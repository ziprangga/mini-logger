use chrono::{DateTime, Utc};
use std::fmt;
use std::time::SystemTime;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimestampPrecision {
    Seconds,
    Millis,
    Micros,
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

#[derive(Copy, Clone)]
pub struct Timestamp {
    pub time: SystemTime,
    pub precision: TimestampPrecision,
}

impl Timestamp {
    pub fn timestamp_seconds(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Seconds,
        }
    }

    pub fn timestamp_millis(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Millis,
        }
    }

    pub fn timestamp_micros(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Micros,
        }
    }

    pub fn timestamp_nanos(&self) -> Self {
        Self {
            time: SystemTime::now(),
            precision: TimestampPrecision::Nanos,
        }
    }

    fn datetime_utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from(self.time)
    }
}

impl Default for Timestamp {
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
