use chrono;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

#[cfg(feature = "buffer")]
const USE_BUFFER: bool = true;

#[cfg(not(feature = "buffer"))]
const USE_BUFFER: bool = false;

#[cfg(feature = "console")]
const USE_CONSOLE: bool = true;

#[cfg(not(feature = "console"))]
const USE_CONSOLE: bool = false;

#[cfg(feature = "async")]
static LOG_SEQ: AtomicUsize = AtomicUsize::new(0);

static DEBUG_CONTEXT: LazyLock<Mutex<Option<Arc<DebugLogInner>>>> =
    LazyLock::new(|| Mutex::new(None));
static LOG_LEVEL: AtomicUsize = AtomicUsize::new(Level::Trace as usize);
static CRATE_LEVELS: LazyLock<RwLock<HashMap<String, Level>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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

impl Level {
    pub fn set_level(self) {
        LOG_LEVEL.store(self as usize, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn enabled(self, module_path: &str) -> bool {
        let map = CRATE_LEVELS.read().unwrap();
        let mut matched_level: Option<(&str, Level)> = None;
        for (prefix, &level) in map.iter() {
            if module_path.starts_with(prefix) {
                // pick the longest match
                if matched_level.is_none() || prefix.len() > matched_level.as_ref().unwrap().0.len()
                {
                    matched_level = Some((prefix, level));
                }
            }
        }

        let effective_level = matched_level
            .map(|(_, l)| l)
            .unwrap_or_else(|| Level::from_usize(LOG_LEVEL.load(Ordering::Relaxed)));

        self as usize <= effective_level as usize
    }

    pub fn from_usize(val: usize) -> Self {
        match val {
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => Level::Off,
        }
    }
}

struct BufWriter<'a>(&'a mut Vec<u8>);

impl<'a> BufWriter<'a> {
    pub fn writer(buf: &'a mut Vec<u8>) -> Self {
        BufWriter(buf)
    }
}

impl<'a> std::fmt::Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

// #[derive(Clone)]
struct DebugLogInner {
    buffer: Option<Arc<Mutex<Vec<u8>>>>,
    console: Option<Arc<Mutex<io::Stdout>>>,

    #[cfg(feature = "async")]
    tx: Mutex<Option<std::sync::mpsc::Sender<(usize, String, Level, String)>>>,
}

impl DebugLogInner {
    #[cfg(not(feature = "async"))]
    fn push_log(&self, module: &str, level: Level, args: std::fmt::Arguments) {
        if !level.enabled(module) {
            return;
        }

        if let Some(out) = &self.console {
            let mut lock = out.lock().unwrap();
            let msg = format_log(module, level, args.clone(), true);
            let _ = writeln!(lock, "{}", msg);
        }

        if let Some(buf) = &self.buffer {
            let mut buf = buf.lock().unwrap();
            use std::fmt::Write;
            let msg = format_log(module, level, args, false);
            let _ = write!(BufWriter::writer(&mut *buf), "{}\n", msg);
        }
    }

    #[cfg(feature = "async")]
    fn push_log(&self, module: &str, level: Level, args: std::fmt::Arguments) {
        if !level.enabled(module) {
            return;
        }

        let seq = LOG_SEQ.fetch_add(1, Ordering::SeqCst);
        let msg = args.to_string();
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.send((seq, module.to_string(), level, msg));
        }
        return;
    }

    #[cfg(feature = "async")]
    fn start_worker(
        rx: std::sync::mpsc::Receiver<(usize, String, Level, String)>,
        buffer: Option<Arc<Mutex<Vec<u8>>>>,
        console: Option<Arc<Mutex<io::Stdout>>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut last_seq = 0;
            let mut pending: std::collections::BTreeMap<usize, (String, Level, String)> =
                std::collections::BTreeMap::new();

            while let Ok((seq, module, level, msg)) = rx.recv() {
                pending.insert(seq, (module, level, msg));
                while let Some((module, level, msg)) = pending.remove(&(last_seq)) {
                    if let Some(out) = &console {
                        let mut lock = out.lock().unwrap();
                        let line = format_log(&module, level, format_args!("{}", msg), true);
                        let _ = writeln!(lock, "{}", line);
                    }

                    if let Some(buf) = &buffer {
                        let mut buf = buf.lock().unwrap();
                        use std::fmt::Write;
                        let line = format_log(&module, level, format_args!("{}", msg), false);
                        let _ = write!(BufWriter::writer(&mut *buf), "{}\n", line);
                    }
                    last_seq += 1;
                }
            }
        })
    }

    fn get_from_buffer(&self) -> Option<String> {
        self.buffer.as_ref().map(|buf| {
            let buf = buf.lock().unwrap();
            String::from_utf8_lossy(&buf).to_string()
        })
    }

    fn save_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(buf) = &self.buffer {
            let buf = buf.lock().unwrap();
            let mut file = File::create(path)?;
            file.write_all(&buf)?;
        }
        Ok(())
    }
}

pub struct DebugLog {
    inner: Arc<DebugLogInner>,

    #[cfg(feature = "async")]
    worker: Option<std::thread::JoinHandle<()>>,
}

impl DebugLog {
    pub fn init(config: Option<&[(&str, Level)]>) -> Self {
        let use_buffer = USE_BUFFER;
        let use_console = USE_CONSOLE;
        // let inner = Arc::new(DebugLogInner {
        //     buffer: if use_buffer {
        //         Some(Arc::new(Mutex::new(Vec::new())))
        //     } else {
        //         None
        //     },
        //     console: if use_console {
        //         Some(Arc::new(Mutex::new(io::stdout())))
        //     } else {
        //         None
        //     },
        // });

        let buffer = if use_buffer {
            Some(Arc::new(Mutex::new(Vec::new())))
        } else {
            None
        };

        let console = if use_console {
            Some(Arc::new(Mutex::new(io::stdout())))
        } else {
            None
        };

        #[cfg(feature = "async")]
        let (inner, worker) = {
            let (tx, rx) = std::sync::mpsc::channel();
            let worker = DebugLogInner::start_worker(rx, buffer.clone(), console.clone());
            (
                Arc::new(DebugLogInner {
                    buffer,
                    console,
                    tx: Mutex::new(Some(tx)),
                }),
                Some(worker),
            )
        };

        #[cfg(not(feature = "async"))]
        let inner = Arc::new(DebugLogInner { buffer, console });

        let mut guard = DEBUG_CONTEXT.lock().unwrap();
        *guard = Some(inner.clone());

        if use_buffer && guard.as_ref().unwrap().buffer.is_none() {
            panic!("DEBUG_CONTEXT compiled without buffer support");
        }
        if use_console && guard.as_ref().unwrap().console.is_none() {
            panic!("DEBUG_CONTEXT compiled without console support");
        }

        if let Some(cfg) = config {
            let mut map = CRATE_LEVELS.write().unwrap();
            for &(crate_name, level) in cfg {
                map.insert(crate_name.to_string(), level);
            }
        } else {
            Self::init_from_env();
            println!("CRATE_LEVELS = {:?}", CRATE_LEVELS.read().unwrap());
        }

        DebugLog {
            inner,
            #[cfg(feature = "async")]
            worker,
        }
    }

    pub fn get_log_from_buffer(&self) -> Option<String> {
        self.inner.get_from_buffer()
    }

    pub fn save_log_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.inner.save_buffer_to_file(path)
    }

    fn init_from_env() {
        if let Ok(env) = std::env::var("RUST_LOG") {
            let mut map = CRATE_LEVELS.write().unwrap();
            for part in env.split(',') {
                let mut kv = part.split('=');
                let crate_name = kv.next().unwrap().trim().to_string();
                let level_str = kv.next().unwrap_or("debug").trim().to_lowercase();
                let level = match level_str.as_str() {
                    "error" => Level::Error,
                    "warn" => Level::Warn,
                    "info" => Level::Info,
                    "debug" => Level::Debug,
                    "trace" => Level::Trace,
                    _ => Level::Debug,
                };
                map.insert(crate_name, level);
            }
        }
    }
}

#[cfg(feature = "async")]
impl Drop for DebugLog {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            // Take ownership of the sender, dropping it
            let _ = self.inner.tx.lock().unwrap().take();
            // worker now sees channel closed → exits
            let _ = worker.join();

            let mut guard = DEBUG_CONTEXT.lock().unwrap();
            *guard = None;
        }
    }
}

// -------------------------------
// Helper
// -------------------------------
/// Get all logs from the global buffer as `Option<String>`.
/// Returns `None` if the buffer is not enabled or not initialized.
pub fn get_log_from_global_buffer() -> Option<String> {
    DEBUG_CONTEXT
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|inner| inner.get_from_buffer())
}

/// Save the global buffer to a file.
/// Returns `Ok(())` if successful, or if buffer is not initialized.
/// Returns an `io::Error` if the file write fails.
pub fn save_global_log_buffer_to_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if let Some(inner) = DEBUG_CONTEXT.lock().unwrap().as_ref() {
        inner.save_buffer_to_file(path)
    } else {
        Ok(())
    }
}

/// Writes a log message using the global debug context.
///
/// If the specified log level is enabled for the given module, this function
/// forwards the message to the shared `DEBUG_CONTEXT`. If the context is not
/// initialized, it will print a warning to stdout.
///
/// # Parameters
/// - `module`: The module path or name for which the log is being recorded.
/// - `level`: The severity level of the log message.
/// - `args`: The formatted arguments for the log message.
#[doc(hidden)]
pub fn write_log(module: &str, level: Level, args: std::fmt::Arguments) {
    if level.enabled(module) {
        let maybe_inner = DEBUG_CONTEXT.lock().unwrap().as_ref().cloned();
        if let Some(inner) = maybe_inner {
            inner.push_log(module, level, args);
        } else {
            println!("DEBUG_CONTEXT not initialized");
        }
    }
}

/// Formats a log message as a string with timestamp, level, module, and content.
///
/// The log is formatted as:
/// `[timestamp LEVEL module] message`
///
/// - `module`: The name or path of the module emitting the log.
/// - `level`: The severity level of the log.
/// - `args`: The formatted log message arguments.
/// - `use_color`: Whether to include ANSI color codes for terminal output.
///
/// Returns the fully formatted log string.
fn format_log(module: &str, level: Level, args: std::fmt::Arguments, use_color: bool) -> String {
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let color_level = color_for_level(level, use_color);
    let module_color = color_for_module(module, use_color);
    let reset = if use_color { "\x1b[0m" } else { "" };
    format!(
        "[{} {}{}{} {}{}{}] {}",
        ts,
        color_level,
        level_as_str(level),
        reset,
        module_color,
        module,
        reset,
        args
    )
}

/// Converts a `Level` enum to its string representation.
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

/// Returns ANSI color code for a log level if `use_color` is true; otherwise returns an empty string.
fn color_for_level(level: Level, use_color: bool) -> &'static str {
    if !use_color {
        return "";
    }
    match level {
        Level::Error => "\x1b[31m", // Red
        Level::Warn => "\x1b[33m",  // Yellow
        Level::Info => "\x1b[32m",  // Green
        Level::Debug => "\x1b[34m", // Blue
        Level::Trace => "\x1b[35m", // Magenta
        Level::Off => "\x1b[0m",    // Reset
    }
}

/// Returns ANSI color code for a module based on its first character if `use_color` is true; otherwise returns an empty string.
fn color_for_module(module: &str, use_color: bool) -> &'static str {
    if !use_color {
        return "";
    }
    match module.chars().next().unwrap_or('a') {
        'a'..='f' => "\x1b[36m", // Cyan
        'g'..='l' => "\x1b[35m", // Magenta
        'm'..='r' => "\x1b[32m", // Green
        's'..='z' => "\x1b[33m", // Yellow
        _ => "\x1b[37m",         // White fallback
    }
}

// -------------------------------
// Macro
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
