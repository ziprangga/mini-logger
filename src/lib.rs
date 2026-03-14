mod log_msg;
mod output;
mod style;

pub use log_msg::*;
pub use output::*;
pub use style::*;

mod writer;
pub use writer::*;
mod formatter;
pub use formatter::*;

// use chrono;
// use std::collections::HashMap;
// use std::collections::VecDeque;
// use std::fs::File;
// use std::io;
// use std::io::Write;
// use std::path::Path;
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::{Arc, LazyLock, Mutex, RwLock};

// #[cfg(feature = "buffer")]
// const USE_BUFFER: bool = true;

// #[cfg(not(feature = "buffer"))]
// const USE_BUFFER: bool = false;

// #[cfg(feature = "console")]
// const USE_CONSOLE: bool = true;

// #[cfg(not(feature = "console"))]
// const USE_CONSOLE: bool = false;

// const MAX_BUFFER_SIZE: usize = 1024 * 1024;

// static LOG_CONTEXT: LazyLock<Mutex<Option<Arc<DebugLogInner>>>> =
//     LazyLock::new(|| Mutex::new(None));

// static MODULE_LOG_LEVELS: LazyLock<RwLock<HashMap<String, Level>>> =
//     LazyLock::new(|| RwLock::new(HashMap::new()));

// struct BufWriter<'a>(&'a mut VecDeque<u8>);

// impl<'a> BufWriter<'a> {
//     pub fn writer(buf: &'a mut VecDeque<u8>) -> Self {
//         BufWriter(buf)
//     }
// }

// impl<'a> std::fmt::Write for BufWriter<'a> {
//     fn write_str(&mut self, s: &str) -> std::fmt::Result {
//         // self.0.extend_from_slice(s.as_bytes());
//         let bytes = s.as_bytes();
//         while self.0.len() + bytes.len() > MAX_BUFFER_SIZE {
//             self.0.pop_front();
//         }

//         self.0.extend(bytes);
//         Ok(())
//     }
// }

// #[derive(Clone)]
// struct DebugLogInner {
//     buffer: Option<Arc<Mutex<VecDeque<u8>>>>,
//     console: Option<Arc<Mutex<io::Stdout>>>,
// }

// impl DebugLogInner {
//     fn push_log(&self, module: &str, level: Level, args: std::fmt::Arguments) {
//         if !level.enabled(module) {
//             return;
//         }

//         if let Some(out) = &self.console {
//             let mut lock = out.lock().expect("Failed to lock console output");
//             let msg = format_log(module, level, args.clone(), true);
//             let _ = writeln!(lock, "{}", msg);
//         }

//         if let Some(buf) = &self.buffer {
//             let mut buf = buf.lock().expect("Failed to lock log buffer");
//             use std::fmt::Write;
//             let msg = format_log(module, level, args, false);
//             // let _ = write!(BufWriter::writer(&mut *buf), "{}\n", msg);
//             let mut writer = BufWriter::writer(&mut *buf);
//             let _ = write!(writer, "{}\n", msg);
//         }
//     }

//     fn get_from_buffer(&self) -> Option<String> {
//         self.buffer.as_ref().map(|buf| {
//             let mut buf = buf.lock().expect("Failed to lock log buffer for reading");
//             // String::from_utf8_lossy(&buf).to_string()
//             String::from_utf8_lossy(&buf.make_contiguous()).to_string()
//         })
//     }

//     fn save_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
//         if let Some(buf) = &self.buffer {
//             let mut buf = buf.lock().expect("Failed to lock log buffer for saving");
//             let mut file = File::create(path)?;
//             // file.write_all(&buf)?;
//             file.write_all(&buf.make_contiguous())?;
//         }
//         Ok(())
//     }
// }

// pub struct DebugLog {
//     inner: Arc<DebugLogInner>,
// }

// impl DebugLog {
//     pub fn init(config: Option<&[(&str, Level)]>) -> Self {
//         let use_buffer = USE_BUFFER;
//         let use_console = USE_CONSOLE;
//         let inner = Arc::new(DebugLogInner {
//             buffer: if use_buffer {
//                 Some(Arc::new(Mutex::new(VecDeque::new())))
//             } else {
//                 None
//             },
//             console: if use_console {
//                 Some(Arc::new(Mutex::new(io::stdout())))
//             } else {
//                 None
//             },
//         });

//         let mut guard = LOG_CONTEXT
//             .lock()
//             .expect("Failed to lock LOG_CONTEXT for initialization");
//         *guard = Some(inner.clone());

//         if use_buffer
//             && guard
//                 .as_ref()
//                 .expect("LOG_CONTEXT is None")
//                 .buffer
//                 .is_none()
//         {
//             panic!("LOG_CONTEXT compiled without buffer support");
//         }
//         if use_console
//             && guard
//                 .as_ref()
//                 .expect("LOG_CONTEXT is None")
//                 .console
//                 .is_none()
//         {
//             panic!("LOG_CONTEXT compiled without console support");
//         }

//         if let Some(cfg) = config {
//             let mut map = MODULE_LOG_LEVELS
//                 .write()
//                 .expect("Failed to acquire write lock for MODULE_LOG_LEVELS");
//             for &(crate_name, level) in cfg {
//                 map.insert(crate_name.to_string(), level);
//             }
//         } else {
//             Self::init_from_env();
//             // Check name of CRATE_LEVEL Value
//             // println!("MODULE_LOG_LEVELS = {:?}", MODULE_LOG_LEVELS.read().unwrap());
//         }

//         DebugLog { inner }
//     }

//     pub fn get_log_from_buffer(&self) -> Option<String> {
//         self.inner.get_from_buffer()
//     }

//     pub fn save_log_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
//         self.inner.save_buffer_to_file(path)
//     }

//     fn init_from_env() {
//         if let Ok(env) = std::env::var("RUST_LOG") {
//             let mut map = MODULE_LOG_LEVELS
//                 .write()
//                 .expect("Failed to acquire write lock for MODULE_LOG_LEVELS");

//             for part in env.split(',') {
//                 let mut kv = part.split('=');
//                 let crate_name = kv
//                     .next()
//                     .expect("Invalid RUST_LOG entry: missing crate name")
//                     .trim()
//                     .to_string();

//                 let level_str = kv.next().unwrap_or("debug").trim().to_lowercase();

//                 let level = match level_str.as_str() {
//                     "error" => Level::Error,
//                     "warn" => Level::Warn,
//                     "info" => Level::Info,
//                     "debug" => Level::Debug,
//                     "trace" => Level::Trace,
//                     unknown => {
//                         eprintln!(
//                             "RUST_LOG: unknown level '{}' for crate '{}', defaulting to DEBUG",
//                             unknown, crate_name
//                         );
//                         Level::Debug
//                     }
//                 };
//                 map.insert(crate_name, level);
//             }
//         }
//     }
// }

// // -------------------------------
// // Helper
// // -------------------------------
// /// Get all logs from the global buffer as `Option<String>`.
// /// Returns `None` if the buffer is not enabled or not initialized.
// pub fn get_log_from_global_buffer() -> Option<String> {
//     LOG_CONTEXT
//         .lock()
//         .expect("Failed to lock LOG_CONTEXT mutex")
//         .as_ref()
//         .and_then(|inner| inner.get_from_buffer())
// }

// /// Save the global buffer to a file.
// /// Returns `Ok(())` if successful, or if buffer is not initialized.
// /// Returns an `io::Error` if the file write fails.
// pub fn save_global_log_buffer_to_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
//     if let Some(inner) = LOG_CONTEXT
//         .lock()
//         .expect("Failed to lock LOG_CONTEXT")
//         .as_ref()
//     {
//         inner.save_buffer_to_file(path)
//     } else {
//         Ok(())
//     }
// }

// /// Writes a log message using the global debug context.
// ///
// /// If the specified log level is enabled for the given module, this function
// /// forwards the message to the shared `LOG_CONTEXT`. If the context is not
// /// initialized, it will print a warning to stdout.
// ///
// /// # Parameters
// /// - `module`: The module path or name for which the log is being recorded.
// /// - `level`: The severity level of the log message.
// /// - `args`: The formatted arguments for the log message.
// #[doc(hidden)]
// pub fn write_log(module: &str, level: Level, args: std::fmt::Arguments) {
//     if level.enabled(module) {
//         let maybe_inner = LOG_CONTEXT
//             .lock()
//             .expect("Failed to lock LOG_CONTEXT")
//             .as_ref()
//             .cloned();
//         if let Some(inner) = maybe_inner {
//             inner.push_log(module, level, args);
//         } else {
//             println!("LOG_CONTEXT not initialized");
//         }
//     }
// }

// /// Formats a log message as a string with timestamp, level, module, and content.
// ///
// /// The log is formatted as:
// /// `[timestamp LEVEL module] message`
// ///
// /// - `module`: The name or path of the module emitting the log.
// /// - `level`: The severity level of the log.
// /// - `args`: The formatted log message arguments.
// /// - `use_color`: Whether to include ANSI color codes for terminal output.
// ///
// /// Returns the fully formatted log string.
// fn format_log(module: &str, level: Level, args: std::fmt::Arguments, use_color: bool) -> String {
//     let ts = chrono::Local::now().format("%b %d %H:%M:%S").to_string();
//     let color_level = color_for_level(level, use_color);
//     let module_color = color_for_module(module, use_color);
//     let reset = if use_color { "\x1b[0m" } else { "" };
//     format!(
//         "[{} {}{}{} {}{}{}] {}",
//         ts,
//         color_level,
//         level_as_str(level),
//         reset,
//         module_color,
//         module,
//         reset,
//         args
//     )
// }

// /// Converts a `Level` enum to its string representation.
// fn level_as_str(level: Level) -> &'static str {
//     match level {
//         Level::Error => "ERROR",
//         Level::Warn => "WARN",
//         Level::Info => "INFO",
//         Level::Debug => "DEBUG",
//         Level::Trace => "TRACE",
//         Level::Off => "OFF",
//     }
// }

// /// Returns ANSI color code for a log level if `use_color` is true; otherwise returns an empty string.
// fn color_for_level(level: Level, use_color: bool) -> &'static str {
//     if !use_color {
//         return "";
//     }
//     match level {
//         Level::Error => "\x1b[31m", // Red
//         Level::Warn => "\x1b[33m",  // Yellow
//         Level::Info => "\x1b[32m",  // Green
//         Level::Debug => "\x1b[34m", // Blue
//         Level::Trace => "\x1b[35m", // Magenta
//         Level::Off => "\x1b[0m",    // Reset
//     }
// }

// /// Returns ANSI color code for a module based on its first character if `use_color` is true; otherwise returns an empty string.
// fn color_for_module(module: &str, use_color: bool) -> &'static str {
//     if !use_color {
//         return "";
//     }
//     match module.chars().next().expect("Module name is empty") {
//         'a'..='f' => "\x1b[36m", // Cyan
//         'g'..='l' => "\x1b[35m", // Magenta
//         'm'..='r' => "\x1b[32m", // Green
//         's'..='z' => "\x1b[33m", // Yellow
//         _ => "\x1b[37m",         // White fallback
//     }
// }

// // -------------------------------
// // Macro
// // -------------------------------
// #[macro_export]
// macro_rules! error_dev {
//     ($($arg:tt)*) => {{
//         $crate::write_log(module_path!(), $crate::Level::Error, format_args!($($arg)*));
//     }};
// }

// #[macro_export]
// macro_rules! warn_dev {
//     ($($arg:tt)*) => {{
//         $crate::write_log(module_path!(), $crate::Level::Warn, format_args!($($arg)*));
//     }};
// }

// #[macro_export]
// macro_rules! info_dev {
//     ($($arg:tt)*) => {{
//         $crate::write_log(module_path!(), $crate::Level::Info, format_args!($($arg)*));
//     }};
// }

// #[macro_export]
// macro_rules! debug_dev {
//     ($($arg:tt)*) => {{
//         $crate::write_log(module_path!(), $crate::Level::Debug, format_args!($($arg)*));
//     }};
// }

// #[macro_export]
// macro_rules! trace_dev {
//     ($($arg:tt)*) => {{
//         $crate::write_log(module_path!(), $crate::Level::Trace, format_args!($($arg)*));
//     }};
// }
