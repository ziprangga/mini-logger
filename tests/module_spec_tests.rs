use simple_debug::*;

fn main() {
    println!("Initializing global logger once for all tests...");
    DebugLog::init(Some(&[
        ("mycrate", Level::Error),
        ("othercrate", Level::Debug),
    ]));

    // -------------------------------
    // Test module-specific log levels
    // -------------------------------
    {
        // No re-init, uses the same global log
        assert!(Level::Error.enabled("mycrate::sub"));
        assert!(!Level::Info.enabled("mycrate::sub"));
        assert!(Level::Debug.enabled("othercrate::sub"));

        println!("test_module_specific_levels passed");
    }
}
