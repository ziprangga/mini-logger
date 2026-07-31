use crate::writer::Output;
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
/// When [`ColorMode::Auto`] is used, the final color behavior is determined
/// from the configured output destination.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ColorMode {
    /// Automatically enable colors when the configured output supports terminal
    /// rendering.
    ///
    /// The final decision is determined when the mode is resolved against an
    /// output destination.
    #[default]
    Auto,

    /// Always emit ANSI color escape sequences.
    Always,

    /// Never emit ANSI color escape sequences.
    Never,
}

impl ColorMode {
    /// Resolves the effective color mode for the specified output destination.
    ///
    /// When the mode is [`ColorMode::Auto`]:
    ///
    /// - terminal outputs resolve to [`ColorMode::Always`] when terminal support
    ///   is detected,
    /// - non-terminal outputs resolve to [`ColorMode::Never`].
    ///
    /// [`ColorMode::Always`] and [`ColorMode::Never`] are returned unchanged.
    pub fn resolve(self, output: &Output) -> Self {
        use std::io::IsTerminal;
        match self {
            Self::Auto => match output {
                Output::Stdout => {
                    if std::io::stdout().is_terminal() {
                        Self::Always
                    } else {
                        Self::Never
                    }
                }
                Output::Stderr => {
                    if std::io::stderr().is_terminal() {
                        Self::Always
                    } else {
                        Self::Never
                    }
                }
                Output::File(_) => Self::Never,
            },

            Self::Always => Self::Always,
            Self::Never => Self::Never,
        }
    }
}

impl std::str::FromStr for ColorMode {
    type Err = ();

    /// Parses a color mode from a string.
    ///
    /// Accepted values are:
    ///
    /// - `"auto"`
    /// - `"always"`
    /// - `"never"`
    ///
    /// Parsing is case-insensitive.
    ///
    /// Returns an error if the value is not recognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Err(()),
        }
    }
}
