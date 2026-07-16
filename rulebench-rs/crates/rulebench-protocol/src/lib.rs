//! Stable transport protocol surfaces for Rust authority.
//!
//! This crate owns wire DTO contracts, protocol metadata, and explicit mapping
//! from portable authority values. It does not own transport implementations
//! or rule semantics.

#![forbid(unsafe_code)]

mod authored_action;
mod authoring;
mod bridge;
mod capabilities;
mod content;
mod experiment;
mod live;
mod reaction;
mod recovery;
mod replay;
mod session;
mod typescript;
mod viewer;

pub use authored_action::{
    AuthoredActionDefinitionDto, AuthoredActionResourceCostDto, AuthoredActionRollPolicyDto,
    AuthoredAreaShapeDto, AuthoredAreaTargetingDeclarationDto, AuthoredCheckDeclarationDto,
    AuthoredDefenseReferenceDto, AuthoredEffectOperationDto, AuthoredModifierDefinitionDto,
    AuthoredModifierDurationPolicyDto, AuthoredModifierStackingPolicyDto,
    AuthoredModifierStatAdjustmentDto, AuthoredModifierTenureDto,
    AuthoredMovementActionDeclarationDto, AuthoredMovementKindDto, AuthoredMovementTopologyDto,
    AuthoredOperationPipelineDto, AuthoredReactionOptionDeclarationDto,
    AuthoredReactionParticipantSelectorDto, AuthoredReactionWindowDto,
    AuthoredTargetFailurePolicyDto, AuthoredTargetKindDto, AuthoredTargetOrderPolicyDto,
    AuthoredTargetSelectionDto, AuthoredTargetTeamConstraintDto, AuthoredTargetingDeclarationDto,
    AuthoredVisibilityRequirementDto,
};
pub use authoring::{
    validate_ruleset_definition, RuleModuleConfigurationDto, RuleModuleDeclarationDto,
    RulesetAuthoringError, RulesetDefinitionDto,
};
pub use bridge::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatAutomationNoCandidateBehaviorDto,
    CombatAutomationPolicyDto, CombatControlCommandDto, CombatControlCommandKindDto,
    CombatSessionCreateRequestDto, CombatSessionIntentCommandDto, CommandRollModeDto,
    ProtocolHandshakeDto, ProtocolRequestContextDto, ScenarioOptionDto,
    ScenarioParticipantOptionDto, UseActionIntentDto, PROTOCOL_ID, PROTOCOL_VERSION,
};
pub use capabilities::{
    CapabilityEntryDto, CapabilityIdentityDto, CapabilitySupportDto, HostCapabilityProfileDto,
    RulebenchCapabilityManifestDto, RulesetProviderDto,
};
pub use content::{
    AuthoredAbilityDefinitionDto, AuthoredAbilityDefinitionKindDto, AuthoredContentCatalogsDto,
    AuthoredContentDecodeError, AuthoredContentPackDocumentDto, AuthoredContentPackDto,
    AuthoredContentProvenanceDto, AuthoredContentSourceKindDto, AuthoredDamageAdjustmentDto,
    AuthoredDamageAdjustmentPolicyDto, AuthoredEntityDefinitionDto, ContentAuditEntryDto,
    ContentDefinitionChangeDto, ContentDefinitionSummaryDto, ContentFingerprintDto,
    ContentImportAttemptDto, ContentImportDiagnosticDto, ContentImportOutcomeDto,
    ContentImportReadoutDto, ContentImportRequestDto, ContentPackDiffDto, ContentPackIdentityDto,
    ContentPackReferenceDto, ContentPackReviewDto, ContentPayloadRequestDto,
    ContentReferenceRequestDto, ContentReplacementPolicyDto, ContentWorkspaceDto,
    StoredContentPackSummaryDto, AUTHORED_CONTENT_PACK_FORMAT, AUTHORED_CONTENT_PACK_VERSION,
    AUTHORED_CONTENT_PACK_VERSION_V1, AUTHORED_CONTENT_PACK_VERSION_V2,
};
pub use experiment::{
    AutomationPolicyCatalogEntryDto, ExperimentComparisonReadoutDto,
    ExperimentComparisonRequestDto, ExperimentDecisionEvidenceDto, ExperimentMatrixRequestDto,
    ExperimentMetricsDto, ExperimentReadoutDto, ExperimentTrialReadoutDto,
    PolicyRulesetCompatibilityDto,
};
pub use live::{
    LiveActionOptionDto, LiveActionResourceCostDto, LiveActionResourceStateDto, LiveAuditEntryDto,
    LiveAutomaticRunDto, LiveAutomaticStepDto, LiveBoardCellDto, LiveBoardDto, LiveCandidateDto,
    LiveCandidateSummaryDto, LiveCellOptionDto, LiveCombatEndDto, LiveCombatLogEntryDto,
    LiveCommandExecutionDto, LiveCommandStepDto, LiveControlExecutionDto,
    LiveCurrentActorOptionsDto, LiveDomainEventDto, LiveFinalizationDto, LiveGeneratedRollDto,
    LiveGridPositionDto, LiveParticipantDto, LivePreflightDto, LiveReactionExecutionDto,
    LiveResourceChangeDto, LiveRollEvidenceDto, LiveSessionSnapshotDto, LiveStateFingerprintDto,
    LiveTargetOptionDto, LiveTargetResolutionDto, LiveTargetSetOptionDto, LiveTraceEntryDto,
    LiveTransportErrorDto,
};
pub use reaction::{
    ReactionAuditEntryDto, ReactionCommandReadoutDto, ReactionCommandSpecDto, ReactionOptionDto,
    ReactionResponseEntryDto, ReactionResponseKindDto, ReactionWindowDto,
    ReactionWindowLifecycleEntryDto,
};
pub use recovery::{
    SessionRecoveryCatalogDto, SessionRecoveryEntryDto, SessionRecoveryForkRequestDto,
    SessionRecoveryIssueDto,
};
pub use replay::{
    ReplayArchiveErrorDto, ReplayArchiveMetadataDto, ReplayCommandReviewDto,
    ReplayComparisonDifferenceDto, ReplayComparisonReadoutDto, ReplayComparisonRequestDto,
    ReplayMismatchDto, ReplayPackageReviewDto, ReplayStateFingerprintDto, ReplayStepEvidenceDto,
    ReplayVerificationReadoutDto,
};
pub use session::CombatSessionHandleDto;
pub use typescript::{render_api_types, ProtocolAlias, ProtocolField, ProtocolInterface};
pub use viewer::{
    ViewerCombatLogEntryDto, ViewerCombatantDto, ViewerCommandAttemptDto, ViewerDefenseDto,
    ViewerDomainEventDto, ViewerFinalCombatantDto, ViewerFinalStateDto, ViewerScenarioReadoutDto,
    ViewerScenarioSummaryDto, ViewerSelectedActionDto, ViewerSelectedTargetDto,
    ViewerSessionStepReadoutDto, ViewerSessionStepSummaryDto, ViewerSessionSummaryDto,
    ViewerSessionTranscriptDto,
};

#[cfg(test)]
mod tests {
    use rulebench_rules::CombatSessionHandle;

    use super::{render_api_types, CombatSessionHandleDto};

    const COMMITTED_API_TYPES: &str =
        include_str!("../../../../libs/protocol/src/generated/api-types.ts");

    #[test]
    fn committed_typescript_contract_matches_protocol_metadata() {
        assert_eq!(render_api_types(), COMMITTED_API_TYPES);
    }

    #[test]
    fn session_handle_mapping_preserves_the_opaque_authority_identity() {
        let handle = CombatSessionHandle::new("test-session");

        let dto = CombatSessionHandleDto::from(&handle);

        assert_eq!(dto.id, "test-session");
        assert_eq!(dto.to_combat_session_handle(), handle);
    }

    #[test]
    fn typescript_contract_exposes_combat_sides_and_immutable_finalization() {
        let contract = render_api_types();

        assert!(contract.contains("export type RulebenchCombatSideIdDto = string;"));
        assert!(contract.contains("readonly sideId: RulebenchCombatSideIdDto;"));
        assert!(contract.contains("readonly policy: RulebenchCombatEndPolicyDto;"));
        assert!(contract.contains("readonly outcomeKind: RulebenchCombatOutcomeKindDto;"));
        assert!(contract.contains("readonly winningSides: readonly RulebenchCombatSideIdDto[];"));
        assert!(contract.contains("readonly finalization: RulebenchCombatFinalizationDto | null;"));
    }
}
