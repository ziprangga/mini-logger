use mini_logger::*;
use std::io::Write;

/// Demonstrates configuring the logger entirely in code.
///
/// This example:
///
/// - Sets the log filter programmatically.
/// - Installs a custom formatter.
/// - Writes log output to standard output.
///
/// # Filter configuration
///
/// The call:
///
/// ```rust,no_run
/// .filter(Some("sample::test_mini_logger"), FilterLevel::Debug)
/// ```
///
/// enables `Debug` and higher severity log records for the
/// `sample::test_mini_logger` target.
///
/// # Note
///
/// This example does **not** load filter directives from an environment
/// variable.
///
/// Programmatic filters can also be combined with `env_default()` or
/// `from_env()`. When the same target is configured multiple times,
/// the last configuration takes precedence.
pub fn custom() {
    mini_logger::Builder::new()
        .filter(Some("sample"), Level::Debug)
        .format_with(
            |formatter: &Formatter, buffer: &mut Buffer, message: &RecMessage<'_>| {
                let color = match message.level() {
                    Level::Off => Color::Reset,
                    Level::Error => Color::Red,
                    Level::Warn => Color::Yellow,
                    Level::Info => Color::Green,
                    Level::Debug => Color::Blue,
                    Level::Trace => Color::Blue,
                };
                let level_str = message.level().as_str();
                let ts = formatter
                    .style()
                    .time_mode()
                    .timestamp(TimestampPrecision::default());
                let color_mode = ColorMode::Auto;

                writeln!(
                    buffer,
                    "{} [{}{}{}] - {}",
                    ts,
                    color_mode.color(color), // Start style
                    level_str,
                    color_mode.reset(), // Reset style
                    message.msg()
                )
            },
        )
        .output_stdout()
        .init();

    info!("MINI_LOGGER");
    debug!("custom debug");
    info!("This use custom");
}
