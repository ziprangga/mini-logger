//! Core logger runtime and initialization system.
//!
//! This module implements a global logging pipeline with filtering,
//! formatting, and output management. It is designed to be initialized once
//! and then used globally throughout the application.
//!
//! # Overview
//!
//! The logger system is composed of three main components:
//!
//! - **Filter system** – decides whether a log record should be processed
//! - **Format system** – converts log records into text output
//! - **Writer system** – handles final output destinations (stdout, stderr, files)
//!
//! These components are assembled through the [`Builder`] type and stored
//! inside a global singleton [`Logger`] instance.
//!
//! # Runtime Flow
//!
//! A log record goes through the following pipeline:
//!
//! 1. A [`RecordMsg`] is created by the logging macro or user code.
//! 2. The message is passed to [`Logger::rec_msg`].
//! 3. The filter system evaluates whether the record is allowed.
//! 4. If accepted, the record is formatted into a thread-local buffer.
//! 5. The buffer is written using the configured writer.
//! 6. The buffer is cleared for reuse.
//!
//! If thread-local storage is unavailable (e.g. during shutdown),
//! a temporary buffer is created as fallback.
//!
//! # Global Logger
//!
//! The logger is stored in a global [`OnceLock`] instance:
//!
//! ```text
//! static LOGGER: OnceLock<Logger>
//! ```
//!
//! It can only be initialized once. Any subsequent initialization attempt
//! will fail.
//!
//! Access is performed through [`Logger::get`].
//!
//! # Initialization
//!
//! The logger is typically initialized using [`init`] or [`Builder::init`].
//!
//! Example:
//!
//! ```rust
//! Builder::new()
//!     .env_default()
//!     .output_stdout()
//!     .try_init()
//! ```
//!
//! During initialization:
//!
//! - Environment variables may configure filters (e.g. `RUST_LOG`)
//! - Output destinations are configured
//! - Formatting rules are applied
//! - Panic hook may be installed (if enabled)
//!
//! # Filtering
//!
//! Filtering is applied before any formatting occurs.
//! If a record does not match the filter rules, it is immediately discarded.
//!
//! See [`Filter`] and [`FilterBuilder`] for configuration details.
//!
//! # Formatting
//!
//! Formatting is handled by [`Formatter`].
//!
//! The built-in renderer supports configurable metadata fields such as
//! timestamps, log levels, targets, and module paths.
//!
//! Applications can also provide a custom renderer by implementing
//! [`RenderRecord`] or by supplying a compatible closure.
//!
//! # Thread-local buffering
//!
//! To improve performance, each thread stores a reusable [`Buffer`].
//! This avoids repeated allocations during logging.
//!
//! If thread-local storage is unavailable, a temporary buffer is allocated
//! for the current log operation.
//!
//! # Runtime control (optional feature)
//!
//! When the `runtime-control` feature is enabled, logging can be dynamically
//! enabled or disabled at runtime using [`Logger::enable`] and [`Logger::disable`].
//!
//! # Panic integration (optional feature)
//!
//! When the `panic-hook` feature is enabled, a panic hook is installed.
//! Panics are converted into log records and flushed immediately before termination.

use crate::filter::{Filter, FilterBuilder, Level};
use crate::format::{Formatter, FormatterBuilder, RenderRecord};
use crate::record::RecordMsg;
use crate::style::{ColorMode, TimeMode, TimePrecision};
use crate::writer::{Buffer, Writer, WriterBuilder, try_with_buffer_slot};

use std::sync::OnceLock;

#[cfg(feature = "runtime-control")]
use std::sync::atomic::{AtomicBool, Ordering};

static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Sets the global logger instance.
/// Returns an error if already initialized.
#[cfg(feature = "log-control")]
fn set_logger(logger: Logger) -> Result<(), &'static str> {
    LOGGER.set(logger).map_err(|_| "Logger already initialized")
}

#[cfg(not(feature = "log-control"))]
fn set_logger(_logger: Logger) -> Result<(), &'static str> {
    Ok(())
}

/// Returns the global logger instance if it exists.
fn get_logger() -> Option<&'static Logger> {
    LOGGER.get()
}

/// Internal initialization helper used by [`init`].
fn try_init() -> Result<(), &'static str> {
    Builder::new()
        .env_default()
        .trigger_panic_to_output()
        .output_stdout()
        .try_init()
}

/// Initializes the logger with default configuration.
///
/// Uses environment variable `RUST_LOG` and outputs to stdout.
///
/// Panics if logger is already initialized.
pub fn init() {
    try_init().expect("env_logger::init should not be called after logger initialized");
}

/// Builder for constructing a [`Logger`].
///
/// This type provides a fluent API for configuring:
///
/// - filters
/// - output destinations
/// - formatting rules
///
/// # Example
///
/// ```rust
/// let logger = Builder::new()
///     .env_default()
///     .output_stdout()
///     .build();
/// ```
#[derive(Default)]
pub struct Builder {
    filter: FilterBuilder,
    writer: WriterBuilder,
    formatter: FormatterBuilder,
}

impl Builder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures filtering from the `RUST_LOG` environment variable.
    pub fn env_default(mut self) -> Self {
        self.filter.filter_env("RUST_LOG");
        self
    }

    /// Uses a custom environment variable for filtering.
    pub fn from_env(mut self, var_name: &str) -> Self {
        self.filter.filter_env(var_name);
        self
    }

    /// Adds a filter rule for a target.
    pub fn filter(mut self, target: Option<&str>, level: Level) -> Self {
        self.filter.filter_target(target, level);
        self
    }

    /// Writes logs to stdout.
    pub fn output_stdout(mut self) -> Self {
        self.writer.stdout();
        self
    }

    /// Writes logs to stderr.
    pub fn output_stderr(mut self) -> Self {
        self.writer.stderr();
        self
    }

    /// Writes logs to a file.
    pub fn output_file(mut self, path: impl Into<String>) -> Self {
        self.writer.file(path);
        self
    }

    /// Sets the color mode used during formatting.
    pub fn color_mode(mut self, color_mode: ColorMode) -> Self {
        self.formatter.color_mode(color_mode);
        self
    }

    /// Sets the timestamp mode used during formatting.
    pub fn time_mode(mut self, time_mode: TimeMode) -> Self {
        self.formatter.time_mode(time_mode);
        self
    }

    /// Sets the timestamp precision used during formatting.
    pub fn time_precision(mut self, tp: TimePrecision) -> Self {
        self.formatter.time_precision(tp);
        self
    }

    /// Enables or disables writing the log level.
    pub fn format_level(mut self, write: bool) -> Self {
        self.formatter.level(write);
        self
    }

    /// Enables or disables writing the log target.
    pub fn format_target(mut self, write: bool) -> Self {
        self.formatter.target(write);
        self
    }

    /// Enables or disables writing the log path.
    pub fn format_module_path(mut self, write: bool) -> Self {
        self.formatter.module_path(write);
        self
    }

    /// Configures a custom record renderer.
    ///
    /// The supplied renderer replaces the built-in renderer.
    pub fn format_with<F>(mut self, format: F) -> Self
    where
        F: RenderRecord + Send + Sync + 'static,
    {
        self.formatter.format_with(format);
        self
    }

    /// Installs panic hook that redirects panics into logger.
    pub fn trigger_panic_to_output(self) -> Self {
        trigger_panic();
        self
    }

    /// Builds and installs the logger globally.
    pub fn try_init(self) -> Result<(), &'static str> {
        let logger = self.build();

        let max_level = logger.get_max_level();
        let result = set_logger(logger);

        if result.is_ok() {
            Level::set_level(max_level);
        }

        result
    }

    /// Builds and installs the logger globally.
    ///
    /// Panics if a global logger has already been installed.
    pub fn init(self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    /// Builds the logger without installing it globally.
    pub fn build(self) -> Logger {
        let writer = self.writer.build();
        let formatter = self.formatter.build(writer.output());
        Logger {
            writer,
            filter: self.filter.build(),
            formatter,
            #[cfg(feature = "runtime-control")]
            active: AtomicBool::new(true),
        }
    }
}

/// Core logger instance.
///
/// Holds the filter, formatter, and writer components used during logging.
pub struct Logger {
    filter: Filter,
    writer: Writer,
    formatter: Formatter,
    #[cfg(feature = "runtime-control")]
    active: AtomicBool,
}

impl Logger {
    /// Returns the global logger instance.
    pub fn get() -> Option<&'static Self> {
        get_logger()
    }

    /// Enables logging at runtime.
    #[cfg(feature = "runtime-control")]
    pub fn enable(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    /// Disable logging at runtime.
    #[cfg(feature = "runtime-control")]
    pub fn disable(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Checks if logging is enabled at runtime.
    #[cfg(feature = "runtime-control")]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Returns the maximum enabled log level.
    pub fn get_max_level(&self) -> Level {
        self.filter.max_level()
    }

    /// Checks whether a log record passes filtering.
    pub fn matches(&self, record_msg: &RecordMsg<'_>) -> bool {
        self.filter.matches(record_msg)
    }

    /// Processes a log record.
    ///
    /// This performs:
    /// filtering → formatting → writing → buffer clearing
    pub fn record(&self, record_msg: &RecordMsg<'_>) {
        #[cfg(feature = "runtime-control")]
        if !self.is_active() {
            return;
        }

        if !self.matches(record_msg) {
            return;
        }

        let write_and_flush = |buffer: &mut Buffer| {
            let _ = self
                .formatter
                .render_record(buffer, record_msg)
                .and_then(|_| self.writer.write_buffer(buffer));

            buffer.clear();
        };

        //Use thread-local buffer
        let printed = try_with_buffer_slot(|slot| match slot {
            Some(buffer) => {
                write_and_flush(buffer);
            }
            None => {
                let mut buffer = Buffer::new();
                write_and_flush(&mut buffer);
                *slot = Some(buffer);
            }
        })
        .is_some();

        // Fallback if thread-local unavailable (thread shutting down)
        if !printed {
            let mut buffer = Buffer::new();
            write_and_flush(&mut buffer);
        }
    }

    /// Flushes the configured output destination.
    pub fn flush(&self) {
        let _ = self.writer.flush();
    }
}

// ================
// Panic trigger
// ================
/// Installs panic hook that redirects panics into logger output.
///
/// Only active when `panic-hook` feature is enabled.
#[cfg(feature = "panic-hook")]
fn trigger_panic() {
    std::panic::set_hook(Box::new(move |info| {
        if let Some(logger) = crate::Logger::get() {
            let (file, _line) = match info.location() {
                Some(loc) => (loc.file(), loc.line()),
                None => ("unknown", 0),
            };

            let msg = format_args!("panic: {}", info);

            let mut builder = RecordMsg::builder();

            builder
                .level(Level::Debug)
                .target(file)
                .module(Some(file))
                .msg(msg);

            logger.rec_msg(&builder.build());
            logger.flush();
        }
    }))
}

/// No-op panic hook when feature is disabled.
#[cfg(not(feature = "panic-hook"))]
fn trigger_panic() {}
