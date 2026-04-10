use crate::record::FilterLevel;

#[derive(Clone, Debug)]
pub struct FilterTarget {
    target: Option<String>,
    level: FilterLevel,
}

impl FilterTarget {
    pub fn new(target: Option<String>, level: FilterLevel) -> Self {
        Self { target, level }
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn level(&self) -> FilterLevel {
        self.level
    }

    pub fn level_for(&self, target: &str) -> Option<FilterLevel> {
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
