/// Destination where completed log buffers are written.
///
/// [`Output`] defines the final target used by [`Writer`] when flushing
/// formatted log records.
///
/// Supported destinations:
///
/// - [`Output::Stdout`] writes to standard output.
/// - [`Output::Stderr`] writes to standard error.
/// - [`Output::File`] appends records to a configured file path.
#[derive(Default, Clone)]
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
        match self {
            Self::Stdout => write!(f, "stdout"),
            Self::Stderr => write!(f, "stderr"),
            Self::File(_) => write!(f, "file"),
        }
    }
}
