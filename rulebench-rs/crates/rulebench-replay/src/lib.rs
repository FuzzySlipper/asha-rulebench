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
    ReplayPackage, ReplayRollEvidence, ReplayStepEvidence, ReplayTraceEvidence,
    REPLAY_PACKAGE_FINGERPRINT_KIND, REPLAY_PACKAGE_VERSION,
};
pub use package_validation::{
    validate_replay_package, ReplayPackageDiagnostic, ReplayPackageDiagnosticCode,
    ReplayPackageValidationReport,
};
mod verification;
pub use verification::{
    verify_replay_package, ReplayMismatch, ReplayMismatchDimension, ReplayVerificationDecisionKind,
    ReplayVerificationReadout,
};
mod archive;
mod archive_storage;
pub use archive::{ReplayArchive, ReplayArchiveError, ReplayArchiveQuery};
pub use archive_storage::{
    InMemoryReplayArchiveStorage, ReplayArchiveEntry, ReplayArchiveMetadata, ReplayArchiveStorage,
    ReplayArchiveStorageError,
};
mod randomness;
pub use randomness::{
    generate_replay_randomness, reproduce_replay_roll_stream, validate_replay_randomness,
    ReplayCommandRandomnessProvenance, ReplayGeneratedRollRequest, ReplayRandomnessDiagnostic,
    ReplayRandomnessDiagnosticCode, ReplayRandomnessSource, ReplayRandomnessValidationReport,
    ReplayRollGenerationSpec, REPLAY_RANDOMNESS_ALGORITHM_VERSION,
};
mod comparison;
pub use comparison::{
    compare_replay_archive_entries, compare_replay_packages, ReplayComparisonDifference,
    ReplayComparisonDifferenceCode, ReplayComparisonReadout,
};
