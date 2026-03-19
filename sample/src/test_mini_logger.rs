use mini_logger::*;
use std::io::Write;

fn default() {
    mini_logger::init();

    info!("MINI_LOGGER");
    info!("This use default");
}

fn custom() {
    mini_logger::Builder::new()
        .env_default()
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
