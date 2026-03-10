use chrono;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

#[cfg(feature = "log-env-support")]
pub use env_logger;
#[cfg(feature = "log-env-support")]
pub use log;

pub static DEBUG_CONTEXT: LazyLock<Mutex<Option<Arc<DebugLogInner>>>> =
    LazyLock::new(|| Mutex::new(None));
pub static LOG_LEVEL: AtomicUsize = AtomicUsize::new(Level::Trace as usize);
pub static CRATE_LEVELS: LazyLock<RwLock<HashMap<String, Level>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn enabled(self) -> bool {
        let module_path = module_path!();
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

#[derive(Clone)]
pub struct DebugLogInner {
    buffer: Option<Arc<Mutex<Vec<u8>>>>,
    console: Option<Arc<Mutex<io::Stdout>>>,
}

impl DebugLogInner {
    pub fn push_log(&self, args: std::fmt::Arguments) {
        if let Some(out) = &self.console {
            let mut lock = out.lock().unwrap();
            let _ = writeln!(lock, "{}", args);
        }

        if let Some(buf) = &self.buffer {
            let mut buf = buf.lock().unwrap();
            use std::fmt::Write;
            let _ = write!(BufWriter::writer(&mut *buf), "{}\n", args);
        }
    }

    pub fn get_buffer(&self) -> Option<String> {
        self.buffer.as_ref().map(|buf| {
            let buf = buf.lock().unwrap();
            String::from_utf8_lossy(&buf).to_string()
        })
    }

    pub fn save_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(buf) = &self.buffer {
            let buf = buf.lock().unwrap();
            let mut file = File::create(path)?;
            file.write_all(&buf)?;
        }
        Ok(())
    }
}

pub struct DebugLog {
    pub inner: Arc<DebugLogInner>,
}

impl DebugLog {
    pub fn init(config: Option<&[(&str, Level)]>, use_buffer: bool, use_console: bool) -> Self {
        let inner = Arc::new(DebugLogInner {
            buffer: if use_buffer {
                Some(Arc::new(Mutex::new(Vec::new())))
            } else {
                None
            },
            console: if use_console {
                Some(Arc::new(Mutex::new(io::stdout())))
            } else {
                None
            },
        });

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
        }

        #[cfg(feature = "log-env-support")]
        {
            // Convert internal CRATE_LEVELS to &[(&str, Level)] for env_logger
            if use_buffer || use_console {
                let map = CRATE_LEVELS.read().unwrap();
                let config_for_env: Vec<(&str, Level)> =
                    map.iter().map(|(k, &v)| (k.as_str(), v)).collect();
                // If either buffer or console is enabled, forward to EnvForwarder
                Self::use_env_logger_init(&inner, &config_for_env);
            }
        }

        DebugLog { inner }
    }

    pub fn get_debug_buffer(&self) -> String {
        self.inner.get_buffer().unwrap_or_default()
    }

    pub fn save_debug_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
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

    #[cfg(feature = "log-env-support")]
    fn use_env_logger_init(inner: &Arc<DebugLogInner>, config: &[(&str, Level)]) {
        use env_logger;
        use log;
        let mut builder = env_logger::Builder::new();
        builder.filter(None, log::LevelFilter::Off);
        for &(crate_name, level) in config {
            let parsed_level = match level {
                Level::Trace => log::LevelFilter::Trace,
                Level::Debug => log::LevelFilter::Debug,
                Level::Info => log::LevelFilter::Info,
                Level::Warn => log::LevelFilter::Warn,
                Level::Error => log::LevelFilter::Error,
                Level::Off => log::LevelFilter::Off,
            };

            builder.filter(Some(crate_name), parsed_level);
        }

        builder.target(env_logger::Target::Pipe(Box::new(EnvForwarder {
            inner: inner.clone(),
        })));

        let _ = builder.try_init();
    }
}

#[cfg(feature = "log-env-support")]
struct EnvForwarder {
    inner: Arc<DebugLogInner>,
}

#[cfg(feature = "log-env-support")]
impl std::io::Write for EnvForwarder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.inner.push_log(format_args!("{}", s));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// -------------------------------
// Helper
// -------------------------------

#[doc(hidden)]
pub fn write_log(module: &str, level: Level, args: std::fmt::Arguments) {
    if level.enabled() {
        let maybe_inner = DEBUG_CONTEXT.lock().unwrap().as_ref().cloned();
        if let Some(inner) = maybe_inner {
            let msg = format_log(module, level, args);
            let msg_arg = format_args!("{}", msg);
            inner.push_log(msg_arg);
        } else {
            #[cfg(feature = "log-env-support")]
            {
                // fallback: just print to stdout
                let msg = format_log(module, level, args);
                let msg_arg = format_args!("{}", msg);
                println!("{}", msg_arg);
            }

            #[cfg(not(feature = "log-env-support"))]
            {
                // original behavior: panic
                panic!("DEBUG_CONTEXT not initialized");
            }
        }
    }
}

fn format_log(module: &str, level: Level, args: std::fmt::Arguments) -> String {
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let color_level = color_for_level(level);
    let module_color = color_for_module(module);
    let reset = "\x1b[0m";
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

fn color_for_level(level: Level) -> &'static str {
    match level {
        Level::Error => "\x1b[31m", // Red
        Level::Warn => "\x1b[33m",  // Yellow
        Level::Info => "\x1b[32m",  // Green
        Level::Debug => "\x1b[34m", // Blue
        Level::Trace => "\x1b[35m", // Magenta
        Level::Off => "\x1b[0m",    // Reset
    }
}

fn color_for_module(module: &str) -> &'static str {
    // Example: simple hash to pick a color
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
