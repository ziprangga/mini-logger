use std::fmt::Display;
use std::io::{self, Write};

use crate::filter::Level;
use crate::record::RecMessage;
use crate::style::TimestampPrecision;
use crate::writer::Writer;

/// Configuration for the built-in log formatter.
///
/// Each option controls whether a specific header field is included
/// when formatting log records.
///
/// The built-in formatter can write:
///
/// - timestamp with configurable precision
/// - log level
/// - log target
/// - module path
///
/// Header fields are written inside a single bracketed prefix.
/// Fields that are disabled or unavailable are skipped.
pub struct DefaultFormat {
    timestamp: bool,
    level: bool,
    target: bool,
    module_path: bool,
}

impl DefaultFormat {
    /// Enables, disables, or configures timestamp output.
    pub fn timestamp(&mut self, write: bool) -> &mut Self {
        self.timestamp = write;
        self
    }

    /// Enables or disables writing the log level.
    pub fn level(&mut self, write: bool) -> &mut Self {
        self.level = write;
        self
    }

    /// Enables or disables writing the log target.
    pub fn target(&mut self, write: bool) -> &mut Self {
        self.target = write;
        self
    }

    /// Enables or disables writing the module path.
    pub fn module_path(&mut self, write: bool) -> &mut Self {
        self.module_path = write;
        self
    }

    /// Formats a log record using the configured built-in layout.
    ///
    /// The formatter writes enabled header fields followed by the
    /// log message. Header fields are grouped into a single prefix
    /// and omitted fields do not produce empty separators.
    pub fn format_write_layout(
        &self,
        writer: &mut Writer,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        let fmt = FormatLayoutWriter {
            formatter: self,
            writer: writer,
            written_header: false,
        };

        fmt.write(record_msg)
    }
}

impl Default for DefaultFormat {
    /// Creates the default formatting configuration.
    ///
    /// Default settings:
    ///
    /// - timestamp enabled
    /// - log level enabled
    /// - target disabled
    /// - module path enabled
    fn default() -> Self {
        Self {
            timestamp: true,
            level: true,
            target: false,
            module_path: true,
        }
    }
}

/// Internal writer for the built-in log layout.
///
/// Tracks whether any header field has already been written so the
/// formatter can correctly insert separators and avoid producing
/// empty header sections.
struct FormatLayoutWriter<'a> {
    formatter: &'a DefaultFormat,
    writer: &'a mut Writer,
    written_header: bool,
}

impl FormatLayoutWriter<'_> {
    /// Writes the complete formatted log record.
    #[inline]
    pub fn write(mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        self.write_timestamp()?;
        self.write_level(record_msg)?;
        self.write_target(record_msg)?;
        self.write_module(record_msg)?;
        self.finish_header()?;
        self.write_args(record_msg)
    }

    /// Writes a single header field.
    ///
    /// The first header field starts the header with `[`, while subsequent
    /// fields are separated by spaces.
    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header {
            self.written_header = true;
            write!(self.writer, "[{value}")?;
        } else {
            write!(self.writer, " {value}")?;
        }

        Ok(())
    }

    /// Writes the timestamp header field if timestamp formatting is enabled.
    ///
    /// The timestamp mode and formatting behavior are taken from the writer
    /// style configuration.
    fn write_timestamp(&mut self) -> io::Result<()> {
        if !self.formatter.timestamp {
            return Ok(());
        }

        let timestamp = self
            .writer
            .style()
            .time_mode()
            .timestamp(TimestampPrecision::default());

        self.write_header_value(timestamp)
    }

    /// Writes the log level header field if enabled.
    ///
    /// The level text is colorized according to the current writer
    /// color mode and severity level.
    fn write_level(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.formatter.level {
            return Ok(());
        }

        use crate::style::Color;
        let color = match record_msg.level() {
            Level::Off => Color::Reset,
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Blue,
            Level::Trace => Color::Blue,
        };
        let level_str = record_msg.level().as_str();

        self.write_header_value(format_args!(
            "{}{:<5}{}",
            self.writer.style().color_mode().color(color),
            level_str,
            self.writer.style().color_mode().reset()
        ))
    }

    /// Writes the log target header field if enabled.
    ///
    /// Empty targets are ignored.
    fn write_target(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.formatter.target {
            return Ok(());
        }

        let target = record_msg.target();
        if target.is_empty() {
            return Ok(());
        }

        self.write_header_value(target)
    }

    /// Writes the module path header field if enabled.
    ///
    /// Records without a module path do not add a header field.
    fn write_module(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.formatter.module_path {
            return Ok(());
        }
        if let Some(module) = record_msg.module() {
            self.write_header_value(module)
        } else {
            Ok(())
        }
    }

    /// Finishes the header.
    ///
    /// Writes the closing `] ` only if at least one header field was written.
    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header {
            write!(self.writer, "] ")?;
        }
        Ok(())
    }

    /// Writes the log message followed by a newline.
    fn write_args(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        write!(self.writer, "{}\n", record_msg.msg())
    }
}
