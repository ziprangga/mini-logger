mod buffer;

pub use buffer::{Buffer, BufferWriter, Writer};

#[derive(Default)]
pub enum Output {
    #[default]
    Stdout,
    Stderr,
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
