//! Audit, replay packages, and verification.
//!
//! Combat execution remains in `rulebench-combat`; this crate owns replay
//! specifications and immutable evidence comparison.

#![forbid(unsafe_code)]

mod automatic_run;
mod package;
mod package_validation;

pub use automatic_run::{
    verify_automatic_run_replay, CombatSessionAutomaticRunReplayDecisionKind,
    CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
};
pub use package::{
    ReplayAcceptedEvents, ReplayCommand, ReplayCommandRecord, ReplayEvidence, ReplayNarration,
    ReplayPackage, ReplayRollEvidence, ReplayTraceEvidence, REPLAY_PACKAGE_FINGERPRINT_KIND,
    REPLAY_PACKAGE_VERSION,
};
pub use package_validation::{
    validate_replay_package, ReplayPackageDiagnostic, ReplayPackageDiagnosticCode,
    ReplayPackageValidationReport,
};
