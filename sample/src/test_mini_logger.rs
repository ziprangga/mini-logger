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
        .format(|buf, message| {
            let (level_str, level_color) = match message.level() {
                LogLevel::Off => ("OFF", Color::Reset),
                LogLevel::Error => ("ERROR", Color::Red),
                LogLevel::Warn => ("WARN", Color::Yellow),
                LogLevel::Info => ("INFO", Color::Green),
                LogLevel::Debug => ("DEBUG", Color::Blue),
                LogLevel::Trace => ("TRACE", Color::Blue),
            };

            let ts = buf.timestamp();
            let color_mode = ColorMode::Auto;

            writeln!(
                buf,
                "{} [{}{}{}] - {}",
                ts,
                color_mode.color(level_color), // Start style
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
