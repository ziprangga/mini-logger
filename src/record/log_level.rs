use std::sync::atomic::{AtomicUsize, Ordering};

static FILTER_LEVEL: AtomicUsize = AtomicUsize::new(FilterLevel::Off as usize);

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FilterLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl FilterLevel {
    #[inline]
    pub fn set_level(self) {
        FILTER_LEVEL.store(self as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_level() -> FilterLevel {
        match FILTER_LEVEL.load(Ordering::Relaxed) {
            1 => FilterLevel::Error,
            2 => FilterLevel::Warn,
            3 => FilterLevel::Info,
            4 => FilterLevel::Debug,
            5 => FilterLevel::Trace,
            _ => FilterLevel::Off,
        }
    }

    #[inline]
    pub fn from_usize(val: usize) -> Self {
        match val {
            1 => FilterLevel::Error,
            2 => FilterLevel::Warn,
            3 => FilterLevel::Info,
            4 => FilterLevel::Debug,
            5 => FilterLevel::Trace,
            _ => FilterLevel::Off,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterLevel::Off => "OFF",
            FilterLevel::Error => "ERROR",
            FilterLevel::Warn => "WARN",
            FilterLevel::Info => "INFO",
            FilterLevel::Debug => "DEBUG",
            FilterLevel::Trace => "TRACE",
        }
    }
}

impl std::str::FromStr for FilterLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(FilterLevel::Off),
            "error" => Ok(FilterLevel::Error),
            "warn" | "warning" => Ok(FilterLevel::Warn),
            "info" => Ok(FilterLevel::Info),
            "debug" => Ok(FilterLevel::Debug),
            "trace" => Ok(FilterLevel::Trace),
            _ => Err(()),
        }
    }
}

impl Default for FilterLevel {
    fn default() -> Self {
        FilterLevel::Debug
    }
}
