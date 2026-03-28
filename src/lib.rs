mod filter;
mod format;
mod log_config;
mod style;
mod writer;

pub use filter::*;
pub use format::*;
pub use log_config::*;
pub use style::*;
pub use writer::*;

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
    filter: filter::FilterBuilder,
    writer: writer::WriterBuilder,
    format: format::FormatBuilder,
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

    pub fn filter(mut self, module: Option<&str>, level: LogLevel) -> Self {
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
        F: Fn(&mut BufferFormatter, &LogMessage<'_>) -> std::io::Result<()> + Sync + Send + 'static,
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
            LogLevel::set_level(max_level);
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
    writer: writer::Writer,
    filter: filter::Filter,
    format: format::Format,
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

    pub fn get_max_level(&self) -> LogLevel {
        self.filter.max_level()
    }

    pub fn matches(&self, record_msg: &LogMessage<'_>) -> bool {
        self.filter.matches(record_msg)
    }

    pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
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

            let mut builder = LogMessage::builder();

            builder
                .level(LogLevel::Debug)
                .target(file)
                .module(Some(file))
                .msg(msg);

            logger.log_msg(&builder.build());
            logger.flush();
        }
    }))
}

#[cfg(not(feature = "panic-hook"))]
fn trigger_panic() {}

// ================
// MACRO
//=================

fn log_reduce_size(
    logger: &Logger,
    level: LogLevel,
    target: &str,
    module: &'static str,
    msg: std::fmt::Arguments,
) {
    let mut builder = LogMessage::builder();

    builder
        .level(level)
        .target(target)
        .module(Some(module))
        .msg(msg);

    logger.log_msg(&builder.build());
}

pub fn log_build<'a>(
    logger: &Logger,
    level: LogLevel,
    target: &str,
    module: &'static str,
    msg: std::fmt::Arguments,
) {
    log_reduce_size(logger, level, target, module, msg)
}

#[cfg(feature = "log-control")]
#[macro_export]
macro_rules! log {
    // logger + target
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let lvl = $lvl;
        if lvl as usize <= $crate::LogLevel::get_level() as usize {
            $crate::log_build($logger, lvl, $target, module_path!(), format_args!($($arg)+));
        }
    }};
    // logger only
    (logger: $logger:expr, $lvl:expr, $($arg:tt)+) => {
        $crate::log!(logger: $logger, target: module_path!(), $lvl, $($arg)+)
    };
    // target only
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {
        if let Some(logger) = $crate::Logger::get() {
            $crate::log!(logger: logger, target: $target, $lvl, $($arg)+)
        }
    };
    // simple log
    ($lvl:expr, $($arg:tt)+) => {
        $crate::log!(target: module_path!(), $lvl, $($arg)+)
    };
}

#[cfg(not(feature = "log-control"))]
#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => {{
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$target;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (logger: $logger:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$logger;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$logger;
        let _ = &$target;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
}

// Level-specific macros
#[macro_export]
macro_rules! error { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Error, $($arg)+) }; }
#[macro_export]
macro_rules! warn { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Warn, $($arg)+) }; }
#[macro_export]
macro_rules! info { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Info, $($arg)+) }; }
#[macro_export]
macro_rules! debug { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Debug, $($arg)+) }; }
#[macro_export]
macro_rules! trace { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Trace, $($arg)+) }; }
