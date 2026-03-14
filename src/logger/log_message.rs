use super::LogFormatter;
use super::{Level, LogConfig};
use crate::style::TimestampPrecision;

use std::fmt::Display;
use std::io::{self, Write};

#[derive(Clone, Debug)]
pub struct LogMessage<'a> {
    log_config: LogConfig<'a>,
    timestamp: Option<TimestampPrecision>,
    module: Option<&'a str>,
    msg: std::fmt::Arguments<'a>,
}

impl<'a> LogMessage<'a> {
    #[inline]
    pub fn builder() -> LogMessageBuilder<'a> {
        LogMessageBuilder::new()
    }

    #[inline]
    pub fn log_config(&self) -> &LogConfig<'a> {
        &self.log_config
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.log_config.level()
    }

    #[inline]
    pub fn target(&self) -> &'a str {
        self.log_config.target()
    }

    #[inline]
    pub fn timestamp(&self) -> Option<TimestampPrecision> {
        self.timestamp
    }

    #[inline]
    pub fn module(&self) -> Option<&'a str> {
        self.module
    }

    #[inline]
    pub fn msg(&self) -> &std::fmt::Arguments<'a> {
        &self.msg
    }
}

impl Default for LogMessage<'_> {
    fn default() -> Self {
        Self {
            log_config: LogConfig::builder().build(),
            timestamp: Some(TimestampPrecision::Seconds),
            module: None,
            msg: format_args!(""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogMessageBuilder<'a> {
    log_message: LogMessage<'a>,
}

impl<'a> LogMessageBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            log_message: LogMessage::default(),
        }
    }

    #[inline]
    pub fn log_config(&mut self, log_config: LogConfig<'a>) -> &mut Self {
        self.log_message.log_config = log_config;
        self
    }

    #[inline]
    pub fn level(&mut self, level: Level) -> &mut Self {
        let target = self.log_message.log_config.target();
        self.log_message.log_config = LogConfig::builder().level(level).target(target).build();

        self
    }

    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        let level = self.log_message.log_config.level();
        self.log_message.log_config = LogConfig::builder().level(level).target(target).build();
        self
    }

    #[inline]
    pub fn timestamp(&mut self, ts: Option<TimestampPrecision>) -> &mut Self {
        self.log_message.timestamp = ts;
        self
    }

    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.log_message.module = module;
        self
    }

    #[inline]
    pub fn msg(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.log_message.msg = msg;
        self
    }

    #[inline]
    pub fn build(&self) -> LogMessage<'a> {
        self.log_message.clone()
    }

    pub fn build_record(&self, log_formatter: &mut LogFormatter) -> io::Result<()> {
        let msg = self.build();
        let fmt = LogMessageFormatWriter {
            log_message: &msg,
            buf: log_formatter,
            written_header: false,
        };

        fmt.write()
    }
}

impl Default for LogMessageBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogMessageFormatWriter<'a> {
    pub log_message: &'a LogMessage<'a>,
    pub buf: &'a mut LogFormatter,
    written_header: bool,
}

impl<'a> LogMessageFormatWriter<'a> {
    #[inline]
    pub fn write(mut self) -> io::Result<()> {
        self.write_timestamp()?;
        self.write_level()?;
        self.write_target()?;
        self.write_module()?;
        self.finish_header()?;
        self.write_args()
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header {
            self.written_header = true;
            write!(self.buf, "[{value}]")?;
        } else {
            write!(self.buf, " {value}")?;
        }

        Ok(())
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        if let Some(precision) = self.log_message.timestamp() {
            let ts = self.buf.timestamp(precision);
            self.write_header_value(ts)?;
        }

        Ok(())
    }

    fn write_level(&mut self) -> io::Result<()> {
        let level = match self.log_message.level() {
            Level::Off => "OFF",
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };

        self.write_header_value(format_args!("{level:<5}"))
    }

    fn write_target(&mut self) -> io::Result<()> {
        let target = self.log_message.target();
        if target.is_empty() {
            return Ok(());
        }

        self.write_header_value(target)
    }

    fn write_module(&mut self) -> io::Result<()> {
        if let Some(module) = self.log_message.module() {
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

    fn write_args(&mut self) -> io::Result<()> {
        write!(self.buf, "{}", self.log_message.msg())
    }
}

// use std::sync::OnceLock;
// static LOGGER: OnceLock<&'static dyn Logger> = OnceLock::new();

// pub trait Logger: Send + Sync {
//     fn enable(&self, log_config: &LogConfig) -> bool;
//     fn log_msg(&self, log_message: &LogMessage);
//     fn flush(&self);
// }

// pub fn set_logger(logger: &'static dyn Logger) -> Result<(), &'static str> {
//     LOGGER.set(logger).map_err(|_| "Logger already set")
// }

// pub fn logger() -> Option<&'static dyn Logger> {
//     LOGGER.get().copied()
// }
