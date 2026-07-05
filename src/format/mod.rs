//! Log record formatting.
//!
//! This module defines how log records are formatted before being written
//! to the output.
//!
//! Formatting can be configured in two ways:
//!
//! - Use the built-in formatter with [`FormatConfig`].
//! - Provide a custom formatter by implementing [`FormatCustom`].
//!
//! # Built-in Formatter
//!
//! The built-in formatter allows configuring which header fields are
//! included in the output, such as:
//!
//! - timestamp
//! - log level
//! - target
//! - module path
//!
//! # Custom Formatter
//!
//! Applications can completely customize the output format by supplying
//! a closure or type implementing [`FormatCustom`].
//!
//! # Example
//!
//! ## Built-in Formatter
//!
//! ```rust
//! use mini_logger::FormatBuilder;
//!
//! let mut builder = FormatBuilder::default();
//!
//! builder
//!     .format_default()
//!     .target(true)
//!     .module_path(false);
//! ```
//!
//! ## Custom Formatter
//!
//! ```rust
//! use std::io::Write;
//!
//! use mini_logger::FormatBuilder;
//!
//! let mut builder = FormatBuilder::default();
//!
//! builder.format_custom(|buf, record| {
//!     writeln!(
//!         buf,
//!         "[{}] {}",
//!         record.level().as_str(),
//!         record.msg()
//!     )
//! });
//! ```

mod format_config;
pub use format_config::FormatConfig;

use crate::record::RecMessage;
use crate::writer::BufferFormatter;
use std::io;

/// Trait implemented by custom log formatters.
///
/// This trait allows applications to completely customize how log
/// records are written.
pub trait FormatCustom {
    /// Formats a log record using a custom layout.
    ///
    /// Implementations are responsible for writing the complete log record
    /// to the provided buffer.
    fn format_custom_layout(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> std::io::Result<()>;
}

impl<F> FormatCustom for F
where
    F: Fn(&mut BufferFormatter, &RecMessage<'_>) -> io::Result<()>,
{
    fn format_custom_layout(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        (self)(buf_formatter, record_msg)
    }
}

/// Configured log formatter.
///
/// A formatter is either the built-in configurable formatter or a
/// user-provided custom formatter.
pub enum Format {
    Default(FormatConfig),
    Custom(Box<dyn FormatCustom + Send + Sync>),
}

impl Format {
    /// Formats a log record using the configured formatter.
    ///
    /// Calls either the built-in formatter or the user-provided custom
    /// formatter depending on the active [`Format`] variant.
    pub fn format_record(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        match self {
            Format::Default(f) => f.format_write_layout(buf_formatter, record_msg),
            Format::Custom(f) => f.format_custom_layout(buf_formatter, record_msg),
        }
    }
}

impl Default for Format {
    /// Creates the default built-in formatter.
    fn default() -> Self {
        Format::Default(FormatConfig::default())
    }
}

/// Builder for constructing a [`Format`].
///
/// Supports either the built-in formatter or a custom formatter.
/// By default, the built-in formatter is used.
/// Call [`FormatBuilder::format_custom`] to replace it with a custom formatter.
#[derive(Default)]
pub struct FormatBuilder {
    format: Format,
}

impl FormatBuilder {
    /// Selects the built-in formatter.
    ///
    /// Returns the associated [`FormatConfig`] so its options can be
    /// configured.
    pub fn format_default(&mut self) -> &mut FormatConfig {
        self.format = Format::Default(FormatConfig::default());

        match &mut self.format {
            Format::Default(cfg) => cfg,
            _ => unreachable!("Format should now always be Default"),
        }
    }

    /// Replaces the built-in formatter with a custom formatter.
    pub fn format_custom<F>(&mut self, f: F) -> &mut Self
    where
        F: FormatCustom + Send + Sync + 'static,
    {
        self.format = Format::Custom(Box::new(f));
        self
    }

    /// Builds the configured formatter.
    pub fn build(self) -> Format {
        self.format
    }
}
