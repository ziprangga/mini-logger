mod debug;
mod ext_debug;

fn main() {
    ext_debug::init_ext_debug();

    run_test();

    debug::run_debug();
}

fn run_test() {
    log::debug!("Halo from log::debug!");
    ext_debug_log!("Halo from ext_debug macro");
}
