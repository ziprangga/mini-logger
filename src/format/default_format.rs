use std::fmt::Display;
use std::io::{self, Write};

use crate::filter::Level;
// use crate::format::FormatCustom;
use crate::record::RecMessage;
use crate::style::TimestampPrecision;
use crate::writer::Writer;

/// Configuration for the built-in log formatter.
///
/// Each option controls whether a particular header field is included
/// when formatting log records.
pub struct DefaultFormat {
    timestamp: Option<TimestampPrecision>,
    level: bool,
    target: bool,
    module_path: bool,
}

impl DefaultFormat {
    /// Enables, disables, or configures timestamp output.
    pub fn timestamp(&mut self, timestamp: Option<TimestampPrecision>) -> &mut Self {
        self.timestamp = timestamp;
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

    /// Formats a log record using the configured layout.
    ///
    /// Internally constructs a [`FormatLayoutWriter`] to write the
    /// configured header fields and message.
    pub fn format_write_layout(
        &self,
        writer: &mut Writer,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        let fmt = FormatLayoutWriter {
            format_config: self,
            writer: writer,
            written_header: false,
        };

        fmt.write(record_msg)
    }
}

impl Default for DefaultFormat {
    /// Creates the default formatting configuration.
    ///
    /// By default:
    /// - timestamp is enabled
    /// - log level is enabled
    /// - target is disabled
    /// - module path is enabled
    fn default() -> Self {
        Self {
            timestamp: Some(TimestampPrecision::default()),
            level: true,
            target: false,
            module_path: true,
        }
    }
}

// /// Built-in Formatter implementation.
// impl FormatRender for DefaultFormat {
//     fn format(&self, writer: &mut Writer, record_msg: &RecMessage<'_>) -> std::io::Result<()> {
//         let fmt = FormatLayoutWriter {
//             format_config: self,
//             writer,
//             written_header: false,
//         };

//         fmt.write(record_msg)
//     }
// }

/// Writes the built-in log layout.
///
/// The writer keeps track of whether any header fields have been written
/// so separators and brackets are emitted correctly.
struct FormatLayoutWriter<'a> {
    format_config: &'a DefaultFormat,
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

    /// Writes the timestamp if enabled.
    ///
    /// Returns immediately when timestamp output is disabled.
    fn write_timestamp(&mut self) -> io::Result<()> {
        // {
        //     use self::TimestampPrecision::{Micros, Millis, Nanos, Seconds};
        //     let ts = match self.format_config.timestamp {
        //         None => return Ok(()),
        //         Some(Seconds) => self.writer.style().timestamp().timestamp_seconds(),
        //         Some(Millis) => self.writer.style().timestamp().timestamp_millis(),
        //         Some(Micros) => self.writer.style().timestamp().timestamp_micros(),
        //         Some(Nanos) => self.writer.style().timestamp().timestamp_nanos(),
        //     };

        //     self.write_header_value(ts)
        // }
        let precision = match self.format_config.timestamp {
            Some(precision) => precision,
            None => return Ok(()),
        };

        let timestamp = self.writer.style().time_mode().timestamp(precision);

        self.write_header_value(timestamp)
    }

    /// Writes the log level if enabled.
    ///
    /// The level is colorized according to its severity.
    fn write_level(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.format_config.level {
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

    /// Writes the log target if enabled.
    ///
    /// Empty targets are skipped.
    fn write_target(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.format_config.target {
            return Ok(());
        }

        let target = record_msg.target();
        if target.is_empty() {
            return Ok(());
        }

        self.write_header_value(target)
    }

    /// Writes the module path if enabled.
    ///
    /// Records without a module path are skipped.
    fn write_module(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.format_config.module_path {
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
