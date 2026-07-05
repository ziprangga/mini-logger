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
pub use time::{Timestamp, TimestampPrecision};
