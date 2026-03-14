use crate::format::Formatter;
use crate::style::TimestampPrecision;
use std::io::{self, Write};

/// Configuration for how to format log records
pub struct ConfigFormat {
    pub timestamp: Option<TimestampPrecision>,
    pub level: bool,
    pub module_path: bool,
    pub target: bool,
    pub source_file: bool,
    pub source_line_number: bool,
    pub indent: Option<usize>,
    pub suffix: &'static str,
}

// impl ConfigFormat {
//     pub fn format(
//         &self,
//         formatter: &mut Formatter,
//         log_message: &LogMessage<'_>,
//     ) -> io::Result<()> {
//         let fmt = ConfigFormatWriter {
//             config: self,
//             buf: formatter,
//             written_header: false,
//         };

//         fmt.write(record)
//     }
// }

impl Default for ConfigFormat {
    fn default() -> Self {
        Self {
            timestamp: Some(TimestampPrecision::Seconds),
            level: true,
            module_path: false,
            target: true,
            source_file: false,
            source_line_number: false,
            indent: Some(4),
            suffix: "\n",
        }
    }
}

/// Writer for a record using a given config and formatter
pub struct ConfigFormatWriter<'a> {
    pub config: &'a ConfigFormat,
    pub buf: &'a mut Formatter,
    written_header: bool,
}

impl<'a> ConfigFormatWriter<'a> {
    pub fn new(config: &'a ConfigFormat, buf: &'a mut Formatter) -> Self {
        Self {
            config,
            buf,
            written_header: false,
        }
    }

    fn write_header_value<T: std::fmt::Display>(&mut self, value: T) -> std::io::Result<()> {
        if !self.written_header {
            self.written_header = true;
            write!(self.buf, "[{}", value)
        } else {
            write!(self.buf, " {}", value)
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header {
            write!(self.buf, "] ")?;
        }
        Ok(())
    }

    pub fn write_record(
        &mut self,
        level: Option<&str>,
        module_path: Option<&str>,
        target: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
        message: &str,
    ) -> io::Result<()> {
        // Timestamp
        if let Some(ts) = self.config.timestamp {
            let timestamp = self.buf.timestamp(ts);
            self.write_header_value(timestamp)?;
        }

        // Level
        if self.config.level {
            if let Some(level) = level {
                self.write_header_value(level)?;
            }
        }

        // Module path
        if self.config.module_path {
            if let Some(module_path) = module_path {
                self.write_header_value(module_path)?;
            }
        }

        // Source file + line
        if self.config.source_file {
            if let Some(file) = file {
                let value = if self.config.source_line_number {
                    match line {
                        Some(line) => format!("{file}:{line}"),
                        None => file.to_string(),
                    }
                } else {
                    file.to_string()
                };
                self.write_header_value(value)?;
            }
        }

        // Target
        if self.config.target {
            if let Some(target) = target {
                if !target.is_empty() {
                    self.write_header_value(target)?;
                }
            }
        }

        // Finish header
        self.finish_header()?;

        // Message with indent
        if let Some(indent) = self.config.indent {
            for (i, line) in message.lines().enumerate() {
                if i > 0 {
                    write!(self.buf, "{}{}", self.config.suffix, " ".repeat(indent))?;
                }
                write!(self.buf, "{}", line)?;
            }
        } else {
            write!(self.buf, "{}", message)?;
        }

        // Suffix
        write!(self.buf, "{}", self.config.suffix)?;

        Ok(())
    }
}

/// Optional builder to simplify setting up ConfigFormat
pub struct FormatBuilder {
    config: ConfigFormat,
}

impl FormatBuilder {
    pub fn new() -> Self {
        Self {
            config: ConfigFormat::default(),
        }
    }

    pub fn timestamp(mut self, precision: Option<TimestampPrecision>) -> Self {
        self.config.timestamp = precision;
        self
    }

    pub fn level(mut self, enabled: bool) -> Self {
        self.config.level = enabled;
        self
    }

    pub fn module_path(mut self, enabled: bool) -> Self {
        self.config.module_path = enabled;
        self
    }

    pub fn target(mut self, enabled: bool) -> Self {
        self.config.target = enabled;
        self
    }

    pub fn file(mut self, enabled: bool) -> Self {
        self.config.source_file = enabled;
        self
    }

    pub fn line_number(mut self, enabled: bool) -> Self {
        self.config.source_line_number = enabled;
        self
    }

    pub fn indent(mut self, spaces: Option<usize>) -> Self {
        self.config.indent = spaces;
        self
    }

    pub fn suffix(mut self, suffix: &'static str) -> Self {
        self.config.suffix = suffix;
        self
    }

    pub fn build(self) -> ConfigFormat {
        self.config
    }
}
