use chrono::{DateTime, Utc};
use std::fmt;
use std::time::SystemTime;

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
    time: SystemTime,
    precision: TimestampPrecision,
}

impl Timestamp {
    /// Returns a timestamp using second precision.
    pub fn timestamp_seconds(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }

    /// Returns a timestamp using millisecond precision.
    pub fn timestamp_millis(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Millis,
        }
    }

    /// Returns a timestamp using microsecond precision.
    pub fn timestamp_micros(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Micros,
        }
    }

    /// Returns a timestamp using nanosecond precision.
    pub fn timestamp_nanos(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Nanos,
        }
    }

    /// Converts the timestamp into a UTC datetime.
    fn datetime_utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from(self.time)
    }
}

impl Default for Timestamp {
    /// Creates a timestamp using the current system time and second precision.
    fn default() -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dt = self.datetime_utc();

        match self.precision {
            TimestampPrecision::Seconds => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S")),
            TimestampPrecision::Millis => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.3f")),
            TimestampPrecision::Micros => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f")),
            TimestampPrecision::Nanos => write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f")),
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
