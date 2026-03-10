use simple_debug::{debug_dev, info_dev};
pub fn run_debug() {
    debug_dev!("Hello from debug_dev!");
    info_dev!("Hello from info_dev!");
}

// mod debug;
// mod ext_debug;

// fn main() {
//     let debug_log = simple_debug::DebugLog::init(None, true, false);
//     ext_debug::init_ext_debug();
//     // simple_debug::Level::set_level(simple_debug::Level::Info);

//     debug::run_debug();
//     ext_debug::run_ext_debug();

//     let content = debug_log.get_debug_buffer();
//     println!("Buffer content:\n{}, \nBatas", content);
// }
