//! Log filtering levels and the global maximum enabled level.
//!
//! `Level` defines the available log verbosity levels used by the
//! filtering system.
//!
//! The module also maintains a global maximum enabled level in an atomic,
//! allowing logging macros to quickly determine whether a log level is
//! potentially enabled before constructing a log record.
//!
//! Levels are ordered from least to most verbose:
//!
//! `Off < Error < Warn < Info < Debug < Trace`

use std::sync::atomic::{AtomicUsize, Ordering};

static LEVEL: AtomicUsize = AtomicUsize::new(Level::Off as usize);

/// Log verbosity level used to determine whether log records are enabled.
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Level {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,

    #[default]
    Debug = 4,

    Trace = 5,
}

impl Level {
    /// Sets the global maximum enabled log level.
    #[inline]
    pub fn set_level(self) {
        LEVEL.store(self as usize, Ordering::Relaxed);
    }

    /// Returns the currently configured global level.
    #[inline]
    pub fn get_level() -> Level {
        match LEVEL.load(Ordering::Relaxed) {
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => Level::Off,
        }
    }

    /// Converts a numeric representation into a filter level.
    ///
    /// Any value outside the valid range is treated as [`Level::Off`].
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

    /// Returns the uppercase string representation of the level.
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

/// Parses a filter level from a case-insensitive string.
///
/// Also accepts `"warning"` as an alias for `"warn"`.
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
