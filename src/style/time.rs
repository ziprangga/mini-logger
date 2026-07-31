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
/// [`TimeMode`] determines whether timestamps are included in formatted output
/// and which timezone is used when rendering them.
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
    time: SystemTime,
    precision: TimePrecision,
}

impl Timestamp {
    /// Captures the current system time using the specified mode and precision.
    pub fn now(mode: TimeMode, precision: TimePrecision) -> Self {
        Self {
            mode,
            time: SystemTime::now(),
            precision,
        }
    }

    /// Converts the timestamp into a [`chrono::Local`] datetime.
    fn datetime_local(&self) -> chrono::DateTime<chrono::Local> {
        chrono::DateTime::<chrono::Local>::from(self.time)
    }

    /// Converts the timestamp into a [`Utc`] datetime.
    fn datetime_utc(&self) -> chrono::DateTime<Utc> {
        DateTime::<Utc>::from(self.time)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self {
            mode: TimeMode::Utc,
            time: SystemTime::now(),
            precision: TimePrecision::Seconds,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            TimeMode::Off => Ok(()),
            TimeMode::Local => {
                let dt = self.datetime_local();

                match self.precision {
                    TimePrecision::Seconds => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
                    TimePrecision::Millis => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                    }
                    TimePrecision::Micros => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                    }
                    TimePrecision::Nanos => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                    }
                }
            }

            TimeMode::Utc => {
                let dt = self.datetime_utc();

                match self.precision {
                    TimePrecision::Seconds => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
                    TimePrecision::Millis => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                    }
                    TimePrecision::Micros => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                    }
                    TimePrecision::Nanos => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                    }
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
