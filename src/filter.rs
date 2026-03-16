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
    built: bool,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            filter: Filter::new(),
            built: false,
        }
    }

    fn insert_filter(&mut self, mut directive: Directive) {
        if let Some(pos) = self
            .filter
            .directives
            .iter()
            .position(|d| d.name == directive.name)
        {
            std::mem::swap(&mut self.filter.directives[pos], &mut directive);
        } else {
            self.filter.directives.push(directive);
        }
    }

    pub fn filter(&mut self, module: Option<&str>, level: Level) -> &mut Self {
        self.insert_filter(Directive {
            name: module.map(|s| s.to_owned()),
            level,
        });
        self
    }

    pub fn filter_module(&mut self, module: &str, level: Level) -> &mut Self {
        self.filter(Some(module), level)
    }

    pub fn filter_level(&mut self, level: Level) -> &mut Self {
        self.filter(None, level)
    }

    pub fn filter_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.filter.filter_string = Some(s.into());
        self
    }

    pub fn build(&mut self) -> Filter {
        // self.filter
        assert!(!self.built, "attempt to re-use consumed filter builder");
        self.built = true;

        let mut filter_directive = Vec::new();
        if self.filter.directives.is_empty() {
            filter_directive.push(Directive {
                name: None,
                level: Level::Error,
            });
        } else {
            filter_directive = std::mem::take(&mut self.filter.directives);
            filter_directive.sort_by(|a, b| {
                let alen = a.name.as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.name.as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        Filter {
            directives: std::mem::take(&mut filter_directive),
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
                Some(lvl) => (first, lvl), // module=level
                None => ("", first),       // bare level → global
            };

            let level = level_str.parse::<Level>().unwrap_or(Level::Off);

            if module.is_empty() {
                self.builder.filter_level(level);
            } else {
                self.builder.filter_module(module, level);
            }
        }
        self
    }
}
