mod buffer;

use crate::style::ColorStyle;
pub use buffer::{Buffer, BufferWriter};

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

#[derive(Debug, Default)]
pub struct Writer {
    buffer_writer: BufferWriter,
}

impl Writer {
    pub fn builder() -> WriterBuilder {
        WriterBuilder::new()
    }

    pub fn color_style(&self) -> ColorStyle {
        self.buffer_writer.color_style()
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer_writer.buffer()
    }

    pub fn print_out(&self, buf: &Buffer) -> std::io::Result<()> {
        self.buffer_writer.write_buffer(buf)
    }

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

    pub fn color_style(&mut self, color_style: ColorStyle) -> &mut Self {
        self.writer.buffer_writer.set_color_style(color_style);
        self
    }

    pub fn build(self) -> Writer {
        let color = self.writer.color_style();
        let output = self.writer.buffer_writer.output_take();

        use std::io::IsTerminal;

        let color_choice = if color == ColorStyle::Auto {
            match output {
                Output::Stdout => {
                    if std::io::stdout().is_terminal() {
                        ColorStyle::Always
                    } else {
                        ColorStyle::Never
                    }
                }
                Output::Stderr => {
                    if std::io::stderr().is_terminal() {
                        ColorStyle::Always
                    } else {
                        ColorStyle::Never
                    }
                }
                Output::File(_) => ColorStyle::Never,
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
