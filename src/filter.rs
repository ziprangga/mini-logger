use crate::logger::{Level, LogConfig, LogMessage};

#[derive(Clone, Debug)]
pub struct Directive {
    name: Option<String>,
    level: Level,
}

#[derive(Clone, Debug)]
pub struct Filter {
    directives: Vec<Directive>,
    filter_string: Option<String>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            directives: Vec::new(),
            filter_string: None,
        }
    }

    pub fn filter(&self) -> Level {
        self.directives
            .iter()
            .map(|d| d.level)
            .max()
            .unwrap_or(Level::Off)
    }

    pub fn is_match(&self, s: &str) -> bool {
        match &self.filter_string {
            Some(f) => s.contains(f),
            None => true,
        }
    }

    pub fn matches(&self, record_msg: &LogMessage<'_>) -> bool {
        if !self.enabled(record_msg.log_config()) {
            return false;
        }

        if !self.is_match(&record_msg.msg().to_string()) {
            return false;
        }

        true
    }

    pub fn enabled(&self, config: &LogConfig<'_>) -> bool {
        let mut level = Level::Off;

        for d in &self.directives {
            match &d.name {
                Some(name) => {
                    if config.target().starts_with(name) {
                        level = d.level;
                    }
                }
                None => {
                    level = d.level;
                }
            }
        }

        config.level() <= level
    }
}

#[derive(Clone, Debug)]
pub struct FilterBuilder {
    filter: Filter,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            filter: Filter::new(),
        }
    }

    pub fn add(&mut self, name: Option<String>, level: Level) {
        self.filter.directives.push(Directive { name, level });
    }

    pub fn filter_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.filter_string = Some(s.into());
        self
    }

    pub fn build(self) -> Filter {
        self.filter
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
