mod time;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

pub use time::*;

#[cfg(feature = "buffer")]
const USE_BUFFER: bool = true;
#[cfg(not(feature = "buffer"))]
const USE_BUFFER: bool = false;

#[cfg(feature = "console")]
const USE_CONSOLE: bool = true;
#[cfg(not(feature = "console"))]
const USE_CONSOLE: bool = false;

const MAX_BUFFER_SIZE: usize = 1024 * 1024;

// -------------------------------
// Log levels
// -------------------------------
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Level {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

fn level_as_str(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
        Level::Off => "OFF",
    }
}

// -------------------------------
// Global logger
// -------------------------------
static GLOBAL_LOGGER: OnceLock<Arc<DebugLogInner>> = OnceLock::new();

#[derive(Clone)]
pub struct DebugLogInner {
    buffer: Option<Arc<Mutex<VecDeque<u8>>>>,
    console: Option<Arc<Mutex<io::Stdout>>>,
}

impl DebugLogInner {
    pub fn new() -> Self {
        DebugLogInner {
            buffer: if USE_BUFFER {
                Some(Arc::new(Mutex::new(VecDeque::new())))
            } else {
                None
            },
            console: if USE_CONSOLE {
                Some(Arc::new(Mutex::new(io::stdout())))
            } else {
                None
            },
        }
    }

    fn push_log(&self, module: &str, level: Level, args: std::fmt::Arguments) {
        // Use thread-local buffer to format first (like env_logger)
        thread_local! {
            static TLS_BUF: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(1024));
        }

        let msg = TLS_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            buf.clear();
            use std::fmt::Write;
            let _ = write!(
                buf,
                "[{} {}] {}",
                time::get_timestamp(),
                level_as_str(level),
                args
            );
            buf.clone()
        });

        if let Some(console) = &self.console {
            let mut lock = console.lock().unwrap();
            let _ = lock.write_all(&msg);
            let _ = lock.write_all(b"\n");
        }

        if let Some(buffer) = &self.buffer {
            let mut buf = buffer.lock().unwrap();
            buf.extend(msg);
            buf.push(b'\n');
            while buf.len() > MAX_BUFFER_SIZE {
                buf.pop_front();
            }
        }
    }

    pub fn get_from_buffer(&self) -> Option<String> {
        self.buffer.as_ref().map(|buf| {
            let buf = buf.lock().unwrap();
            String::from_utf8_lossy(&buf.make_contiguous()).to_string()
        })
    }

    pub fn save_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(buffer) = &self.buffer {
            let buf = buffer.lock().unwrap();
            let mut file = File::create(path)?;
            file.write_all(&buf.make_contiguous())?;
        }
        Ok(())
    }
}

// -------------------------------
// Public API
// -------------------------------
pub struct DebugLog;

impl DebugLog {
    /// Initialize global logger
    pub fn init() -> Arc<DebugLogInner> {
        GLOBAL_LOGGER
            .get_or_init(|| Arc::new(DebugLogInner::new()))
            .clone()
    }

    /// Access global logger for reading buffer
    pub fn get() -> Arc<DebugLogInner> {
        GLOBAL_LOGGER
            .get()
            .expect("Logger not initialized. Call DebugLog::init() first.")
            .clone()
    }
}

/// Write log to global logger
pub fn write_log(module: &str, level: Level, args: std::fmt::Arguments) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        logger.push_log(module, level, args);
    } else {
        println!("[{} {}] {}", module, level_as_str(level), args);
    }
}

// -------------------------------
// Macros
// -------------------------------
#[macro_export]
macro_rules! error_dev {
    ($($arg:tt)*) => {{
        $crate::write_log(module_path!(), $crate::Level::Error, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn_dev {
    ($($arg:tt)*) => {{
        $crate::write_log(module_path!(), $crate::Level::Warn, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! info_dev {
    ($($arg:tt)*) => {{
        $crate::write_log(module_path!(), $crate::Level::Info, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {{
        $crate::write_log(module_path!(), $crate::Level::Debug, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! trace_dev {
    ($($arg:tt)*) => {{
        $crate::write_log(module_path!(), $crate::Level::Trace, format_args!($($arg)*));
    }};
}
