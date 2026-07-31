//! Styling utilities for log output.
//!
//! This module provides configuration types used to control how log records
//! are rendered before being written by the logger.
//!
//! The available components are:
//!
//! - [`Color`] and [`ColorMode`] control ANSI color rendering.
//! - [`TimeMode`], [`Timestamp`], and [`TimePrecision`] control timestamp
//!   generation and formatting.
//! - [`Style`] groups output appearance settings used during record
//!   formatting.
//!..

mod color;
mod time;

pub use color::{Color, ColorMode};
pub use time::{TimeMode, TimePrecision, Timestamp};

/// Formatting style configuration.
///
/// [`Style`] stores output presentation settings that affect how formatted
/// records are rendered.
///
/// It does not perform formatting itself. The formatter reads these settings
/// while generating the final log output.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Style {
    color_mode: ColorMode,
    time_mode: TimeMode,
    time_precision: TimePrecision,
}

impl Style {
    /// Creates a style configuration with custom color and time modes.
    pub fn new(cm: ColorMode, tm: TimeMode, p: TimePrecision) -> Self {
        Self {
            color_mode: cm,
            time_mode: tm,
            time_precision: p,
        }
    }

    /// Returns the configured color mode.
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Returns the configured time mode.
    pub fn time_mode(&self) -> TimeMode {
        self.time_mode
    }

    pub fn time_precision(&self) -> TimePrecision {
        self.time_precision
    }

    /// Sets the color mode.
    pub fn set_color_mode(&mut self, cm: ColorMode) -> &mut Self {
        self.color_mode = cm;
        self
    }

    /// Sets the time mode.
    pub fn set_time_mode(&mut self, tm: TimeMode) -> &mut Self {
        self.time_mode = tm;
        self
    }

    pub fn set_time_precision(&mut self, tp: TimePrecision) -> &mut Self {
        self.time_precision = tp;
        self
    }
}
