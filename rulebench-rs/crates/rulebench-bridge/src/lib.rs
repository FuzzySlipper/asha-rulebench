//! Host-neutral runtime invocation boundary for Rulebench transports.
//!
//! This crate owns session handles and maps versioned protocol requests to the
//! portable Rust authority. HTTP, JSON, process lifecycle, and UI state belong
//! to concrete adapters outside this crate.

#![forbid(unsafe_code)]

mod content_invocation;
mod error;
mod invocation;

pub use content_invocation::{import_authored_content, ContentInvocationError};
pub use error::{BridgeError, BridgeErrorKind};
pub use invocation::{prepare_replay_scenario, BridgeScenario, RulebenchBridge};

/// Host-neutral types needed by a concrete replay-storage adapter.
///
/// This is the bridge's deliberate storage-composition seam. Concrete hosts
/// should not depend on the portable authority facade directly.
pub mod replay_storage {
    pub use rulebench_rules::{
        record_replay_package, CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec,
        CombatControlCommandSpec, CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec,
        CombatSessionCandidateSelectionSpec, CombatSessionCreateRequest,
        CombatSessionIntentCommandSpec, Combatant, CommandRollMode, ContentFingerprint,
        ContentPackReference, ContentPackSetReference, ContentPackStorage, EquipmentCommandKind,
        EquipmentCommandSpec, GridPosition, ReactionCommandSpec, ReactionResponseKind,
        ReplayArchiveEntry, ReplayArchiveMetadata, ReplayArchiveStorage, ReplayArchiveStorageError,
        ReplayCommand, ReplayCommandRecord, ReplayCommandRecordingSpec, ReplayNarration,
        RulesetArtifactProvenance, UseActionIntent, REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION,
        REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM,
    };
}

/// Host-neutral types and operations needed by a concrete content repository.
pub mod content_storage {
    pub use rulebench_rules::{
        compare_content_packs, CanonicalContentPack, ContentDefinitionKind,
        ContentImportDiagnostic, ContentImportDiagnosticSeverity, ContentImportLimits,
        ContentPackDiffReadout, ContentPackReference, ContentPackSetReference, ContentPackStorage,
        ContentStorageError, ContentStorageRecord, ImportedContentPack, RulesetArtifactProvenance,
        StorageReplacementPolicy, StoredContentPayload,
    };
}

#[cfg(test)]
mod tests;
