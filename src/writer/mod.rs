//! Buffered output and writing infrastructure.
//!
//! This module is responsible for writing formatted log records to their
//! configured destination.
//!
//! The writing pipeline consists of:
//!
//! - [`Writer`] configures the output destination and color mode.
//! - [`BufferFormatter`] formats log records into an in-memory buffer.
//! - [`BufferWriter`] writes the completed buffer to the configured output.
//! - [`Buffer`] stores the formatted bytes before they are written.
//!
//! Buffering allows formatting to complete before any output is written,
//! reducing partial writes and keeping formatting independent from output.

mod buffer_formatter;
mod buffer_writer;

pub use buffer_formatter::{BufferFormatter, try_with_buf_formatter_slot};
pub use buffer_writer::{Buffer, BufferWriter};

use crate::style::ColorMode;

/// Output destination used by the logger.
#[derive(Default)]
pub enum Output {
    /// Write log records to standard output.
    #[default]
    Stdout,
    /// Write log records to standard error.
    Stderr,
    /// Append log records to the specified file.
    File(String),
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stdout => "stdout",
                Self::Stderr => "stderr",
                Self::File(_) => "file",
            }
        )
    }
}

/// High-level writer used by the logger.
///
/// Owns the configured output destination and provides buffers used
/// during log formatting.
#[derive(Debug, Default)]
pub struct Writer {
    buffer_writer: BufferWriter,
}

impl Writer {
    /// Creates a new [`WriterBuilder`].
    pub fn builder() -> WriterBuilder {
        WriterBuilder::new()
    }

    /// Returns the configured color mode.
    pub fn color_mode(&self) -> ColorMode {
        self.buffer_writer.color_mode()
    }

    /// Creates a new empty output buffer.
    pub fn buffer(&self) -> Buffer {
        self.buffer_writer.buffer()
    }

    /// Writes the buffer to the configured output.
    pub fn print_out(&self, buf: &Buffer) -> std::io::Result<()> {
        self.buffer_writer.write_buffer(buf)
    }

    /// Flushes the configured output stream.
    pub fn flush(&self) -> std::io::Result<()> {
        use std::io::Write as _;
        match self.buffer_writer.output_ref() {
            Output::Stdout => std::io::stdout().flush(),
            Output::Stderr => std::io::stderr().flush(),
            Output::File(path) => {
                let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
                file.flush()
            }
        }
    }
}

/// Builder for constructing a [`Writer`].
#[derive(Debug, Default)]
pub struct WriterBuilder {
    writer: Writer,
}

impl WriterBuilder {
    pub fn new() -> Self {
        Self {
            writer: Writer::default(),
        }
    }

    pub fn stdout(&mut self) -> &mut Self {
        self.writer.buffer_writer.set_output(Output::Stdout);
        self
    }

    pub fn stderr(&mut self) -> &mut Self {
        self.writer.buffer_writer.set_output(Output::Stderr);
        self
    }

    pub fn file(&mut self, path: impl Into<String>) -> &mut Self {
        self.writer
            .buffer_writer
            .set_output(Output::File(path.into()));
        self
    }

    pub fn color_mode(&mut self, color_mode: ColorMode) -> &mut Self {
        self.writer.buffer_writer.set_color_mode(color_mode);
        self
    }

    /// Builds the configured writer.
    ///
    /// When the color mode is [`ColorMode::Auto`], the final color mode is
    /// determined from the selected output destination. Terminal outputs
    /// enable colors, while file output disables them.
    pub fn build(self) -> Writer {
        let color = self.writer.color_mode();
        let output = self.writer.buffer_writer.output_take();

        use std::io::IsTerminal;

        let color_choice = if color == ColorMode::Auto {
            match output {
                Output::Stdout => {
                    if std::io::stdout().is_terminal() {
                        ColorMode::Always
                    } else {
                        ColorMode::Never
                    }
                }
                Output::Stderr => {
                    if std::io::stderr().is_terminal() {
                        ColorMode::Always
                    } else {
                        ColorMode::Never
                    }
                }
                Output::File(_) => ColorMode::Never,
            }
        } else {
            color
        };

        let writer = match output {
            Output::Stdout => BufferWriter::new(Output::Stdout, color_choice),
            Output::Stderr => BufferWriter::new(Output::Stderr, color_choice),
            Output::File(string) => BufferWriter::new(Output::File(string), color_choice),
        };

        Writer {
            buffer_writer: writer,
        }
    }
}
