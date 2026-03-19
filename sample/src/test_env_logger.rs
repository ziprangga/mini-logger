use env_logger::fmt::style;
use log::{debug, error, info, warn};
use std::io::Write;

fn default_env_logger() {
    env_logger::builder()
        .format_timestamp_secs() // Default is seconds
        .init();

    info!("ENV_LOGGER");
    info!("This uses the standard default format.");
}

fn custom_env_logger() {
    env_logger::builder()
        .format(|buf, record| {
            // 1. Handle Colors/Styles
            let level_style = match record.level() {
                log::Level::Error => style::AnsiColor::Red
                    .on_default()
                    .effects(style::Effects::BOLD),
                log::Level::Warn => style::AnsiColor::Yellow.on_default(),
                log::Level::Info => style::AnsiColor::Green.on_default(),
                _ => style::Style::new(),
            };

            // 2. Handle Timestamps (requires 'humantime' feature, enabled by default)
            let ts = buf.timestamp();

            // 3. Write the final formatted line
            writeln!(
                buf,
                "{} [{}{}{}] - {}",
                ts,
                level_style.render(), // Start style
                record.level(),
                level_style.render_reset(), // Reset style
                record.args()
            )
        })
        .init();

    info!("ENV_LOGGER");
    info!("This uses the custom format.");
}

pub fn run_test_env_logger(default: bool) {
    if default {
        default_env_logger()
    } else {
        custom_env_logger()
    }

    info!("System initialized.");
    warn!("Low disk space.");
    error!("Critical failure!");

    debug!("======================");
    info!("END_OF_ENV_LOGGER");
    debug!("======================");
}
