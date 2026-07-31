//! Log record formatting.
//!
//! This module defines how log records are rendered before being written to an
//! output destination.
//!
//! Formatting can be configured using [`FormatterBuilder`].
//!
//! The built-in renderer supports configurable header fields such as:
//!
//! - timestamp
//! - log level
//! - target
//! - module path
//!
//! Applications can also provide a custom renderer by implementing
//! [`RenderRecord`] or by supplying a compatible closure.
//! ```

mod default_render;

use crate::record::RecMessage;
use crate::style::{ColorMode, Style, TimeMode, TimePrecision};
use crate::writer::Buffer;
use crate::writer::Output;

/// Renders a log record into a [`Buffer`].
///
/// Implementations are responsible for formatting record data and writing the
/// resulting bytes into the provided buffer.
pub trait RenderRecord {
    fn render(
        &self,
        formatter: &Formatter,
        buffer: &mut Buffer,
        record: &RecMessage<'_>,
    ) -> std::io::Result<()>;
}

impl<F> RenderRecord for F
where
    F: Fn(&Formatter, &mut Buffer, &RecMessage<'_>) -> std::io::Result<()> + Send + Sync,
{
    fn render(
        &self,
        formatter: &Formatter,
        buffer: &mut Buffer,
        record: &RecMessage<'_>,
    ) -> std::io::Result<()> {
        (self)(formatter, buffer, record)
    }
}

/// Log record formatting configuration.
///
/// A formatter controls:
///
/// - style settings such as colors and timestamps,
/// - which metadata fields are included,
/// - the renderer used to produce the final output.
///
/// If no custom renderer is configured, the built-in renderer is used.
pub struct Formatter {
    style: Style,
    level: bool,
    target: bool,
    module_path: bool,
    renderer: Option<Box<dyn RenderRecord + Send + Sync>>,
}

impl Formatter {
    /// Creates a new [`FormatterBuilder`].
    pub fn builder() -> FormatterBuilder {
        FormatterBuilder::new()
    }

    /// Enables or disables writing the log level.
    pub fn level(&self) -> bool {
        self.level
    }

    /// Enables or disables writing the log target.
    pub fn target(&self) -> bool {
        self.target
    }

    /// Enables or disables writing the module path.
    pub fn module_path(&self) -> bool {
        self.module_path
    }

    /// Returns the active style configuration.
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Renders a record into the provided buffer.
    ///
    /// The configured custom renderer is used when present. Otherwise the
    /// built-in renderer is used.
    pub fn render_record(
        &self,
        buffer: &mut Buffer,
        record: &RecMessage<'_>,
    ) -> std::io::Result<()> {
        match &self.renderer {
            Some(renderer) => renderer.render(self, buffer, record),
            None => default_render::DefaultRenderer.render(self, buffer, record),
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self {
            style: Style::default(),
            level: true,
            target: false,
            module_path: true,
            renderer: None,
        }
    }
}

/// Builder for constructing a [`Formatter`].
pub struct FormatterBuilder {
    formatter: Formatter,
}

impl FormatterBuilder {
    /// Creates a builder with default formatting configuration.
    pub fn new() -> Self {
        Self {
            formatter: Formatter::default(),
        }
    }

    /// Enables or disables writing the log level.
    pub fn level(&mut self, write: bool) -> &mut Self {
        self.formatter.level = write;
        self
    }

    /// Enables or disables writing the log target.
    pub fn target(&mut self, write: bool) -> &mut Self {
        self.formatter.target = write;
        self
    }

    /// Enables or disables writing the module path.
    pub fn module_path(&mut self, write: bool) -> &mut Self {
        self.formatter.module_path = write;
        self
    }

    /// Sets the color mode.
    pub fn color_mode(&mut self, color_mode: ColorMode) -> &mut Self {
        self.formatter.style.set_color_mode(color_mode);
        self
    }

    /// Sets the timestamp mode.
    pub fn time_mode(&mut self, time_mode: TimeMode) -> &mut Self {
        self.formatter.style.set_time_mode(time_mode);
        self
    }

    /// Sets the timestamp precision.
    pub fn time_precision(&mut self, time_precision: TimePrecision) -> &mut Self {
        self.formatter.style.set_time_precision(time_precision);
        self
    }

    /// Configures a custom record renderer.
    ///
    /// The supplied renderer replaces the built-in renderer.
    pub fn format_with<F>(&mut self, format: F) -> &mut Self
    where
        F: RenderRecord + Send + Sync + 'static,
    {
        self.formatter.renderer = Some(Box::new(format));
        self
    }

    /// Builds the configured formatter.
    ///
    /// Automatic color mode resolution is performed against the specified output
    /// destination before the formatter is returned.
    pub fn build(self, output: &Output) -> Formatter {
        let mut format = self.formatter;

        let color_choice = format.style().color_mode().resolve(output);

        format.style.set_color_mode(color_choice);

        format
    }
}

impl Default for FormatterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
