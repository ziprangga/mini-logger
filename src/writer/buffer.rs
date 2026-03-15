use super::Output;
use crate::style::ColorStyle;

pub struct Buffer(Vec<u8>);
impl Buffer {
    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.extend(buffer);
        Ok(buffer.len())
    }

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

#[derive(Debug, Default)]
pub struct BufferWriter {
    output: Output,
    color_style: ColorStyle,
}

impl BufferWriter {
    pub fn new(output: Output, color_style: ColorStyle) -> Self {
        Self {
            output,
            color_style,
        }
    }

    pub fn stdout(color_style: ColorStyle) -> Self {
        Self {
            output: Output::Stdout,
            color_style,
        }
    }

    pub fn stderr(color_style: ColorStyle) -> Self {
        Self {
            output: Output::Stderr,
            color_style,
        }
    }

    pub fn file(path: impl Into<String>, color_style: ColorStyle) -> Self {
        Self {
            output: Output::File(path.into()),
            color_style,
        }
    }

    pub fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub fn color_style(&self) -> ColorStyle {
        self.color_style
    }

    pub fn write(&self, buf: &Buffer) -> std::io::Result<()> {
        use std::io::Write as _;

        let buf_bytes = buf.as_bytes();

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
}

#[derive(Debug)]
pub struct Writer {
    buffer_writer: BufferWriter,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            buffer_writer: BufferWriter::default(),
        }
    }

    pub fn stdout(mut self) -> Self {
        self.buffer_writer.output = Output::Stdout;
        self
    }

    pub fn stderr(mut self) -> Self {
        self.buffer_writer.output = Output::Stderr;
        self
    }

    pub fn file(mut self, path: impl Into<String>) -> Self {
        self.buffer_writer.output = Output::File(path.into());
        self
    }

    pub fn color(mut self, color_style: ColorStyle) -> Self {
        self.buffer_writer.color_style = color_style;
        self
    }

    pub fn build(self) -> BufferWriter {
        self.buffer_writer
    }
}
