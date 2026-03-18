#[derive(Clone, Copy, Debug)]
pub enum Color {
    Reset,
    Red,
    Green,
    Yellow,
    Blue,
}

impl Color {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reset => "\x1b[0m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl ColorMode {
    fn enabled(self) -> bool {
        use std::io::IsTerminal;
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }

    pub fn color(self, color: Color) -> &'static str {
        if self.enabled() { color.as_str() } else { "" }
    }

    pub fn reset(self) -> &'static str {
        if self.enabled() {
            Color::Reset.as_str()
        } else {
            ""
        }
    }
}

impl std::str::FromStr for ColorMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Ok(ColorMode::default()),
        }
    }
}
