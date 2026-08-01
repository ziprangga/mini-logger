// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mini logger crate.
//!
//! This crate provides a lightweight, configurable logging system with:
//!
//! - Level and target filtering
//! - Flexible record formatting
//! - ANSI color support
//! - Multiple output destinations (stdout, stderr, file)
//! - Thread-local buffering for performance
//! - Optional runtime control features
//! - Optional panic hook integration
//!
//! # Inspiration
//!
//! This crate is inspired by the [`log`](https://docs.rs/log) and
//! [`env_logger`](https://docs.rs/env_logger) crates.
//!
//! It follows the same general logging model while integrating functionality
//! that is often split across multiple crates.
//!
//! # Platform Support
//!
//! This crate does not support `no_std` environments.
//!
//! It is intended for high-level application development and relies on
//! standard library facilities such as file I/O, thread-local storage,
//! and synchronization primitives.
//!
//! # Architecture
//!
//! The crate is organized into several internal modules:
//!
//! - `logger` → global logger runtime and initialization
//! - `filter` → log filtering rules (level, target, environment support)
//! - `format` → log record formatting and rendering
//! - `writer` → output destinations and buffer management
//! - `record` → log record representation ([`RecMessage`])
//! - `style` → color, timestamp, and formatting-related configuration
//!
//! Internal-only modules:
//!
//! - `macros` → logging macros implementation
//! - `__private_helper` → internal utilities
//!
//! # Usage
//!
//! Basic initialization:
//!
//! ```rust
//! use mini_logger::init;
//!
//! init();
//! ```
//!
//! Custom configuration:
//!
//! ```rust
//! use mini_logger::Builder;
//!
//! Builder::new()
//!     .env_default()
//!     .output_stdout()
//!     .init();
//! ```
//!
//! # Re-exported API
//!
//! The crate re-exports its primary types so applications typically do not
//! need to access internal modules directly.
//!
//! ## Core Types
//!
//! - [`Logger`]
//! - [`Builder`]
//! - [`RecMessage`]
//! - [`RecMessageBuilder`]
//!
//! ## Filtering
//!
//! - [`Filter`]
//! - [`FilterBuilder`]
//! - [`Level`]
//!
//! ## Formatting
//!
//! - [`Formatter`]
//! - [`FormatterBuilder`]
//! - [`RenderRecord`]
//!
//! ## Output
//!
//! - [`Writer`]
//! - [`WriterBuilder`]
//! - [`Output`]
//! - [`Buffer`]
//! - [`try_with_buffer_slot`]
//!
//! ## Styling
//!
//! - [`Color`]
//! - [`ColorMode`]
//! - [`TimeMode`]
//! - [`TimePrecision`]
//! - [`Timestamp`]
//!
//! ## Macros
//!
//! Logging macros are exported by the crate and are available automatically
//! after importing the crate.
//!
//! ### Core Macro
//!
//! - [`log!`] — flexible logging macro supporting multiple forms:
//!
//! ```rust
//! log!(info, "message");
//! log!(target: "network", debug, "message");
//! ```
//!
//! ### Level Convenience Macros
//!
//! These macros are shorthand wrappers around [`log!`]:
//!
//! - [`error!`]
//! - [`warn!`]
//! - [`info!`]
//! - [`debug!`]
//! - [`trace!`]
//!
//! Example:
//!
//! ```rust
//! error!("something failed");
//! warn!("warning message");
//! info!("information");
//! debug!("debug details");
//! trace!("trace details");
//! ```
//!...

pub mod __private_helper;
mod filter;
mod format;
mod logger;
mod record;
mod style;
mod writer;

#[macro_use]
mod macros;

pub use filter::Filter;
pub use filter::FilterBuilder;
pub use filter::Level;
pub use format::Formatter;
pub use format::FormatterBuilder;
pub use format::RenderRecord;
pub use logger::Builder;
pub use logger::Logger;
pub use logger::init;
pub use record::RecordMsg;
pub use record::RecordMsgBuilder;
pub use style::Color;
pub use style::ColorMode;
pub use style::TimeMode;
pub use style::TimePrecision;
pub use style::Timestamp;
pub use writer::Buffer;
pub use writer::Output;
pub use writer::Writer;
pub use writer::WriterBuilder;
pub use writer::try_with_buffer_slot;
