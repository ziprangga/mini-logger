use std::fmt::Display;
use std::io::{self, Write};

use crate::logger::LogLevel;
use crate::logger::LogMessage;
use crate::style::TimestampPrecision;
use crate::writer::BufferFormat;

pub trait FormatRecord {
    fn format_record(
        &self,
        buf_format: &mut BufferFormat,
        record_msg: &LogMessage<'_>,
    ) -> std::io::Result<()>;
}

impl<F> FormatRecord for F
where
    F: Fn(&mut BufferFormat, &LogMessage<'_>) -> io::Result<()>,
{
    fn format_record(
        &self,
        buf_format: &mut BufferFormat,
        record_msg: &LogMessage<'_>,
    ) -> io::Result<()> {
        (self)(buf_format, record_msg)
    }
}

pub type FormatLog = Box<dyn FormatRecord + Sync + Send>;

#[derive(Default)]
pub struct FormatBuilder {
    format_default: FormatConfig,
    format_custom: Option<FormatLog>,
}

impl FormatBuilder {
    pub fn build(self) -> FormatLog {
        if let Some(fmt) = self.format_custom {
            fmt
        } else {
            Box::new(self.format_default)
        }
    }
}

pub struct FormatConfig {
    pub timestamp: Option<TimestampPrecision>,
    pub level: bool,
    pub target: bool,
    pub module_path: bool,
}

impl FormatConfig {
    pub fn timestamp(&mut self, timestamp: Option<TimestampPrecision>) -> &mut Self {
        self.timestamp = timestamp;
        self
    }
    pub fn level(&mut self, write: bool) -> &mut Self {
        self.level = write;
        self
    }
    pub fn target(&mut self, write: bool) -> &mut Self {
        self.target = write;
        self
    }
    pub fn module_path(&mut self, write: bool) -> &mut Self {
        self.module_path = write;
        self
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            timestamp: Some(TimestampPrecision::default()),
            level: true,
            target: false,
            module_path: true,
        }
    }
}

impl FormatRecord for FormatConfig {
    fn format_record(
        &self,
        buf_format: &mut BufferFormat,
        record_msg: &LogMessage<'_>,
    ) -> io::Result<()> {
        let fmt = FormatWriter {
            format: self,
            buf_format: buf_format,
            written_header: false,
        };

        fmt.write(record_msg)
    }
}

struct FormatWriter<'a> {
    pub format: &'a FormatConfig,
    pub buf_format: &'a mut BufferFormat,
    written_header: bool,
}

impl FormatWriter<'_> {
    #[inline]
    pub fn write(mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        self.write_timestamp()?;
        self.write_level(record_msg)?;
        self.write_target(record_msg)?;
        self.write_module(record_msg)?;
        self.finish_header()?;
        self.write_args(record_msg)
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header {
            self.written_header = true;
            write!(self.buf_format, "[{value}")?;
        } else {
            write!(self.buf_format, " {value}")?;
        }

        Ok(())
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        {
            use self::TimestampPrecision::{Micros, Millis, Nanos, Seconds};
            let ts = match self.format.timestamp {
                None => return Ok(()),
                Some(Seconds) => self.buf_format.timestamp().timestamp_seconds(),
                Some(Millis) => self.buf_format.timestamp().timestamp_millis(),
                Some(Micros) => self.buf_format.timestamp().timestamp_micros(),
                Some(Nanos) => self.buf_format.timestamp().timestamp_nanos(),
            };

            self.write_header_value(ts)
        }
    }

    fn write_level(&mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        if !self.format.level {
            return Ok(());
        }

        use crate::style::Color;
        let (level_str, color) = match record_msg.level() {
            LogLevel::Off => ("OFF", Color::Reset),
            LogLevel::Error => ("ERROR", Color::Red),
            LogLevel::Warn => ("WARN", Color::Yellow),
            LogLevel::Info => ("INFO", Color::Green),
            LogLevel::Debug => ("DEBUG", Color::Blue),
            LogLevel::Trace => ("TRACE", Color::Blue),
        };
        self.write_header_value(format_args!(
            "{}{:<5}{}",
            self.buf_format.color_mode().color(color),
            level_str,
            self.buf_format.color_mode().reset()
        ))
    }

    fn write_target(&mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        if !self.format.target {
            return Ok(());
        }

        let target = record_msg.target();
        if target.is_empty() {
            return Ok(());
        }

        self.write_header_value(target)
    }

    fn write_module(&mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        if !self.format.module_path {
            return Ok(());
        }
        if let Some(module) = record_msg.module() {
            self.write_header_value(module)
        } else {
            Ok(())
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header {
            write!(self.buf_format, "] ")?;
        }
        Ok(())
    }

    fn write_args(&mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        write!(self.buf_format, "{}\n", record_msg.msg())
    }
}
