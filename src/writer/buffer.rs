/// In-memory buffer used by [`Writer`] to store formatted log output before it
/// is written to the configured destination.
///
/// The buffer implements a lightweight byte storage layer for formatting.
/// Data is accumulated during record formatting and cleared after the writer
/// completes output handling.
///
/// The buffer does not perform any output operations itself. Flushing is
/// handled by [`Writer`] when the completed buffer is written to the selected
#[derive(Default, Clone)]
pub struct Buffer(Vec<u8>);
impl Buffer {
    /// Clears all stored formatted bytes
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Appends bytes into the buffer.
    ///
    /// Returns the number of bytes written.
    pub fn write_out(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.extend(buffer);
        Ok(buffer.len())
    }

    /// Does nothing because [`Buffer`] does not own an output destination.
    ///
    /// Actual flushing is performed by [`Writer`] when writing the buffer to
    /// the configured [`Output`].
    pub fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
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
