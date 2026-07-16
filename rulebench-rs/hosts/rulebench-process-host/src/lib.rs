//! Process-backed HTTP/JSON adapter for the Rulebench bridge.
//!
//! This crate is Rulebench-local host infrastructure. It owns HTTP parsing,
//! JSON serialization, process lifecycle, and fixture registration. It does
//! not own rule semantics or application state.

#![forbid(unsafe_code)]

mod artifact_repository;
mod capability_artifact;
mod content_workspace;
mod http;
mod router;
mod server;

pub use artifact_repository::{
    ArtifactRepositoryIssue, FileReplayArchiveStorage, FileSessionRecoveryStorage,
    ReplayStorageOpenReport, SessionRecoveryOpenReport,
};
pub use capability_artifact::render_capability_manifest_artifact;
pub use http::{HttpMethod, HttpRequest, HttpResponse};
pub use router::{
    build_durable_rulebench_router, build_rulebench_bridge, ArtifactRepositoryConfig,
    ArtifactRepositoryStatus, ProcessHostRouter,
};
pub use server::{serve_until, ServerError};

#[cfg(test)]
mod tests;
