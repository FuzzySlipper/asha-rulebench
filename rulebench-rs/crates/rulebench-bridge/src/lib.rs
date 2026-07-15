//! Host-neutral runtime invocation boundary for Rulebench transports.
//!
//! This crate owns session handles and maps versioned protocol requests to the
//! portable Rust authority. HTTP, JSON, process lifecycle, and UI state belong
//! to concrete adapters outside this crate.

#![forbid(unsafe_code)]

mod error;
mod invocation;

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
        CombatSessionIntentCommandSpec, Combatant, CommandRollMode, ContentPackStorage,
        EquipmentCommandKind, EquipmentCommandSpec, GridPosition, ReactionCommandSpec,
        ReactionResponseKind, ReplayArchiveEntry, ReplayArchiveMetadata, ReplayArchiveStorage,
        ReplayArchiveStorageError, ReplayCommand, ReplayCommandRecord, ReplayCommandRecordingSpec,
        ReplayNarration, RulesetArtifactProvenance, UseActionIntent,
    };
}

#[cfg(test)]
mod tests;
