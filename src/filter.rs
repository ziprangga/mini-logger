use crate::logger::{Level, LogConfig};

#[derive(Clone, Debug)]
pub struct Filter<'a> {
    name: Option<String>,
    log_config: LogConfig<'a>,
}

impl<'a> Filter<'a> {
    pub fn builder() -> FilterBuilder<'a> {
        FilterBuilder::new()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn log_config(&self) -> &LogConfig<'a> {
        &self.log_config
    }

    /// Returns true if a log record passes the filter
    pub fn matches(&self, record_target: &str, record_level: Level) -> bool {
        // Level must be equal or higher
        let level_ok = (record_level as usize) <= (self.log_config.level() as usize);

        // Target must match (or empty = wildcard)
        let target_ok =
            self.log_config.target().is_empty() || self.log_config.target() == record_target;

        level_ok && target_ok
    }
}

impl<'a> Default for Filter<'a> {
    fn default() -> Self {
        Self {
            name: None,
            log_config: LogConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FilterBuilder<'a> {
    name: Option<String>,
    log_config: LogConfig<'a>,
}

impl<'a> FilterBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            log_config: LogConfig::default(),
        }
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn log_config(&mut self, log_config: LogConfig<'a>) -> &mut Self {
        self.log_config = log_config;
        self
    }

    pub fn build(&self) -> Filter<'a> {
        Filter {
            name: self.name.clone(),
            log_config: self.log_config,
        }
    }
}
