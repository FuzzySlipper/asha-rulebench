//! Host-neutral runtime invocation boundary for Rulebench transports.
//!
//! This crate owns session handles and maps versioned protocol requests to the
//! portable Rust authority. HTTP, JSON, process lifecycle, and UI state belong
//! to concrete adapters outside this crate.

#![forbid(unsafe_code)]

mod error;
mod invocation;

pub use error::{BridgeError, BridgeErrorKind};
pub use invocation::{BridgeScenario, RulebenchBridge};

#[cfg(test)]
mod tests;
