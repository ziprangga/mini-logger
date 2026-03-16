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

    // pub fn flush(&self) {
    //     //
    // }
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
