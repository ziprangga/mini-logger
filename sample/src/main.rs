mod test_mini_logger;

// ============================================================================
// RUN CUSTOM COLOR MODE (Default Behavior):
//    $ cargo run
//    -> Runs `custom()`. Uses code-defined filters and applies ANSI color styles.
//
// RUN SYSTEM ENVIRONMENT LOGGER MODE:
//    $ cargo run -- true
//    -> Runs `default()`. Relies entirely on the "RUST_LOG" environment variable.
//
// RUN ENVIRONMENT LOGGER WITH TARGET OVERRIDES:
//    Linux/macOS:  $ RUST_LOG=debug cargo run -- true
//    PowerShell:   $env:RUST_LOG="debug"; cargo run -- true
//    -> Forces the system backend to expose all logs down to the Debug tier.
// ============================================================================
fn main() {
    let arg = std::env::args().nth(1);
    let flag = matches!(arg.as_deref(), Some("true"));

    test_mini_logger::run_test_mini_logger(flag);
}
