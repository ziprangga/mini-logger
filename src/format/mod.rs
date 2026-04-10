mod format_config;
pub use format_config::FormatConfig;

use crate::record::RecMessage;
use crate::writer::BufferFormatter;
use std::io;

pub trait FormatCustom {
    fn format_custom_layout(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> std::io::Result<()>;
}

impl<F> FormatCustom for F
where
    F: Fn(&mut BufferFormatter, &RecMessage<'_>) -> io::Result<()>,
{
    fn format_custom_layout(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        (self)(buf_formatter, record_msg)
    }
}

pub enum Format {
    Default(FormatConfig),
    Custom(Box<dyn FormatCustom + Send + Sync>),
}

impl Format {
    pub fn format_record(
        &self,
        buf_formatter: &mut BufferFormatter,
        record_msg: &RecMessage<'_>,
    ) -> io::Result<()> {
        match self {
            Format::Default(f) => f.format_write_layout(buf_formatter, record_msg),
            Format::Custom(f) => f.format_custom_layout(buf_formatter, record_msg),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Default(FormatConfig::default())
    }
}

#[derive(Default)]
pub struct FormatBuilder {
    format: Format,
}

impl FormatBuilder {
    pub fn format_default(&mut self) -> &mut FormatConfig {
        self.format = Format::Default(FormatConfig::default());

        match &mut self.format {
            Format::Default(cfg) => cfg,
            _ => unreachable!("Format should now always be Default"),
        }
    }

    pub fn format_custom<F>(&mut self, f: F) -> &mut Self
    where
        F: FormatCustom + Send + Sync + 'static,
    {
        self.format = Format::Custom(Box::new(f));
        self
    }

    pub fn build(self) -> Format {
        self.format
    }
}
