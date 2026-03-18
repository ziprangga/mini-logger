use mini_logger::*;
mod try_module;

fn init_using_builder_or_direct(direct: bool) {
    if direct {
        mini_logger::init()
    } else {
        mini_logger::Builder::new()
            .env_default()
            .output_stdout()
            .init()
    }
}

fn main() {
    // // Initialize the global logger
    init_using_builder_or_direct(true);

    // Log messages at different levels
    error!("This is an error");
    warn!("This is a warning");
    info!("This is info");
    debug!("This is debug");
    trace!("This is trace");

    info!("Another info after builder init");

    try_module::run()
}
