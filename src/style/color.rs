/// ANSI terminal colors used by the built-in formatter.
///
/// Each variant represents a standard ANSI color escape sequence used when
/// rendering formatted log records.
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

/// Controls when ANSI color escape sequences are emitted.
///
/// The selected mode is stored as part of [`crate::style::Style`] and is
/// resolved during writer construction when [`ColorMode::Auto`] is used.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ColorMode {
    /// Automatically enable colors when output supports terminal rendering.
    ///
    /// The final decision is resolved by [`crate::writer::WriterBuilder`]
    /// during build, after the output destination is known.
    #[default]
    Auto,
    /// Always emit ANSI color escape sequences.
    Always,
    /// Never emit ANSI color escape sequences.
    Never,
}

impl ColorMode {
    /// Returns whether ANSI colors should currently be emitted.
    ///
    /// [`ColorMode::Auto`] checks terminal availability on standard output or
    /// standard error.
    fn is_enabled(self) -> bool {
        use std::io::IsTerminal;
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => std::io::stdout().is_terminal() || std::io::stderr().is_terminal(),
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
