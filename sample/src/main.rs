use simple_debug::{DebugLog, debug_dev, info_dev};
fn main() {
    DebugLog::init(None, false, true);

    debug_dev!("Hello from debug_dev!");
    info_dev!("Hello from info_dev!");
}
