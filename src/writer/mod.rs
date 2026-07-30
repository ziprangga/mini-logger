//! Buffered output and writing infrastructure.
//!
//! This module provides the output layer used by the logger.
//!
//! The writing pipeline consists of:
//!
//! - [`Writer`] stores the reusable formatting buffer, output destination,
//!   and output style configuration.
//! - [`Buffer`] stores formatted bytes before they are written.
//! - [`Output`] defines the destination where completed buffers are written.
//!
//! The writer uses thread-local storage to reuse a [`Writer`] instance per
//! thread during logging. This avoids repeated buffer allocations while
//! keeping each thread's temporary formatting state independent.
//!
//! The configured writer is created by [`WriterBuilder`] and finalized during
//! [`WriterBuilder::build`]. Automatic color selection is resolved at build
//! time after the output destination has been configured, ensuring the result
//! is independent of the order in which builder methods are called.
//!
//! Buffering allows formatting to complete before output is written, reducing
//! partial writes and keeping formatting separate from output handling.
//!..

mod buffer;
mod output;
pub use buffer::Buffer;
pub use output::Output;

use crate::style::{ColorMode, Style, TimeMode};
use std::cell::RefCell;

thread_local! {
    static WRITER: RefCell<Option<Writer>> = const {RefCell::new(None)};
}

/// Executes a closure with access to the thread-local [`Writer`] slot.
///
/// The slot stores a reusable writer instance for the current thread.
/// Reusing this writer avoids allocating a new buffer for every log record.
///
/// The slot may already contain a writer from a previous log call, or may be
/// empty when the thread performs logging for the first time.
///
/// Returns `None` when thread-local access is unavailable, such as during
/// thread shutdown.
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

/// Writer backed by an in-memory buffer.
///
/// A writer combines three responsibilities:
///
/// - storing formatted bytes in [`Buffer`],
/// - holding the configured [`Output`] destination,
/// - applying [`Style`] options during formatting.
///
/// The completed buffer can later be written to the configured destination.
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
///
/// Configures:
///
/// - output destination,
/// - color mode,
/// - time mode.
///
/// The final writer state is produced by [`WriterBuilder::build`].
/// Automatic color mode resolution happens during build because the selected
/// output destination is required to determine whether terminal colors should
/// be enabled.
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
    /// If [`ColorMode::Auto`] is configured, the final color mode is resolved
    /// from the selected output destination:
    ///
    /// - terminal output enables colors when supported,
    /// - file output disables colors.
    ///
    /// Resolution is delayed until build time so calls such as:
    ///
    /// ```text
    /// color_mode(Auto) -> output_stdout()
    /// ```
    ///
    /// and:
    ///
    /// ```text
    /// output_stdout() -> color_mode(Auto)
    /// ```
    ///
    /// produce the same final writer configuration.
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
