/// ANSI terminal colors used by the built-in formatter.
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Reset,
    Red,
    Green,
    Yellow,
    Blue,
}

impl Color {
    /// Returns the ANSI escape sequence for the color.
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

/// Controls when ANSI colors are emitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ColorMode {
    /// Enable colors only when writing to a terminal.
    #[default]
    Auto,
    /// Always emit ANSI color escape sequences.
    Always,
    /// Never emit ANSI color escape sequences.
    Never,
}

impl ColorMode {
    /// Returns whether ANSI colors should be emitted.
    fn is_enabled(self) -> bool {
        use std::io::IsTerminal;
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }

    /// Returns the ANSI escape sequence for the color.
    ///
    /// Returns an empty string when colors are disabled.
    pub fn color(self, color: Color) -> &'static str {
        if self.is_enabled() {
            color.as_str()
        } else {
            ""
        }
    }

    /// Returns the ANSI reset escape sequence.
    ///
    /// Returns an empty string when colors are disabled.
    pub fn reset(self) -> &'static str {
        if self.is_enabled() {
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
