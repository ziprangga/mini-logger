//! Parses logger filter directives from an environment variable.
//!
//! The parser converts a comma-separated filter string into a collection
//! of [`FilterTarget`] directives.
//!
//! Supports directives in the form:
//!
//! - `info`
//! - `debug,my_crate=trace`
//! - `warn,network=debug,db=trace`
//!
//! A directive without a target becomes the global filter level, while
//! `target=level` applies only to log targets with the specified prefix.

use super::FilterLevel;
use super::FilterTarget;

/// Filter directives loaded from an environment variable.
#[derive(Debug)]
pub struct FilterEnv {
    env: String,
}

impl FilterEnv {
    /// Loads filter directives from an environment variable.
    ///
    /// Returns `None` if the variable does not exist.
    pub fn from_env_var(var_name: &str) -> Option<Self> {
        std::env::var(var_name).ok().map(|env| Self { env })
    }

    /// Parses the filter string into target filter directives.
    ///
    /// Directives may be either `level` for the global filter or
    /// `target=level` for a target-specific filter.
    ///
    /// Invalid log levels are treated as [`FilterLevel::Off`].
    pub fn parse_filter_string(self) -> Vec<FilterTarget> {
        let mut out = Vec::new();

        for directive in self.env.split(',') {
            let directive = directive.trim();
            if directive.is_empty() {
                continue;
            }

            let mut parts = directive.splitn(2, '=');
            let first = parts.next().unwrap().trim();
            let second = parts.next().map(|s| s.trim());

            let (target, level_str) = match second {
                Some(lvl) => (first, lvl),
                None => ("", first),
            };

            let level_filter = level_str.parse::<FilterLevel>().unwrap_or(FilterLevel::Off);

            let target_filter = if target.is_empty() {
                None
            } else {
                Some(target.to_owned())
            };

            out.push(FilterTarget::new(target_filter, level_filter));
        }

        out
    }
}
