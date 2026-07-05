//! Target-specific filter directives.
//!
//! Each directive associates an optional target prefix with a maximum
//! enabled log level. A missing target represents the global default.

use crate::filter::FilterLevel;

/// A filter directive consisting of an optional target prefix and log level.
#[derive(Clone, Debug)]
pub struct FilterTarget {
    target: Option<String>,
    level: FilterLevel,
}

impl FilterTarget {
    /// Creates a new target filter directive.
    pub fn new(target: Option<String>, level: FilterLevel) -> Self {
        Self { target, level }
    }

    /// Returns the target prefix, or `None` for the global directive.
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Returns the configured log level.
    pub fn level(&self) -> FilterLevel {
        self.level
    }

    /// Returns the configured level if this directive applies to the target.
    ///
    /// A directive matches when the target starts with the configured target
    /// prefix. If no target prefix is configured, the directive acts as the
    /// global default and always matches.
    pub fn level_for(&self, target: &str) -> Option<FilterLevel> {
        match &self.target {
            Some(name) => {
                if target.starts_with(name) {
                    Some(self.level)
                } else {
                    None
                }
            }
            None => Some(self.level),
        }
    }
}
