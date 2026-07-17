//! Temporary Rulebench product adapter over extracted RPG authority.
//!
//! Existing protocol, bridge, and fixture consumers still need one combined
//! product surface while task #5938 migrates them to their permanent owners.
//! The reusable primitives, normalized rule declarations, and RuntimeSession
//! fabric are imported from the exact public `asha-rpg` revision; this crate
//! contains no duplicate implementation of those owners.
//!
//! Removal: #5938 deletes this adapter after product call sites import focused
//! product owners and the supported `asha-rpg` boundary directly.

#![forbid(unsafe_code)]

mod capabilities;

pub use capabilities::{
    assemble_capability_manifest, executable_conformance_capabilities, CapabilityEntry,
    CapabilityIdentity, CapabilityKind, CapabilityManifestError, CapabilityRegistryInput,
    CapabilitySupport, HostCapabilityProfile, RulebenchCapabilityManifest,
    RulesetProviderManifestEntry, CAPABILITY_ARTIFACT_SCHEMA, CAPABILITY_MANIFEST_ID,
    CAPABILITY_MANIFEST_VERSION,
};

pub use rpg_core::Team;
pub use rpg_ir::{
    CombatEndPolicy, EffectOperationId, OperationPipelineV2, RuleModuleId,
    RulesetArtifactProvenance, RulesetModuleProvenance, RulesetProviderCapability,
    RulesetProviderCatalog, RulesetProviderCatalogError, RulesetProviderCompatibilityError,
    RulesetProviderDescriptor, TargetingOperationId,
};
pub use rulebench_combat::model::*;
pub use rulebench_combat::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    evaluate_effective_stats_for_combatant, fingerprint_projected_state, fingerprint_projection,
    resolve_use_action, validate_combat_automation_policy,
    validate_combat_automation_policy_for_context, validate_intent_shape,
    CombatAutomationCandidateEvidence, CombatAutomationNoCandidateBehavior,
    CombatAutomationPolicyContext, CombatAutomationPolicyDecisionEvidence,
    CombatAutomationPolicyRegistration, CombatAutomationPolicyRequirement,
    CombatAutomationPolicySelector, CombatAutomationPolicySpec,
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
    COMBAT_AUTOMATION_POLICY_REGISTRY, FIRST_ACCEPTED_CANDIDATE_POLICY_ID,
    FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION, LOWEST_VITALITY_TARGET_POLICY_ID,
    LOWEST_VITALITY_TARGET_POLICY_VERSION, OBJECTIVE_SIDE_PRESSURE_POLICY_ID,
    OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use rulebench_content::{
    authored_scenario_definition, bind_authored_action, canonicalize_content_pack,
    compare_content_packs, fingerprint_authored_action, fingerprint_content_pack_set,
    import_content_pack, materialize_authored_scenario, validate_scenario_content,
    validate_scenario_content_report, AuthoredActionAbilityGrantReceipt,
    AuthoredActionBindingError, AuthoredActionBindingReceipt, AuthoredActionBindingRequest,
    AuthoredActionDefinition, AuthoredContentPack, AuthoredEffectOperation,
    AuthoredModifierEffectOperation, AuthoredReactionHookEffectOperation,
    AuthoredReactionOptionDeclaration, AuthoredScenarioActionGrant, AuthoredScenarioBindingError,
    AuthoredScenarioBindingReceipt, AuthoredScenarioControl, AuthoredScenarioControlMode,
    AuthoredScenarioDefinition, AuthoredScenarioParticipant, AuthoredScenarioParticipantReceipt,
    AuthoredTargetingDeclaration, CanonicalContentPack, ContentDefinitionChange,
    ContentDefinitionChangeKind, ContentDefinitionKind, ContentFingerprint, ContentImportContext,
    ContentImportDiagnostic, ContentImportDiagnosticCode, ContentImportDiagnosticSeverity,
    ContentImportLimits, ContentImportReport, ContentPackCanonicalVersion, ContentPackCatalogs,
    ContentPackCollisionPolicy, ContentPackDefinition, ContentPackDiagnosticCode,
    ContentPackDiffReadout, ContentPackIdentity, ContentPackMetadataChangeKind,
    ContentPackProvenance, ContentPackReference, ContentPackSetReference, ContentPackSourceKind,
    ContentPackStorage, ContentStorageError, ContentStorageRecord, ContentStorageStartupIssue,
    EntityDefinition, ImportedContentPack, ReactionParticipantSelector, StorageReplacementPolicy,
    StoredContentPayload, AUTHORED_ACTION_BINDING_VERSION,
    AUTHORED_ACTION_CHECK_VOCABULARY_VERSION, AUTHORED_ACTION_DEFINITION_FINGERPRINT_ALGORITHM,
    AUTHORED_ACTION_REACTION_EXPANSION_LIMIT, AUTHORED_SCENARIO_BINDING_VERSION,
    CONTENT_PACK_FINGERPRINT_ALGORITHM, CONTENT_PACK_FINGERPRINT_ALGORITHM_V1,
    CONTENT_PACK_FINGERPRINT_ALGORITHM_V2, CONTENT_PACK_SET_FINGERPRINT_ALGORITHM,
};
pub use rulebench_replay::{
    canonical_replay_archive_payload, canonical_replay_archive_payload_fingerprint,
    compare_replay_packages, inspect_replay_package, record_replay_package,
    verify_automatic_run_replay, verify_replay_package,
    CombatSessionAutomaticRunReplayDecisionKind, CombatSessionAutomaticRunReplayReadout,
    CombatSessionAutomaticRunReplaySpec, InMemoryReplayArchiveStorage,
    InMemorySessionRecoveryStorage, RecoveredSession, ReplayArchive, ReplayArchiveEntry,
    ReplayArchiveError, ReplayArchiveMetadata, ReplayArchiveQuery, ReplayArchiveStorage,
    ReplayArchiveStorageError, ReplayCommand, ReplayCommandInspection, ReplayCommandRecord,
    ReplayCommandRecordingSpec, ReplayComparisonDifference, ReplayComparisonDifferenceCode,
    ReplayComparisonReadout, ReplayEvidence, ReplayMismatch, ReplayMismatchDimension,
    ReplayNarration, ReplayPackage, ReplayPackageInspection, ReplayPackageValidationReport,
    ReplayRandomnessSource, ReplayStepEvidence, ReplayVerificationDecisionKind,
    ReplayVerificationReadout, SessionRecoveryError, SessionRecoveryFrame, SessionRecoveryPackage,
    SessionRecoveryStorage, SessionRecoveryStorageError, REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION,
    REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM, SESSION_RECOVERY_PACKAGE_VERSION,
};

#[cfg(test)]
mod tests {
    use super::{
        verify_automatic_run_replay, CombatAutomationPolicySpec, CombatSessionApi,
        CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
        CombatSessionHandle, RulesetMetadata,
    };

    #[test]
    fn adapter_exposes_the_current_product_migration_contract() {
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
