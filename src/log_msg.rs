use std::sync::atomic::{AtomicUsize, Ordering};

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(Level::Trace as usize);

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

#[derive(Clone, Debug)]
pub struct LogMessage<'a> {
    log_config: LogConfig<'a>,
    module: Option<&'a str>,
    msg: std::fmt::Arguments<'a>,
}

impl<'a> LogMessage<'a> {
    #[inline]
    pub fn builder() -> LogMessageBuilder<'a> {
        LogMessageBuilder::new()
    }

    #[inline]
    pub fn log_config(&self) -> &LogConfig<'a> {
        &self.log_config
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.log_config.level()
    }

    #[inline]
    pub fn target(&self) -> &'a str {
        self.log_config.target()
    }

    #[inline]
    pub fn module(&self) -> Option<&'a str> {
        self.module
    }

    #[inline]
    pub fn msg(&self) -> &std::fmt::Arguments<'a> {
        &self.msg
    }
}

impl Default for LogMessage<'_> {
    fn default() -> Self {
        Self {
            log_config: LogConfig::builder().build(),
            module: None,
            msg: format_args!(""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogMessageBuilder<'a> {
    log_message: LogMessage<'a>,
}

impl<'a> LogMessageBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            log_message: LogMessage::default(),
        }
    }

    #[inline]
    pub fn log_config(&mut self, log_config: LogConfig<'a>) -> &mut Self {
        self.log_message.log_config = log_config;
        self
    }

    #[inline]
    pub fn level(&mut self, level: Level) -> &mut Self {
        self.log_message.log_config.level = level;
        self
    }

    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.log_message.log_config.target = target;
        self
    }

    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.log_message.module = module;
        self
    }

    #[inline]
    pub fn msg(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.log_message.msg = msg;
        self
    }

    #[inline]
    pub fn build(&self) -> LogMessage<'a> {
        self.log_message.clone()
    }
}

impl Default for LogMessageBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::OnceLock;
static LOGGER: OnceLock<&'static dyn Logger> = OnceLock::new();

pub trait Logger: Send + Sync {
    fn enable(&self, log_config: &LogConfig) -> bool;
    fn log_msg(&self, log_message: &LogMessage);
    fn flush(&self);
}

pub fn set_logger(logger: &'static dyn Logger) -> Result<(), &'static str> {
    LOGGER.set(logger).map_err(|_| "Logger already set")
}

pub fn logger() -> Option<&'static dyn Logger> {
    LOGGER.get().copied()
}
