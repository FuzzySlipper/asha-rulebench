//! Host-neutral runtime invocation boundary for Rulebench transports.
//!
//! This crate owns session handles and maps versioned protocol requests to the
//! portable Rust authority. HTTP, JSON, process lifecycle, and UI state belong
//! to concrete adapters outside this crate.

#![forbid(unsafe_code)]

mod content_invocation;
mod error;
mod experiment;
mod invocation;

pub use content_invocation::{import_authored_content, ContentInvocationError};
pub use error::{BridgeError, BridgeErrorKind};
pub use invocation::{prepare_replay_scenario, BridgeScenario, RulebenchBridge};

/// Host-neutral types needed by a concrete replay-storage adapter.
///
/// This is the bridge's deliberate storage-composition seam. Concrete hosts
/// should not depend on the portable authority facade directly.
pub mod replay_storage {
    pub use rpg_core::GridPosition;
    pub use rpg_ir::RulesetArtifactProvenance;
    pub use rulebench_combat::{
        CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec, CombatControlCommandSpec,
        CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec,
        CombatSessionCandidateSelectionSpec, CombatSessionCreateRequest,
        CombatSessionIntentCommandSpec, CommandRollMode, EquipmentCommandKind,
        EquipmentCommandSpec, ReactionCommandSpec, ReactionResponseKind, StateFingerprint,
    };
    pub use rulebench_content::{
        bind_authored_action, materialize_authored_scenario, AuthoredActionAbilityGrantReceipt,
        AuthoredActionBindingReceipt, AuthoredActionBindingRequest, Combatant, ContentFingerprint,
        ContentPackReference, ContentPackSetReference, ContentPackStorage, UseActionIntent,
    };
    pub use rulebench_replay::{
        record_replay_package, ReplayArchiveEntry, ReplayArchiveMetadata, ReplayArchiveStorage,
        ReplayArchiveStorageError, ReplayCommand, ReplayCommandRecord, ReplayCommandRecordingSpec,
        ReplayNarration, SessionRecoveryError, SessionRecoveryFrame, SessionRecoveryPackage,
        SessionRecoveryStorage, SessionRecoveryStorageError,
        REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION, REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM,
        SESSION_RECOVERY_PACKAGE_VERSION,
    };
}

/// Host-neutral types and operations needed by a concrete content repository.
pub mod content_storage {
    pub use rpg_ir::{RulesetArtifactProvenance, RulesetProviderCatalog};
    pub use rulebench_content::{
        compare_content_packs, materialize_authored_scenario, AuthoredScenarioControlMode,
        CanonicalContentPack, ContentDefinitionKind, ContentImportDiagnostic,
        ContentImportDiagnosticSeverity, ContentImportLimits, ContentPackDiffReadout,
        ContentPackReference, ContentPackSetReference, ContentPackStorage, ContentStorageError,
        ContentStorageRecord, ImportedContentPack, StorageReplacementPolicy, StoredContentPayload,
    };
}

#[cfg(test)]
mod tests;
