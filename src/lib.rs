mod filter;
mod logger;
mod style;
mod writer;

pub use filter::*;
pub use logger::*;
pub use style::{Color, ColorStyle, Timestamp, TimestampPrecision};
pub use writer::*;

use std::cell::RefCell;
use std::sync::OnceLock;
use std::thread_local;

static LOGGER: OnceLock<Logger> = OnceLock::new();

thread_local! {
    static FORMATTER: RefCell<Option<LogFormatter>> = const {RefCell::new(None)};
}

fn set_logger(logger: Logger) -> Result<(), &'static str> {
    LOGGER.set(logger).map_err(|_| "Logger already initialized")
}

fn try_init() -> Result<(), &'static str> {
    Builder::new().env_default().output_stdout().try_init()
}

pub fn init() {
    try_init().expect("env_logger::init should not be called after logger initialized");
}

#[derive(Default)]
pub struct Builder {
    filter: filter::FilterBuilder,
    writer: writer::WriterBuilder,
    format: logger::FormatBuilder,
    built: bool,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env_default(&mut self) -> &mut Self {
        let mut env_filter = FilterEnv::new(&mut self.filter);
        env_filter.parse_env_var("RUST_LOG");
        self
    }

    pub fn from_env(&mut self, var_name: &str) -> &mut Self {
        let mut env_filter = FilterEnv::new(&mut self.filter);
        env_filter.parse_env_var(var_name);
        self
    }

    pub fn filter_module(&mut self, module: &str, level: Level) -> &mut Self {
        self.filter.filter_module(module, level);
        self
    }

    pub fn filter_level(&mut self, level: Level) -> &mut Self {
        self.filter.filter_level(level);
        self
    }

    pub fn filter(&mut self, module: Option<&str>, level: Level) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    pub fn output(&mut self, target: Output) -> &mut Self {
        self.writer.output(target);
        self
    }

    pub fn output_stdout(&mut self) -> &mut Self {
        self.writer.stdout();
        self
    }

    pub fn output_stderr(&mut self) -> &mut Self {
        self.writer.stderr();
        self
    }
    pub fn output_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.writer.file(path);
        self
    }

    pub fn color_style(&mut self, color_style: ColorStyle) -> &mut Self {
        self.writer.color_style(color_style);
        self
    }

    pub fn try_init(&mut self) -> Result<(), &'static str> {
        let logger = self.build();

        let max_level = logger.filter();
        let result = set_logger(logger);

        if result.is_ok() {
            Level::set_level(max_level);
        }

        result
    }

    pub fn init(&mut self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    pub fn build(&mut self) -> Logger {
        assert!(!self.built, "attempt to re-use consumed logger builder");
        self.built = true;
        Logger {
            writer: self.writer.build(),
            filter: self.filter.build(),
            format: self.format.build(),
        }
    }
}

pub struct Logger {
    writer: writer::Writer,
    filter: filter::Filter,
    format: logger::FormatFn,
}

impl Logger {
    pub fn filter(&self) -> Level {
        self.filter.filter()
    }
    pub fn matches(&self, record_msg: &LogMessage<'_>) -> bool {
        self.filter.matches(record_msg)
    }
    pub fn enabled(&self, log_config: &LogConfig<'_>) -> bool {
        self.filter.enabled(log_config)
    }

    pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
        if self.matches(record_msg) {
            let write_and_flush = |formatter: &mut LogFormatter, record_msg: &LogMessage<'_>| {
                let _ = self
                    .format
                    .format_record(formatter, record_msg)
                    .and_then(|_| formatter.print(&self.writer));

                // Clear buffer for next log
                formatter.clear();
            };

            // Use thread-local buffer
            let printed = FORMATTER
                .try_with(|tl_buf| {
                    if let Ok(mut slot) = tl_buf.try_borrow_mut() {
                        if let Some(ref mut formatter) = *slot {
                            if formatter.color_style() != self.writer.color_style() {
                                *formatter = LogFormatter::new(&self.writer);
                            }
                            write_and_flush(formatter, record_msg);
                        } else {
                            let mut formatter = LogFormatter::new(&self.writer);
                            write_and_flush(&mut formatter, record_msg);
                            *slot = Some(formatter);
                        }
                    } else {
                        write_and_flush(&mut LogFormatter::new(&self.writer), record_msg);
                    }
                })
                .is_ok();

            // Fallback if thread-local unavailable (thread shutting down)
            if !printed {
                write_and_flush(&mut LogFormatter::new(&self.writer), record_msg);
            }
        }
    }

    pub fn flush(&self) {
        // Flush all thread-local formatters
        let _ = FORMATTER.try_with(|tl_buf| {
            if let Ok(mut slot) = tl_buf.try_borrow_mut() {
                if let Some(ref mut formatter) = *slot {
                    // print buffer
                    let _ = formatter.print(&self.writer);
                    // reset buffer
                    formatter.clear();
                }
            }
        });

        // Flush the underlying writer's buffer
        let _ = self.writer.print_out(&self.writer.buffer());
    }
}

//==================================================================
// use std::sync::OnceLock;
// static LOGGER: OnceLock<&'static dyn Logger> = OnceLock::new();
//
// use std::sync::{OnceLock, Mutex};

// static LOGGER: OnceLock<Mutex<LogStruct>> = OnceLock::new();

// pub trait Logger: Send + Sync {
//     fn enable(&self, log_config: &LogConfig) -> bool;
//     fn log_msg(&self, log_message: &LogMessage);
//     fn flush(&self);
// }

// pub fn set_logger(logger: &'static dyn Logger) -> Result<(), &'static str> {
//     LOGGER.set(logger).map_err(|_| "Logger already set")
// }

// pub fn logger() -> Option<&'static dyn Logger> {
//     LOGGER.get().copied()
// }
