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

mod buffer;
mod output;
pub use buffer::Buffer;
pub use output::Output;

use crate::style::{ColorMode, Style, TimeMode};
use std::cell::RefCell;

thread_local! {
    static WRITER: RefCell<Option<Writer>> = const {RefCell::new(None)};
}

/// Executes a closure with access to a thread-local [`BufferFormatter`] slot.
///
/// This slot is used to reuse a formatter per thread to avoid allocating a
/// new buffer for every log record.
///
/// If the thread-local storage is unavailable (e.g. during thread shutdown),
/// returns `None`.
///
/// The slot may contain an existing formatter, or be empty if this is the
/// first log call on the thread.
pub fn try_with_buf_formatter_slot<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Option<Writer>) -> R,
{
    WRITER
        .try_with(|tl| {
            let mut slot = tl.try_borrow_mut().ok()?;
            Some(f(&mut slot))
        })
        .ok()
        .flatten()
}

/// Formatter backed by an in-memory buffer.
///
/// Formatting writes into the buffer through the standard [`std::io::Write`]
/// interface. The completed buffer can later be written by a [`Writer`].
#[derive(Debug, Default, Clone)]
pub struct Writer {
    buffer: Buffer,
    output: Output,
    style: Style,
}

impl Writer {
    /// Creates a new [`WriterBuilder`].
    pub fn builder() -> WriterBuilder {
        WriterBuilder::new()
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn write_buffer(&self) -> std::io::Result<()> {
        use std::io::Write as _;

        let buf_bytes = self.buffer.as_bytes();

        match &self.output {
            Output::Stdout => {
                let mut stream = std::io::stdout().lock();
                stream.write_all(buf_bytes)?;
                stream.flush()?;
            }
            Output::Stderr => {
                let mut stream = std::io::stderr().lock();
                stream.write_all(buf_bytes)?;
                stream.flush()?;
            }
            Output::File(path) => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                file.write_all(buf_bytes)?;
                file.flush()?;
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Flushes the configured output stream.
    pub fn flush(&self) -> std::io::Result<()> {
        use std::io::Write as _;
        match self.output() {
            Output::Stdout => std::io::stdout().flush(),
            Output::Stderr => std::io::stderr().flush(),
            Output::File(path) => {
                let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
                file.flush()
            }
        }
    }
}

impl std::io::prelude::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write_out(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }
}

/// Builder for constructing a [`Writer`].
#[derive(Debug, Default, Clone)]
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
        self.writer.output = Output::Stdout;
        self
    }

    pub fn stderr(&mut self) -> &mut Self {
        self.writer.output = Output::Stderr;
        self
    }

    pub fn file(&mut self, path: impl Into<String>) -> &mut Self {
        self.writer.output = Output::File(path.into());
        self
    }

    pub fn color_mode(&mut self, color_mode: ColorMode) -> &mut Self {
        self.writer.style.set_color_mode(color_mode);
        self
    }

    pub fn time_mode(&mut self, time_mode: TimeMode) -> &mut Self {
        self.writer.style.set_time_mode(time_mode);
        self
    }

    /// Builds the configured writer.
    ///
    /// When the color mode is [`ColorMode::Auto`], the final color mode is
    /// determined from the selected output destination. Terminal outputs
    /// enable colors, while file output disables them.
    pub fn build(self) -> Writer {
        use std::io::IsTerminal;
        let mut writer = self.writer;

        let color_choice = if writer.style.color_mode() == ColorMode::Auto {
            match writer.output() {
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
            writer.style.color_mode()
        };
        writer.style.set_color_mode(color_choice);

        writer
    }
}
