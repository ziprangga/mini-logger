/// Output destination used by the logger.
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
