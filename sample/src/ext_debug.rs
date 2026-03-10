// -------------------------------
// Debug-only Logger Init
// -------------------------------
#[allow(dead_code)]
pub fn init_ext_debug() {
    // Initialize logger; ignore errors if already initialized
    let _ = env_logger::builder()
        .filter_module("sample", log::LevelFilter::Debug)
        .try_init();
}

// -------------------------------
// Debug-only macro
// -------------------------------
#[macro_export]
macro_rules! ext_debug_log {
    ($($arg:tt)*) => {
        {
            ::log::debug!($($arg)*);
        }
    };
}

pub fn run_ext_debug() {
    log::debug!("Halo from log::debug!");
    ext_debug_log!("Halo from ext_debug macro");
}
