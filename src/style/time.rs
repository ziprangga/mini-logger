//! Timestamp configuration and formatting utilities.
//!
//! This module provides types used to control timestamp generation and
//! formatting in log records.
//!
//! The available components are:
//!
//! - [`TimeMode`] controls whether timestamps are disabled, local time, or UTC.
//! - [`TimestampPrecision`] controls the fractional time precision.
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

impl TimeMode {
    /// Returns whether timestamps are enabled.
    pub fn is_enabled(self) -> bool {
        match self {
            Self::Off => false,
            Self::Local | Self::Utc => true,
        }
    }

    /// Creates a timestamp using the current system time.
    ///
    /// Returns a disabled timestamp when [`TimeMode::Off`] is selected.
    pub fn timestamp(self, precision: TimestampPrecision) -> Timestamp {
        if !self.is_enabled() {
            return self.reset();
        }

        Timestamp {
            mode: self,
            time: SystemTime::now(),
            precision,
        }
    }

    /// Creates a default UTC timestamp.
    ///
    /// This is used when timestamp output is disabled and provides a neutral
    /// timestamp value for internal handling.
    pub fn reset(self) -> Timestamp {
        Timestamp {
            mode: TimeMode::Utc,
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }
}

impl std::str::FromStr for TimeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(TimeMode::Off),
            "local" => Ok(TimeMode::Local),
            "utc" => Ok(TimeMode::Utc),
            _ => Ok(TimeMode::default()),
        }
    }
}

/// Precision used when formatting timestamps.
///
/// Controls the number of fractional seconds included in formatted output.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimestampPrecision {
    /// Whole seconds.
    Seconds,
    /// Millisecond precision.
    Millis,
    /// Microsecond precision.
    Micros,
    /// Nanosecond precision.
    Nanos,
}

impl std::str::FromStr for TimestampPrecision {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "seconds" => Ok(TimestampPrecision::Seconds),
            "millis" => Ok(TimestampPrecision::Millis),
            "micros" => Ok(TimestampPrecision::Micros),
            "nanos" => Ok(TimestampPrecision::Nanos),
            _ => Ok(TimestampPrecision::Seconds),
        }
    }
}

impl Default for TimestampPrecision {
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
/// - the requested [`TimestampPrecision`].
///
/// Formatting is performed when the timestamp is displayed.
#[derive(Copy, Clone)]
pub struct Timestamp {
    mode: TimeMode,
    time: SystemTime,
    precision: TimestampPrecision,
}

impl Timestamp {
    /// Converts the timestamp into a Local datetime.
    fn datetime_local(&self) -> chrono::DateTime<chrono::Local> {
        chrono::DateTime::<chrono::Local>::from(self.time)
    }

    /// Converts the timestamp into a UTC datetime.
    fn datetime_utc(&self) -> chrono::DateTime<Utc> {
        DateTime::<Utc>::from(self.time)
    }
}

impl Default for Timestamp {
    /// Creates a UTC timestamp using the current system time and second
    /// precision.
    fn default() -> Self {
        Self {
            mode: TimeMode::Utc,
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mode == TimeMode::Off {
            return Ok(());
        }

        match self.mode {
            TimeMode::Local => {
                let dt = self.datetime_local();

                match self.precision {
                    TimestampPrecision::Seconds => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
                    TimestampPrecision::Millis => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                    }
                    TimestampPrecision::Micros => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                    }
                    TimestampPrecision::Nanos => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                    }
                }
            }

            TimeMode::Utc => {
                let dt = self.datetime_utc();

                match self.precision {
                    TimestampPrecision::Seconds => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
                    TimestampPrecision::Millis => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f"))
                    }
                    TimestampPrecision::Micros => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"))
                    }
                    TimestampPrecision::Nanos => {
                        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f"))
                    }
                }
            }

            TimeMode::Off => Ok(()),
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
