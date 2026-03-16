use std::cell::RefCell;
use std::fmt::Display;
use std::io::{self, Write};
use std::rc::Rc;
// use std::time::SystemTime;

use crate::logger::log_config::Level;
use crate::logger::log_message::LogMessage;
use crate::style::{ColorStyle, Timestamp, TimestampPrecision};
use crate::writer::{Buffer, Writer};

pub struct LogFormatter {
    buffer: Rc<RefCell<Buffer>>,
    color_style: ColorStyle,
}

impl LogFormatter {
    pub fn new(writer: &Writer) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(writer.buffer())),
            color_style: writer.color_style(),
        }
    }

    pub fn color_style(&self) -> ColorStyle {
        self.color_style
    }

    pub fn print(&self, writer: &Writer) -> std::io::Result<()> {
        writer.print_out(&self.buffer.borrow())
    }

    pub fn clear(&mut self) {
        self.buffer.borrow_mut().clear();
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::default()
    }
}

impl std::io::prelude::Write for LogFormatter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.borrow_mut().write_out(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.borrow_mut().flush()
    }
}

pub trait FormatRecord {
    fn format_record(
        &self,
        format: &mut LogFormatter,
        record_msg: &LogMessage<'_>,
    ) -> std::io::Result<()>;
}

impl<F> FormatRecord for F
where
    F: Fn(&mut LogFormatter, &LogMessage<'_>) -> io::Result<()>,
{
    fn format_record(
        &self,
        formatter: &mut LogFormatter,
        record_msg: &LogMessage<'_>,
    ) -> io::Result<()> {
        (self)(formatter, record_msg)
    }
}

pub type FormatFn = Box<dyn FormatRecord + Sync + Send>;

#[derive(Default)]
pub struct FormatBuilder {
    format_default: FormatConfig,
    format_custom: Option<FormatFn>,
    built: bool,
}

impl FormatBuilder {
    pub fn build(&mut self) -> FormatFn {
        // if let Some(fmt) = self.format_custom {
        //     fmt
        // } else {
        //     Box::new(self.format_default)
        // }

        assert!(!self.built, "attempt to re-use consumed format builder");
        let built = std::mem::replace(
            self,
            FormatBuilder {
                built: true,
                ..Default::default()
            },
        );
        if let Some(fmt) = built.format_custom {
            fmt
        } else {
            Box::new(built.format_default)
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
            target: true,
            module_path: true,
        }
    }
}

impl FormatRecord for FormatConfig {
    fn format_record(
        &self,
        formatter: &mut LogFormatter,
        record_msg: &LogMessage<'_>,
    ) -> io::Result<()> {
        let fmt = FormatWriter {
            format: self,
            buf: formatter,
            written_header: false,
        };

        fmt.write(record_msg)
    }
}

struct FormatWriter<'a> {
    pub format: &'a FormatConfig,
    pub buf: &'a mut LogFormatter,
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
            write!(self.buf, "[{value}")?;
        } else {
            write!(self.buf, " {value}")?;
        }

        Ok(())
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        {
            use self::TimestampPrecision::{Micros, Millis, Nanos, Seconds};
            let ts = match self.format.timestamp {
                None => return Ok(()),
                Some(Seconds) => self.buf.timestamp().timestamp_seconds(),
                Some(Millis) => self.buf.timestamp().timestamp_millis(),
                Some(Micros) => self.buf.timestamp().timestamp_micros(),
                Some(Nanos) => self.buf.timestamp().timestamp_nanos(),
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
            Level::Off => ("OFF", Color::Reset),
            Level::Error => ("ERROR", Color::Red),
            Level::Warn => ("WARN", Color::Yellow),
            Level::Info => ("INFO", Color::Green),
            Level::Debug => ("DEBUG", Color::Blue),
            Level::Trace => ("TRACE", Color::Blue),
        };
        self.write_header_value(format_args!(
            "{}{:<5}{}",
            self.buf.color_style().color(color),
            level_str,
            self.buf.color_style().reset()
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
            write!(self.buf, "] ")?;
        }
        Ok(())
    }

    fn write_args(&mut self, record_msg: &LogMessage<'_>) -> io::Result<()> {
        write!(self.buf, "{}", record_msg.msg())
    }
}
