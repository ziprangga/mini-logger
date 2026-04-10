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

// pub struct FilterEnv<'a> {
//     builder: &'a mut FilterBuilder,
// }

// impl<'a> FilterEnv<'a> {
//     pub fn new(builder: &'a mut FilterBuilder) -> Self {
//         Self { builder }
//     }

//     pub fn parse_env_var(&mut self, var_name: &str) -> &mut Self {
//         if let Ok(env_val) = std::env::var(var_name) {
//             self.parse_filter_string(&env_val);
//         }
//         self
//     }

//     pub fn parse_filter_string(&mut self, s: &str) -> &mut Self {
//         for directive in s.split(',') {
//             let directive = directive.trim();
//             if directive.is_empty() {
//                 continue;
//             }

//             // Split by '='
//             let mut parts = directive.splitn(2, '=');
//             let first = parts.next().unwrap().trim();
//             let second = parts.next().map(|s| s.trim());

//             let (module, level_str) = match second {
//                 // module=level
//                 Some(lvl) => (first, lvl),
//                 // bare level → global
//                 None => ("", first),
//             };

//             let level = level_str.parse::<FilterLevel>().unwrap_or(FilterLevel::Off);

//             let module_opt = if module.is_empty() {
//                 None
//             } else {
//                 Some(module)
//             };

//             self.builder.filter_target(module_opt, level);
//         }
//         self
//     }
// }
