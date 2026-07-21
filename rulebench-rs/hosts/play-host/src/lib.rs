#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Mutex,
};

use rpg_compiler::{
    compile_prepared_play_bundle_json, load_compiled_play_bundle_json, CompiledPlayBundle,
    RpgCompileFailure, RpgDiagnostic, RpgDiagnosticSeverity, RpgDiagnosticStage,
};
use rpg_core::{
    BoundedValue, GridPosition, RpgDomainEvent, RpgRandomEvidence, RpgRandomRequest,
    RpgRandomRequestKind, RpgReactionRequest, RpgResolutionReceipt, RpgResolutionRejection,
    RpgTeamId, RpgTraceStep,
};
use rpg_ir::{
    CompiledPlayBundleArtifact, ContentConflictPolicy, ContentExtensionPolicy, ContentImpactPlane,
    ContentPackDependencyRelationship, ContentRelationshipKind, MaterializedContentDefinitionKind,
    MaterializedContentVisibility, RulesetValueKind,
};
use rpg_runtime::{
    RpgActionProposal, RpgAuthoritySession, RpgAutomaticCommandFailure, RpgBoardSetup,
    RpgCellCapabilitySetup, RpgCellCapabilityValue, RpgCellSetup, RpgCheckpointPhase,
    RpgCommandOutcome, RpgEncounterOutcomeView, RpgInitialCapability, RpgParticipantSetup,
    RpgRandomSource, RpgRandomSourceBinding, RpgRandomSourceFailure, RpgReactionProposal,
    RpgReplayBoundary, RpgReplayEntry, RpgReplayFailure, RpgReplayOperation, RpgReplayPhase,
    RpgScenario, RpgSchemaIdentity, RpgSessionCheckpoint, RpgTurnControl, RpgTurnControlProposal,
    RpgTurnControlReceipt, RpgTurnInitialization,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum PlayBundleLifecycleStatus {
    NoActivePlayBundle,
    CompiledCandidate,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayDiagnosticDto {
    pub stage: String,
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
    pub package_id: Option<String>,
    pub definition_id: Option<String>,
    pub source: Option<PlayDiagnosticSourceDto>,
    pub graph_path: Option<Vec<String>>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayDiagnosticSourceDto {
    pub module: String,
    pub declaration: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VersionedIdentityDto {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentPackSummaryDto {
    pub id: String,
    pub version: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentPackLockEntryDto {
    pub requester: String,
    pub package_id: String,
    pub requested_version: String,
    pub resolved_version: String,
    pub source_fingerprint: String,
    pub import_as: String,
    pub relationship: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VersionedRequirementDto {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentDefinitionDto {
    pub id: String,
    pub fingerprint: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub catalog: Option<String>,
    pub catalog_id: Option<String>,
    pub kind: String,
    pub visibility: String,
    pub extension_policy: String,
    pub references: Vec<String>,
    pub package_id: String,
    pub package_version: String,
    pub source_module: String,
    pub source_declaration: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetValueDto {
    pub kind: String,
    pub id: String,
    pub label: String,
    pub numeric_domain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ParticipantProfileDto {
    pub definition_id: String,
    pub profile_id: String,
    pub label: String,
    pub description: Option<String>,
    pub role: String,
    pub definition_ids: Vec<String>,
    pub capabilities: Vec<ScenarioInitialCapabilityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentRelationshipDto {
    pub kind: String,
    pub source: String,
    pub target: String,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleFingerprintDto {
    pub source: String,
    pub semantic: String,
    pub presentation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentPatchChangeDto {
    pub plane: String,
    pub path: String,
    pub before: String,
    pub after: String,
    pub effective: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentMixinProvenanceDto {
    pub identity: String,
    pub fingerprint: String,
    pub parameters: Vec<String>,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentDerivationProvenanceDto {
    pub definition_id: String,
    pub owner: String,
    pub base: String,
    pub base_fingerprint: String,
    pub mixins: Vec<ContentMixinProvenanceDto>,
    pub local_patch_fingerprint: String,
    pub materialized_fingerprint: String,
    pub changes: Vec<ContentPatchChangeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContentOverlayProvenanceDto {
    pub overlay: String,
    pub target: String,
    pub expected_fingerprint: String,
    pub before_fingerprint: String,
    pub after_fingerprint: String,
    pub plane: String,
    pub conflict_policy: String,
    pub patch_fingerprint: String,
    pub order: usize,
    pub changes: Vec<ContentPatchChangeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleArtifactSummaryDto {
    pub schema: VersionedIdentityDto,
    pub artifact_id: String,
    pub play_bundle: VersionedIdentityDto,
    pub ruleset: VersionedIdentityDto,
    pub language: VersionedIdentityDto,
    pub content_packs: Vec<ContentPackSummaryDto>,
    pub dependency_lock: Vec<ContentPackLockEntryDto>,
    pub required_operations: Vec<VersionedRequirementDto>,
    pub required_capabilities: Vec<VersionedRequirementDto>,
    pub required_values: Vec<String>,
    pub required_numeric_domains: Vec<String>,
    pub ruleset_values: Vec<RulesetValueDto>,
    pub participant_profiles: Vec<ParticipantProfileDto>,
    pub exported_roots: Vec<String>,
    pub definitions: Vec<ContentDefinitionDto>,
    pub policy_binding_ids: Vec<String>,
    pub relationships: Vec<ContentRelationshipDto>,
    pub derivation_slots: usize,
    pub overlay_slots: usize,
    pub derivations: Vec<ContentDerivationProvenanceDto>,
    pub overlays: Vec<ContentOverlayProvenanceDto>,
    pub fingerprints: PlayBundleFingerprintDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleUpgradeFieldDto {
    pub plane: String,
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleUpgradeDefinitionDto {
    pub definition_id: String,
    pub change: String,
    pub descendant: bool,
    pub causes: Vec<String>,
    pub fields: Vec<PlayBundleUpgradeFieldDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleUpgradeImpactDto {
    pub from_artifact_id: String,
    pub to_artifact_id: String,
    pub source_changes: Vec<String>,
    pub definitions: Vec<PlayBundleUpgradeDefinitionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioSchemaDto {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioPositionDto {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioBoundedValueDto {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(tag = "kind", rename_all = "camelCase")]
pub enum ScenarioCellCapabilityValueDto {
    Traversal { passable: bool, movement_cost: u32 },
    Flag { value: bool },
    Integer { value: i32 },
    Identifier { value_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioCellCapabilityDto {
    pub id: String,
    pub version: u32,
    pub definition_id: Option<String>,
    pub value: ScenarioCellCapabilityValueDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioCellDto {
    pub id: String,
    pub position: ScenarioPositionDto,
    pub capabilities: Vec<ScenarioCellCapabilityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioBoardDto {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<ScenarioCellDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(
    tag = "owner",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(tag = "owner", rename_all = "camelCase")]
pub enum ScenarioInitialCapabilityDto {
    Vitality {
        value: ScenarioBoundedValueDto,
    },
    Stat {
        id: String,
        value: i32,
    },
    Defense {
        id: String,
        value: i32,
    },
    Resource {
        id: String,
        value: ScenarioBoundedValueDto,
    },
    Modifier {
        stacking_group: String,
        id: String,
        value: i32,
        remaining_turns: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioParticipantDto {
    pub id: String,
    pub label: String,
    pub team_id: String,
    pub position: ScenarioPositionDto,
    pub definition_ids: Vec<String>,
    pub capabilities: Vec<ScenarioInitialCapabilityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioTurnDto {
    pub initiative_order: Vec<String>,
    pub current_actor_id: String,
    pub round: u32,
    pub turn: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioRandomSourceDto {
    pub policy_id: String,
    pub policy_version: u32,
    pub source_id: String,
    pub source_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioSetupRequestDto {
    pub schema: ScenarioSchemaDto,
    pub play_bundle_id: String,
    pub board: ScenarioBoardDto,
    pub participants: Vec<ScenarioParticipantDto>,
    pub turn: ScenarioTurnDto,
    pub random_source: ScenarioRandomSourceDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioTemplatePresentationDto {
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ScenarioTemplateDto {
    pub schema: ScenarioSchemaDto,
    pub identity: VersionedIdentityDto,
    pub play_bundle: VersionedIdentityDto,
    pub presentation: ScenarioTemplatePresentationDto,
    pub board: ScenarioBoardDto,
    pub participants: Vec<ScenarioParticipantDto>,
    pub turn: ScenarioTurnDto,
    pub random_source: ScenarioRandomSourceDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayUnavailableDto {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayActionOptionsDto {
    pub participant_ids: Vec<String>,
    pub cell_ids: Vec<String>,
    pub area_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayAuthorityActionDto {
    pub definition_id: String,
    pub label: String,
    pub available: bool,
    pub unavailable: Option<GameplayUnavailableDto>,
    pub maximum_targets: u32,
    pub options: GameplayActionOptionsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayTurnControlDto {
    pub kind: String,
    pub label: String,
    pub available: bool,
    pub unavailable: Option<GameplayUnavailableDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayLogEntryDto {
    pub sequence: String,
    pub state_revision: String,
    pub actor_id: String,
    pub action_id: String,
    pub events: Vec<GameplayEventDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayOutcomeDto {
    pub status: String,
    pub winning_team_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayCostDto {
    pub resource_id: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayRandomRequestDto {
    pub kind: String,
    pub count: u32,
    pub sides: u32,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum GameplayRandomPlanConditionKindDto {
    WhenThen,
    WhenOtherwise,
    CheckHit,
    CheckMiss,
    CheckSaved,
    CheckFailed,
    CheckNoRoll,
    AllPreviousTrue,
    AnyPreviousFalse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayRandomPlanConditionDto {
    pub kind: GameplayRandomPlanConditionKindDto,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayRandomPlanEntryDto {
    pub request: GameplayRandomRequestDto,
    pub conditions: Vec<GameplayRandomPlanConditionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayActionDto {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub team: String,
    pub maximum_range: u32,
    pub maximum_targets: u32,
    pub costs: Vec<GameplayCostDto>,
    pub random_plan: Vec<GameplayRandomPlanEntryDto>,
    pub candidate_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayPreflightDto {
    pub action_id: String,
    pub target_id: String,
    pub available: bool,
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayNamedValueDto {
    pub id: String,
    pub current: i32,
    pub maximum: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayModifierDto {
    pub stacking_group: String,
    pub id: String,
    pub value: i32,
    pub remaining_turns: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayEntityDto {
    pub id: String,
    pub label: String,
    pub team_id: String,
    pub x: u32,
    pub y: u32,
    pub definition_ids: Vec<String>,
    pub vitality: GameplayNamedValueDto,
    pub stats: Vec<GameplayNamedValueDto>,
    pub defenses: Vec<GameplayNamedValueDto>,
    pub resources: Vec<GameplayNamedValueDto>,
    pub modifiers: Vec<GameplayModifierDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayEventDto {
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayTraceDto {
    pub path: String,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayReactionOptionDto {
    pub id: String,
    pub label: String,
    pub damage_reduction: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayReactionDto {
    pub reaction_id: String,
    pub actor_id: String,
    pub target_id: String,
    pub action_id: String,
    pub options: Vec<GameplayReactionOptionDto>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayRandomEvidenceDto {
    pub kind: String,
    pub count: u32,
    pub sides: u32,
    pub path: String,
    pub values: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayResultDto {
    pub status: String,
    pub code: Option<String>,
    pub message: String,
    pub events: Vec<GameplayEventDto>,
    pub trace: Vec<GameplayTraceDto>,
    pub random_consumed: String,
    pub random_evidence: Vec<GameplayRandomEvidenceDto>,
    pub state_revision: u32,
    pub random_request: Option<GameplayRandomRequestDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayReplayBoundaryDto {
    pub revision: String,
    pub accepted_random_position: String,
    pub phase: String,
    pub state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayReplayEntryDto {
    pub sequence: usize,
    pub operation: String,
    pub outcome: String,
    pub before: GameplayReplayBoundaryDto,
    pub after: GameplayReplayBoundaryDto,
    pub random_evidence: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplayArchiveDto {
    pub checkpoint_schema: String,
    pub replay_schema_version: u32,
    pub event_schema_version: u32,
    pub artifact_id: String,
    pub artifact_schema: String,
    pub play_bundle: String,
    pub ruleset: String,
    pub operation_schemas: Vec<String>,
    pub capability_schemas: Vec<String>,
    pub content_packs: Vec<String>,
    pub dependency_lock: Vec<String>,
    pub fingerprints: PlayBundleFingerprintDto,
    pub definition_fingerprints: Vec<String>,
    pub state_revision: String,
    pub accepted_random_position: String,
    pub phase: String,
    pub state_hash: String,
    pub checkpoint_bytes: usize,
    pub replay_entries: Vec<GameplayReplayEntryDto>,
    pub verification_status: String,
    pub verification_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GameplaySessionDto {
    pub artifact_id: String,
    pub actor_id: String,
    pub state_revision: u32,
    pub accepted_random_values: String,
    pub random_source: ScenarioRandomSourceDto,
    pub board: ScenarioBoardDto,
    pub turn: ScenarioTurnDto,
    pub actions: Vec<GameplayAuthorityActionDto>,
    pub controls: Vec<GameplayTurnControlDto>,
    pub entities: Vec<GameplayEntityDto>,
    pub pending_reaction: Option<GameplayReactionDto>,
    pub log: Vec<GameplayLogEntryDto>,
    pub outcome: GameplayOutcomeDto,
    pub last_result: Option<GameplayResultDto>,
    pub archive: GameplayArchiveDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlayWorkspaceResponseDto {
    pub ok: bool,
    pub status: PlayBundleLifecycleStatus,
    pub active_artifact: Option<PlayBundleArtifactSummaryDto>,
    pub candidate_artifact: Option<PlayBundleArtifactSummaryDto>,
    pub upgrade_impact: Option<PlayBundleUpgradeImpactDto>,
    pub activation_revision: u32,
    pub host_random_source: ScenarioRandomSourceDto,
    pub supported_random_sources: Vec<ScenarioRandomSourceDto>,
    pub scenario_setup_required: bool,
    pub gameplay_available: bool,
    pub gameplay: Option<GameplaySessionDto>,
    pub diagnostics: Vec<PlayDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConfiguredRulesetLocationDto {
    pub id: String,
    pub label: String,
    pub ruleset_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct RulesetLocationConfigDto {
    pub schema_version: u32,
    pub rulesets: Vec<ConfiguredRulesetLocationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct RulesetCatalogRequestDto {
    pub ruleset_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetCatalogContentPackDto {
    pub id: String,
    pub version: String,
    pub label: String,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetCatalogPlayBundleDto {
    pub id: String,
    pub version: String,
    pub content_pack_ids: Vec<String>,
    pub compatible: bool,
    pub diagnostics: Vec<PlayDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetCatalogDto {
    pub ruleset_root: String,
    pub ruleset: VersionedIdentityDto,
    pub content_packs: Vec<RulesetCatalogContentPackDto>,
    pub play_bundles: Vec<RulesetCatalogPlayBundleDto>,
    pub scenarios: Vec<ScenarioTemplateDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetCatalogResponseDto {
    pub ok: bool,
    pub catalog: Option<RulesetCatalogDto>,
    pub diagnostics: Vec<PlayDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct PlayBundleCompileRequestDto {
    pub ruleset_root: String,
    pub content_pack_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct PreparedPlayBundleCompileRequestDto {
    pub prepared_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct GameplayCommandRequestDto {
    pub expected_revision: u32,
    pub action_id: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct GameplayReactionRequestDto {
    pub expected_revision: u32,
    pub reaction_id: String,
    pub option_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct GameplayTurnControlRequestDto {
    pub expected_revision: u32,
    pub actor_id: String,
    pub kind: String,
}

#[derive(Debug)]
pub struct SystemGameplayRandomSource {
    binding: RpgRandomSourceBinding,
}

impl Default for SystemGameplayRandomSource {
    fn default() -> Self {
        Self {
            binding: random_source_binding("random.system"),
        }
    }
}

impl RpgRandomSource for SystemGameplayRandomSource {
    fn binding(&self) -> &RpgRandomSourceBinding {
        &self.binding
    }

    fn draw(&mut self, request: &RpgRandomRequest) -> Result<Vec<u32>, RpgRandomSourceFailure> {
        if request.sides == 0 {
            return Err(random_source_failure(
                "SESSION_RANDOM_REQUEST_SIDES_INVALID",
                &request.path,
                "authority requested a die with zero sides",
            ));
        }
        (0..request.count)
            .map(|_| system_die_value(request.sides, &request.path))
            .collect()
    }
}

#[derive(Debug)]
pub struct ScriptedGameplayRandomSource {
    binding: RpgRandomSourceBinding,
    values: VecDeque<u32>,
}

impl ScriptedGameplayRandomSource {
    pub fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            binding: random_source_binding("random.roll-tape"),
            values: values.into_iter().collect(),
        }
    }

    pub fn remaining(&self) -> usize {
        self.values.len()
    }
}

impl RpgRandomSource for ScriptedGameplayRandomSource {
    fn binding(&self) -> &RpgRandomSourceBinding {
        &self.binding
    }

    fn draw(&mut self, request: &RpgRandomRequest) -> Result<Vec<u32>, RpgRandomSourceFailure> {
        if request.sides == 0 {
            return Err(random_source_failure(
                "SESSION_RANDOM_REQUEST_SIDES_INVALID",
                &request.path,
                "authority requested a die with zero sides",
            ));
        }
        let count = usize::try_from(request.count).map_err(|_| {
            random_source_failure(
                "SESSION_RANDOM_REQUEST_COUNT_INVALID",
                &request.path,
                "authority random request count exceeds this host's address space",
            )
        })?;
        if self.values.len() < count {
            return Err(random_source_failure(
                "SESSION_RANDOM_TAPE_EXHAUSTED",
                &request.path,
                format!(
                    "authority requested {}d{}, but the configured roll tape has {} value(s) remaining",
                    request.count,
                    request.sides,
                    self.values.len()
                ),
            ));
        }
        let candidate = self.values.iter().take(count).copied().collect::<Vec<_>>();
        if let Some((index, value)) = candidate
            .iter()
            .enumerate()
            .find(|(_, value)| **value == 0 || **value > request.sides)
        {
            return Err(random_source_failure(
                "SESSION_RANDOM_TAPE_VALUE_INVALID",
                &request.path,
                format!(
                    "roll tape value {} at request offset {} is outside 1..={} for {}",
                    value, index, request.sides, request.path
                ),
            ));
        }
        for _ in 0..count {
            self.values.pop_front();
        }
        Ok(candidate)
    }
}

fn system_die_value(sides: u32, path: &str) -> Result<u32, RpgRandomSourceFailure> {
    let unbiased_range = u32::MAX - (u32::MAX % sides);
    loop {
        let mut bytes = [0_u8; 4];
        getrandom::fill(&mut bytes).map_err(|error| {
            random_source_failure(
                "SESSION_SYSTEM_RANDOM_UNAVAILABLE",
                path,
                format!("system random source failed: {error}"),
            )
        })?;
        let value = u32::from_le_bytes(bytes);
        if value < unbiased_range {
            return Ok((value % sides) + 1);
        }
    }
}

fn random_source_failure(
    code: &str,
    path: &str,
    message: impl Into<String>,
) -> RpgRandomSourceFailure {
    RpgRandomSourceFailure {
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.into(),
        expected_request: None,
        actual_request: None,
    }
}

fn random_source_binding(source_id: &str) -> RpgRandomSourceBinding {
    RpgRandomSourceBinding {
        policy_id: "random.automatic".to_owned(),
        policy_version: 1,
        source_id: source_id.to_owned(),
        source_version: 1,
    }
}

#[derive(Debug)]
struct ActivePlayBundle {
    bundle: CompiledPlayBundle,
    encounter: Option<ActiveEncounter>,
}

#[derive(Debug)]
struct ActiveEncounter {
    session: RpgAuthoritySession,
    last_result: Option<GameplayResultDto>,
    initial_checkpoint: RpgSessionCheckpoint,
    latest_checkpoint: RpgSessionCheckpoint,
    latest_checkpoint_bytes: Vec<u8>,
    replay_entries: Vec<RpgReplayEntry>,
    verification_status: String,
    verification_message: String,
}

#[derive(Debug)]
struct ActivationSlots {
    candidate: Option<CompiledPlayBundle>,
    active: Option<ActivePlayBundle>,
    activation_revision: u32,
    random_source_binding: RpgRandomSourceBinding,
}

impl Default for ActivationSlots {
    fn default() -> Self {
        Self {
            candidate: None,
            active: None,
            activation_revision: 0,
            random_source_binding: random_source_binding("random.system"),
        }
    }
}

impl ActivationSlots {
    fn stage(&mut self, candidate: CompiledPlayBundle) {
        self.candidate = Some(candidate);
    }

    fn clear_candidate(&mut self) {
        self.candidate = None;
    }

    fn activate(&mut self) -> bool {
        let Some(candidate) = self.candidate.take() else {
            return false;
        };
        self.active = Some(ActivePlayBundle {
            bundle: candidate,
            encounter: None,
        });
        self.activation_revision += 1;
        true
    }

    fn status(&self) -> PlayBundleLifecycleStatus {
        if self.candidate.is_some() {
            PlayBundleLifecycleStatus::CompiledCandidate
        } else if self.active.is_some() {
            PlayBundleLifecycleStatus::Active
        } else {
            PlayBundleLifecycleStatus::NoActivePlayBundle
        }
    }
}

impl ActiveEncounter {
    fn store_entry(&mut self, entry: RpgReplayEntry) -> Result<(), RpgReplayFailure> {
        let checkpoint = self.session.checkpoint()?;
        let checkpoint_bytes = self.session.checkpoint_json()?;
        self.replay_entries.push(entry);
        self.latest_checkpoint = checkpoint;
        self.latest_checkpoint_bytes = checkpoint_bytes;
        self.verification_status = "notRun".to_owned();
        self.verification_message = "Stored replay changed; verification is required".to_owned();
        Ok(())
    }
}

pub struct PlayHost {
    slots: Mutex<ActivationSlots>,
    random_source: Mutex<Box<dyn RpgRandomSource>>,
}

impl Default for PlayHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayHost {
    pub fn new() -> Self {
        Self::with_random_source(SystemGameplayRandomSource::default())
    }

    pub fn with_random_source(source: impl RpgRandomSource + 'static) -> Self {
        let random_source_binding = source.binding().clone();
        Self {
            slots: Mutex::new(ActivationSlots {
                random_source_binding,
                ..ActivationSlots::default()
            }),
            random_source: Mutex::new(Box::new(source)),
        }
    }

    pub fn status(&self) -> PlayWorkspaceResponseDto {
        let slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        response_from_slots(true, &slots, Vec::new())
    }

    pub fn compile_candidate(&self, prepared_source: &str) -> PlayWorkspaceResponseDto {
        let compilation = compile_prepared_play_bundle_json(prepared_source.as_bytes());
        match compilation {
            Ok(bundle) => match close_portable_artifact(bundle) {
                Ok(loaded) => {
                    let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                    slots.stage(loaded);
                    response_from_slots(true, &slots, Vec::new())
                }
                Err(diagnostics) => {
                    let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                    slots.clear_candidate();
                    response_from_slots(false, &slots, diagnostics)
                }
            },
            Err(failure) => {
                let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                slots.clear_candidate();
                response_from_slots(false, &slots, diagnostics_from_failure(failure))
            }
        }
    }

    pub fn activate_candidate(&self) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        match slots.activate() {
            true => response_from_slots(true, &slots, Vec::new()),
            false => response_from_slots(
                false,
                &slots,
                vec![PlayDiagnosticDto {
                    stage: "activation".to_owned(),
                    severity: "error".to_owned(),
                    code: "PLAY_BUNDLE_ACTIVATION_CANDIDATE_REQUIRED".to_owned(),
                    path: "$.candidateArtifact".to_owned(),
                    message: "compile an accepted artifact before activation".to_owned(),
                    package_id: None,
                    definition_id: None,
                    source: None,
                    graph_path: None,
                    expected: None,
                    actual: None,
                }],
            ),
        }
    }

    pub fn start_encounter(&self, request: ScenarioSetupRequestDto) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(bundle) = slots.active.as_ref().map(|active| active.bundle.clone()) else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic_at_stage(
                    "setup",
                    "RPG_SETUP_ACTIVE_ARTIFACT_REQUIRED",
                    "$.playBundleId",
                    "activate a compiled PlayBundle before creating a Scenario",
                )],
            );
        };
        let setup = scenario(request);
        if setup.random_source != slots.random_source_binding {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic_at_stage(
                    "setup",
                    "RPG_RANDOM_SOURCE_BINDING_MISMATCH",
                    "$.randomSource",
                    "encounter random source must match the source selected by this host",
                )],
            );
        }
        let session = match RpgAuthoritySession::from_scenario(bundle, setup) {
            Ok(session) => session,
            Err(failure) => {
                return response_from_slots(false, &slots, diagnostics_from_setup_failure(failure));
            }
        };
        let checkpoint = match session.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(failure) => {
                return response_from_slots(
                    false,
                    &slots,
                    diagnostics_from_replay_failure(failure),
                );
            }
        };
        let checkpoint_bytes = match session.checkpoint_json() {
            Ok(checkpoint_bytes) => checkpoint_bytes,
            Err(failure) => {
                return response_from_slots(
                    false,
                    &slots,
                    diagnostics_from_replay_failure(failure),
                );
            }
        };
        let encounter = ActiveEncounter {
            session,
            last_result: None,
            initial_checkpoint: checkpoint.clone(),
            latest_checkpoint: checkpoint,
            latest_checkpoint_bytes: checkpoint_bytes,
            replay_entries: Vec::new(),
            verification_status: "notRun".to_owned(),
            verification_message: "No replay verification has run yet".to_owned(),
        };
        if let Some(active) = &mut slots.active {
            active.encounter = Some(encounter);
        }
        response_from_slots(true, &slots, Vec::new())
    }

    pub fn execute_command(&self, request: GameplayCommandRequestDto) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = slots
            .active
            .as_mut()
            .and_then(|active| active.encounter.as_mut())
        else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_SESSION_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "create an encounter before submitting gameplay",
                )],
            );
        };
        let mut random_source = self
            .random_source
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let recorded = active.session.submit_with_random_source_recorded(
            RpgActionProposal {
                expected_revision: u64::from(request.expected_revision),
                action_id: request.action_id,
                actor_id: request.actor_id,
                target_ids: request.target_ids,
            },
            random_source.as_mut(),
        );
        let (outcome, entry) = match recorded {
            Ok(recorded) => recorded,
            Err(failure) => {
                return response_from_slots(
                    false,
                    &slots,
                    diagnostics_from_automatic_failure(failure),
                );
            }
        };
        if let Err(failure) = active.store_entry(entry) {
            return response_from_slots(false, &slots, diagnostics_from_replay_failure(failure));
        }
        active.last_result = Some(gameplay_result(&outcome, active.session.state().revision()));
        response_from_slots(
            !matches!(outcome, RpgCommandOutcome::Rejected(_)),
            &slots,
            Vec::new(),
        )
    }

    pub fn resolve_reaction(
        &self,
        request: GameplayReactionRequestDto,
    ) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = slots
            .active
            .as_mut()
            .and_then(|active| active.encounter.as_mut())
        else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_SESSION_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "create an encounter before resolving a reaction",
                )],
            );
        };
        let mut random_source = self
            .random_source
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let recorded = active.session.react_with_random_source_recorded(
            RpgReactionProposal {
                expected_revision: u64::from(request.expected_revision),
                reaction_id: request.reaction_id,
                option_id: request.option_id,
            },
            random_source.as_mut(),
        );
        let (outcome, entry) = match recorded {
            Ok(recorded) => recorded,
            Err(failure) => {
                return response_from_slots(
                    false,
                    &slots,
                    diagnostics_from_automatic_failure(failure),
                );
            }
        };
        if let Err(failure) = active.store_entry(entry) {
            return response_from_slots(false, &slots, diagnostics_from_replay_failure(failure));
        }
        active.last_result = Some(gameplay_result(&outcome, active.session.state().revision()));
        response_from_slots(
            !matches!(outcome, RpgCommandOutcome::Rejected(_)),
            &slots,
            Vec::new(),
        )
    }

    pub fn execute_turn_control(
        &self,
        request: GameplayTurnControlRequestDto,
    ) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = slots
            .active
            .as_mut()
            .and_then(|active| active.encounter.as_mut())
        else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_SESSION_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "create an encounter before submitting turn control",
                )],
            );
        };
        let control = match request.kind.as_str() {
            "endTurn" => RpgTurnControl::EndTurn,
            _ => {
                let message = format!("unsupported turn control {}", request.kind);
                return response_from_slots(
                    false,
                    &slots,
                    vec![host_diagnostic(
                        "RPG_TURN_CONTROL_UNSUPPORTED",
                        "$.kind",
                        &message,
                    )],
                );
            }
        };
        let recorded = active.session.control_recorded(RpgTurnControlProposal {
            expected_revision: u64::from(request.expected_revision),
            actor_id: request.actor_id,
            control,
        });
        let (outcome, entry) = match recorded {
            Ok(recorded) => recorded,
            Err(failure) => {
                return response_from_slots(
                    false,
                    &slots,
                    diagnostics_from_replay_failure(failure),
                );
            }
        };
        if let Err(failure) = active.store_entry(entry) {
            return response_from_slots(false, &slots, diagnostics_from_replay_failure(failure));
        }
        active.last_result = Some(gameplay_result(&outcome, active.session.state().revision()));
        response_from_slots(
            !matches!(outcome, RpgCommandOutcome::Rejected(_)),
            &slots,
            Vec::new(),
        )
    }

    pub fn restore_latest_checkpoint(&self) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = slots
            .active
            .as_mut()
            .and_then(|active| active.encounter.as_mut())
        else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_CHECKPOINT_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "create an encounter before restoring a checkpoint",
                )],
            );
        };
        match RpgAuthoritySession::restore_checkpoint_json(&active.latest_checkpoint_bytes) {
            Ok(restored) => {
                active.session = restored;
                active.verification_status = "checkpointRestored".to_owned();
                active.verification_message = format!(
                    "Restored checkpoint {} at revision {}",
                    active.latest_checkpoint.state_hash.value,
                    active.latest_checkpoint.state.revision
                );
                response_from_slots(true, &slots, Vec::new())
            }
            Err(failure) => {
                response_from_slots(false, &slots, diagnostics_from_replay_failure(failure))
            }
        }
    }

    pub fn replay_archive(&self) -> PlayWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = slots
            .active
            .as_mut()
            .and_then(|active| active.encounter.as_mut())
        else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_REPLAY_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "create an encounter before replaying stored records",
                )],
            );
        };
        match active
            .session
            .replay_into(active.initial_checkpoint.clone(), &active.replay_entries)
        {
            Ok(()) => match active.session.checkpoint() {
                Ok(checkpoint) => {
                    let checkpoint_bytes = match active.session.checkpoint_json() {
                        Ok(checkpoint_bytes) => checkpoint_bytes,
                        Err(failure) => {
                            return response_from_slots(
                                false,
                                &slots,
                                diagnostics_from_replay_failure(failure),
                            );
                        }
                    };
                    active.latest_checkpoint = checkpoint;
                    active.latest_checkpoint_bytes = checkpoint_bytes;
                    active.verification_status = "verified".to_owned();
                    active.verification_message = format!(
                        "Rust replay verified {} record(s) at state hash {}",
                        active.replay_entries.len(),
                        active.latest_checkpoint.state_hash.value
                    );
                    response_from_slots(true, &slots, Vec::new())
                }
                Err(failure) => {
                    response_from_slots(false, &slots, diagnostics_from_replay_failure(failure))
                }
            },
            Err(failure) => {
                active.verification_status = "failed".to_owned();
                active.verification_message = failure.to_string();
                response_from_slots(false, &slots, diagnostics_from_replay_failure(failure))
            }
        }
    }
}

fn close_portable_artifact(
    bundle: CompiledPlayBundle,
) -> Result<CompiledPlayBundle, Vec<PlayDiagnosticDto>> {
    let encoded = serde_json::to_vec(bundle.artifact()).map_err(|error| {
        vec![PlayDiagnosticDto {
            stage: "artifact".to_owned(),
            severity: "error".to_owned(),
            code: "RULESET_ARTIFACT_ENCODING_FAILED".to_owned(),
            path: "$".to_owned(),
            message: error.to_string(),
            package_id: None,
            definition_id: None,
            source: None,
            graph_path: None,
            expected: None,
            actual: None,
        }]
    })?;
    load_compiled_play_bundle_json(&encoded).map_err(diagnostics_from_failure)
}

fn response_from_slots(
    ok: bool,
    slots: &ActivationSlots,
    diagnostics: Vec<PlayDiagnosticDto>,
) -> PlayWorkspaceResponseDto {
    let upgrade_impact = match (&slots.active, &slots.candidate) {
        (Some(active), Some(candidate)) => Some(upgrade_impact(
            active.bundle.artifact(),
            candidate.artifact(),
        )),
        _ => None,
    };
    PlayWorkspaceResponseDto {
        ok,
        status: slots.status(),
        active_artifact: slots
            .active
            .as_ref()
            .map(|active| artifact_summary(active.bundle.artifact())),
        candidate_artifact: slots
            .candidate
            .as_ref()
            .map(|bundle| artifact_summary(bundle.artifact())),
        upgrade_impact,
        activation_revision: slots.activation_revision,
        host_random_source: encounter_random_source(&slots.random_source_binding),
        supported_random_sources: vec![encounter_random_source(&slots.random_source_binding)],
        scenario_setup_required: slots
            .active
            .as_ref()
            .is_some_and(|active| active.encounter.is_none()),
        gameplay_available: slots
            .active
            .as_ref()
            .is_some_and(|active| active.encounter.is_some()),
        gameplay: slots
            .active
            .as_ref()
            .and_then(|active| active.encounter.as_ref())
            .map(gameplay_session),
        diagnostics,
    }
}

fn upgrade_impact(
    active: &CompiledPlayBundleArtifact,
    candidate: &CompiledPlayBundleArtifact,
) -> PlayBundleUpgradeImpactDto {
    let active_sources = active
        .content_packs
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let candidate_sources = candidate
        .content_packs
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let mut source_changes = Vec::new();
    let source_ids = active_sources
        .keys()
        .chain(candidate_sources.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for source_id in source_ids {
        let before = active_sources.get(source_id);
        let after = candidate_sources.get(source_id);
        let change = match (before, after) {
            (Some(before), Some(after))
                if before.version != after.version
                    || before.source_fingerprint != after.source_fingerprint =>
            {
                Some(format!(
                    "{source_id}: {} ({}) → {} ({})",
                    before.version,
                    before.source_fingerprint,
                    after.version,
                    after.source_fingerprint
                ))
            }
            (Some(before), None) => Some(format!(
                "{source_id}: {} ({}) → removed",
                before.version, before.source_fingerprint
            )),
            (None, Some(after)) => Some(format!(
                "{source_id}: added {} ({})",
                after.version, after.source_fingerprint
            )),
            _ => None,
        };
        if let Some(change) = change {
            source_changes.push(change);
        }
    }

    let active_definitions = active
        .materialized_definitions
        .iter()
        .map(|definition| (definition.id.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    let candidate_definitions = candidate
        .materialized_definitions
        .iter()
        .map(|definition| (definition.id.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    let mut definitions = Vec::new();
    let definition_ids = active_definitions
        .keys()
        .chain(candidate_definitions.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for definition_id in definition_ids {
        let before = active_definitions.get(definition_id).copied();
        let after = candidate_definitions.get(definition_id).copied();
        if before.map(|definition| &definition.fingerprint)
            == after.map(|definition| &definition.fingerprint)
        {
            continue;
        }
        let mut fields = Vec::new();
        diff_json_values(
            "semantic",
            "$.semantic",
            before.map(|definition| &definition.semantic),
            after.map(|definition| &definition.semantic),
            &mut fields,
        );
        diff_json_values(
            "presentation",
            "$.presentation",
            before.map(|definition| &definition.presentation),
            after.map(|definition| &definition.presentation),
            &mut fields,
        );
        let change = match (before, after) {
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (Some(_), Some(_)) => "changed",
            (None, None) => continue,
        };
        definitions.push(PlayBundleUpgradeDefinitionDto {
            definition_id: definition_id.to_owned(),
            change: change.to_owned(),
            descendant: is_derived_definition(active, definition_id)
                || is_derived_definition(candidate, definition_id),
            causes: upgrade_causes(active, candidate, definition_id, before, after),
            fields,
        });
    }

    PlayBundleUpgradeImpactDto {
        from_artifact_id: active.artifact_id.clone(),
        to_artifact_id: candidate.artifact_id.clone(),
        source_changes,
        definitions,
    }
}

fn is_derived_definition(artifact: &CompiledPlayBundleArtifact, definition_id: &str) -> bool {
    artifact
        .derivation_provenance
        .iter()
        .any(|provenance| provenance.definition_id == definition_id)
}

fn upgrade_causes(
    active: &CompiledPlayBundleArtifact,
    candidate: &CompiledPlayBundleArtifact,
    definition_id: &str,
    before: Option<&rpg_ir::MaterializedContentDefinition>,
    after: Option<&rpg_ir::MaterializedContentDefinition>,
) -> Vec<String> {
    let mut causes = Vec::new();
    if before.map(|definition| {
        (
            definition.provenance.package_id.as_str(),
            definition.provenance.package_version.as_str(),
        )
    }) != after.map(|definition| {
        (
            definition.provenance.package_id.as_str(),
            definition.provenance.package_version.as_str(),
        )
    }) {
        causes.push("definition owner version changed".to_owned());
    }
    let active_derivation = active
        .derivation_provenance
        .iter()
        .find(|provenance| provenance.definition_id == definition_id);
    let candidate_derivation = candidate
        .derivation_provenance
        .iter()
        .find(|provenance| provenance.definition_id == definition_id);
    match (active_derivation, candidate_derivation) {
        (Some(before), Some(after)) => {
            if before.base_package_id != after.base_package_id
                || before.base_package_version != after.base_package_version
                || before.base_definition_id != after.base_definition_id
                || before.base_fingerprint != after.base_fingerprint
            {
                causes.push("primary base identity or fingerprint changed".to_owned());
            }
            if before.mixins != after.mixins {
                causes.push(
                    "ordered mixin identities, parameters, or fingerprints changed".to_owned(),
                );
            }
            if before.local_patch_fingerprint != after.local_patch_fingerprint {
                causes.push("local patch fingerprint changed".to_owned());
            }
        }
        (None, Some(_)) => causes.push("definition became derived".to_owned()),
        (Some(_), None) => causes.push("definition is no longer derived".to_owned()),
        (None, None) => {}
    }
    let active_overlays = active
        .overlay_provenance
        .iter()
        .filter(|provenance| provenance.target_definition_id == definition_id)
        .collect::<Vec<_>>();
    let candidate_overlays = candidate
        .overlay_provenance
        .iter()
        .filter(|provenance| provenance.target_definition_id == definition_id)
        .collect::<Vec<_>>();
    if active_overlays != candidate_overlays {
        causes.push("PlayBundle-ordered overlay provenance changed".to_owned());
    }
    if causes.is_empty() {
        causes.push("materialized definition fields changed".to_owned());
    }
    causes
}

fn diff_json_values(
    plane: &str,
    path: &str,
    before: Option<&Value>,
    after: Option<&Value>,
    fields: &mut Vec<PlayBundleUpgradeFieldDto>,
) {
    if before == after {
        return;
    }
    match (before, after) {
        (Some(Value::Object(before)), Some(Value::Object(after))) => {
            for key in before.keys().chain(after.keys()) {
                let child_path = json_field_path(path, key);
                let before_value = before.get(key);
                let after_value = after.get(key);
                if fields.last().is_some_and(|field| field.path == child_path) {
                    continue;
                }
                diff_json_values(plane, &child_path, before_value, after_value, fields);
            }
        }
        (Some(Value::Array(before)), Some(Value::Array(after))) => {
            let length = before.len().max(after.len());
            for index in 0..length {
                diff_json_values(
                    plane,
                    &format!("{path}[{index}]"),
                    before.get(index),
                    after.get(index),
                    fields,
                );
            }
        }
        _ => fields.push(PlayBundleUpgradeFieldDto {
            plane: plane.to_owned(),
            path: path.to_owned(),
            before: before
                .map(json_value)
                .unwrap_or_else(|| "<missing>".to_owned()),
            after: after
                .map(json_value)
                .unwrap_or_else(|| "<missing>".to_owned()),
        }),
    }
}

fn json_field_path(parent: &str, field: &str) -> String {
    if field
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        format!("{parent}.{field}")
    } else {
        format!("{parent}[{}]", json_value(&Value::String(field.to_owned())))
    }
}

fn scenario(request: ScenarioSetupRequestDto) -> RpgScenario {
    RpgScenario {
        schema: RpgSchemaIdentity {
            id: request.schema.id,
            version: request.schema.version,
        },
        play_bundle_id: request.play_bundle_id,
        board: RpgBoardSetup {
            width: request.board.width,
            height: request.board.height,
            cells: request
                .board
                .cells
                .into_iter()
                .map(|cell| RpgCellSetup {
                    id: cell.id,
                    position: grid_position(cell.position),
                    capabilities: cell
                        .capabilities
                        .into_iter()
                        .map(|capability| RpgCellCapabilitySetup {
                            id: capability.id,
                            version: capability.version,
                            definition_id: capability.definition_id,
                            value: match capability.value {
                                ScenarioCellCapabilityValueDto::Traversal {
                                    passable,
                                    movement_cost,
                                } => RpgCellCapabilityValue::Traversal {
                                    passable,
                                    movement_cost,
                                },
                                ScenarioCellCapabilityValueDto::Flag { value } => {
                                    RpgCellCapabilityValue::Flag { value }
                                }
                                ScenarioCellCapabilityValueDto::Integer { value } => {
                                    RpgCellCapabilityValue::Integer { value }
                                }
                                ScenarioCellCapabilityValueDto::Identifier { value_id } => {
                                    RpgCellCapabilityValue::Identifier { value_id }
                                }
                            },
                        })
                        .collect(),
                })
                .collect(),
        },
        participants: request
            .participants
            .into_iter()
            .map(|participant| RpgParticipantSetup {
                id: participant.id,
                label: participant.label,
                team_id: RpgTeamId::named(participant.team_id),
                position: grid_position(participant.position),
                definition_ids: participant.definition_ids,
                capabilities: participant
                    .capabilities
                    .into_iter()
                    .map(initial_capability)
                    .collect(),
            })
            .collect(),
        turn: RpgTurnInitialization {
            initiative_order: request.turn.initiative_order,
            current_actor_id: request.turn.current_actor_id,
            round: u64::from(request.turn.round),
            turn: u64::from(request.turn.turn),
        },
        random_source: RpgRandomSourceBinding {
            policy_id: request.random_source.policy_id,
            policy_version: request.random_source.policy_version,
            source_id: request.random_source.source_id,
            source_version: request.random_source.source_version,
        },
    }
}

fn grid_position(position: ScenarioPositionDto) -> GridPosition {
    GridPosition {
        x: position.x,
        y: position.y,
    }
}

fn initial_capability(capability: ScenarioInitialCapabilityDto) -> RpgInitialCapability {
    match capability {
        ScenarioInitialCapabilityDto::Vitality { value } => RpgInitialCapability::Vitality {
            value: bounded_value(value),
        },
        ScenarioInitialCapabilityDto::Stat { id, value } => {
            RpgInitialCapability::Stat { id, value }
        }
        ScenarioInitialCapabilityDto::Defense { id, value } => {
            RpgInitialCapability::Defense { id, value }
        }
        ScenarioInitialCapabilityDto::Resource { id, value } => RpgInitialCapability::Resource {
            id,
            value: bounded_value(value),
        },
        ScenarioInitialCapabilityDto::Modifier {
            stacking_group,
            id,
            value,
            remaining_turns,
        } => RpgInitialCapability::Modifier {
            stacking_group,
            id,
            value,
            remaining_turns,
        },
    }
}

fn bounded_value(value: ScenarioBoundedValueDto) -> BoundedValue {
    BoundedValue {
        current: value.current,
        max: value.max,
    }
}

fn encounter_random_source(binding: &RpgRandomSourceBinding) -> ScenarioRandomSourceDto {
    ScenarioRandomSourceDto {
        policy_id: binding.policy_id.clone(),
        policy_version: binding.policy_version,
        source_id: binding.source_id.clone(),
        source_version: binding.source_version,
    }
}

fn encounter_board(board: &RpgBoardSetup) -> ScenarioBoardDto {
    ScenarioBoardDto {
        width: board.width,
        height: board.height,
        cells: board
            .cells
            .iter()
            .map(|cell| ScenarioCellDto {
                id: cell.id.clone(),
                position: ScenarioPositionDto {
                    x: cell.position.x,
                    y: cell.position.y,
                },
                capabilities: cell
                    .capabilities
                    .iter()
                    .map(|capability| ScenarioCellCapabilityDto {
                        id: capability.id.clone(),
                        version: capability.version,
                        definition_id: capability.definition_id.clone(),
                        value: match &capability.value {
                            RpgCellCapabilityValue::Traversal {
                                passable,
                                movement_cost,
                            } => ScenarioCellCapabilityValueDto::Traversal {
                                passable: *passable,
                                movement_cost: *movement_cost,
                            },
                            RpgCellCapabilityValue::Flag { value } => {
                                ScenarioCellCapabilityValueDto::Flag { value: *value }
                            }
                            RpgCellCapabilityValue::Integer { value } => {
                                ScenarioCellCapabilityValueDto::Integer { value: *value }
                            }
                            RpgCellCapabilityValue::Identifier { value_id } => {
                                ScenarioCellCapabilityValueDto::Identifier {
                                    value_id: value_id.clone(),
                                }
                            }
                        },
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn encounter_turn(turn: &rpg_runtime::RpgTurnState) -> ScenarioTurnDto {
    ScenarioTurnDto {
        initiative_order: turn.initiative_order.clone(),
        current_actor_id: turn.current_actor_id.clone(),
        round: dto_revision(turn.round),
        turn: dto_revision(turn.turn),
    }
}

fn gameplay_session(active: &ActiveEncounter) -> GameplaySessionDto {
    let view = active.session.encounter_view();
    GameplaySessionDto {
        artifact_id: view.artifact_id.clone(),
        actor_id: view.turn.current_actor_id.clone(),
        state_revision: dto_revision(view.state_revision),
        accepted_random_values: view.accepted_random_position.to_string(),
        random_source: encounter_random_source(&view.random_source),
        board: encounter_board(&view.board),
        turn: encounter_turn(&view.turn),
        actions: view.actions.iter().map(gameplay_authority_action).collect(),
        controls: view.controls.iter().map(gameplay_turn_control).collect(),
        entities: view.participants.iter().map(gameplay_entity).collect(),
        pending_reaction: view.pending_reaction.as_ref().map(gameplay_reaction),
        log: view.log.iter().map(gameplay_log_entry).collect(),
        outcome: gameplay_outcome(&view.outcome),
        last_result: active.last_result.clone(),
        archive: gameplay_archive(active),
    }
}

fn gameplay_archive(active: &ActiveEncounter) -> GameplayArchiveDto {
    let checkpoint = &active.latest_checkpoint;
    let binding = &checkpoint.artifact_binding;
    GameplayArchiveDto {
        checkpoint_schema: format!("{}@{}", checkpoint.schema.id, checkpoint.schema.version),
        replay_schema_version: checkpoint.schemas.replay_entry,
        event_schema_version: checkpoint.schemas.event,
        artifact_id: binding.artifact_id.clone(),
        artifact_schema: format!(
            "{}@{}",
            binding.artifact_schema.identity, binding.artifact_schema.major
        ),
        play_bundle: format!("{}@{}", binding.play_bundle.id, binding.play_bundle.version),
        ruleset: format!("{}@{}", binding.ruleset.id, binding.ruleset.version),
        operation_schemas: checkpoint
            .schemas
            .operations
            .iter()
            .map(|requirement| format!("{}@{}", requirement.id, requirement.version))
            .collect(),
        capability_schemas: checkpoint
            .schemas
            .capabilities
            .iter()
            .map(|requirement| format!("{}@{}", requirement.id, requirement.version))
            .collect(),
        content_packs: binding
            .content_packs
            .iter()
            .map(|package| {
                format!(
                    "{}@{} · {}",
                    package.id, package.version, package.source_fingerprint
                )
            })
            .collect(),
        dependency_lock: binding
            .dependency_lock
            .iter()
            .map(|entry| {
                format!(
                    "{} --{} {} → {}--> {} as {} · {}",
                    entry.requester,
                    dependency_relationship(entry.relationship),
                    entry.requested_version,
                    entry.resolved_version,
                    entry.package_id,
                    entry.import_as,
                    entry.source_fingerprint
                )
            })
            .collect(),
        fingerprints: PlayBundleFingerprintDto {
            source: binding.fingerprints.source.clone(),
            semantic: binding.fingerprints.semantic.clone(),
            presentation: binding.fingerprints.presentation.clone(),
        },
        definition_fingerprints: binding
            .definitions
            .iter()
            .map(|definition| format!("{} · {}", definition.id, definition.fingerprint))
            .collect(),
        state_revision: checkpoint.state.revision.to_string(),
        accepted_random_position: checkpoint.accepted_random_position.to_string(),
        phase: checkpoint_phase_label(&checkpoint.phase),
        state_hash: format!(
            "{}:{}",
            checkpoint.state_hash.algorithm, checkpoint.state_hash.value
        ),
        checkpoint_bytes: active.latest_checkpoint_bytes.len(),
        replay_entries: active
            .replay_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| gameplay_replay_entry(index, entry))
            .collect(),
        verification_status: active.verification_status.clone(),
        verification_message: active.verification_message.clone(),
    }
}

fn gameplay_replay_entry(index: usize, entry: &RpgReplayEntry) -> GameplayReplayEntryDto {
    let (random_evidence, events) = match &entry.outcome {
        RpgCommandOutcome::Accepted(receipt) => (
            receipt
                .random_evidence
                .iter()
                .map(gameplay_random_evidence_label)
                .collect(),
            receipt
                .events
                .iter()
                .map(|event| gameplay_event(event).summary)
                .collect(),
        ),
        RpgCommandOutcome::ControlAccepted(receipt) => (
            Vec::new(),
            receipt
                .events
                .iter()
                .map(|event| gameplay_event(event).summary)
                .collect(),
        ),
        RpgCommandOutcome::AwaitingReaction(pending) => (
            pending
                .random_evidence
                .iter()
                .map(gameplay_random_evidence_label)
                .collect(),
            Vec::new(),
        ),
        RpgCommandOutcome::Rejected(rejection) => (
            rejection
                .random_evidence
                .iter()
                .map(gameplay_random_evidence_label)
                .collect(),
            Vec::new(),
        ),
    };
    GameplayReplayEntryDto {
        sequence: index + 1,
        operation: match &entry.operation {
            RpgReplayOperation::Submit { command } => {
                format!(
                    "submit {} by {} → {} at revision {}",
                    command.intent.action_id,
                    command.intent.actor_id,
                    command.intent.target_ids.join(", "),
                    command.expected_revision
                )
            }
            RpgReplayOperation::React { command } => {
                format!(
                    "react {} with {} at revision {}",
                    command.reaction_id,
                    command.option_id.as_deref().unwrap_or("decline"),
                    command.expected_revision
                )
            }
            RpgReplayOperation::TurnControl { command } => format!(
                "{} by {} at revision {}",
                command.control.id(),
                command.actor_id,
                command.expected_revision
            ),
        },
        outcome: match &entry.outcome {
            RpgCommandOutcome::Accepted(_) => "accepted",
            RpgCommandOutcome::ControlAccepted(_) => "controlAccepted",
            RpgCommandOutcome::AwaitingReaction(_) => "awaitingReaction",
            RpgCommandOutcome::Rejected(_) => "rejected",
        }
        .to_owned(),
        before: gameplay_replay_boundary(&entry.before),
        after: gameplay_replay_boundary(&entry.after),
        random_evidence,
        events,
    }
}

fn gameplay_replay_boundary(boundary: &RpgReplayBoundary) -> GameplayReplayBoundaryDto {
    GameplayReplayBoundaryDto {
        revision: boundary.revision.to_string(),
        accepted_random_position: boundary.accepted_random_position.to_string(),
        phase: replay_phase_label(&boundary.phase),
        state_hash: format!(
            "{}:{}",
            boundary.state_hash.algorithm, boundary.state_hash.value
        ),
    }
}

fn checkpoint_phase_label(phase: &RpgCheckpointPhase) -> String {
    match phase {
        RpgCheckpointPhase::Ready => "ready".to_owned(),
        RpgCheckpointPhase::AwaitingReaction { pending, .. } => {
            format!("awaitingReaction {}", pending.request.reaction_id)
        }
    }
}

fn replay_phase_label(phase: &RpgReplayPhase) -> String {
    match phase {
        RpgReplayPhase::Ready => "ready".to_owned(),
        RpgReplayPhase::AwaitingReaction { reaction_id } => {
            format!("awaitingReaction {reaction_id}")
        }
    }
}

fn gameplay_random_evidence_label(evidence: &RpgRandomEvidence) -> String {
    format!(
        "{} {}d{} at {} = {}",
        gameplay_random_request(&evidence.request).kind,
        evidence.request.count,
        evidence.request.sides,
        evidence.request.path,
        evidence
            .values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn gameplay_authority_action(action: &rpg_runtime::RpgActionView) -> GameplayAuthorityActionDto {
    GameplayAuthorityActionDto {
        definition_id: action.definition_id.clone(),
        label: action.label.clone(),
        available: action.available,
        unavailable: action
            .unavailable
            .as_ref()
            .map(|rejection| GameplayUnavailableDto {
                code: rejection.code.clone(),
                path: rejection.path.clone(),
                message: rejection.message.clone(),
            }),
        maximum_targets: action.maximum_targets,
        options: GameplayActionOptionsDto {
            participant_ids: action.options.participant_ids.clone(),
            cell_ids: action.options.cell_ids.clone(),
            area_ids: action.options.area_ids.clone(),
        },
    }
}

fn gameplay_turn_control(control: &rpg_runtime::RpgTurnControlView) -> GameplayTurnControlDto {
    GameplayTurnControlDto {
        kind: match control.control {
            RpgTurnControl::EndTurn => "endTurn",
        }
        .to_owned(),
        label: control.label.clone(),
        available: control.available,
        unavailable: control
            .unavailable
            .as_ref()
            .map(|rejection| GameplayUnavailableDto {
                code: rejection.code.clone(),
                path: rejection.path.clone(),
                message: rejection.message.clone(),
            }),
    }
}

fn gameplay_log_entry(entry: &rpg_runtime::RpgEncounterLogEntry) -> GameplayLogEntryDto {
    GameplayLogEntryDto {
        sequence: entry.sequence.to_string(),
        state_revision: entry.state_revision.to_string(),
        actor_id: entry.actor_id.clone(),
        action_id: entry.action_id.clone(),
        events: entry.events.iter().map(gameplay_event).collect(),
    }
}

fn gameplay_outcome(outcome: &RpgEncounterOutcomeView) -> GameplayOutcomeDto {
    match outcome {
        RpgEncounterOutcomeView::InProgress => GameplayOutcomeDto {
            status: "inProgress".to_owned(),
            winning_team_ids: Vec::new(),
        },
        RpgEncounterOutcomeView::Completed { winning_team_ids } => GameplayOutcomeDto {
            status: "completed".to_owned(),
            winning_team_ids: winning_team_ids.iter().map(ToString::to_string).collect(),
        },
    }
}

fn gameplay_entity(entity: &rpg_runtime::RpgParticipantView) -> GameplayEntityDto {
    GameplayEntityDto {
        id: entity.id.clone(),
        label: entity.label.clone(),
        team_id: entity.team_id.to_string(),
        x: entity.position.x,
        y: entity.position.y,
        definition_ids: entity.definition_ids.clone(),
        vitality: GameplayNamedValueDto {
            id: "vitality".to_owned(),
            current: entity.vitality.current,
            maximum: Some(entity.vitality.max),
        },
        stats: entity
            .stats
            .iter()
            .map(|value| named_value(&value.id, value.value, None))
            .collect(),
        defenses: entity
            .defenses
            .iter()
            .map(|value| named_value(&value.id, value.value, None))
            .collect(),
        resources: entity
            .resources
            .iter()
            .map(|value| named_value(&value.id, value.value.current, Some(value.value.max)))
            .collect(),
        modifiers: entity
            .modifiers
            .iter()
            .map(|modifier| GameplayModifierDto {
                stacking_group: modifier.stacking_group.clone(),
                id: modifier.id.clone(),
                value: modifier.value,
                remaining_turns: modifier.remaining_turns,
            })
            .collect(),
    }
}

fn named_value(id: &str, current: i32, maximum: Option<i32>) -> GameplayNamedValueDto {
    GameplayNamedValueDto {
        id: id.to_owned(),
        current,
        maximum,
    }
}

fn gameplay_reaction(request: &RpgReactionRequest) -> GameplayReactionDto {
    GameplayReactionDto {
        reaction_id: request.reaction_id.clone(),
        actor_id: request.actor_id.clone(),
        target_id: request.target_id.clone(),
        action_id: request.action_id.clone(),
        options: request
            .options
            .iter()
            .map(|option| GameplayReactionOptionDto {
                id: option.id.clone(),
                label: option.label.clone(),
                damage_reduction: option.damage_reduction,
            })
            .collect(),
        path: request.path.clone(),
    }
}

fn gameplay_result(outcome: &RpgCommandOutcome, current_revision: u64) -> GameplayResultDto {
    match outcome {
        RpgCommandOutcome::Accepted(receipt) => accepted_result(receipt),
        RpgCommandOutcome::ControlAccepted(receipt) => control_accepted_result(receipt),
        RpgCommandOutcome::AwaitingReaction(pending) => GameplayResultDto {
            status: "awaitingReaction".to_owned(),
            code: None,
            message: format!("Awaiting reaction {}", pending.request.reaction_id),
            events: Vec::new(),
            trace: pending.trace.iter().map(gameplay_trace).collect(),
            random_consumed: pending.random_attempted.to_string(),
            random_evidence: pending
                .random_evidence
                .iter()
                .map(gameplay_random_evidence)
                .collect(),
            state_revision: dto_revision(current_revision),
            random_request: None,
        },
        RpgCommandOutcome::Rejected(rejection) => rejected_result(rejection, current_revision),
    }
}

fn control_accepted_result(receipt: &RpgTurnControlReceipt) -> GameplayResultDto {
    GameplayResultDto {
        status: "accepted".to_owned(),
        code: None,
        message: format!(
            "Accepted {} at state revision {}",
            receipt.control.id(),
            receipt.state_revision
        ),
        events: receipt.events.iter().map(gameplay_event).collect(),
        trace: Vec::new(),
        random_consumed: "0".to_owned(),
        random_evidence: Vec::new(),
        state_revision: dto_revision(receipt.state_revision),
        random_request: None,
    }
}

fn accepted_result(receipt: &RpgResolutionReceipt) -> GameplayResultDto {
    GameplayResultDto {
        status: "accepted".to_owned(),
        code: None,
        message: format!(
            "Accepted {} at state revision {}",
            receipt.action_id, receipt.state_revision
        ),
        events: receipt.events.iter().map(gameplay_event).collect(),
        trace: receipt.trace.iter().map(gameplay_trace).collect(),
        random_consumed: receipt.random_consumed.to_string(),
        random_evidence: receipt
            .random_evidence
            .iter()
            .map(gameplay_random_evidence)
            .collect(),
        state_revision: dto_revision(receipt.state_revision),
        random_request: None,
    }
}

fn rejected_result(rejection: &RpgResolutionRejection, current_revision: u64) -> GameplayResultDto {
    GameplayResultDto {
        status: "rejected".to_owned(),
        code: Some(rejection.code.clone()),
        message: rejection.message.clone(),
        events: Vec::new(),
        trace: rejection.trace.iter().map(gameplay_trace).collect(),
        random_consumed: rejection.random_attempted.to_string(),
        random_evidence: rejection
            .random_evidence
            .iter()
            .map(gameplay_random_evidence)
            .collect(),
        state_revision: dto_revision(current_revision),
        random_request: rejection
            .random_request
            .as_deref()
            .map(gameplay_random_request),
    }
}

fn dto_revision(revision: u64) -> u32 {
    u32::try_from(revision).unwrap_or(u32::MAX)
}

fn gameplay_random_request(request: &RpgRandomRequest) -> GameplayRandomRequestDto {
    GameplayRandomRequestDto {
        kind: match request.kind {
            RpgRandomRequestKind::AttackCheck => "attackCheck",
            RpgRandomRequestKind::SavingThrowCheck => "savingThrowCheck",
            RpgRandomRequestKind::FormulaDice => "formulaDice",
        }
        .to_owned(),
        count: request.count,
        sides: request.sides,
        path: request.path.clone(),
    }
}

fn gameplay_random_evidence(evidence: &RpgRandomEvidence) -> GameplayRandomEvidenceDto {
    GameplayRandomEvidenceDto {
        kind: match evidence.request.kind {
            RpgRandomRequestKind::AttackCheck => "attackCheck",
            RpgRandomRequestKind::SavingThrowCheck => "savingThrowCheck",
            RpgRandomRequestKind::FormulaDice => "formulaDice",
        }
        .to_owned(),
        count: evidence.request.count,
        sides: evidence.request.sides,
        path: evidence.request.path.clone(),
        values: evidence.values.clone(),
    }
}

fn gameplay_trace(trace: &RpgTraceStep) -> GameplayTraceDto {
    GameplayTraceDto {
        path: trace.path.clone(),
        code: trace.code.clone(),
        detail: trace.detail.clone(),
    }
}

fn gameplay_event(event: &RpgDomainEvent) -> GameplayEventDto {
    let (kind, summary) = match event {
        RpgDomainEvent::ResourceSpent {
            entity_id,
            resource_id,
            amount,
            remaining,
        } => (
            "resourceSpent",
            format!("{entity_id} spent {amount} {resource_id}; {remaining} remains"),
        ),
        RpgDomainEvent::AttackResolved {
            actor_id,
            target_id,
            roll,
            total,
            defense_id,
            defense,
            hit,
        } => (
            "attackResolved",
            format!(
                "{actor_id} rolled {roll} for {total} against {target_id} {defense_id} {defense}; hit={hit}"
            ),
        ),
        RpgDomainEvent::SavingThrowResolved {
            target_id,
            roll,
            total,
            difficulty,
            saved,
        } => (
            "savingThrowResolved",
            format!(
                "{target_id} rolled {roll} for {total} against {difficulty}; saved={saved}"
            ),
        ),
        RpgDomainEvent::DamageApplied {
            source_id,
            target_id,
            amount,
            damage_type,
            remaining_vitality,
        } => (
            "damageApplied",
            format!(
                "{source_id} dealt {amount} {damage_type} to {target_id}; vitality {remaining_vitality}"
            ),
        ),
        RpgDomainEvent::HealingApplied {
            source_id,
            target_id,
            amount,
            current_vitality,
        } => (
            "healingApplied",
            format!(
                "{source_id} healed {target_id} for {amount}; vitality {current_vitality}"
            ),
        ),
        RpgDomainEvent::ResourceChanged {
            entity_id,
            resource_id,
            delta,
            current,
        } => (
            "resourceChanged",
            format!("{entity_id} changed {resource_id} by {delta}; current {current}"),
        ),
        RpgDomainEvent::ModifierApplied {
            target_id,
            modifier_id,
            value,
            remaining_turns,
            ..
        } => (
            "modifierApplied",
            format!(
                "{target_id} gained {modifier_id} {value} for {remaining_turns} turn(s)"
            ),
        ),
        RpgDomainEvent::ModifierDurationChanged {
            target_id,
            modifier_id,
            remaining_turns,
            ..
        } => (
            "modifierDurationChanged",
            format!(
                "{target_id} {modifier_id} duration changed to {remaining_turns} turn(s)"
            ),
        ),
        RpgDomainEvent::ModifierExpired {
            target_id,
            modifier_id,
            ..
        } => (
            "modifierExpired",
            format!("{target_id} {modifier_id} expired"),
        ),
        RpgDomainEvent::PositionChanged {
            entity_id,
            previous,
            current,
            provokes,
            ..
        } => (
            "positionChanged",
            format!(
                "{entity_id} moved ({},{}) to ({},{}); provokes={provokes}",
                previous.x, previous.y, current.x, current.y
            ),
        ),
        RpgDomainEvent::ReactionOpened {
            reaction_id,
            target_id,
            ..
        } => (
            "reactionOpened",
            format!("opened {reaction_id} for {target_id}"),
        ),
        RpgDomainEvent::ReactionResolved {
            reaction_id,
            option_id,
            damage_reduction,
        } => (
            "reactionResolved",
            format!(
                "resolved {reaction_id} with {}; damage reduction {damage_reduction}",
                option_id.as_deref().unwrap_or("decline")
            ),
        ),
    };
    GameplayEventDto {
        kind: kind.to_owned(),
        summary,
    }
}

fn host_diagnostic(code: &str, path: &str, message: &str) -> PlayDiagnosticDto {
    host_diagnostic_at_stage("gameplay", code, path, message)
}

fn host_diagnostic_at_stage(
    stage: &str,
    code: &str,
    path: &str,
    message: &str,
) -> PlayDiagnosticDto {
    PlayDiagnosticDto {
        stage: stage.to_owned(),
        severity: "error".to_owned(),
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.to_owned(),
        package_id: None,
        definition_id: None,
        source: None,
        graph_path: None,
        expected: None,
        actual: None,
    }
}

fn random_source_diagnostic(failure: RpgRandomSourceFailure) -> PlayDiagnosticDto {
    host_diagnostic(&failure.code, &failure.path, &failure.message)
}

fn diagnostics_from_automatic_failure(
    failure: RpgAutomaticCommandFailure,
) -> Vec<PlayDiagnosticDto> {
    match failure {
        RpgAutomaticCommandFailure::RandomSource(failure) => {
            vec![random_source_diagnostic(failure)]
        }
        RpgAutomaticCommandFailure::Replay(failure) => diagnostics_from_replay_failure(failure),
    }
}

fn diagnostics_from_setup_failure(
    failure: rpg_runtime::RpgScenarioFailure,
) -> Vec<PlayDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            host_diagnostic_at_stage(
                "setup",
                &diagnostic.code,
                &diagnostic.path,
                &diagnostic.message,
            )
        })
        .collect()
}

fn diagnostics_from_replay_failure(failure: RpgReplayFailure) -> Vec<PlayDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(|diagnostic| PlayDiagnosticDto {
            stage: "replay".to_owned(),
            severity: "error".to_owned(),
            code: diagnostic.code,
            path: diagnostic.path,
            message: diagnostic.message,
            package_id: None,
            definition_id: None,
            source: None,
            graph_path: None,
            expected: diagnostic.expected,
            actual: diagnostic.actual,
        })
        .collect()
}

fn diagnostics_from_failure(failure: RpgCompileFailure) -> Vec<PlayDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(diagnostic_dto)
        .collect()
}

fn diagnostic_dto(diagnostic: RpgDiagnostic) -> PlayDiagnosticDto {
    PlayDiagnosticDto {
        stage: diagnostic_stage(diagnostic.stage).to_owned(),
        severity: diagnostic_severity(diagnostic.severity).to_owned(),
        code: diagnostic.code,
        path: diagnostic.path,
        message: diagnostic.message,
        package_id: None,
        definition_id: None,
        source: None,
        graph_path: None,
        expected: None,
        actual: None,
    }
}

fn diagnostic_stage(stage: RpgDiagnosticStage) -> &'static str {
    match stage {
        RpgDiagnosticStage::Decode => "decode",
        RpgDiagnosticStage::Compatibility => "compatibility",
        RpgDiagnosticStage::Requirements => "requirements",
        RpgDiagnosticStage::References => "references",
        RpgDiagnosticStage::Semantics => "semantics",
        RpgDiagnosticStage::Artifact => "artifact",
    }
}

fn diagnostic_severity(severity: RpgDiagnosticSeverity) -> &'static str {
    match severity {
        RpgDiagnosticSeverity::Error => "error",
    }
}

fn artifact_summary(artifact: &CompiledPlayBundleArtifact) -> PlayBundleArtifactSummaryDto {
    PlayBundleArtifactSummaryDto {
        schema: VersionedIdentityDto {
            id: artifact.artifact_schema.identity.clone(),
            version: artifact.artifact_schema.major.to_string(),
        },
        artifact_id: artifact.artifact_id.clone(),
        play_bundle: VersionedIdentityDto {
            id: artifact.play_bundle_identity.id.clone(),
            version: artifact.play_bundle_identity.version.clone(),
        },
        ruleset: VersionedIdentityDto {
            id: artifact.ruleset.identity.id.clone(),
            version: artifact.ruleset.identity.version.clone(),
        },
        language: VersionedIdentityDto {
            id: artifact.ruleset.language.id.clone(),
            version: artifact.ruleset.language.version.clone(),
        },
        content_packs: artifact
            .content_packs
            .iter()
            .map(|source| ContentPackSummaryDto {
                id: source.id.clone(),
                version: source.version.clone(),
                source_fingerprint: source.source_fingerprint.clone(),
            })
            .collect(),
        dependency_lock: artifact
            .dependency_lock
            .iter()
            .map(|entry| ContentPackLockEntryDto {
                requester: entry.requester.clone(),
                package_id: entry.package_id.clone(),
                requested_version: entry.requested_version.clone(),
                resolved_version: entry.resolved_version.clone(),
                source_fingerprint: entry.source_fingerprint.clone(),
                import_as: entry.import_as.clone(),
                relationship: dependency_relationship(entry.relationship).to_owned(),
            })
            .collect(),
        required_operations: artifact
            .content_requirements
            .operations
            .iter()
            .map(|entry| VersionedRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        required_capabilities: artifact
            .content_requirements
            .capabilities
            .iter()
            .map(|entry| VersionedRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        required_values: artifact
            .content_requirements
            .values
            .iter()
            .map(|value| format!("{:?}:{}", value.kind, value.id))
            .collect(),
        required_numeric_domains: artifact.content_requirements.numeric_domains.clone(),
        ruleset_values: artifact
            .ruleset
            .provides
            .values
            .iter()
            .map(|value| RulesetValueDto {
                kind: ruleset_value_kind(value.kind).to_owned(),
                id: value.id.clone(),
                label: value.label.clone(),
                numeric_domain_id: value.numeric_domain_id.clone(),
            })
            .collect(),
        participant_profiles: artifact
            .materialized_definitions
            .iter()
            .filter_map(participant_profile)
            .collect(),
        exported_roots: artifact.exported_roots.clone(),
        definitions: artifact
            .materialized_definitions
            .iter()
            .map(|definition| ContentDefinitionDto {
                id: definition.id.clone(),
                fingerprint: definition.fingerprint.clone(),
                label: definition
                    .presentation
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                description: definition
                    .presentation
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                tags: definition
                    .presentation
                    .get("tags")
                    .and_then(serde_json::Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
                catalog: definition
                    .semantic
                    .get("catalog")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                catalog_id: definition
                    .semantic
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                kind: definition_kind(definition.kind).to_owned(),
                visibility: definition_visibility(definition.visibility).to_owned(),
                extension_policy: extension_policy(definition.extension_policy).to_owned(),
                references: definition.references.clone(),
                package_id: definition.provenance.package_id.clone(),
                package_version: definition.provenance.package_version.clone(),
                source_module: definition.provenance.source.module.clone(),
                source_declaration: definition.provenance.source.declaration.clone(),
            })
            .collect(),
        policy_binding_ids: artifact
            .compiled_policy_bindings
            .iter()
            .map(|binding| binding.id.clone())
            .collect(),
        relationships: artifact
            .relationships
            .iter()
            .map(|relationship| ContentRelationshipDto {
                kind: relationship_kind(relationship.kind).to_owned(),
                source: relationship.source.clone(),
                target: relationship.target.clone(),
                order: relationship.order,
            })
            .collect(),
        derivation_slots: artifact.derivation_provenance.len(),
        overlay_slots: artifact.overlay_provenance.len(),
        derivations: artifact
            .derivation_provenance
            .iter()
            .map(|provenance| ContentDerivationProvenanceDto {
                definition_id: provenance.definition_id.clone(),
                owner: format!("{}@{}", provenance.package_id, provenance.package_version),
                base: format!(
                    "{}@{}#{}",
                    provenance.base_package_id,
                    provenance.base_package_version,
                    provenance.base_definition_id
                ),
                base_fingerprint: provenance.base_fingerprint.clone(),
                mixins: provenance
                    .mixins
                    .iter()
                    .map(|mixin| ContentMixinProvenanceDto {
                        identity: format!(
                            "{}@{}#{}",
                            mixin.package_id, mixin.package_version, mixin.definition_id
                        ),
                        fingerprint: mixin.fingerprint.clone(),
                        parameters: mixin
                            .parameters
                            .iter()
                            .map(|(id, value)| format!("{id}={}", json_value(value)))
                            .collect(),
                        order: mixin.order,
                    })
                    .collect(),
                local_patch_fingerprint: provenance.local_patch_fingerprint.clone(),
                materialized_fingerprint: provenance.materialized_fingerprint.clone(),
                changes: provenance.changes.iter().map(patch_change).collect(),
            })
            .collect(),
        overlays: artifact
            .overlay_provenance
            .iter()
            .map(|provenance| ContentOverlayProvenanceDto {
                overlay: format!(
                    "{}@{}",
                    provenance.overlay_package_id, provenance.overlay_package_version
                ),
                target: format!(
                    "{}@{}#{}",
                    provenance.target_package_id,
                    provenance.target_package_version,
                    provenance.target_definition_id
                ),
                expected_fingerprint: provenance.expected_fingerprint.clone(),
                before_fingerprint: provenance.before_fingerprint.clone(),
                after_fingerprint: provenance.after_fingerprint.clone(),
                plane: impact_plane(provenance.plane).to_owned(),
                conflict_policy: conflict_policy(provenance.conflict_policy).to_owned(),
                patch_fingerprint: provenance.patch_fingerprint.clone(),
                order: provenance.order,
                changes: provenance.changes.iter().map(patch_change).collect(),
            })
            .collect(),
        fingerprints: PlayBundleFingerprintDto {
            source: artifact.fingerprints.source.clone(),
            semantic: artifact.fingerprints.semantic.clone(),
            presentation: artifact.fingerprints.presentation.clone(),
        },
    }
}

fn patch_change(change: &rpg_ir::ContentPatchChangeProvenance) -> ContentPatchChangeDto {
    ContentPatchChangeDto {
        plane: impact_plane(change.plane).to_owned(),
        path: change.path.clone(),
        before: json_value(&change.before),
        after: json_value(&change.after),
        effective: change.effective,
    }
}

fn json_value(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unencodable>".to_owned())
}

fn impact_plane(plane: ContentImpactPlane) -> &'static str {
    match plane {
        ContentImpactPlane::Semantic => "semantic",
        ContentImpactPlane::Presentation => "presentation",
        ContentImpactPlane::Both => "both",
    }
}

fn conflict_policy(policy: ContentConflictPolicy) -> &'static str {
    match policy {
        ContentConflictPolicy::Reject => "reject",
        ContentConflictPolicy::Replace => "replace",
    }
}

fn dependency_relationship(relationship: ContentPackDependencyRelationship) -> &'static str {
    match relationship {
        ContentPackDependencyRelationship::DependsOn => "dependsOn",
        ContentPackDependencyRelationship::Contributes => "contributes",
        ContentPackDependencyRelationship::Patches => "patches",
    }
}

fn participant_profile(
    definition: &rpg_ir::MaterializedContentDefinition,
) -> Option<ParticipantProfileDto> {
    if definition.semantic.get("catalog")?.as_str()? != "participantProfile" {
        return None;
    }
    let data = definition.semantic.get("data")?;
    let definition_ids = data
        .get("definitionIds")?
        .as_array()?
        .iter()
        .map(|value| value.as_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?;
    let capabilities = serde_json::from_value::<Vec<ScenarioInitialCapabilityDto>>(
        data.get("capabilities")?.clone(),
    )
    .ok()?;
    Some(ParticipantProfileDto {
        definition_id: definition.id.clone(),
        profile_id: definition.semantic.get("id")?.as_str()?.to_owned(),
        label: definition
            .presentation
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(&definition.id)
            .to_owned(),
        description: definition
            .presentation
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned),
        role: data.get("role")?.as_str()?.to_owned(),
        definition_ids,
        capabilities,
    })
}

fn ruleset_value_kind(kind: RulesetValueKind) -> &'static str {
    match kind {
        RulesetValueKind::Defense => "defense",
        RulesetValueKind::Stat => "stat",
    }
}

fn definition_kind(kind: MaterializedContentDefinitionKind) -> &'static str {
    match kind {
        MaterializedContentDefinitionKind::Action => "action",
        MaterializedContentDefinitionKind::Support => "support",
    }
}

fn definition_visibility(visibility: MaterializedContentVisibility) -> &'static str {
    match visibility {
        MaterializedContentVisibility::Exported => "exported",
        MaterializedContentVisibility::Support => "support",
    }
}

fn extension_policy(policy: ContentExtensionPolicy) -> &'static str {
    match policy {
        ContentExtensionPolicy::Sealed => "sealed",
        ContentExtensionPolicy::Derivable => "derivable",
        ContentExtensionPolicy::Patchable => "patchable",
        ContentExtensionPolicy::Configurable => "configurable",
    }
}

fn relationship_kind(kind: ContentRelationshipKind) -> &'static str {
    match kind {
        ContentRelationshipKind::DependsOn => "dependsOn",
        ContentRelationshipKind::Contributes => "contributes",
        ContentRelationshipKind::DerivesFrom => "derivesFrom",
        ContentRelationshipKind::Patches => "patches",
        ContentRelationshipKind::Configures => "configures",
        ContentRelationshipKind::Exports => "exports",
    }
}

pub fn generated_protocol() -> String {
    let declarations = [
        PlayBundleLifecycleStatus::decl(),
        PlayDiagnosticDto::decl(),
        PlayDiagnosticSourceDto::decl(),
        VersionedIdentityDto::decl(),
        ContentPackSummaryDto::decl(),
        ContentPackLockEntryDto::decl(),
        VersionedRequirementDto::decl(),
        ContentDefinitionDto::decl(),
        RulesetValueDto::decl(),
        ParticipantProfileDto::decl(),
        ContentRelationshipDto::decl(),
        PlayBundleFingerprintDto::decl(),
        ContentPatchChangeDto::decl(),
        ContentMixinProvenanceDto::decl(),
        ContentDerivationProvenanceDto::decl(),
        ContentOverlayProvenanceDto::decl(),
        PlayBundleArtifactSummaryDto::decl(),
        PlayBundleUpgradeFieldDto::decl(),
        PlayBundleUpgradeDefinitionDto::decl(),
        PlayBundleUpgradeImpactDto::decl(),
        ScenarioSchemaDto::decl(),
        ScenarioPositionDto::decl(),
        ScenarioBoundedValueDto::decl(),
        ScenarioCellCapabilityValueDto::decl(),
        ScenarioCellCapabilityDto::decl(),
        ScenarioCellDto::decl(),
        ScenarioBoardDto::decl(),
        ScenarioInitialCapabilityDto::decl(),
        ScenarioParticipantDto::decl(),
        ScenarioTurnDto::decl(),
        ScenarioRandomSourceDto::decl(),
        ScenarioSetupRequestDto::decl(),
        ScenarioTemplatePresentationDto::decl(),
        ScenarioTemplateDto::decl(),
        GameplayUnavailableDto::decl(),
        GameplayActionOptionsDto::decl(),
        GameplayAuthorityActionDto::decl(),
        GameplayTurnControlDto::decl(),
        GameplayLogEntryDto::decl(),
        GameplayOutcomeDto::decl(),
        GameplayRandomRequestDto::decl(),
        GameplayNamedValueDto::decl(),
        GameplayModifierDto::decl(),
        GameplayEntityDto::decl(),
        GameplayEventDto::decl(),
        GameplayTraceDto::decl(),
        GameplayReactionOptionDto::decl(),
        GameplayReactionDto::decl(),
        GameplayRandomEvidenceDto::decl(),
        GameplayResultDto::decl(),
        GameplayReplayBoundaryDto::decl(),
        GameplayReplayEntryDto::decl(),
        GameplayArchiveDto::decl(),
        GameplaySessionDto::decl(),
        PlayWorkspaceResponseDto::decl(),
        ConfiguredRulesetLocationDto::decl(),
        RulesetLocationConfigDto::decl(),
        RulesetCatalogRequestDto::decl(),
        RulesetCatalogContentPackDto::decl(),
        RulesetCatalogPlayBundleDto::decl(),
        RulesetCatalogDto::decl(),
        RulesetCatalogResponseDto::decl(),
        PlayBundleCompileRequestDto::decl(),
        PreparedPlayBundleCompileRequestDto::decl(),
        GameplayCommandRequestDto::decl(),
        GameplayReactionRequestDto::decl(),
        GameplayTurnControlRequestDto::decl(),
    ];
    let exports = declarations
        .into_iter()
        .map(|declaration| format!("export {declaration}"))
        .collect::<Vec<_>>();
    format!(
        "// @generated by rulebench-play-host. Do not edit.\n\n{}\n",
        exports.join("\n\n")
    )
}

#[cfg(test)]
mod tests {
    use rpg_core::{RpgRandomRequest, RpgRandomRequestKind};
    use rpg_runtime::RpgRandomSource;

    use super::{
        PlayBundleLifecycleStatus, PlayHost, ScriptedGameplayRandomSource,
        SystemGameplayRandomSource,
    };

    fn random_request(count: u32, sides: u32) -> RpgRandomRequest {
        RpgRandomRequest {
            kind: RpgRandomRequestKind::FormulaDice,
            count,
            sides,
            path: "$.test".to_owned(),
        }
    }

    #[test]
    fn failed_compilation_cannot_create_a_candidate_or_active_artifact() {
        let host = PlayHost::new();

        let compilation = host.compile_candidate(r#"{"unexpected":true}"#);
        assert!(!compilation.ok);
        assert_eq!(
            compilation.status,
            PlayBundleLifecycleStatus::NoActivePlayBundle
        );
        assert!(compilation.candidate_artifact.is_none());
        assert!(compilation.active_artifact.is_none());
        assert_eq!(compilation.activation_revision, 0);
        assert_eq!(
            compilation.diagnostics[0].code,
            "PLAY_BUNDLE_PREPARED_DECODE_FAILED"
        );

        let activation = host.activate_candidate();
        assert!(!activation.ok);
        assert_eq!(
            activation.status,
            PlayBundleLifecycleStatus::NoActivePlayBundle
        );
        assert_eq!(activation.activation_revision, 0);
        assert_eq!(
            activation.diagnostics[0].code,
            "PLAY_BUNDLE_ACTIVATION_CANDIDATE_REQUIRED"
        );
    }

    #[test]
    fn scripted_random_source_consumes_only_an_accepted_exact_request() {
        let mut source = ScriptedGameplayRandomSource::new([2, 6, 4]);

        assert_eq!(source.draw(&random_request(2, 6)).unwrap(), vec![2, 6]);
        assert_eq!(source.remaining(), 1);

        let failure = source.draw(&random_request(2, 6)).unwrap_err();
        assert_eq!(failure.code, "SESSION_RANDOM_TAPE_EXHAUSTED");
        assert_eq!(source.remaining(), 1);
    }

    #[test]
    fn scripted_random_source_rejects_an_out_of_range_value_without_consuming_it() {
        let mut source = ScriptedGameplayRandomSource::new([7, 3]);

        let failure = source.draw(&random_request(1, 6)).unwrap_err();
        assert_eq!(failure.code, "SESSION_RANDOM_TAPE_VALUE_INVALID");
        assert_eq!(source.remaining(), 2);
    }

    #[test]
    fn system_random_source_stays_inside_authority_die_bounds() {
        let mut source = SystemGameplayRandomSource::default();

        let values = source.draw(&random_request(128, 20)).unwrap();
        assert_eq!(values.len(), 128);
        assert!(values.iter().all(|value| (1..=20).contains(value)));
    }
}
