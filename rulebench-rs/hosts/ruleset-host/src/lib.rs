#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

use rpg_compiler::{
    compile_prepared_ruleset_json, load_compiled_ruleset_artifact_json, CompiledRpgAction,
    CompiledRulesetBundle, RpgCompileFailure, RpgDiagnostic, RpgDiagnosticSeverity,
    RpgDiagnosticStage, RpgRandomPlanCondition, RpgRandomPlanConditionKind, RpgRandomPlanEntry,
};
use rpg_core::{
    ActiveRpgModifier, GridPosition, RpgCapabilityState, RpgDomainEvent, RpgEntityState, RpgIntent,
    RpgRandomEvidence, RpgRandomRequest, RpgRandomRequestKind, RpgReactionRequest,
    RpgResolutionReceipt, RpgResolutionRejection, RpgTraceStep, Team,
};
use rpg_ir::{
    CompiledRulesetArtifact, MaterializedRulesetDefinitionKind, MaterializedRulesetVisibility,
    RulesetConflictPolicy, RulesetDependencyRelationship, RulesetExtensionPolicy,
    RulesetImpactPlane, RulesetRelationshipKind,
};
use rpg_runtime::{
    RpgAuthorityCommand, RpgAuthoritySession, RpgCheckpointPhase, RpgCommandOutcome,
    RpgReactionCommand, RpgReplayBoundary, RpgReplayEntry, RpgReplayFailure, RpgReplayOperation,
    RpgReplayPhase, RpgSessionCheckpoint,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum RulesetLifecycleStatus {
    NoActiveRuleset,
    CompiledCandidate,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDiagnosticDto {
    pub stage: String,
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
    pub package_id: Option<String>,
    pub definition_id: Option<String>,
    pub source: Option<RulesetDiagnosticSourceDto>,
    pub graph_path: Option<Vec<String>>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDiagnosticSourceDto {
    pub module: String,
    pub declaration: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetIdentityDto {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetSourcePackageDto {
    pub id: String,
    pub version: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetLockEntryDto {
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
pub struct RulesetRequirementDto {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDefinitionDto {
    pub id: String,
    pub fingerprint: String,
    pub label: Option<String>,
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
pub struct RulesetRelationshipDto {
    pub kind: String,
    pub source: String,
    pub target: String,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetFingerprintDto {
    pub source: String,
    pub semantic: String,
    pub presentation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetPatchChangeDto {
    pub plane: String,
    pub path: String,
    pub before: String,
    pub after: String,
    pub effective: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetMixinProvenanceDto {
    pub identity: String,
    pub fingerprint: String,
    pub parameters: Vec<String>,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDerivationProvenanceDto {
    pub definition_id: String,
    pub owner: String,
    pub base: String,
    pub base_fingerprint: String,
    pub mixins: Vec<RulesetMixinProvenanceDto>,
    pub local_patch_fingerprint: String,
    pub materialized_fingerprint: String,
    pub changes: Vec<RulesetPatchChangeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetOverlayProvenanceDto {
    pub overlay: String,
    pub target: String,
    pub expected_fingerprint: String,
    pub before_fingerprint: String,
    pub after_fingerprint: String,
    pub plane: String,
    pub conflict_policy: String,
    pub patch_fingerprint: String,
    pub order: usize,
    pub changes: Vec<RulesetPatchChangeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetArtifactSummaryDto {
    pub schema: RulesetIdentityDto,
    pub artifact_id: String,
    pub composition: RulesetIdentityDto,
    pub language: RulesetIdentityDto,
    pub source_packages: Vec<RulesetSourcePackageDto>,
    pub dependency_lock: Vec<RulesetLockEntryDto>,
    pub required_operations: Vec<RulesetRequirementDto>,
    pub required_capabilities: Vec<RulesetRequirementDto>,
    pub exported_roots: Vec<String>,
    pub definitions: Vec<RulesetDefinitionDto>,
    pub policy_binding_ids: Vec<String>,
    pub relationships: Vec<RulesetRelationshipDto>,
    pub derivation_slots: usize,
    pub overlay_slots: usize,
    pub derivations: Vec<RulesetDerivationProvenanceDto>,
    pub overlays: Vec<RulesetOverlayProvenanceDto>,
    pub fingerprints: RulesetFingerprintDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetUpgradeFieldDto {
    pub plane: String,
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetUpgradeDefinitionDto {
    pub definition_id: String,
    pub change: String,
    pub descendant: bool,
    pub causes: Vec<String>,
    pub fields: Vec<RulesetUpgradeFieldDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetUpgradeImpactDto {
    pub from_artifact_id: String,
    pub to_artifact_id: String,
    pub source_changes: Vec<String>,
    pub definitions: Vec<RulesetUpgradeDefinitionDto>,
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
    pub team: String,
    pub x: u32,
    pub y: u32,
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
pub struct GameplayResultDto {
    pub status: String,
    pub code: Option<String>,
    pub message: String,
    pub events: Vec<GameplayEventDto>,
    pub trace: Vec<GameplayTraceDto>,
    pub random_consumed: String,
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
    pub composition: String,
    pub language: String,
    pub operation_schemas: Vec<String>,
    pub capability_schemas: Vec<String>,
    pub source_packages: Vec<String>,
    pub dependency_lock: Vec<String>,
    pub fingerprints: RulesetFingerprintDto,
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
    pub actor_id: String,
    pub state_revision: u32,
    pub accepted_random_values: String,
    pub actions: Vec<GameplayActionDto>,
    pub preflights: Vec<GameplayPreflightDto>,
    pub entities: Vec<GameplayEntityDto>,
    pub pending_reaction: Option<GameplayReactionDto>,
    pub last_result: Option<GameplayResultDto>,
    pub archive: GameplayArchiveDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetWorkspaceResponseDto {
    pub ok: bool,
    pub status: RulesetLifecycleStatus,
    pub active_artifact: Option<RulesetArtifactSummaryDto>,
    pub candidate_artifact: Option<RulesetArtifactSummaryDto>,
    pub upgrade_impact: Option<RulesetUpgradeImpactDto>,
    pub activation_revision: u32,
    pub gameplay_available: bool,
    pub gameplay: Option<GameplaySessionDto>,
    pub diagnostics: Vec<RulesetDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct RulesetCompileRequestDto {
    pub ruleset_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct PreparedRulesetCompileRequestDto {
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
    pub random_values: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct GameplayReactionRequestDto {
    pub expected_revision: u32,
    pub reaction_id: String,
    pub option_id: Option<String>,
    pub additional_random_values: Vec<u32>,
}

#[derive(Debug)]
struct ActiveRuleset {
    bundle: CompiledRulesetBundle,
    session: RpgAuthoritySession,
    last_result: Option<GameplayResultDto>,
    initial_checkpoint: RpgSessionCheckpoint,
    latest_checkpoint: RpgSessionCheckpoint,
    latest_checkpoint_bytes: Vec<u8>,
    replay_entries: Vec<RpgReplayEntry>,
    verification_status: String,
    verification_message: String,
}

#[derive(Debug, Default)]
struct ActivationSlots {
    candidate: Option<CompiledRulesetBundle>,
    active: Option<ActiveRuleset>,
    activation_revision: u32,
}

impl ActivationSlots {
    fn stage(&mut self, candidate: CompiledRulesetBundle) {
        self.candidate = Some(candidate);
    }

    fn clear_candidate(&mut self) {
        self.candidate = None;
    }

    fn activate(&mut self) -> Result<bool, RpgReplayFailure> {
        let Some(candidate) = self.candidate.take() else {
            return Ok(false);
        };
        let session =
            RpgAuthoritySession::from_compiled_ruleset(candidate.clone(), initial_gameplay_state());
        let checkpoint = match session.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(failure) => {
                self.candidate = Some(candidate);
                return Err(failure);
            }
        };
        let checkpoint_bytes = match session.checkpoint_json() {
            Ok(checkpoint_bytes) => checkpoint_bytes,
            Err(failure) => {
                self.candidate = Some(candidate);
                return Err(failure);
            }
        };
        self.active = Some(ActiveRuleset {
            bundle: candidate,
            session,
            last_result: None,
            initial_checkpoint: checkpoint.clone(),
            latest_checkpoint: checkpoint,
            latest_checkpoint_bytes: checkpoint_bytes,
            replay_entries: Vec::new(),
            verification_status: "notRun".to_owned(),
            verification_message: "No replay verification has run yet".to_owned(),
        });
        self.activation_revision += 1;
        Ok(true)
    }

    fn status(&self) -> RulesetLifecycleStatus {
        if self.candidate.is_some() {
            RulesetLifecycleStatus::CompiledCandidate
        } else if self.active.is_some() {
            RulesetLifecycleStatus::Active
        } else {
            RulesetLifecycleStatus::NoActiveRuleset
        }
    }
}

impl ActiveRuleset {
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

pub struct RulesetHost {
    slots: Mutex<ActivationSlots>,
}

impl Default for RulesetHost {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesetHost {
    pub fn new() -> Self {
        Self {
            slots: Mutex::new(ActivationSlots::default()),
        }
    }

    pub fn status(&self) -> RulesetWorkspaceResponseDto {
        let slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        response_from_slots(true, &slots, Vec::new())
    }

    pub fn compile_candidate(&self, prepared_source: &str) -> RulesetWorkspaceResponseDto {
        let compilation = compile_prepared_ruleset_json(prepared_source.as_bytes());
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

    pub fn activate_candidate(&self) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        match slots.activate() {
            Ok(true) => response_from_slots(true, &slots, Vec::new()),
            Ok(false) => response_from_slots(
                false,
                &slots,
                vec![RulesetDiagnosticDto {
                    stage: "activation".to_owned(),
                    severity: "error".to_owned(),
                    code: "RULESET_ACTIVATION_CANDIDATE_REQUIRED".to_owned(),
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
            Err(failure) => {
                response_from_slots(false, &slots, diagnostics_from_replay_failure(failure))
            }
        }
    }

    pub fn execute_command(
        &self,
        request: GameplayCommandRequestDto,
    ) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = &mut slots.active else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_SESSION_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "activate a compiled artifact before submitting gameplay",
                )],
            );
        };
        let recorded = active.session.submit_recorded(RpgAuthorityCommand {
            expected_revision: u64::from(request.expected_revision),
            intent: RpgIntent {
                action_id: request.action_id,
                actor_id: request.actor_id,
                target_ids: request.target_ids,
            },
            random_values: request.random_values,
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

    pub fn resolve_reaction(
        &self,
        request: GameplayReactionRequestDto,
    ) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = &mut slots.active else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_SESSION_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "activate a compiled artifact before resolving a reaction",
                )],
            );
        };
        let recorded = active.session.react_recorded(RpgReactionCommand {
            expected_revision: u64::from(request.expected_revision),
            reaction_id: request.reaction_id,
            option_id: request.option_id,
            additional_random_values: request.additional_random_values,
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

    pub fn restore_latest_checkpoint(&self) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = &mut slots.active else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_CHECKPOINT_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "activate a compiled artifact before restoring a checkpoint",
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

    pub fn replay_archive(&self) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        let Some(active) = &mut slots.active else {
            return response_from_slots(
                false,
                &slots,
                vec![host_diagnostic(
                    "RPG_REPLAY_ACTIVE_ARTIFACT_REQUIRED",
                    "$.activeArtifact",
                    "activate a compiled artifact before replaying stored records",
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
    bundle: CompiledRulesetBundle,
) -> Result<CompiledRulesetBundle, Vec<RulesetDiagnosticDto>> {
    let encoded = serde_json::to_vec(bundle.artifact()).map_err(|error| {
        vec![RulesetDiagnosticDto {
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
    load_compiled_ruleset_artifact_json(&encoded).map_err(diagnostics_from_failure)
}

fn response_from_slots(
    ok: bool,
    slots: &ActivationSlots,
    diagnostics: Vec<RulesetDiagnosticDto>,
) -> RulesetWorkspaceResponseDto {
    let upgrade_impact = match (&slots.active, &slots.candidate) {
        (Some(active), Some(candidate)) => Some(upgrade_impact(
            active.bundle.artifact(),
            candidate.artifact(),
        )),
        _ => None,
    };
    RulesetWorkspaceResponseDto {
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
        gameplay_available: slots.active.is_some(),
        gameplay: slots.active.as_ref().map(gameplay_session),
        diagnostics,
    }
}

fn upgrade_impact(
    active: &CompiledRulesetArtifact,
    candidate: &CompiledRulesetArtifact,
) -> RulesetUpgradeImpactDto {
    let active_sources = active
        .source_packages
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let candidate_sources = candidate
        .source_packages
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
        definitions.push(RulesetUpgradeDefinitionDto {
            definition_id: definition_id.to_owned(),
            change: change.to_owned(),
            descendant: is_derived_definition(active, definition_id)
                || is_derived_definition(candidate, definition_id),
            causes: upgrade_causes(active, candidate, definition_id, before, after),
            fields,
        });
    }

    RulesetUpgradeImpactDto {
        from_artifact_id: active.artifact_id.clone(),
        to_artifact_id: candidate.artifact_id.clone(),
        source_changes,
        definitions,
    }
}

fn is_derived_definition(artifact: &CompiledRulesetArtifact, definition_id: &str) -> bool {
    artifact
        .derivation_provenance
        .iter()
        .any(|provenance| provenance.definition_id == definition_id)
}

fn upgrade_causes(
    active: &CompiledRulesetArtifact,
    candidate: &CompiledRulesetArtifact,
    definition_id: &str,
    before: Option<&rpg_ir::MaterializedRulesetDefinition>,
    after: Option<&rpg_ir::MaterializedRulesetDefinition>,
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
        causes.push("composition-ordered overlay provenance changed".to_owned());
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
    fields: &mut Vec<RulesetUpgradeFieldDto>,
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
        _ => fields.push(RulesetUpgradeFieldDto {
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

fn initial_gameplay_state() -> RpgCapabilityState {
    let hero = RpgEntityState::new("hero", Team::Ally, GridPosition { x: 0, y: 0 }, 28)
        .with_stat("power", 3)
        .with_defense("guard", 13)
        .with_resource("focus", 2, 2);
    let raider = RpgEntityState::new("raider", Team::Enemy, GridPosition { x: 4, y: 0 }, 36)
        .with_stat("power", 2)
        .with_defense("guard", 12)
        .with_resource("focus", 2, 2);
    let mut state = RpgCapabilityState::default();
    state.insert_entity(hero);
    state.insert_entity(raider);
    state
}

fn gameplay_session(active: &ActiveRuleset) -> GameplaySessionDto {
    let actor_id = "hero";
    let state = active.session.state();
    let actions = active
        .session
        .ruleset()
        .actions()
        .map(|action| gameplay_action(&active.session, actor_id, action))
        .collect();
    let mut preflights = Vec::new();
    for action in active.session.ruleset().actions() {
        for target in state.entities().filter(|entity| entity.id() != actor_id) {
            let intent = RpgIntent {
                action_id: action.id.clone(),
                actor_id: actor_id.to_owned(),
                target_ids: vec![target.id().to_owned()],
            };
            let result = active.session.ruleset().preflight(state, &intent);
            preflights.push(match result {
                Ok(()) => GameplayPreflightDto {
                    action_id: action.id.clone(),
                    target_id: target.id().to_owned(),
                    available: true,
                    code: None,
                    message: "Rust authority accepts this intent at the active revision".to_owned(),
                },
                Err(rejection) => GameplayPreflightDto {
                    action_id: action.id.clone(),
                    target_id: target.id().to_owned(),
                    available: false,
                    code: Some(rejection.code),
                    message: rejection.message,
                },
            });
        }
    }
    GameplaySessionDto {
        actor_id: actor_id.to_owned(),
        state_revision: dto_revision(state.revision()),
        accepted_random_values: active.session.accepted_random_values().to_string(),
        actions,
        preflights,
        entities: state.entities().map(gameplay_entity).collect(),
        pending_reaction: active
            .session
            .pending_reaction()
            .map(|pending| gameplay_reaction(&pending.request)),
        last_result: active.last_result.clone(),
        archive: gameplay_archive(active),
    }
}

fn gameplay_archive(active: &ActiveRuleset) -> GameplayArchiveDto {
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
        composition: format!("{}@{}", binding.composition.id, binding.composition.version),
        language: format!("{}@{}", binding.language.id, binding.language.version),
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
        source_packages: binding
            .source_packages
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
        fingerprints: RulesetFingerprintDto {
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
                .map(gameplay_random_evidence)
                .collect(),
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
                .map(gameplay_random_evidence)
                .collect(),
            Vec::new(),
        ),
        RpgCommandOutcome::Rejected(rejection) => (
            rejection
                .random_evidence
                .iter()
                .map(gameplay_random_evidence)
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
        },
        outcome: match &entry.outcome {
            RpgCommandOutcome::Accepted(_) => "accepted",
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

fn gameplay_random_evidence(evidence: &RpgRandomEvidence) -> String {
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

fn gameplay_action(
    session: &RpgAuthoritySession,
    actor_id: &str,
    action: CompiledRpgAction,
) -> GameplayActionDto {
    let candidate_ids = session
        .ruleset()
        .candidate_ids(session.state(), actor_id, &action.id)
        .unwrap_or_default();
    GameplayActionDto {
        id: action.id,
        name: action.name,
        source_path: action.source_path,
        team: match action.targets.team {
            rpg_ir::RpgIrTeamConstraint::Hostile => "hostile",
            rpg_ir::RpgIrTeamConstraint::Ally => "ally",
            rpg_ir::RpgIrTeamConstraint::Any => "any",
        }
        .to_owned(),
        maximum_range: action.targets.maximum_range,
        maximum_targets: action.targets.maximum_targets,
        costs: action
            .costs
            .into_iter()
            .map(|cost| GameplayCostDto {
                resource_id: cost.resource_id,
                amount: cost.amount,
            })
            .collect(),
        random_plan: action
            .random_plan
            .iter()
            .map(gameplay_random_plan_entry)
            .collect(),
        candidate_ids,
    }
}

fn gameplay_entity(entity: &RpgEntityState) -> GameplayEntityDto {
    GameplayEntityDto {
        id: entity.id().to_owned(),
        team: match entity.team() {
            Team::Ally => "ally",
            Team::Enemy => "enemy",
        }
        .to_owned(),
        x: entity.position().x,
        y: entity.position().y,
        vitality: GameplayNamedValueDto {
            id: "vitality".to_owned(),
            current: entity.vitality().current,
            maximum: Some(entity.vitality().max),
        },
        stats: entity
            .stats()
            .map(|(id, value)| named_value(id, value, None))
            .collect(),
        defenses: entity
            .defenses()
            .map(|(id, value)| named_value(id, value, None))
            .collect(),
        resources: entity
            .resources()
            .map(|(id, value)| named_value(id, value.current, Some(value.max)))
            .collect(),
        modifiers: entity
            .modifiers()
            .map(|(group, modifier)| gameplay_modifier(group, modifier))
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

fn gameplay_modifier(group: &str, modifier: &ActiveRpgModifier) -> GameplayModifierDto {
    GameplayModifierDto {
        stacking_group: group.to_owned(),
        id: modifier.id().to_owned(),
        value: modifier.value(),
        remaining_turns: modifier.remaining_turns(),
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
        RpgCommandOutcome::AwaitingReaction(pending) => GameplayResultDto {
            status: "awaitingReaction".to_owned(),
            code: None,
            message: format!("Awaiting reaction {}", pending.request.reaction_id),
            events: Vec::new(),
            trace: pending.trace.iter().map(gameplay_trace).collect(),
            random_consumed: pending.random_attempted.to_string(),
            state_revision: dto_revision(current_revision),
            random_request: None,
        },
        RpgCommandOutcome::Rejected(rejection) => rejected_result(rejection, current_revision),
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

fn gameplay_random_plan_entry(entry: &RpgRandomPlanEntry) -> GameplayRandomPlanEntryDto {
    GameplayRandomPlanEntryDto {
        request: gameplay_random_request(&entry.request),
        conditions: entry
            .conditions
            .iter()
            .map(gameplay_random_plan_condition)
            .collect(),
    }
}

fn gameplay_random_plan_condition(
    condition: &RpgRandomPlanCondition,
) -> GameplayRandomPlanConditionDto {
    GameplayRandomPlanConditionDto {
        kind: match condition.kind {
            RpgRandomPlanConditionKind::WhenThen => GameplayRandomPlanConditionKindDto::WhenThen,
            RpgRandomPlanConditionKind::WhenOtherwise => {
                GameplayRandomPlanConditionKindDto::WhenOtherwise
            }
            RpgRandomPlanConditionKind::CheckHit => GameplayRandomPlanConditionKindDto::CheckHit,
            RpgRandomPlanConditionKind::CheckMiss => GameplayRandomPlanConditionKindDto::CheckMiss,
            RpgRandomPlanConditionKind::CheckSaved => {
                GameplayRandomPlanConditionKindDto::CheckSaved
            }
            RpgRandomPlanConditionKind::CheckFailed => {
                GameplayRandomPlanConditionKindDto::CheckFailed
            }
            RpgRandomPlanConditionKind::CheckNoRoll => {
                GameplayRandomPlanConditionKindDto::CheckNoRoll
            }
            RpgRandomPlanConditionKind::AllPreviousTrue => {
                GameplayRandomPlanConditionKindDto::AllPreviousTrue
            }
            RpgRandomPlanConditionKind::AnyPreviousFalse => {
                GameplayRandomPlanConditionKindDto::AnyPreviousFalse
            }
        },
        path: condition.path.clone(),
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

fn host_diagnostic(code: &str, path: &str, message: &str) -> RulesetDiagnosticDto {
    RulesetDiagnosticDto {
        stage: "gameplay".to_owned(),
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

fn diagnostics_from_replay_failure(failure: RpgReplayFailure) -> Vec<RulesetDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(|diagnostic| RulesetDiagnosticDto {
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

fn diagnostics_from_failure(failure: RpgCompileFailure) -> Vec<RulesetDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(diagnostic_dto)
        .collect()
}

fn diagnostic_dto(diagnostic: RpgDiagnostic) -> RulesetDiagnosticDto {
    RulesetDiagnosticDto {
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

fn artifact_summary(artifact: &CompiledRulesetArtifact) -> RulesetArtifactSummaryDto {
    RulesetArtifactSummaryDto {
        schema: RulesetIdentityDto {
            id: artifact.artifact_schema.identity.clone(),
            version: artifact.artifact_schema.major.to_string(),
        },
        artifact_id: artifact.artifact_id.clone(),
        composition: RulesetIdentityDto {
            id: artifact.composition_identity.id.clone(),
            version: artifact.composition_identity.version.clone(),
        },
        language: RulesetIdentityDto {
            id: artifact.language_identity.id.clone(),
            version: artifact.language_identity.version.clone(),
        },
        source_packages: artifact
            .source_packages
            .iter()
            .map(|source| RulesetSourcePackageDto {
                id: source.id.clone(),
                version: source.version.clone(),
                source_fingerprint: source.source_fingerprint.clone(),
            })
            .collect(),
        dependency_lock: artifact
            .dependency_lock
            .iter()
            .map(|entry| RulesetLockEntryDto {
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
            .required_operations
            .iter()
            .map(|entry| RulesetRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        required_capabilities: artifact
            .required_capabilities
            .iter()
            .map(|entry| RulesetRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        exported_roots: artifact.exported_roots.clone(),
        definitions: artifact
            .materialized_definitions
            .iter()
            .map(|definition| RulesetDefinitionDto {
                id: definition.id.clone(),
                fingerprint: definition.fingerprint.clone(),
                label: definition
                    .presentation
                    .get("label")
                    .and_then(serde_json::Value::as_str)
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
            .map(|relationship| RulesetRelationshipDto {
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
            .map(|provenance| RulesetDerivationProvenanceDto {
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
                    .map(|mixin| RulesetMixinProvenanceDto {
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
            .map(|provenance| RulesetOverlayProvenanceDto {
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
        fingerprints: RulesetFingerprintDto {
            source: artifact.fingerprints.source.clone(),
            semantic: artifact.fingerprints.semantic.clone(),
            presentation: artifact.fingerprints.presentation.clone(),
        },
    }
}

fn patch_change(change: &rpg_ir::RulesetPatchChangeProvenance) -> RulesetPatchChangeDto {
    RulesetPatchChangeDto {
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

fn impact_plane(plane: RulesetImpactPlane) -> &'static str {
    match plane {
        RulesetImpactPlane::Semantic => "semantic",
        RulesetImpactPlane::Presentation => "presentation",
        RulesetImpactPlane::Both => "both",
    }
}

fn conflict_policy(policy: RulesetConflictPolicy) -> &'static str {
    match policy {
        RulesetConflictPolicy::Reject => "reject",
        RulesetConflictPolicy::Replace => "replace",
    }
}

fn dependency_relationship(relationship: RulesetDependencyRelationship) -> &'static str {
    match relationship {
        RulesetDependencyRelationship::DependsOn => "dependsOn",
        RulesetDependencyRelationship::Contributes => "contributes",
        RulesetDependencyRelationship::Patches => "patches",
    }
}

fn definition_kind(kind: MaterializedRulesetDefinitionKind) -> &'static str {
    match kind {
        MaterializedRulesetDefinitionKind::Action => "action",
        MaterializedRulesetDefinitionKind::Support => "support",
    }
}

fn definition_visibility(visibility: MaterializedRulesetVisibility) -> &'static str {
    match visibility {
        MaterializedRulesetVisibility::Exported => "exported",
        MaterializedRulesetVisibility::Support => "support",
    }
}

fn extension_policy(policy: RulesetExtensionPolicy) -> &'static str {
    match policy {
        RulesetExtensionPolicy::Sealed => "sealed",
        RulesetExtensionPolicy::Derivable => "derivable",
        RulesetExtensionPolicy::Patchable => "patchable",
        RulesetExtensionPolicy::Configurable => "configurable",
    }
}

fn relationship_kind(kind: RulesetRelationshipKind) -> &'static str {
    match kind {
        RulesetRelationshipKind::DependsOn => "dependsOn",
        RulesetRelationshipKind::Contributes => "contributes",
        RulesetRelationshipKind::DerivesFrom => "derivesFrom",
        RulesetRelationshipKind::Patches => "patches",
        RulesetRelationshipKind::Configures => "configures",
        RulesetRelationshipKind::Exports => "exports",
    }
}

pub fn generated_protocol() -> String {
    let declarations = [
        RulesetLifecycleStatus::decl(),
        RulesetDiagnosticDto::decl(),
        RulesetDiagnosticSourceDto::decl(),
        RulesetIdentityDto::decl(),
        RulesetSourcePackageDto::decl(),
        RulesetLockEntryDto::decl(),
        RulesetRequirementDto::decl(),
        RulesetDefinitionDto::decl(),
        RulesetRelationshipDto::decl(),
        RulesetFingerprintDto::decl(),
        RulesetPatchChangeDto::decl(),
        RulesetMixinProvenanceDto::decl(),
        RulesetDerivationProvenanceDto::decl(),
        RulesetOverlayProvenanceDto::decl(),
        RulesetArtifactSummaryDto::decl(),
        RulesetUpgradeFieldDto::decl(),
        RulesetUpgradeDefinitionDto::decl(),
        RulesetUpgradeImpactDto::decl(),
        GameplayCostDto::decl(),
        GameplayRandomRequestDto::decl(),
        GameplayRandomPlanConditionKindDto::decl(),
        GameplayRandomPlanConditionDto::decl(),
        GameplayRandomPlanEntryDto::decl(),
        GameplayActionDto::decl(),
        GameplayPreflightDto::decl(),
        GameplayNamedValueDto::decl(),
        GameplayModifierDto::decl(),
        GameplayEntityDto::decl(),
        GameplayEventDto::decl(),
        GameplayTraceDto::decl(),
        GameplayReactionOptionDto::decl(),
        GameplayReactionDto::decl(),
        GameplayResultDto::decl(),
        GameplayReplayBoundaryDto::decl(),
        GameplayReplayEntryDto::decl(),
        GameplayArchiveDto::decl(),
        GameplaySessionDto::decl(),
        RulesetWorkspaceResponseDto::decl(),
        RulesetCompileRequestDto::decl(),
        PreparedRulesetCompileRequestDto::decl(),
        GameplayCommandRequestDto::decl(),
        GameplayReactionRequestDto::decl(),
    ];
    let exports = declarations
        .into_iter()
        .map(|declaration| format!("export {declaration}"))
        .collect::<Vec<_>>();
    format!(
        "// @generated by rulebench-ruleset-host. Do not edit.\n\n{}\n",
        exports.join("\n\n")
    )
}

#[cfg(test)]
mod tests {
    use super::{initial_gameplay_state, RulesetHost, RulesetLifecycleStatus};

    #[test]
    fn initial_gameplay_state_is_explicit_and_inactive_until_artifact_activation() {
        let state = initial_gameplay_state();
        assert_eq!(state.revision(), 0);
        assert_eq!(state.entity("hero").unwrap().position().x, 0);
        assert_eq!(state.entity("raider").unwrap().position().x, 4);
        assert_eq!(
            state
                .entity("hero")
                .unwrap()
                .resource("focus")
                .unwrap()
                .current,
            2
        );
    }

    #[test]
    fn failed_compilation_cannot_create_a_candidate_or_active_artifact() {
        let host = RulesetHost::new();

        let compilation = host.compile_candidate(r#"{"unexpected":true}"#);
        assert!(!compilation.ok);
        assert_eq!(compilation.status, RulesetLifecycleStatus::NoActiveRuleset);
        assert!(compilation.candidate_artifact.is_none());
        assert!(compilation.active_artifact.is_none());
        assert_eq!(compilation.activation_revision, 0);
        assert_eq!(
            compilation.diagnostics[0].code,
            "RULESET_PREPARED_DECODE_FAILED"
        );

        let activation = host.activate_candidate();
        assert!(!activation.ok);
        assert_eq!(activation.status, RulesetLifecycleStatus::NoActiveRuleset);
        assert_eq!(activation.activation_revision, 0);
    }
}
