use std::sync::atomic::{AtomicUsize, Ordering};

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(Level::Off as usize);

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Level {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Level {
    #[inline]
    pub fn set_level(self) {
        LOG_LEVEL.store(self as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_level() -> Level {
        match LOG_LEVEL.load(Ordering::Relaxed) {
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => Level::Off,
        }
    }

    #[inline]
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

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Off => "OFF",
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }
}

impl std::str::FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(Level::Off),
            "error" => Ok(Level::Error),
            "warn" | "warning" => Ok(Level::Warn),
            "info" => Ok(Level::Info),
            "debug" => Ok(Level::Debug),
            "trace" => Ok(Level::Trace),
            _ => Err(()),
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Level::Debug
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LogConfig<'a> {
    level: Level,
    target: &'a str,
}

impl<'a> LogConfig<'a> {
    #[inline]
    pub fn builder() -> LogConfigBuilder<'a> {
        LogConfigBuilder::new()
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }
}

impl Default for LogConfig<'_> {
    fn default() -> Self {
        Self {
            level: Level::Debug,
            target: "",
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LogConfigBuilder<'a> {
    log_config: LogConfig<'a>,
}

impl<'a> LogConfigBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            log_config: LogConfig::default(),
        }
    }

    #[inline]
    pub fn level(&mut self, level: Level) -> &mut Self {
        self.log_config.level = level;
        self
    }

    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.log_config.target = target;
        self
    }

    #[inline]
    pub fn build(&self) -> LogConfig<'a> {
        self.log_config.clone()
    }
}

impl Default for LogConfigBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
