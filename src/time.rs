use chrono;
use std::sync::{LazyLock, Mutex};

#[derive(Copy, Clone, Debug)]
pub enum TimeMode {
    Utc,
    Local,
}

#[derive(Copy, Clone, Debug)]
pub enum TimeFormat {
    Iso8601,
    HumanReadable,
}

// Global configuration
static TIME_MODE: LazyLock<Mutex<TimeMode>> = LazyLock::new(|| Mutex::new(TimeMode::Utc));
static TIME_FORMAT: LazyLock<Mutex<TimeFormat>> = LazyLock::new(|| Mutex::new(TimeFormat::Iso8601));

pub fn set_time_mode(mode: TimeMode) {
    let mut guard = TIME_MODE.lock().unwrap_or_else(|poisoned| {
        eprintln!("TIME_MODE lock poisoned, using fallback");
        poisoned.into_inner()
    });
    *guard = mode;
}

pub fn set_time_format(format: TimeFormat) {
    let mut guard = TIME_FORMAT.lock().unwrap_or_else(|poisoned| {
        eprintln!("TIME_FORMAT lock poisoned, using fallback");
        poisoned.into_inner()
    });
    *guard = format;
}

// Updated timestamp in format_log
pub fn get_timestamp() -> String {
    let mode = *TIME_MODE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let format = *TIME_FORMAT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    match (mode, format) {
        (TimeMode::Utc, TimeFormat::Iso8601) => {
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }
        (TimeMode::Utc, TimeFormat::HumanReadable) => {
            chrono::Utc::now().format("%b %d %H:%M:%S").to_string()
        }
        (TimeMode::Local, TimeFormat::Iso8601) => chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%:z")
            .to_string(),
        (TimeMode::Local, TimeFormat::HumanReadable) => {
            chrono::Local::now().format("%b %d %H:%M:%S").to_string()
        }
    }
}
