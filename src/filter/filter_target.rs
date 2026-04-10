use crate::record::LogLevel;

#[derive(Clone, Debug)]
pub struct FilterTarget {
    target: Option<String>,
    level: LogLevel,
}

impl FilterTarget {
    pub fn new(target: Option<String>, level: LogLevel) -> Self {
        Self { target, level }
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn level_for(&self, target: &str) -> Option<LogLevel> {
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
