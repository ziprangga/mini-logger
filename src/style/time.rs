use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TimestampPrecision {
    Seconds,
    Millis,
    Micros,
    Nanos,
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
        let duration = match self.time.duration_since(UNIX_EPOCH) {
            Ok(d) => d,
            Err(_) => return Err(fmt::Error),
        };

        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();

        match self.precision {
            TimestampPrecision::Seconds => {
                write!(f, "{secs}")
            }
            TimestampPrecision::Millis => {
                write!(f, "{secs}.{:03}", nanos / 1_000_000)
            }
            TimestampPrecision::Micros => {
                write!(f, "{secs}.{:06}", nanos / 1_000)
            }
            TimestampPrecision::Nanos => {
                write!(f, "{secs}.{:09}", nanos)
            }
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
