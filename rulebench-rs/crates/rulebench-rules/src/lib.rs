//! Supported portable convenience facade for ASHA Rulebench rule authority.
//!
//! This crate re-exports the stable public API from the portable authority
//! layers: core values, ruleset declarations, canonical content, combat
//! execution, and replay verification. New portable consumers may depend on
//! the focused owner crates when they need a narrower surface; this facade is
//! the supported one-crate entry point for consumers that need the complete
//! authority contract.
//!
//! Stability: public items re-exported here are the local `v0` portable
//! contract. Rulebench-only fixtures, generated artifacts, bridge adapters,
//! and UI concerns are intentionally absent.
//!
//! Non-claims: this is not a generic rules-engine API, a host runtime, or a
//! promise that every currently public owner-crate item has cross-project
//! compatibility beyond the documented Rulebench portable contract.

#![forbid(unsafe_code)]

pub use rulebench_combat::model::*;
pub use rulebench_combat::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    evaluate_effective_stats_for_combatant, fingerprint_projected_state, fingerprint_projection,
    resolve_use_action, validate_combat_automation_policy, validate_intent_shape,
    CombatAutomationCandidateEvidence, CombatAutomationNoCandidateBehavior,
    CombatAutomationPolicyDecisionEvidence, CombatAutomationPolicySpec,
    CombatAutomationPolicyValidationCode, CombatAutomationPolicyValidationReadout,
    CombatSessionApi, CombatSessionApiError, CombatSessionArchive,
    CombatSessionAutoCandidateCommandSpec, CombatSessionAutoCandidateDecisionKind,
    CombatSessionAutoCandidateExecutionReadout, CombatSessionAutoCandidatePlanReadout,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepOperationKind,
    CombatSessionAutomaticStepPlanReadout, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
    CombatSessionCommandSpec, CombatSessionCreateReadout, CombatSessionCreateRequest,
    CombatSessionHandle, CombatSessionIntentCommandSpec, CombatSessionScriptCommandKind,
    CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
    CombatSessionState, CombatState, EffectiveStatEvaluationError,
    FIRST_ACCEPTED_CANDIDATE_POLICY_ID, FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION,
    PROJECTION_FINGERPRINT_ALGORITHM, STATE_FINGERPRINT_ALGORITHM,
};
pub use rulebench_content::{
    canonicalize_content_pack, compare_content_packs, fingerprint_content_pack_set,
    import_content_pack, validate_scenario_content, validate_scenario_content_report,
    AuthoredContentPack, CanonicalContentPack, ContentDefinitionChange,
    ContentDefinitionChangeKind, ContentDefinitionKind, ContentFingerprint, ContentImportContext,
    ContentImportDiagnostic, ContentImportDiagnosticCode, ContentImportDiagnosticSeverity,
    ContentImportLimits, ContentImportReport, ContentPackCatalogs, ContentPackCollisionPolicy,
    ContentPackDefinition, ContentPackDiagnosticCode, ContentPackDiffReadout, ContentPackIdentity,
    ContentPackMetadataChangeKind, ContentPackProvenance, ContentPackReference,
    ContentPackSetReference, ContentPackSourceKind, EntityDefinition, ImportedContentPack,
};
pub use rulebench_core::Team;
pub use rulebench_replay::{
    compare_replay_packages, inspect_replay_package, record_replay_package,
    verify_automatic_run_replay, verify_replay_package,
    CombatSessionAutomaticRunReplayDecisionKind, CombatSessionAutomaticRunReplayReadout,
    CombatSessionAutomaticRunReplaySpec, InMemoryReplayArchiveStorage, ReplayArchive,
    ReplayArchiveError, ReplayArchiveMetadata, ReplayArchiveQuery, ReplayArchiveStorageError,
    ReplayCommand, ReplayCommandInspection, ReplayCommandRecord, ReplayCommandRecordingSpec,
    ReplayComparisonDifference, ReplayComparisonDifferenceCode, ReplayComparisonReadout,
    ReplayEvidence, ReplayMismatch, ReplayMismatchDimension, ReplayNarration, ReplayPackage,
    ReplayPackageInspection, ReplayPackageValidationReport, ReplayRandomnessSource,
    ReplayStepEvidence, ReplayVerificationDecisionKind, ReplayVerificationReadout,
};
pub use rulebench_ruleset::CombatEndPolicy;

#[cfg(test)]
mod tests {
    use super::{
        verify_automatic_run_replay, CombatAutomationPolicySpec, CombatSessionApi,
        CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
        CombatSessionHandle, RulesetMetadata,
    };

    #[test]
    fn facade_exposes_the_documented_portable_contract() {
        let api = CombatSessionApi::new();
        let handle = CombatSessionHandle::new("portable-session");
        let ruleset = RulesetMetadata {
            id: "portable.ruleset".to_string(),
            name: "Portable Ruleset".to_string(),
            version: "0.1.0".to_string(),
            summary: "Facade contract smoke.".to_string(),
            modules: Vec::new(),
        };
        let replay_verifier: fn(
            CombatSessionAutomaticRunReplaySpec,
        ) -> CombatSessionAutomaticRunReplayReadout = verify_automatic_run_replay;
        let automation_policy = CombatAutomationPolicySpec::first_accepted_candidate();

        assert!(api.list_active_sessions().is_empty());
        assert_eq!(ruleset.id, "portable.ruleset");
        assert_eq!(handle.id, "portable-session");
        assert_eq!(automation_policy.id, "firstAcceptedCandidate");
        let _ = replay_verifier;
    }
}
