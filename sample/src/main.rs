//! Example runner.
//!
//! Each module demonstrates a different aspect of `mini_logger`.
//!
//! # Note
//!
//! Logger initialization is global and can only occur once per process.
//! Therefore, only one initialization example should be run at a time.
//!
//! Uncomment either `custom` or `default` to initialize the logger.
//! The `multi_thread` example can be combined with either initialization
//! example to demonstrate logging from multiple threads.
//!
//! See the documentation in the `custom` and `default` modules for
//! configuration details and usage notes.

// mod custom;
mod default;
mod multi_thread;
// mod use_builder;

fn main() {
    // custom::custom();
    default::default();
    // use_builder::use_builder();
    multi_thread::multi_thread();
}
