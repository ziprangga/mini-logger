mod filter_env;
mod filter_level;
mod filter_target;

pub use filter_env::FilterEnv;
pub use filter_level::FilterLevel;
pub use filter_target::FilterTarget;

use crate::record::RecMessage;

#[derive(Clone, Debug, Default)]
pub struct Filter {
    filter_target: Vec<FilterTarget>,
    filter_string: Option<String>,
}

impl Filter {
    pub fn max_level(&self) -> FilterLevel {
        self.filter_target
            .iter()
            .map(|d| d.level())
            .max()
            .unwrap_or(FilterLevel::Off)
    }

    pub fn matches(&self, record_msg: &RecMessage<'_>) -> bool {
        if !self.enabled(record_msg.target(), &record_msg.level()) {
            return false;
        }

        if !self.is_match(&record_msg.msg().to_string()) {
            return false;
        }
        true
    }

    fn is_match(&self, s: &str) -> bool {
        match &self.filter_string {
            Some(f) => s.contains(f),
            None => true,
        }
    }

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

#[derive(Debug)]
pub struct FilterBuilder {
    filter: Filter,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            filter: Filter::default(),
        }
    }

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

    pub fn filter_target(&mut self, module: Option<&str>, level: FilterLevel) -> &mut Self {
        self.insert_filter(FilterTarget::new(module.map(|s| s.to_owned()), level));
        self
    }

    pub fn filter_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.filter_string = Some(s.into());
        self
    }

    pub fn filter_env(&mut self, var_name: &str) -> &mut Self {
        if let Some(env) = FilterEnv::from_env_var(var_name) {
            for filter_target in env.parse_filter_string() {
                self.insert_filter(filter_target);
            }
        }
        self
    }

    pub fn build(mut self) -> Filter {
        let mut filter_target = Vec::new();

        if self.filter.filter_target.is_empty() {
            filter_target.push(FilterTarget::new(None, FilterLevel::Error));
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
