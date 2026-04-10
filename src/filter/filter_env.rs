use super::FilterLevel;
use super::FilterTarget;

#[derive(Debug)]
pub struct FilterEnv {
    env: String,
}

impl FilterEnv {
    pub fn from_env_var(var_name: &str) -> Option<Self> {
        std::env::var(var_name).ok().map(|env| Self { env })
    }

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

            let (module, level_str) = match second {
                Some(lvl) => (first, lvl),
                None => ("", first),
            };

            let level = level_str.parse::<FilterLevel>().unwrap_or(FilterLevel::Off);

            let target = if module.is_empty() {
                None
            } else {
                Some(module.to_owned())
            };

            out.push(FilterTarget::new(target, level));
        }

        out
    }
}
