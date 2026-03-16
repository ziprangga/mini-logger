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
    static LOG_FORMATTER: RefCell<Option<LogFormatter>> = const {RefCell::new(None)};
}

fn set_logger(logger: Logger) -> Result<(), &'static str> {
    LOGGER.set(logger).map_err(|_| "Logger already initialized")
}

fn get_logger() -> Option<&'static Logger> {
    LOGGER.get()
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

    pub fn filter(&mut self, module: Option<&str>, level: LogLevel) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    pub fn color_style(&mut self, color_style: ColorStyle) -> &mut Self {
        self.writer.color_style(color_style);
        self
    }

    // pub fn default_format(&mut self) -> &mut Self {
    //     self.format = Default::default();
    //     self
    // }

    // pub fn filter_module(&mut self, module: &str, level: Level) -> &mut Self {
    //     self.filter.filter_module(module, level);
    //     self
    // }

    // pub fn filter_level(&mut self, level: Level) -> &mut Self {
    //     self.filter.filter_level(level);
    //     self
    // }

    // pub fn output(&mut self, target: Output) -> &mut Self {
    //     self.writer.output(target);
    //     self
    // }

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

    pub fn try_init(&mut self) -> Result<(), &'static str> {
        let logger = self.build();

        let max_level = logger.filter();
        let result = set_logger(logger);

        if result.is_ok() {
            LogLevel::set_level(max_level);
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
    format: logger::LogFormat,
}

impl Logger {
    pub fn set(logger: Logger) -> Result<(), &'static str> {
        set_logger(logger)
    }

    pub fn get() -> Option<&'static Self> {
        get_logger()
    }

    pub fn filter(&self) -> LogLevel {
        self.filter.filter()
    }

    pub fn matches(&self, record_msg: &LogMessage<'_>) -> bool {
        self.filter.matches(record_msg)
    }

    // pub fn enabled(&self, log_config: &LogConfig<'_>) -> bool {
    //     self.filter.enabled(log_config)
    // }

    pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
        if self.matches(record_msg) {
            let write_and_flush = |log_formatter: &mut LogFormatter,
                                   record_msg: &LogMessage<'_>| {
                let _ = self
                    .format
                    .format_record(log_formatter, record_msg)
                    .and_then(|_| log_formatter.print(&self.writer));

                // Clear buffer for next log
                log_formatter.clear();
            };

            // Use thread-local buffer
            let printed = LOG_FORMATTER
                .try_with(|tl_buf| {
                    if let Ok(mut slot) = tl_buf.try_borrow_mut() {
                        if let Some(ref mut log_formatter) = *slot {
                            if log_formatter.color_style() != self.writer.color_style() {
                                *log_formatter = LogFormatter::new(&self.writer);
                            }
                            write_and_flush(log_formatter, record_msg);
                        } else {
                            let mut log_formatter = LogFormatter::new(&self.writer);
                            write_and_flush(&mut log_formatter, record_msg);
                            *slot = Some(log_formatter);
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

    // pub fn flush(&self) {
    //     // Flush all thread-local formatters
    //     let _ = LOG_FORMATTER.try_with(|tl_buf| {
    //         if let Ok(mut slot) = tl_buf.try_borrow_mut() {
    //             if let Some(ref mut log_formatter) = *slot {
    //                 // print buffer
    //                 let _ = log_formatter.print(&self.writer);
    //                 // reset buffer
    //                 log_formatter.clear();
    //             }
    //         }
    //     });

    //     // Flush the underlying writer's buffer
    //     let _ = self.writer.print_out(&self.writer.buffer());
    // }
}

// ================
// MACRO
//=================

#[macro_export]
macro_rules! log {
    // logger + target
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let lvl = $lvl;
        if lvl as usize <= $crate::LogLevel::get_level() as usize {
            let msg = $crate::LogMessage::builder()
                .level(lvl)
                .target($target)
                .module(Some(module_path!()))
                .msg(format_args!($($arg)+))
                .build();
            $logger.log_msg(&msg);
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
