use mini_logger::*;

pub fn use_builder() {
    Builder::new()
        .env_default()
        .color_mode(ColorMode::Never)
        .output_stdout()
        .init();

    info!("MINI_LOGGER");
    debug!("use builder debug");
    info!("This use use builder init");
}
