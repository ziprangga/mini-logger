use std::sync::atomic::{AtomicUsize, Ordering};

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(LogLevel::Off as usize);

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    #[inline]
    pub fn set_level(self) {
        LOG_LEVEL.store(self as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_level() -> LogLevel {
        match LOG_LEVEL.load(Ordering::Relaxed) {
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            5 => LogLevel::Trace,
            _ => LogLevel::Off,
        }
    }

    #[inline]
    pub fn from_usize(val: usize) -> Self {
        match val {
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            5 => LogLevel::Trace,
            _ => LogLevel::Off,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(()),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Debug
    }
}

// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
// pub struct LogConfig<'a> {
//     level: LogLevel,
//     target: &'a str,
// }

// impl<'a> LogConfig<'a> {
//     #[inline]
//     pub fn builder() -> LogConfigBuilder<'a> {
//         LogConfigBuilder::new()
//     }

//     #[inline]
//     pub fn level(&self) -> LogLevel {
//         self.level
//     }

//     #[inline]
//     pub fn target(&self) -> &'a str {
//         self.target
//     }
// }

// impl Default for LogConfig<'_> {
//     fn default() -> Self {
//         Self {
//             level: LogLevel::Debug,
//             target: "",
//         }
//     }
// }

// #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
// pub struct LogConfigBuilder<'a> {
//     log_config: LogConfig<'a>,
// }

// impl<'a> LogConfigBuilder<'a> {
//     #[inline]
//     pub fn new() -> Self {
//         Self {
//             log_config: LogConfig::default(),
//         }
//     }

//     #[inline]
//     pub fn level(&mut self, level: LogLevel) -> &mut Self {
//         self.log_config.level = level;
//         self
//     }

//     #[inline]
//     pub fn target(&mut self, target: &'a str) -> &mut Self {
//         self.log_config.target = target;
//         self
//     }

//     #[inline]
//     pub fn build(&self) -> LogConfig<'a> {
//         self.log_config.clone()
//     }
// }

// impl Default for LogConfigBuilder<'_> {
//     fn default() -> Self {
//         Self::new()
//     }
// }
