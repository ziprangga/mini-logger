mod test_env_logger;
mod test_mini_logger;

fn main() {
    // // Initialize the global logger
    test_mini_logger::run_test_mini_logger(false);

    test_env_logger::run_test_env_logger(false);
}
