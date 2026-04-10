use crate::filter::{Filter, FilterBuilder, FilterEnv};
use crate::format::{Format, FormatBuilder};
use crate::record::FilterLevel;
use crate::record::RecMessage;
use crate::style::ColorMode;
use crate::writer::{BufferFormatter, Writer, WriterBuilder, try_with_buf_formatter_slot};

use std::sync::OnceLock;

#[cfg(feature = "runtime-control")]
use std::sync::atomic::{AtomicBool, Ordering};

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[cfg(feature = "log-control")]
fn set_logger(logger: Logger) -> Result<(), &'static str> {
    LOGGER.set(logger).map_err(|_| "Logger already initialized")
}

#[cfg(not(feature = "log-control"))]
fn set_logger(_logger: Logger) -> Result<(), &'static str> {
    Ok(())
}

fn get_logger() -> Option<&'static Logger> {
    LOGGER.get()
}

fn try_init() -> Result<(), &'static str> {
    Builder::new()
        .env_default()
        .trigger_panic_to_output()
        .output_stdout()
        .try_init()
}

pub fn init() {
    try_init().expect("env_logger::init should not be called after logger initialized");
}

#[derive(Default)]
pub struct Builder {
    filter: FilterBuilder,
    writer: WriterBuilder,
    format: FormatBuilder,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env_default(mut self) -> Self {
        let mut env_filter = FilterEnv::new(&mut self.filter);
        env_filter.parse_env_var("RUST_LOG");
        self
    }

    pub fn from_env(mut self, var_name: &str) -> Self {
        let mut env_filter = FilterEnv::new(&mut self.filter);
        env_filter.parse_env_var(var_name);
        self
    }

    pub fn filter(mut self, module: Option<&str>, level: FilterLevel) -> Self {
        self.filter.filter_target(module, level);
        self
    }

    pub fn color_mode(mut self, color_mode: ColorMode) -> Self {
        self.writer.color_mode(color_mode);
        self
    }

    pub fn format_level(mut self, write: bool) -> Self {
        self.format.format_default().level(write);
        self
    }

    pub fn format_target(mut self, write: bool) -> Self {
        self.format.format_default().target(write);
        self
    }

    pub fn format_module_path(mut self, write: bool) -> Self {
        self.format.format_default().module_path(write);
        self
    }

    pub fn format_custom<F>(mut self, format: F) -> Self
    where
        F: Fn(&mut BufferFormatter, &RecMessage<'_>) -> std::io::Result<()> + Sync + Send + 'static,
    {
        self.format.format_custom(format);
        self
    }

    pub fn output_stdout(mut self) -> Self {
        self.writer.stdout();
        self
    }

    pub fn output_stderr(mut self) -> Self {
        self.writer.stderr();
        self
    }
    pub fn output_file(mut self, path: impl Into<String>) -> Self {
        self.writer.file(path);
        self
    }

    pub fn trigger_panic_to_output(self) -> Self {
        trigger_panic();
        self
    }

    pub fn try_init(self) -> Result<(), &'static str> {
        let logger = self.build();

        let max_level = logger.get_max_level();
        let result = set_logger(logger);

        if result.is_ok() {
            FilterLevel::set_level(max_level);
        }

        result
    }

    pub fn init(self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    pub fn build(self) -> Logger {
        Logger {
            writer: self.writer.build(),
            filter: self.filter.build(),
            format: self.format.build(),
            #[cfg(feature = "runtime-control")]
            active: AtomicBool::new(true),
        }
    }
}

pub struct Logger {
    filter: Filter,
    writer: Writer,
    format: Format,
    #[cfg(feature = "runtime-control")]
    active: AtomicBool,
}

impl Logger {
    pub fn get() -> Option<&'static Self> {
        get_logger()
    }

    #[cfg(feature = "runtime-control")]
    pub fn enable(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "runtime-control")]
    pub fn disable(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    #[cfg(feature = "runtime-control")]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn get_max_level(&self) -> FilterLevel {
        self.filter.max_level()
    }

    pub fn matches(&self, record_msg: &RecMessage<'_>) -> bool {
        self.filter.matches(record_msg)
    }

    pub fn rec_msg(&self, record_msg: &RecMessage<'_>) {
        #[cfg(feature = "runtime-control")]
        if !self.is_active() {
            return;
        }

        if !self.matches(record_msg) {
            return;
        }

        let write_and_flush = |buf_formatter: &mut BufferFormatter| {
            let _ = self
                .format
                .format_record(buf_formatter, record_msg)
                .and_then(|_| buf_formatter.print(&self.writer));
            // Clear buffer for next log
            buf_formatter.clear();
        };

        //Use thread-local buffer
        let printed = try_with_buf_formatter_slot(|slot| match slot {
            Some(buf_formatter) => {
                if buf_formatter.color_mode() != self.writer.color_mode() {
                    *buf_formatter = BufferFormatter::new(&self.writer);
                }
                write_and_flush(buf_formatter);
            }
            None => {
                let mut buf_formatter = BufferFormatter::new(&self.writer);
                write_and_flush(&mut buf_formatter);
                *slot = Some(buf_formatter);
            }
        })
        .is_some();

        // Fallback if thread-local unavailable (thread shutting down)
        if !printed {
            let mut buf_formatter = BufferFormatter::new(&self.writer);
            write_and_flush(&mut buf_formatter);
        }
    }

    pub fn flush(&self) {
        // Flush all thread-local formatters
        let _ = try_with_buf_formatter_slot(|slot| {
            if let Some(buf_formatter) = slot {
                let _ = buf_formatter.print(&self.writer);
                buf_formatter.clear();
            }
        });
        // Flush the underlying writer's buffer
        let _ = self.writer.flush();
    }
}

// ================
// Panic trigger
// ================
#[cfg(feature = "panic-hook")]
fn trigger_panic() {
    std::panic::set_hook(Box::new(move |info| {
        if let Some(logger) = crate::Logger::get() {
            let (file, _line) = match info.location() {
                Some(loc) => (loc.file(), loc.line()),
                None => ("unknown", 0),
            };

            let msg = format_args!("panic: {}", info);

            let mut builder = RecMessage::builder();

            builder
                .level(FilterLevel::Debug)
                .target(file)
                .module(Some(file))
                .msg(msg);

            logger.rec_msg(&builder.build());
            logger.flush();
        }
    }))
}

#[cfg(not(feature = "panic-hook"))]
fn trigger_panic() {}
