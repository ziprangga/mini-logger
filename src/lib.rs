mod filter;
mod format;
mod logger;
mod style;
mod writer;

pub use filter::*;
pub use format::{LogFormatter, try_with_log_formatter_slot};
pub use logger::*;
pub use style::{Color, ColorMode, Timestamp, TimestampPrecision};
pub use writer::*;

use std::sync::OnceLock;

static LOGGER: OnceLock<Logger> = OnceLock::new();

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
        }
    }
}

pub struct Logger {
    writer: writer::Writer,
    filter: filter::Filter,
    format: logger::FormatLog,
}

impl Logger {
    pub fn get() -> Option<&'static Self> {
        get_logger()
    }

    pub fn get_max_level(&self) -> LogLevel {
        self.filter.max_level()
    }

    pub fn matches(&self, record_msg: &LogMessage<'_>) -> bool {
        self.filter.matches(record_msg)
    }

    pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
        if !self.matches(record_msg) {
            return;
        }

        let write_and_flush = |log_formatter: &mut LogFormatter| {
            let _ = self
                .format
                .format_record(log_formatter, record_msg)
                .and_then(|_| log_formatter.print(&self.writer));
            // Clear buffer for next log
            log_formatter.clear();
        };

        //Use thread-local buffer
        let printed = try_with_log_formatter_slot(|slot| match slot {
            Some(log_formatter) => {
                if log_formatter.color_mode() != self.writer.color_mode() {
                    *log_formatter = LogFormatter::new(&self.writer);
                }
                write_and_flush(log_formatter);
            }
            None => {
                let mut log_formatter = LogFormatter::new(&self.writer);
                write_and_flush(&mut log_formatter);
                *slot = Some(log_formatter);
            }
        })
        .is_some();

        // Fallback if thread-local unavailable (thread shutting down)
        if !printed {
            let mut log_formatter = LogFormatter::new(&self.writer);
            write_and_flush(&mut log_formatter);
        }
    }

    pub fn flush(&self) {
        // Flush all thread-local formatters
        let _ = try_with_log_formatter_slot(|slot| {
            if let Some(log_formatter) = slot {
                let _ = log_formatter.print(&self.writer);
                log_formatter.clear();
            }
        });
        // Flush the underlying writer's buffer
        let _ = self.writer.flush();
    }
}

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
