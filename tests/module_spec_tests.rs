use mini_logger::*; // your new logger crate

fn main() {
    println!("Initializing global logger once for all tests...");

    // Initialize global logger with module-specific levels
    Builder::new()
        .filter_module("mycrate", Level::Error)
        .filter_module("othercrate", Level::Debug)
        .output_stdout()
        .try_init()
        .expect("Failed to initialize mini_logger");

    // -------------------------------
    // Test module-specific log levels
    // -------------------------------
    {
        // Access global logger
        let logger = LOGGER.get().expect("Logger not initialized");

        // Check if logger enables messages for specific modules
        assert!(
            logger
                .filter()
                .matches_module_level("mycrate::sub", Level::Error)
        );
        assert!(
            !logger
                .filter()
                .matches_module_level("mycrate::sub", Level::Info)
        );
        assert!(
            logger
                .filter()
                .matches_module_level("othercrate::sub", Level::Debug)
        );

        println!("test_module_specific_levels passed");
    }
}
