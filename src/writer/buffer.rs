use std::cell::RefCell;

thread_local! {
    static BUFFER: RefCell<Option<Buffer>> = const {RefCell::new(None)};
}

/// Executes a closure with access to the thread-local [`Buffer`] slot.
///
/// The slot stores a reusable [`Buffer`] for the current thread.
/// Reusing this buffer avoids allocating a new buffer for every log record.
///
/// The slot may already contain a buffer from a previous log call, or may be
/// empty when the thread performs logging for the first time.
///
/// Returns `None` when thread-local access is unavailable, such as during
/// thread shutdown.
pub fn try_with_buffer_slot<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Option<Buffer>) -> R,
{
    BUFFER
        .try_with(|tl| {
            let mut slot = tl.try_borrow_mut().ok()?;
            Some(f(&mut slot))
        })
        .ok()
        .flatten()
}

/// In-memory buffer used by [`Writer`] to store formatted log output before it
/// is written to the configured destination.
///
/// The buffer implements a lightweight byte storage layer for formatting.
/// Data is accumulated during record formatting and cleared after the writer
/// completes output handling.
///
/// The buffer does not perform any output operations itself. Flushing is
/// handled by [`Writer`] when the completed buffer is written to the configured
/// [`Output`].
#[derive(Default)]
pub struct Buffer(Vec<u8>);
impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all stored formatted bytes.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Returns the stored bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(self.as_bytes()).fmt(f)
    }
}

impl std::io::Write for Buffer {
    /// Appends bytes into the buffer.
    ///
    /// Returns the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    /// Does nothing because [`Buffer`] does not own an output destination.
    ///
    /// Actual flushing is performed by [`Writer`] when writing the buffer to
    /// the configured [`Output`].
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
