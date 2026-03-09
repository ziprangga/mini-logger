use arrayvec;
use once_cell::sync::OnceCell;
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

pub static DEBUG_CONTEXT: OnceCell<Arc<DebugLogInner>> = OnceCell::new();
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

pub struct RingBuffer {
    buf: Box<[u8]>,
    write: AtomicUsize,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buf: vec![0u8; size].into_boxed_slice(),
            write: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, bytes: &[u8]) {
        let start = self.write.fetch_add(bytes.len(), Ordering::Relaxed);
        let len = self.buf.len();

        for (i, b) in bytes.iter().enumerate() {
            let idx = (start + i) % len;
            unsafe {
                let ptr = self.buf.as_ptr().add(idx) as *mut u8;
                *ptr = *b;
            }
        }
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.buf.to_vec()
    }
}

pub struct DebugLogInner {
    buffer: Option<RingBuffer>,
    console: Option<io::Stdout>,
}

impl DebugLogInner {
    pub fn write_fmt(&self, args: std::fmt::Arguments) {
        if let Some(out) = &self.console {
            let mut lock = out.lock();
            let _ = lock.write_fmt(args);
            let _ = lock.write_all(b"\n");
        }

        if let Some(buf) = &self.buffer {
            use std::fmt::Write;
            let mut tmp = arrayvec::ArrayString::<512>::new();
            let _ = write!(tmp, "{}\n", args);
            buf.push(tmp.as_bytes());
        }
    }

    pub fn get_buffer(&self) -> Option<String> {
        self.buffer
            .as_ref()
            .map(|b| String::from_utf8_lossy(&b.snapshot()).to_string())
    }

    pub fn save_buffer_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(buf) = &self.buffer {
            let mut file = File::create(path)?;
            file.write_all(&buf.snapshot())?;
        }
        Ok(())
    }
}

pub struct DebugLog {
    debug_log_inner: OnceCell<Arc<DebugLogInner>>,
}

impl DebugLog {
    pub fn init(config: Option<&[(&str, Level)]>, use_buffer: bool, use_console: bool) -> Self {
        let inner = Arc::new(DebugLogInner {
            buffer: if use_buffer {
                Some(RingBuffer::new(1024 * 1024))
            } else {
                None
            },
            console: if use_console {
                Some(io::stdout())
            } else {
                None
            },
        });

        let debug_log = DebugLog {
            debug_log_inner: OnceCell::new(),
        };
        let _ = debug_log.debug_log_inner.set(inner.clone());

        let _ = DEBUG_CONTEXT.set(inner);

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
            let map = CRATE_LEVELS.read().unwrap();
            let config_for_env: Vec<(&str, Level)> =
                map.iter().map(|(k, &v)| (k.as_str(), v)).collect();

            // Disable others only if user passed explicit config
            Self::use_config_init(&config_for_env);
        }

        debug_log
    }

    #[doc(hidden)]
    pub fn write_log(level: Level, args: std::fmt::Arguments) {
        if level.enabled() {
            let inner = DEBUG_CONTEXT
                .get()
                .expect("DEBUG_CONTEXT not initialized, need init function");
            inner.write_fmt(args);
        }
    }

    #[doc(hidden)]
    pub fn forward_to_log(_level: Level, _args: std::fmt::Arguments) {
        #[cfg(feature = "log-env-support")]
        match _level {
            Level::Error => log::error!("{}", _args),
            Level::Warn => log::warn!("{}", _args),
            Level::Info => log::info!("{}", _args),
            Level::Debug => log::debug!("{}", _args),
            Level::Trace => log::trace!("{}", _args),
            Level::Off => {}
        }
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
    fn use_config_init(config: &[(&str, Level)]) {
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

        builder.format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "[ENV_LOGGER {} {}] {}",
                record.level(),
                record.module_path().unwrap_or("<unknown>"),
                record.args()
            )
        });

        let _ = builder.try_init();
    }
}

// -------------------------------
// Debug macro
// Wrapper macro from crate log
// use it for print message to console or manipulate it
// for your need in the function you build
// debug_dev!("Starting main app in debug mode...");
// can be combined with another value or message
// let msg = some_value;
// debug-dev!("message {}", msg.to_string());
// just like "println!"
// -------------------------------
#[macro_export]
macro_rules! error_dev {
    ($($arg:tt)*) => {{
        DebugLog::write_log($crate::Level::Error, format_args!($($arg)*));
        DebugLog::forward_to_log($crate::Level::Error, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn_dev {
    ($($arg:tt)*) => {{
        DebugLog::write_log($crate::Level::Warn, format_args!($($arg)*));
        DebugLog::forward_to_log($crate::Level::Warn, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! info_dev {
    ($($arg:tt)*) => {{
        DebugLog::write_log($crate::Level::Info, format_args!($($arg)*));
        DebugLog::forward_to_log($crate::Level::Info, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {{
        DebugLog::write_log($crate::Level::Debug, format_args!($($arg)*));
        DebugLog::forward_to_log($crate::Level::Debug, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! trace_dev {
    ($($arg:tt)*) => {{
        DebugLog::write_log($crate::Level::Trace, format_args!($($arg)*));
        DebugLog::forward_to_log($crate::Level::Trace, format_args!($($arg)*));
    }};
}
