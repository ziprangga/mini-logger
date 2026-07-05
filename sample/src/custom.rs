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
        .filter(Some("sample"), FilterLevel::Debug)
        .format_custom(|buf, message| {
            let color = match message.level() {
                FilterLevel::Off => Color::Reset,
                FilterLevel::Error => Color::Red,
                FilterLevel::Warn => Color::Yellow,
                FilterLevel::Info => Color::Green,
                FilterLevel::Debug => Color::Blue,
                FilterLevel::Trace => Color::Blue,
            };
            let level_str = message.level().as_str();
            let ts = buf.timestamp();
            let color_mode = ColorMode::Auto;

            writeln!(
                buf,
                "{} [{}{}{}] - {}",
                ts,
                color_mode.color(color), // Start style
                level_str,
                color_mode.reset(), // Reset style
                message.msg()
            )
        })
        .output_stdout()
        .init();

    info!("MINI_LOGGER");
    info!("This use custom");
}
