//! Log record representation.
//!
//! This module defines [`RecordMsg`], the internal representation of a
//! log record passed through filtering, formatting, and writing.
//!
//! A log record contains:
//! - the log level,
//! - the log target,
//! - the optional module path,
//! - the formatted log message.
//!
//! [`RecordMsgBuilder`] provides a convenient way to construct log records.

use crate::filter::Level;

/// Represents a single log record.
///
/// A log record carries all information required by the logger pipeline,
/// including filtering, formatting, and writing.
#[derive(Clone, Debug)]
pub struct RecordMsg<'a> {
    level: Level,
    target: &'a str,
    module: Option<&'a str>,
    message: std::fmt::Arguments<'a>,
}

impl<'a> RecordMsg<'a> {
    /// Creates a new [`RecordMsgBuilder`].
    #[inline]
    pub fn builder() -> RecordMsgBuilder<'a> {
        RecordMsgBuilder::new()
    }

    /// Returns the log level.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns the log target.
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
    pub fn message(&self) -> &std::fmt::Arguments<'a> {
        &self.message
    }
}

impl<'a> Default for RecordMsg<'a> {
    fn default() -> Self {
        Self {
            level: Level::default(),
            target: "",
            module: None,
            message: format_args!(""),
        }
    }
}

/// Builder for constructing a [`RecordMsg`].
#[derive(Clone, Debug)]
pub struct RecordMsgBuilder<'a> {
    level: Level,
    target: &'a str,
    module: Option<&'a str>,
    message: std::fmt::Arguments<'a>,
}

impl<'a> RecordMsgBuilder<'a> {
    /// Creates a new empty log record builder.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the log level.
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    /// Sets the log target.
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.target = target;
        self
    }

    /// Sets the module path.
    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.module = module;
        self
    }

    /// Sets the formatted log message.
    #[inline]
    pub fn message(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.message = msg;
        self
    }

    /// Builds a log record.
    ///
    /// The builder remains unchanged and can be used to build additional
    /// records with modified fields.
    #[inline]
    pub fn build(&self) -> RecordMsg<'a> {
        RecordMsg {
            level: self.level,
            target: self.target,
            module: self.module,
            message: self.message,
        }
    }
}

impl<'a> Default for RecordMsgBuilder<'a> {
    fn default() -> Self {
        Self {
            level: Level::default(),
            target: "",
            module: None,
            message: format_args!(""),
        }
    }
}
