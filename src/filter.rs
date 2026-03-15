use crate::logger::{Level, LogConfig};

#[derive(Clone, Debug)]
pub struct Filter {
    directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct Directive {
    name: Option<String>,
    level: Level,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            directives: Vec::new(),
        }
    }

    pub fn add(&mut self, name: Option<String>, level: Level) {
        self.directives.push(Directive { name, level });
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
