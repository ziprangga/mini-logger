use mini_logger::*;

/// Demonstrates the default logger configuration.
///
/// `init()` initializes the logger using the default configuration:
///
/// - Loads filter directives from the `RUST_LOG` environment variable.
/// - Enables panic hook integration (when the feature is enabled).
/// - Writes log output to standard output.
///
/// # Example
///
/// ```text
/// RUST_LOG=info
/// ```
///
/// or
///
/// ```text
/// RUST_LOG=sample=debug
/// ```
pub fn default() {
    mini_logger::init();

    info!("MINI_LOGGER");
    info!("This use default");
}
