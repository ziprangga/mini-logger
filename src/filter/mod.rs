//! Filtering system for controlling which log records are emitted.
//!
//! A filter consists of two independent parts:
//! - **Level filtering** determines whether a log level is enabled for a target.
//! - **Message filtering** optionally matches log messages by substring.
//!
//! Filtering can be configured programmatically with [`FilterBuilder`] or loaded
//! from an environment variable using [`FilterEnv`].
//!
//! # Filter Configuration
//!
//! Filters can be configured in two ways:
//!
//! ## 1. Programmatically
//!
//! Configure filters directly in code.
//!
//! ```rust
//! use mini_logger::{FilterBuilder, FilterLevel};
//!
//! let filter = FilterBuilder::new()
//!     .filter_target(None, FilterLevel::Info)
//!     .filter_target(Some("my_crate"), FilterLevel::Debug)
//!     .build();
//! ```
//!
//! ## 2. Environment Variable
//!
//! Load filter directives from an environment variable.
//!
//! ```text
//! MY_LOG=info,my_crate=debug,network=trace
//! ```
//!
//! ```rust
//! use mini_logger::FilterBuilder;
//!
//! let filter = FilterBuilder::new()
//!     .filter_env("MY_LOG")
//!     .build();
//! ```
//!
//! Both approaches can also be combined. When the same target is configured
//! multiple times, the later configuration replaces the earlier one.

mod filter_env;
mod filter_level;
mod filter_target;

pub use filter_env::FilterEnv;
pub use filter_level::FilterLevel;
pub use filter_target::FilterTarget;

use crate::record::RecMessage;

/// Compiled filter used to determine whether a log record should be emitted.
///
/// A filter evaluates:
/// - the record target,
/// - the record level,
/// - and optionally whether the message contains a configured substring.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    filter_target: Vec<FilterTarget>,
    filter_string: Option<String>,
}

impl Filter {
    /// Returns the highest log level configured by this filter.
    pub fn max_level(&self) -> FilterLevel {
        self.filter_target
            .iter()
            .map(|d| d.level())
            .max()
            .unwrap_or(FilterLevel::Off)
    }

    /// Returns `true` if the log record passes both level and message filtering.
    pub fn matches(&self, record_msg: &RecMessage<'_>) -> bool {
        if !self.enabled(record_msg.target(), &record_msg.level()) {
            return false;
        }

        if !self.is_match(&record_msg.msg().to_string()) {
            return false;
        }
        true
    }

    /// Returns whether the message satisfies the configured string filter.
    ///
    /// If no message filter is configured, every message matches. Otherwise,
    /// the message must contain the configured substring.
    fn is_match(&self, s: &str) -> bool {
        match &self.filter_string {
            Some(f) => s.contains(f),
            None => true,
        }
    }

    /// Determines whether a log level is enabled for the given target.
    ///
    /// The effective level is computed by evaluating every matching directive.
    /// Directives are sorted by target length during `build()`, so broader
    /// target prefixes are evaluated first and more specific prefixes override
    /// them.
    fn enabled(&self, target: &str, log_level: &FilterLevel) -> bool {
        let mut level = FilterLevel::Off;

        for d in &self.filter_target {
            if let Some(lvl) = d.level_for(target) {
                level = lvl;
            }
        }
        *log_level <= level
    }
}

/// Builder for constructing a [`Filter`].
///
/// Supports configuration from code or environment variables. Duplicate
/// targets replace earlier definitions, and target directives are ordered
/// from least-specific to most-specific before the filter is built.
#[derive(Debug)]
pub struct FilterBuilder {
    filter: Filter,
}

impl FilterBuilder {
    /// Creates a new filter builder with the default configuration.
    pub fn new() -> Self {
        Self {
            filter: Filter::default(),
        }
    }

    /// Inserts a filter directive.
    ///
    /// If another directive exists for the same target, it is replaced.
    /// This guarantees that each target appears at most once before
    /// the filter is built.
    fn insert_filter(&mut self, mut filter_target: FilterTarget) {
        if let Some(pos) = self
            .filter
            .filter_target
            .iter()
            .position(|d| d.target() == filter_target.target())
        {
            std::mem::swap(&mut self.filter.filter_target[pos], &mut filter_target);
        } else {
            self.filter.filter_target.push(filter_target);
        }
    }

    /// Adds or replaces a target-specific filter directive.
    pub fn filter_target(&mut self, target: Option<&str>, level: FilterLevel) -> &mut Self {
        self.insert_filter(FilterTarget::new(target.map(|s| s.to_owned()), level));
        self
    }

    /// Restricts output to log messages containing the given substring.
    pub fn filter_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.filter_string = Some(s.into());
        self
    }

    /// Loads filter directives from an environment variable.
    pub fn filter_env(&mut self, var_name: &str) -> &mut Self {
        if let Some(env) = FilterEnv::from_env_var(var_name) {
            for filter_target in env.parse_filter_string() {
                self.insert_filter(filter_target);
            }
        }
        self
    }

    /// Builds the final filter.
    ///
    /// If no filter directives are configured, a global `Debug` filter is
    /// inserted.
    ///
    /// Target directives are sorted by target length (shortest first).
    /// During matching, every matching directive updates the effective level,
    /// allowing more specific target prefixes to override broader ones.
    pub fn build(mut self) -> Filter {
        let mut filter_target = Vec::new();

        if self.filter.filter_target.is_empty() {
            filter_target.push(FilterTarget::new(None, FilterLevel::Debug));
        } else {
            filter_target = std::mem::take(&mut self.filter.filter_target);
            filter_target.sort_by(|a, b| {
                let alen = a.target().as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.target().as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        Filter {
            filter_target: std::mem::take(&mut filter_target),
            filter_string: std::mem::take(&mut self.filter.filter_string),
        }
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
