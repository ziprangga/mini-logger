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
//! - [`Style`] stores resolved formatting settings used during record
//!   formatting.
//! - [`StyleBuilder`] configures style options before building a [`Style`].
//!..

mod color;
mod time;

pub use color::{Color, ColorMode};
pub use time::{TimeMode, TimePrecision, Timestamp};

use crate::writer::Output;

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
    /// Creates a [`StyleBuilder`] with default configuration values.
    pub fn builder() -> StyleBuilder {
        StyleBuilder::new()
    }

    /// Returns the configured color mode.
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Returns the configured time mode.
    pub fn time_mode(&self) -> TimeMode {
        self.time_mode
    }

    /// Returns the configured timestamp precision.
    pub fn time_precision(&self) -> TimePrecision {
        self.time_precision
    }
}

/// Builder for constructing a [`Style`] configuration.
///
/// [`StyleBuilder`] allows configuring output styling options before creating
/// an immutable [`Style`] instance.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct StyleBuilder {
    color_mode: ColorMode,
    time_mode: TimeMode,
    time_precision: TimePrecision,
}

impl StyleBuilder {
    /// Creates a [`StyleBuilder`] with default configuration values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color mode.
    pub fn color_mode(&mut self, cm: ColorMode) -> &mut Self {
        self.color_mode = cm;
        self
    }

    /// Sets the time mode.
    pub fn time_mode(&mut self, tm: TimeMode) -> &mut Self {
        self.time_mode = tm;
        self
    }

    /// Sets the timestamp precision.
    pub fn time_precision(&mut self, tp: TimePrecision) -> &mut Self {
        self.time_precision = tp;
        self
    }

    /// Builds the final [`Style`] configuration.
    ///
    /// Resolves automatic color selection using the provided output destination
    /// before creating the immutable style configuration.
    pub fn build(self, output: &Output) -> Style {
        Style {
            color_mode: self.color_mode.resolve(output),
            time_mode: self.time_mode,
            time_precision: self.time_precision,
        }
    }
}
