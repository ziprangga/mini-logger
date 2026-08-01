//! Timestamp configuration and formatting utilities.
//!
//! This module provides types used to control timestamp generation and
//! formatting in log records.
//!
//! The available components are:
//!
//! - [`TimeMode`] controls whether timestamps are disabled, local time, or UTC.
//! - [`TimePrecision`] controls the fractional time precision.
//! - [`Timestamp`] stores a captured system time together with its formatting
//!   configuration.
//!..

use chrono::{DateTime, Utc};
use std::fmt;
use std::time::SystemTime;

/// Controls timestamp generation and timezone selection.
///
/// [`TimeMode`] controls timestamp generation and timezone selection.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimeMode {
    /// Disable timestamps.
    Off,

    /// Use the local system timezone.
    Local,

    /// Use UTC timezone.
    #[default]
    Utc,
}

impl TimeMode {
    /// Resolves the time mode into a captured timestamp.
    ///
    /// [`TimeMode::Off`] creates an empty timestamp that renders nothing.
    /// Other modes capture the current system time with the selected
    /// precision.
    pub fn resolve(self, tp: TimePrecision) -> Timestamp {
        match self {
            Self::Off => Timestamp {
                mode: Self::Off,
                time: None,
                precision: None,
            },

            Self::Local => Timestamp::now(Self::Local, tp),

            Self::Utc => Timestamp::now(Self::Utc, tp),
        }
    }
}

impl std::str::FromStr for TimeMode {
    type Err = ();

    /// Parses a time mode from a string.
    ///
    /// Accepted values are:
    ///
    /// - `"off"`
    /// - `"local"`
    /// - `"utc"`
    ///
    /// Parsing is case-insensitive.
    ///
    /// Returns an error if the value is not recognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(TimeMode::Off),
            "local" => Ok(TimeMode::Local),
            "utc" => Ok(TimeMode::Utc),
            _ => Err(()),
        }
    }
}

/// Precision used when formatting timestamps.
///
/// Controls the number of fractional seconds included in formatted output.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimePrecision {
    /// Whole seconds.
    Seconds,
    /// Millisecond precision.
    Millis,
    /// Microsecond precision.
    Micros,
    /// Nanosecond precision.
    Nanos,
}

impl std::str::FromStr for TimePrecision {
    type Err = ();

    /// Parses a timestamp precision from a string.
    ///
    /// Accepted values are:
    ///
    /// - `"seconds"`
    /// - `"millis"`
    /// - `"micros"`
    /// - `"nanos"`
    ///
    /// Parsing is case-insensitive.
    ///
    /// Unknown values resolve to [`TimePrecision::Seconds`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "seconds" => Ok(TimePrecision::Seconds),
            "millis" => Ok(TimePrecision::Millis),
            "micros" => Ok(TimePrecision::Micros),
            "nanos" => Ok(TimePrecision::Nanos),
            _ => Ok(TimePrecision::Seconds),
        }
    }
}

impl Default for TimePrecision {
    fn default() -> Self {
        Self::Seconds
    }
}

/// Captured timestamp used by the built-in formatter.
///
/// A timestamp stores:
///
/// - the selected [`TimeMode`],
/// - the captured [`SystemTime`],
/// - the requested [`TimePrecision`].
///
/// Formatting is performed when the timestamp is displayed.
#[derive(Copy, Clone)]
pub struct Timestamp {
    mode: TimeMode,
    time: Option<SystemTime>,
    precision: Option<TimePrecision>,
}

impl Timestamp {
    /// Captures the current system time using the specified mode and precision.
    pub fn now(mode: TimeMode, precision: TimePrecision) -> Self {
        Self {
            mode,
            time: Some(SystemTime::now()),
            precision: Some(precision),
        }
    }

    /// Converts the timestamp into a [`chrono::Local`] datetime.
    fn datetime_local(&self) -> Option<chrono::DateTime<chrono::Local>> {
        self.time.map(chrono::DateTime::<chrono::Local>::from)
    }

    /// Converts the timestamp into a [`chrono::Utc`] datetime.
    fn datetime_utc(&self) -> Option<chrono::DateTime<Utc>> {
        self.time.map(DateTime::<Utc>::from)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self {
            mode: TimeMode::Utc,
            time: Some(SystemTime::now()),
            precision: Some(TimePrecision::Seconds),
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            TimeMode::Off => Ok(()),

            TimeMode::Local => {
                if let Some(dt) = self.datetime_local() {
                    match self.precision {
                        Some(TimePrecision::Seconds) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S"))
                        }
                        Some(TimePrecision::Millis) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                        }
                        Some(TimePrecision::Micros) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                        }
                        Some(TimePrecision::Nanos) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                        }
                        None => Ok(()),
                    }
                } else {
                    Ok(())
                }
            }

            TimeMode::Utc => {
                if let Some(dt) = self.datetime_utc() {
                    match self.precision {
                        Some(TimePrecision::Seconds) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S"))
                        }
                        Some(TimePrecision::Millis) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                        }
                        Some(TimePrecision::Micros) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                        }
                        Some(TimePrecision::Nanos) => {
                            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                        }
                        None => Ok(()),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
