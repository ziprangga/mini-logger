# mini-logger

A **lightweight, thread-safe Rust logger** designed for simplicity, flexibility, and minimal boilerplate. Supports custom formatting, color output, file logging, and panic capture.

Inspired by env_logger and log. Implemented as a lightweight custom logger based on their design, with adapted patterns to fit project-specific goals.


## Features

* Configurable color modes for stdout/stderr.
* Output to stdout, stderr, or file.
* Panic hook integration to automatically log panics.
* Thread-local buffers for fast logging.
* Custom formatting functions for advanced use.
* Environment-variable-based filtering (`RUST_LOG`) or custom filters per module.
* Simple macros for levels: `error!`, `warn!`, `info!`, `debug!`, `trace!`.

## Example

A complete working example is included in the [`sample`](./sample) crate.

### Run the example

```bash
cd sample
cargo run
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mini-logger = "0.1"
```

## Usage

### Initialize a simple logger

```rust
use mini_logger::{Builder, LogLevel};

// Initialize with default environment filter and stdout output
Builder::new()
    .env_default()
    .output_stdout()
    .init();
```
or

```rust    
// Same initialization, but non-panicking (skip init if it already initialize)
Builder::new()
    .env_default()
    .output_stdout()
    .try_init();
```

### Logging messages

```rust
info!("This is an info message");
warn!("This is a warning");
error!("This is an error");
debug!("Debug details: {}", 26);
trace!("Trace output here");
```

### Using a custom logger

```rust
use mini_logger::{Builder, Logger, LogLevel};

let logger = Builder::new()
    .filter(Some("my_module"), LogLevel::Debug)
    .output_file("app.log")
    .build();

log!(logger: logger, LogLevel::Info, "Custom logger message");
```

### Custom formatting

```rust
use mini_logger::{Builder, LogMessage, BufferFormatter};

Builder::new().format_custom(|buf, msg: &LogMessage| {
    writeln!(buf, "[{}] {}", msg.level(), msg.msg)
}).init();
```

### Panic logging

```rust
Builder::new()
    .trigger_panic_to_output() // automatically logs panics
    .output_file("panic.log")
    .init();
```

## License

MIT or Apache-2.0 (choose one).
