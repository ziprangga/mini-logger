//! Log record representation.
//!
//! This module defines [`RecMessage`], the internal representation of a
//! log record passed through filtering, formatting, and writing.
//!
//! A log record contains:
//! - the log level,
//! - the log target,
//! - the optional module path,
//! - the formatted log message.
//!
//! [`RecMessageBuilder`] provides a convenient way to construct log records.

use crate::filter::FilterLevel;

/// Represents a single log record.
///
/// A log record carries all information required by the logger pipeline,
/// including filtering, formatting, and writing.
#[derive(Clone, Debug)]
pub struct RecMessage<'a> {
    level: FilterLevel,
    target: &'a str,
    module: Option<&'a str>,
    msg: std::fmt::Arguments<'a>,
}

impl<'a> RecMessage<'a> {
    /// Creates a new [`RecMessageBuilder`].
    #[inline]
    pub fn builder() -> RecMessageBuilder<'a> {
        RecMessageBuilder::new()
    }

    /// Returns the log level.
    #[inline]
    pub fn level(&self) -> FilterLevel {
        self.level
    }

    /// Returns the log level.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }

    /// Returns the module path, if available.
    #[inline]
    pub fn module(&self) -> Option<&'a str> {
        self.module
    }

    /// Returns the formatted log message.
    #[inline]
    pub fn msg(&self) -> &std::fmt::Arguments<'a> {
        &self.msg
    }
}

impl Default for RecMessage<'_> {
    fn default() -> Self {
        Self {
            level: FilterLevel::default(),
            target: "",
            module: None,
            msg: format_args!(""),
        }
    }
}

/// Builder for constructing a [`RecMessage`].
#[derive(Clone, Debug)]
pub struct RecMessageBuilder<'a> {
    record_msg: RecMessage<'a>,
}

impl<'a> RecMessageBuilder<'a> {
    /// Creates a new empty log record builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            record_msg: RecMessage::default(),
        }
    }

    /// Sets the log level.
    #[inline]
    pub fn level(&mut self, level: FilterLevel) -> &mut Self {
        self.record_msg.level = level;
        self
    }

    /// Sets the log target.
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.record_msg.target = target;
        self
    }

    /// Sets the module path.
    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.record_msg.module = module;
        self
    }

    /// Sets the formatted log message.
    #[inline]
    pub fn msg(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.record_msg.msg = msg;
        self
    }

    /// Builds a log record.
    ///
    /// The builder remains unchanged and can be used to build additional
    /// records with modified fields.
    #[inline]
    pub fn build(&self) -> RecMessage<'a> {
        self.record_msg.clone()
    }
}

impl Default for RecMessageBuilder<'_> {
    /// Creates a new empty log record builder.
    fn default() -> Self {
        Self::new()
    }
}
