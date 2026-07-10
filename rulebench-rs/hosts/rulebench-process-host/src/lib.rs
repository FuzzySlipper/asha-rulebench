//! Process-backed HTTP/JSON adapter for the Rulebench bridge.
//!
//! This crate is Rulebench-local host infrastructure. It owns HTTP parsing,
//! JSON serialization, process lifecycle, and fixture registration. It does
//! not own rule semantics or application state.

#![forbid(unsafe_code)]

mod http;
mod router;
mod server;

pub use http::{HttpMethod, HttpRequest, HttpResponse};
pub use router::{build_rulebench_bridge, ProcessHostRouter};
pub use server::{serve_until, ServerError};

#[cfg(test)]
mod tests;
