use std::fmt::Display;
use std::io::{self, Write};

use crate::filter::Level;
use crate::format::{Formatter, RenderRecord};
use crate::record::RecMessage;
use crate::style::Color;
use crate::style::ColorMode;
use crate::writer::Buffer;

/// Built-in renderer for formatted log records.
///
/// The renderer writes enabled record metadata as a bracketed header followed
/// by the log message.
///
/// Depending on the active [`Formatter`] configuration, the header may include:
///
/// - timestamp
/// - log level
/// - log target
/// - module path
///
/// Fields that are disabled or unavailable are omitted.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRenderer;

impl RenderRecord for DefaultRenderer {
    fn render(
        &self,
        formatter: &Formatter,
        buffer: &mut Buffer,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        let fmt = FormatLayoutWriter {
            formatter,
            buffer,
            written_header: false,
        };

        fmt.write(record_msg)
    }
}

/// Internal writer for the built-in log layout.
///
/// Tracks whether any header field has already been written so the
/// formatter can correctly insert separators and avoid producing
/// empty header sections.
struct FormatLayoutWriter<'a> {
    formatter: &'a Formatter,
    buffer: &'a mut Buffer,
    written_header: bool,
}

impl FormatLayoutWriter<'_> {
    /// Writes the complete formatted log record.
    #[inline]
    pub fn write(mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        self.write_timestamp(record_msg)?;
        self.write_level(record_msg)?;
        self.write_target(record_msg)?;
        self.write_module(record_msg)?;
        self.finish_header()?;
        self.write_args(record_msg)
    }

    /// Returns whether ANSI color output is enabled.
    fn color_enabled(&self) -> bool {
        match self.formatter.style().color_mode() {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => unreachable!("ColorMode::Auto must be resolved during build"),
        }
    }

    /// Returns the color associated with the record severity level.
    fn level_color(record_msg: &RecMessage<'_>) -> crate::style::Color {
        match record_msg.level() {
            Level::Off => Color::Reset,
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Blue,
            Level::Trace => Color::Blue,
        }
    }

    /// Writes a single header field.
    ///
    /// The first header field starts the header with `[`, while subsequent
    /// fields are separated by spaces.
    fn write_header_value<T>(&mut self, record_msg: &RecMessage<'_>, value: T) -> io::Result<()>
    where
        T: Display,
    {
        let color_enabled = self.color_enabled();

        let color_head = if color_enabled {
            Color::Yellow.as_str()
        } else {
            ""
        };

        let color_reset = if color_enabled {
            Color::Reset.as_str()
        } else {
            ""
        };

        if !self.written_header {
            self.written_header = true;

            write!(self.buffer, "{}[{}", color_head, color_reset)?;
        } else {
            write!(self.buffer, " ")?;
        }

        let color_ctx = Self::level_color(record_msg);

        let color_value = if color_enabled {
            color_ctx.as_str()
        } else {
            ""
        };

        write!(self.buffer, "{}{}{}", color_value, value, color_reset)
    }

    /// Writes the timestamp header field if timestamp formatting is enabled.
    ///
    /// Timestamp generation and formatting behavior are taken from the
    /// formatter style configuration.
    fn write_timestamp(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        let style = self.formatter.style();

        let timestamp = style.time_mode().resolve(style.time_precision());

        self.write_header_value(record_msg, timestamp)
    }

    /// Writes the log level header field if enabled.
    ///
    /// The level text is colorized according to the active color mode and
    /// record severity level.
    fn write_level(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.formatter.level {
            return Ok(());
        }

        let level_str = record_msg.level().as_str();
        self.write_header_value(record_msg, format_args!("{:<5}", level_str))
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

        self.write_header_value(record_msg, target)
    }

    /// Writes the module path header field if enabled.
    ///
    /// Records without a module path do not add a header field.
    fn write_module(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        if !self.formatter.module_path {
            return Ok(());
        }
        if let Some(module) = record_msg.module() {
            self.write_header_value(record_msg, module)
        } else {
            Ok(())
        }
    }

    /// Finishes the header.
    ///
    /// Writes the closing header delimiter and message separator only if at least
    /// one header field was written.
    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header {
            let color_enabled = self.color_enabled();
            let color_tail = if color_enabled {
                Color::Yellow.as_str()
            } else {
                ""
            };
            let color_reset = if color_enabled {
                Color::Reset.as_str()
            } else {
                ""
            };

            write!(self.buffer, "{}]{} - ", color_tail, color_reset)?;
        }

        Ok(())
    }

    /// Writes the log message followed by a newline.
    fn write_args(&mut self, record_msg: &RecMessage<'_>) -> io::Result<()> {
        write!(self.buffer, "{}\n", record_msg.msg())
    }
}
