//! Buffered output and writing infrastructure.
//!
//! This module provides the output layer used by the logger.
//!
//! The writing pipeline consists of:
//!
//! - [`Writer`] stores the configured output destination.
//! - [`Buffer`] stores formatted bytes before they are written.
//! - [`Output`] defines the destination where completed buffers are written.
//!
//! The logger uses thread-local storage to reuse a [`Buffer`] per thread
//! during logging. This avoids repeated buffer allocations while keeping each
//! thread's temporary formatting state independent.
//!
//! The configured writer is created by [`WriterBuilder`] and finalized during
//! [`WriterBuilder::build`].
//!
//! Buffering allows formatting to complete before output is written, reducing
//! partial writes and keeping formatting separate from output handling.
//!..

mod buffer;
mod output;
pub use buffer::{Buffer, try_with_buffer_slot};
pub use output::Output;

/// Writes completed [`Buffer`] contents to a configured [`Output`].
///
/// A writer stores the output destination used when emitting formatted log
/// records.
///
/// Formatted bytes are accumulated separately in a [`Buffer`] and are written
/// through the writer once formatting has completed.
#[derive(Debug, Default)]
pub struct Writer {
    output: Output,
}

impl Writer {
    /// Creates a new [`WriterBuilder`].
    pub fn builder() -> WriterBuilder {
        WriterBuilder::new()
    }

    /// Returns the configured output destination.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// Writes the contents of a completed [`Buffer`] to the configured output
    /// destination.
    ///
    /// The buffer itself is not modified.
    pub fn write_buffer(&self, buffer: &Buffer) -> std::io::Result<()> {
        use std::io::Write as _;

        let buf_bytes = buffer.as_bytes();

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

    /// Flushes the configured output destination.
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

/// Builder for constructing a [`Writer`].
///
/// Configures the output destination used by the writer.
///
/// The final writer state is produced by [`WriterBuilder::build`].
#[derive(Debug, Default)]
pub struct WriterBuilder {
    writer: Writer,
}

impl WriterBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            writer: Writer::default(),
        }
    }

    /// Configures the writer to write to standard output.
    pub fn stdout(&mut self) -> &mut Self {
        self.writer.output = Output::Stdout;
        self
    }

    /// Configures the writer to write to standard error.
    pub fn stderr(&mut self) -> &mut Self {
        self.writer.output = Output::Stderr;
        self
    }

    /// Configures the writer to append records to the specified file.
    pub fn file(&mut self, path: impl Into<String>) -> &mut Self {
        self.writer.output = Output::File(path.into());
        self
    }

    /// Builds the configured writer.
    pub fn build(self) -> Writer {
        self.writer
    }
}
