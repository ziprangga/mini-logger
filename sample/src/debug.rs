use simple_debug::{debug_dev, info_dev};
pub fn run_debug() {
    debug_dev!("Hello from debug_dev!");
    info_dev!("Hello from info_dev!");

    for i in 0..3 {
        std::thread::spawn(move || {
            for j in 0..5 {
                simple_debug::debug_dev!("thread {} log {}", i, j);
                println!("this is thread")
            }
        });
    }
}
