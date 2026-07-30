//! Styling utilities for log output.
//!
//! This module provides reusable types for formatting log records,
//! including ANSI color support and timestamp formatting.
//!
//! The available components are:
//! - [`Color`] and [`ColorMode`] for terminal colors.
//! - [`Timestamp`] and [`TimestampPrecision`] for timestamp formatting.

mod color;
mod time;

pub use color::{Color, ColorMode};
pub use time::{TimeMode, Timestamp, TimestampPrecision};

#[derive(Debug, Default, Clone)]
pub struct Style {
    color_mode: ColorMode,
    time_mode: TimeMode,
}

impl Style {
    pub fn new(cm: ColorMode, tm: TimeMode) -> Self {
        Self {
            color_mode: cm,
            time_mode: tm,
        }
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn time_mode(&self) -> TimeMode {
        self.time_mode
    }

    pub fn set_color_mode(&mut self, cm: ColorMode) -> &mut Self {
        self.color_mode = cm;
        self
    }

    pub fn set_time_mode(&mut self, tm: TimeMode) -> &mut Self {
        self.time_mode = tm;
        self
    }
}
