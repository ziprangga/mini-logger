use chrono::{DateTime, Utc};
use std::fmt;
use std::time::SystemTime;

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimeMode {
    Off,

    Local,

    #[default]
    Utc,
}

impl TimeMode {
    pub fn is_enabled(self) -> bool {
        match self {
            Self::Off => false,
            Self::Local => true,
            Self::Utc => true,
        }
    }

    /// Returns the formatted timestamp.
    ///
    /// Returns an empty string when timestamps are disabled.
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

/// Timestamp used by the built-in formatter.
///
/// A timestamp stores both the captured time and the precision used
/// when formatting it.
#[derive(Copy, Clone)]
pub struct Timestamp {
    mode: TimeMode,
    time: SystemTime,
    precision: TimestampPrecision,
}

impl Timestamp {
    /// Returns a timestamp using second precision.
    pub fn timestamp_seconds(&self) -> Self {
        Self {
            mode: self.mode,
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }

    /// Returns a timestamp using millisecond precision.
    pub fn timestamp_millis(&self) -> Self {
        Self {
            mode: self.mode,
            time: SystemTime::now(),
            precision: TimestampPrecision::Millis,
        }
    }

    /// Returns a timestamp using microsecond precision.
    pub fn timestamp_micros(&self) -> Self {
        Self {
            mode: self.mode,
            time: SystemTime::now(),
            precision: TimestampPrecision::Micros,
        }
    }

    /// Returns a timestamp using nanosecond precision.
    pub fn timestamp_nanos(&self) -> Self {
        Self {
            mode: self.mode,
            time: SystemTime::now(),
            precision: TimestampPrecision::Nanos,
        }
    }

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
    /// Creates a timestamp using the current system time and second precision.
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
