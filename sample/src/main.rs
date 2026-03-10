mod debug;
mod ext_debug;

fn main() {
    let debug_log = simple_debug::DebugLog::init(None);
    ext_debug::init_ext_debug();
    // simple_debug::Level::set_level(simple_debug::Level::Info);

    debug::run_debug();
    ext_debug::run_ext_debug();

    if let Some(content) = debug_log.get_log_from_buffer() {
        println!("Buffer content:\n{}, \nBatas", content);
    };
}
