mod filter;
mod logger;
mod style;
mod writer;

pub use filter::*;
pub use logger::*;
pub use style::{Color, ColorStyle, Timestamp, TimestampPrecision};
pub use writer::*;

// use std::cell::RefCell;
// use std::sync::OnceLock;
// use std::thread_local;

// static LOGGER: OnceLock<Logger> = OnceLock::new();

// thread_local! {
//     static FORMATTER: RefCell<Option<LogFormatter>> = const {RefCell::new(None)};
// }

// pub struct Logger {
//     writer: writer::Writer,
//     // filter: filter::Filter,
//     format: logger::LogFormatter,
// }

// impl Logger {

//     pub fn from_env(env: )

//     pub fn filter(&self) -> LevelFilter {
//             self.filter.filter()
//         }

//     pub fn matches(&self, log_message: &LogMessage<'_>) -> bool {
//                 self.filter.matches(record)
//             }

//     fn enable(&self, _log_config: &LogConfig) -> bool {
//         // to do after filter first
//         true
//     }
//     fn log_msg(&self, log_message: &LogMessage) {

//     };
//     fn flush(&self);
// }

// pub struct ThreadLocalLogger;

// impl ThreadLocalLogger {
//     pub fn log<'a>(msg: &LogMessage<'a>) {
//         // Closure to format and flush
//         let write_and_flush = |formatter: &mut LogFormatter, msg: &LogMessage<'a>| {
//             let _ = LogMessageFormatWriter {
//                 log_message: msg,
//                 buf: formatter,
//                 written_header: false,
//             }
//             .write();
//
//              if let Some(global_logger) = LOGGER.get() {
//               global_logger.log_msg(msg); // call concrete logger
//            }

//             // Flush to global logger (replace with your actual global LOGGER)
//             formatter.flush_to_logger();

//             // Clear buffer for next log
//             formatter.clear();
//         };

//         // Use thread-local buffer
//         let printed = FORMATTER
//             .try_with(|tl_buf| {
//                 if let Ok(mut slot) = tl_buf.try_borrow_mut() {
//                     if let Some(ref mut formatter) = *slot {
//                         // Reuse existing formatter
//                         write_and_flush(formatter, msg);
//                     } else {
//                         // No formatter yet, create and store it
//                         let mut formatter = LogFormatter::new();
//                         write_and_flush(&mut formatter, msg);
//                         *slot = Some(formatter);
//                     }
//                 } else {
//                     // Re-entrant logging: use a temporary formatter
//                     write_and_flush(&mut LogFormatter::new(), msg);
//                 }
//             })
//             .is_ok();

//         // Fallback if thread-local unavailable (thread shutting down)
//         if !printed {
//             write_and_flush(&mut LogFormatter::new(), msg);
//         }
//     }
// }

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
