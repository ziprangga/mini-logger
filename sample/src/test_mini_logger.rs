use mini_logger::*;
use std::io::Write;

// default use the env "RUST_LOG" to configuration level and target
fn default() {
    mini_logger::init();

    info!("MINI_LOGGER");
    info!("This use default");
}

// custom use ".filter(Some("sample::test_mini_logger"), LogLevel::Debug)" to configuration level and target
// this configuration can be combine using ".env_default()" or ".from_env",
// Builder::new() .env_default() .filter(Some("my_crate::submodule"), FilterLevel::Info) .output_stdout() .init();
// Both apply.
// env_default() → loads rules from "RUST_LOG"
// .filter(Some(...), ...) → adds/overrides for that target
// Final filter = env rules + your manual rule
// If same target appears in both → last one wins ".filter(Some(..), ..)"

fn custom() {
    mini_logger::Builder::new()
        .filter(Some("sample::test_mini_logger"), LogLevel::Debug)
        .format_custom(|buf, message| {
            let color = match message.level() {
                LogLevel::Off => Color::Reset,
                LogLevel::Error => Color::Red,
                LogLevel::Warn => Color::Yellow,
                LogLevel::Info => Color::Green,
                LogLevel::Debug => Color::Blue,
                LogLevel::Trace => Color::Blue,
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

fn multi_thread() {
    info!("this from another module");

    let mut handles = Vec::new();

    for i in 0..4 {
        let handle = std::thread::spawn(move || {
            let msg = format!(
                "this from another module with multiple thread, thread {}",
                i
            );
            debug!("{}", msg);
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}

pub fn run_test_mini_logger(def: bool) {
    if def {
        default()
    } else {
        custom()
    }

    error!("This is an error");
    warn!("This is a warning");
    info!("This is info");
    debug!("This is debug");
    trace!("This is trace");

    info!("Another info after builder init");
    debug!("======================");
    multi_thread();
    debug!("======================");
    info!("END_OF_MINI_LOGGER");
    debug!("======================");
}
