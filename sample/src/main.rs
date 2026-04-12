mod test_mini_logger;

fn main() {
    let arg = std::env::args().nth(1);
    let flag = matches!(arg.as_deref(), Some("true"));

    test_mini_logger::run_test_mini_logger(flag);
}
