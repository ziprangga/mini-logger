// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mini logger crate.
//!
//! This crate provides a lightweight, configurable logging system with:
//!
//! - Level + target filtering
//! - Flexible formatting system
//! - Support color
//! - Multiple output backends (stdout, stderr, file)
//! - Thread-local buffering for performance
//! - Optional runtime control features
//! - Optional panic hook integration
//!
//! # Inspiration
//!
//! This crate is inspired by the [`log`](https://docs.rs/log) and
//! [`env_logger`](https://docs.rs/env_logger) crates.
//!
//! It follows the same general logging model but integrates functionality
//! that is typically split across multiple crates.
//!
//! - This crate **does not support `no_std` environments**.
//!   It is designed specifically for **high-level application development**, not embedded or constrained systems.
//!
//! - This reduces the need for additional dependencies that are typically used
//!   in high-level application logging setups.
//!
//! - This crate provides **additional control over runtime behavior**
//!   (such as enabling/disabling logging at runtime).
//!
//! - This crate provides a **configurable single output destination per logger instance**
//!   (stdout, stderr, or file).
//!
//! - This crate is a **single all-in-one logging implementation** that includes:
//!   - logger runtime
//!   - filtering system
//!   - formatting system
//!   - output handling
//!
//! # Architecture
//!
//! The crate is split into several internal modules:
//!
//! - [`logger`] → global logger runtime and initialization
//! - [`filter`] → log filtering rules (level + target + env support)
//! - [`format`] → log formatting system (default + custom)
//! - [`writer`] → output handling (stdout/stderr/file + buffering)
//! - [`record`] → log record structure (`RecMessage`)
//! - [`style`] → color + timestamp formatting utilities
//!
//! Internal-only modules:
//!
//! - `macros` → logging macros (not public API)
//! - `__private_helper` → internal utilities
//!
//! # Usage
//!
//! Typical usage:
//!
//! ```rust
//! use mini_logger::init;
//!
//! init();
//! ```
//!
//! Or advanced configuration:
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
//! This crate re-exports its main components for convenience,
//! so users do not need to access internal modules directly.
//!
//! ## Core types
//!
//! - [`Logger`]
//! - [`Builder`]
//! - [`RecMessage`]
//!
//! ## Filtering
//!
//! - [`Filter`]
//! - [`FilterBuilder`]
//! - [`FilterLevel`]
//! - [`FilterEnv`]
//! - [`FilterTarget`]
//!
//! ## Formatting
//!
//! - [`Format`]
//! - [`FormatBuilder`]
//! - [`FormatConfig`]
//! - [`FormatCustom`]
//!
//! ## Output / Writer
//!
//! - [`Writer`]
//! - [`Output`]
//! - [`BufferWriter`]
//! - [`BufferFormatter`]
//! - [`Buffer`]
//! - [`try_with_buf_formatter_slot`]
//!
//! ## Styling
//!
//! - [`Color`]
//! - [`ColorMode`]
//! - [`Timestamp`]
//! - [`TimestampPrecision`]
//!
//! ## Macros
//!
//! Logging macros are included via `#[macro_use] mod macros`
//! and are available automatically after importing the crate.
//!
//! Macros provides a set of logging macros for ergonomic usage.
//!
//! ### Main macro
//!
//! - [`log!`] → core logging macro supporting multiple forms:
//!
//! ```rust
//! log!(info, "message");
//! log!(target: "network", debug, "msg");
//! ```
//!
//! ### Level convenience macros
//!
//! These are shorthand wrappers around [`log!`]:
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
//! info!("info message");
//! debug!("debug message");
//! trace!("trace message");
//!..

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
pub use filter::FilterEnv;
pub use filter::FilterLevel;
pub use filter::FilterTarget;
pub use format::Format;
pub use format::FormatBuilder;
pub use format::FormatConfig;
pub use format::FormatCustom;
pub use logger::Builder;
pub use logger::Logger;
pub use logger::init;

pub use record::RecMessage;
pub use record::RecMessageBuilder;
pub use style::Color;
pub use style::ColorMode;
pub use style::Timestamp;
pub use style::TimestampPrecision;
pub use writer::Buffer;
pub use writer::BufferFormatter;
pub use writer::BufferWriter;
pub use writer::Output;
pub use writer::Writer;
pub use writer::try_with_buf_formatter_slot;
