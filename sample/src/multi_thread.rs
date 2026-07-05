use mini_logger::*;
/// Demonstrates logging from multiple threads.
///
/// # Note
///
/// This example does not initialize the logger.
///
/// Call `mini_logger::init()` or initialize a [`Builder`] before invoking
/// this function. When using the examples in this directory, run this
/// example together with either the `default` or `custom` module, which
/// performs the global logger initialization.
///
/// Each thread reuses its own thread-local formatter to reduce buffer
/// allocations during logging.
pub fn multi_thread() {
    info!("=============this from another module==============");

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
