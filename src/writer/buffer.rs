/// In-memory buffer used while formatting log records
#[derive(Default, Clone)]
pub struct Buffer(Vec<u8>);
impl Buffer {
    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn write_out(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.extend(buffer);
        Ok(buffer.len())
    }

    /// No-op.
    ///
    /// Buffers are flushed when written by [`BufferWriter`].
    pub fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(self.as_bytes()).fmt(f)
    }
}
