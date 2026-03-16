use mini_logger::*;

fn main() {
    // // Initialize the global logger
    // mini_logger::init(); // sets defaults from RUST_LOG or Info

    // // Log messages at different levels
    // error!("This is an error");
    // warn!("This is a warning");
    // info!("This is info");
    // debug!("This is debug");
    // trace!("This is trace");

    //You can also test with the builder
    mini_logger::Builder::new()
        .filter_level(Level::Info)
        .default_format()
        .output_stdout()
        .init();

    info!("Another info after builder init");
}
