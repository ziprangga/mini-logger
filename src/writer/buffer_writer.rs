use super::Output;
use crate::style::ColorMode;

pub struct Buffer(Vec<u8>);
impl Buffer {
    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn write_out(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
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
    color_mode: ColorMode,
}

impl BufferWriter {
    pub fn new(output: Output, color_mode: ColorMode) -> Self {
        Self { output, color_mode }
    }

    pub fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub fn output_ref(&self) -> &Output {
        &self.output
    }

    pub fn output_take(self) -> Output {
        self.output
    }

    pub fn set_output(&mut self, output: Output) -> &mut Self {
        self.output = output;
        self
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn set_color_mode(&mut self, color_mode: ColorMode) -> &mut Self {
        self.color_mode = color_mode;
        self
    }

    pub fn write_buffer(&self, buf: &Buffer) -> std::io::Result<()> {
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
