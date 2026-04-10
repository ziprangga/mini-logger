mod filter_target;

pub use filter_target::FilterTarget;

use crate::record::{LogLevel, RecMessage};

#[derive(Clone, Debug, Default)]
pub struct Filter {
    filter_target: Vec<FilterTarget>,
    filter_string: Option<String>,
}

impl Filter {
    pub fn max_level(&self) -> LogLevel {
        self.filter_target
            .iter()
            .map(|d| d.level())
            .max()
            .unwrap_or(LogLevel::Off)
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

    fn enabled(&self, target: &str, log_level: &LogLevel) -> bool {
        let mut level = LogLevel::Off;

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

    pub fn filter_target(&mut self, module: Option<&str>, level: LogLevel) -> &mut Self {
        // self.insert_filter(FilterTarget {
        //     target: module.map(|s| s.to_owned()),
        //     level,
        // });
        self.insert_filter(FilterTarget::new(module.map(|s| s.to_owned()), level));
        self
    }

    pub fn filter_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.filter_string = Some(s.into());
        self
    }

    pub fn build(mut self) -> Filter {
        let mut filter_target = Vec::new();

        if self.filter.filter_target.is_empty() {
            // filter_target.push(FilterTarget {
            //     target: None,
            //     level: LogLevel::Error,
            // });
            filter_target.push(FilterTarget::new(None, LogLevel::Error));
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

pub struct FilterEnv<'a> {
    builder: &'a mut FilterBuilder,
}

impl<'a> FilterEnv<'a> {
    pub fn new(builder: &'a mut FilterBuilder) -> Self {
        Self { builder }
    }

    pub fn parse_env_var(&mut self, var_name: &str) -> &mut Self {
        if let Ok(env_val) = std::env::var(var_name) {
            self.parse_filter_string(&env_val);
        }
        self
    }

    pub fn parse_filter_string(&mut self, s: &str) -> &mut Self {
        for directive in s.split(',') {
            let directive = directive.trim();
            if directive.is_empty() {
                continue;
            }

            // Split by '='
            let mut parts = directive.splitn(2, '=');
            let first = parts.next().unwrap().trim();
            let second = parts.next().map(|s| s.trim());

            let (module, level_str) = match second {
                // module=level
                Some(lvl) => (first, lvl),
                // bare level → global
                None => ("", first),
            };

            let level = level_str.parse::<LogLevel>().unwrap_or(LogLevel::Off);

            let module_opt = if module.is_empty() {
                None
            } else {
                Some(module)
            };

            self.builder.filter_target(module_opt, level);
        }
        self
    }
}
