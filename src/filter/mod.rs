//! Filtering system for controlling which log records are emitted.
//!
//! A filter consists of two independent parts:
//! - **Level filtering** determines whether a log level is enabled for a target.
//! - **Message filtering** optionally matches log messages by substring.
//!
//! Filtering can be configured programmatically with [`FilterBuilder`] or loaded
//! from an environment variable using [`FilterBuilder::filter_env`].
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
//! use mini_logger::{FilterBuilder, Level};
//!
//! let filter = FilterBuilder::new()
//!     .filter_target(None, Level::Info)
//!     .filter_target(Some("my_crate"), Level::Debug)
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

mod level;

pub use level::Level;

use crate::record::RecMessage;

/// A filter directive consisting of an optional target prefix
/// and an associated maximum enabled log level.
///
/// A `None` target represents the global/default filter level.
#[derive(Clone, Debug)]
pub struct Target {
    target: Option<String>,
    level: Level,
}

impl Target {
    /// Creates a new target filter directive.
    pub fn new(tgt: Option<String>, lvl: Level) -> Self {
        Self {
            target: tgt,
            level: lvl,
        }
    }

    /// Returns the target prefix, or `None` for the global directive.
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Returns the configured log level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns the configured level if this directive applies to the target.
    ///
    /// A directive matches when the target starts with the configured target
    /// prefix. If no target prefix is configured, the directive acts as the
    /// global default and always matches.
    pub fn level_for(&self, tgt: &str) -> Option<Level> {
        match &self.target {
            Some(name) => {
                if tgt.starts_with(name) {
                    Some(self.level)
                } else {
                    None
                }
            }
            None => Some(self.level),
        }
    }
}

/// Compiled filter used to determine whether a log record should be emitted.
///
/// A filter evaluates:
/// - the record target,
/// - the record level,
/// - and optionally the record message substring.
///
/// A record must pass both level filtering and message filtering.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    targets: Vec<Target>,
    message: Option<String>,
}

impl Filter {
    pub fn builder() -> FilterBuilder {
        FilterBuilder::new()
    }

    /// Returns the highest log level configured by this filter.
    pub fn max_level(&self) -> Level {
        self.targets
            .iter()
            .map(|d| d.level())
            .max()
            .unwrap_or(Level::Off)
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
        match &self.message {
            Some(f) => s.contains(f),
            None => true,
        }
    }

    /// Determines whether a log level is enabled for the given target.
    ///
    /// Every matching directive is evaluated in order.
    ///
    /// Directives are sorted by prefix length during `build()`, causing
    /// broader prefixes to be applied first and more specific prefixes to
    /// replace the effective level later.
    fn enabled(&self, tgt: &str, lvl: &Level) -> bool {
        let mut level = Level::Off;

        for d in &self.targets {
            if let Some(lvl) = d.level_for(tgt) {
                level = lvl;
            }
        }
        *lvl <= level
    }
}

/// Builder for constructing a [`Filter`].
///
/// Supports configuration from code or environment variables.
///
/// Duplicate targets replace earlier definitions. Before building, target
/// directives are sorted from shorter prefixes to longer prefixes so that
/// more specific targets override broader target prefixes during matching.
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

    /// Adds or replaces a target-specific filter directive.
    pub fn filter_target(&mut self, tgt: Option<&str>, lvl: Level) -> &mut Self {
        self.insert_filter(Target::new(tgt.map(|s| s.to_owned()), lvl));
        self
    }

    /// Restricts output to log messages containing the given substring.
    pub fn filter_message(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.message = Some(s.into());
        self
    }

    /// Loads filter directives from an environment variable.
    ///
    /// The environment variable value must contain a comma-separated
    /// filter string such as:
    ///
    /// ```text
    /// MY_LOG=info,my_crate=debug
    /// ```
    ///
    /// Parsed directives are merged into the builder configuration.
    pub fn filter_env(&mut self, var_name: &str) -> &mut Self {
        let value = match std::env::var(var_name) {
            Ok(v) => v,
            Err(_) => return self,
        };

        for target in parse_var_str(&value) {
            self.insert_filter(target);
        }

        self
    }

    /// Builds the final filter.
    ///
    /// If no filter directives are configured, a global `Debug` filter is
    /// inserted.
    ///
    /// Target directives are sorted by target length (shortest first).
    /// This allows longer and more specific target prefixes to override
    /// broader matches during filtering.
    pub fn build(mut self) -> Filter {
        let mut in_targets = Vec::new();

        if self.filter.targets.is_empty() {
            in_targets.push(Target::new(None, Level::Debug));
        } else {
            in_targets = std::mem::take(&mut self.filter.targets);
            in_targets.sort_by(|a, b| {
                let alen = a.target().as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.target().as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        Filter {
            targets: std::mem::take(&mut in_targets),
            message: std::mem::take(&mut self.filter.message),
        }
    }

    /// Inserts a filter directive.
    ///
    /// If another directive exists for the same target, it is replaced.
    /// This guarantees that each target appears at most once before
    /// the filter is built.
    fn insert_filter(&mut self, mut tgt: Target) {
        if let Some(pos) = self
            .filter
            .targets
            .iter()
            .position(|d| d.target() == tgt.target())
        {
            std::mem::swap(&mut self.filter.targets[pos], &mut tgt);
        } else {
            self.filter.targets.push(tgt);
        }
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a comma-separated filter string into target directives.
///
/// Directives may be either:
///
/// - `level` for a global filter level.
/// - `target=level` for a target-specific filter.
///
/// Examples:
///
/// ```text
/// info
/// my_crate=debug
/// info,my_crate=debug,network=trace
/// ```
///
/// Invalid log levels are treated as [`Level::Off`].
fn parse_var_str(var_name: &str) -> Vec<Target> {
    let mut out = Vec::new();

    for directive in var_name.split(',') {
        let directive = directive.trim();
        if directive.is_empty() {
            continue;
        }

        let mut parts = directive.splitn(2, '=');
        let first = parts.next().unwrap().trim();
        let second = parts.next().map(|s| s.trim());

        let (target, level_str) = match second {
            Some(lvl) => (Some(first.to_owned()), lvl),
            None => (None, first),
        };

        let level = level_str.parse::<Level>().unwrap_or(Level::Off);

        out.push(Target::new(target, level));
    }

    out
}
