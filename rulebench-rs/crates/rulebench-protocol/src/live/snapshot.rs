use rulebench_rules::{
    ActionResourceCost, ActionResourceRefreshPolicy, ActionResourceState,
    CombatEndConditionReadout, CombatFinalizationReadout, CombatLogEntry, CombatSessionSnapshot,
    CommandAuditEntry, CurrentActorActionOption, CurrentActorCellOption, CurrentActorOptionSummary,
    CurrentActorTargetOption, CurrentActorTargetSetOption, FinalCombatantState, SpatialBoardState,
    SpatialCellState, StateFingerprint,
};
use serde::{Deserialize, Serialize};

use crate::{ReactionAuditEntryDto, ReactionWindowDto, ReactionWindowLifecycleEntryDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveStateFingerprintDto {
    pub algorithm: String,
    pub value: String,
}

impl From<&StateFingerprint> for LiveStateFingerprintDto {
    fn from(value: &StateFingerprint) -> Self {
        Self {
            algorithm: value.algorithm.clone(),
            value: value.value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveParticipantDto {
    pub id: String,
    pub name: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
    pub temporary_vitality: i32,
    pub defeated: bool,
    pub conditions: Vec<String>,
    pub position: LiveGridPositionDto,
    pub movement_remaining: u32,
    pub movement_maximum: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveGridPositionDto {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveBoardCellDto {
    pub position: LiveGridPositionDto,
    pub terrain_tags: Vec<String>,
    pub blocks_movement: bool,
    pub occupant_ids: Vec<String>,
}

impl From<&SpatialCellState> for LiveBoardCellDto {
    fn from(value: &SpatialCellState) -> Self {
        Self {
            position: LiveGridPositionDto {
                x: value.position.x,
                y: value.position.y,
            },
            terrain_tags: value.terrain_tags.clone(),
            blocks_movement: value.blocks_movement,
            occupant_ids: value.occupant_ids.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveBoardDto {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<LiveBoardCellDto>,
}

impl From<&SpatialBoardState> for LiveBoardDto {
    fn from(value: &SpatialBoardState) -> Self {
        Self {
            id: value.id.clone(),
            width: value.width,
            height: value.height,
            cells: value.cells.iter().map(LiveBoardCellDto::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveActionResourceCostDto {
    pub resource_id: String,
    pub amount: u32,
}

impl From<&ActionResourceCost> for LiveActionResourceCostDto {
    fn from(value: &ActionResourceCost) -> Self {
        Self {
            resource_id: value.resource_id.clone(),
            amount: value.amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveActionResourceStateDto {
    pub resource_id: String,
    pub source_id: String,
    pub kind: String,
    pub current: i32,
    pub max: i32,
    pub available: bool,
    pub refresh_policy: String,
    pub refresh_turns: Option<u32>,
    pub remaining_refresh_turns: Option<u32>,
}

impl From<&ActionResourceState> for LiveActionResourceStateDto {
    fn from(value: &ActionResourceState) -> Self {
        Self {
            resource_id: value.resource_id.clone(),
            source_id: value.source_id.clone(),
            kind: value.kind.code().to_string(),
            current: value.current,
            max: value.max,
            available: value.available,
            refresh_policy: value.refresh_policy.code().to_string(),
            refresh_turns: match value.refresh_policy {
                ActionResourceRefreshPolicy::Turns(turns) => Some(turns),
                ActionResourceRefreshPolicy::Never
                | ActionResourceRefreshPolicy::CombatStart
                | ActionResourceRefreshPolicy::TurnStart => None,
            },
            remaining_refresh_turns: value.remaining_refresh_turns,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveTargetOptionDto {
    pub target_id: String,
    pub target_name: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
    pub reason: String,
}

impl From<&CurrentActorTargetOption> for LiveTargetOptionDto {
    fn from(value: &CurrentActorTargetOption) -> Self {
        Self {
            target_id: value.target_id.clone(),
            target_name: value.target_name.clone(),
            current_hit_points: value.current_hit_points,
            max_hit_points: value.max_hit_points,
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveTargetSetOptionDto {
    pub id: String,
    pub target_ids: Vec<String>,
    pub target_cell: Option<LiveGridPositionDto>,
    pub roll_policy: String,
    pub reason: String,
}

impl From<&CurrentActorTargetSetOption> for LiveTargetSetOptionDto {
    fn from(value: &CurrentActorTargetSetOption) -> Self {
        Self {
            id: value.id.clone(),
            target_ids: value.target_ids.clone(),
            target_cell: value.target_cell.map(|cell| LiveGridPositionDto {
                x: cell.x,
                y: cell.y,
            }),
            roll_policy: value.roll_policy.code().to_string(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCellOptionDto {
    pub position: LiveGridPositionDto,
    pub reason: String,
}

impl From<&CurrentActorCellOption> for LiveCellOptionDto {
    fn from(value: &CurrentActorCellOption) -> Self {
        Self {
            position: LiveGridPositionDto {
                x: value.position.x,
                y: value.position.y,
            },
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveActionOptionDto {
    pub action_id: String,
    pub ability_id: String,
    pub action_name: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub resource_costs: Vec<LiveActionResourceCostDto>,
    pub resource_states: Vec<LiveActionResourceStateDto>,
    pub target_mode: String,
    pub targets: Vec<LiveTargetOptionDto>,
    pub target_sets: Vec<LiveTargetSetOptionDto>,
    pub destinations: Vec<LiveCellOptionDto>,
}

impl From<&CurrentActorActionOption> for LiveActionOptionDto {
    fn from(value: &CurrentActorActionOption) -> Self {
        Self {
            action_id: value.action_id.clone(),
            ability_id: value.ability_id.clone(),
            action_name: value.action_name.clone(),
            available: value.available,
            unavailable_reason: value.unavailable_reason.clone(),
            resource_costs: value
                .resource_costs
                .iter()
                .map(LiveActionResourceCostDto::from)
                .collect(),
            resource_states: value
                .resource_states
                .iter()
                .map(LiveActionResourceStateDto::from)
                .collect(),
            target_mode: value.target_mode.code().to_string(),
            targets: value
                .target_options
                .iter()
                .map(LiveTargetOptionDto::from)
                .collect(),
            target_sets: value
                .target_set_options
                .iter()
                .map(LiveTargetSetOptionDto::from)
                .collect(),
            destinations: value
                .destination_options
                .iter()
                .map(LiveCellOptionDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCurrentActorOptionsDto {
    pub round_number: u32,
    pub turn_index: u32,
    pub lifecycle_phase: String,
    pub current_actor_id: Option<String>,
    pub current_actor_defeated: bool,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub actions: Vec<LiveActionOptionDto>,
}

impl From<&CurrentActorOptionSummary> for LiveCurrentActorOptionsDto {
    fn from(value: &CurrentActorOptionSummary) -> Self {
        Self {
            round_number: value.round_number,
            turn_index: value.turn_index,
            lifecycle_phase: value.lifecycle_phase.code().to_string(),
            current_actor_id: value.current_actor_id.clone(),
            current_actor_defeated: value.current_actor_defeated,
            available: value.available,
            unavailable_reason: value
                .unavailable_reason
                .map(|reason| reason.code().to_string()),
            actions: value
                .actions
                .iter()
                .map(LiveActionOptionDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCombatLogEntryDto {
    pub id: String,
    pub step_id: String,
    pub log_index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: String,
    pub event_types: Vec<String>,
}

impl From<&CombatLogEntry> for LiveCombatLogEntryDto {
    fn from(value: &CombatLogEntry) -> Self {
        Self {
            id: value.id.clone(),
            step_id: value.step_id.clone(),
            log_index: value.log_index,
            title: value.title.clone(),
            summary: value.summary.clone(),
            outcome_class: value.outcome_class.code().to_string(),
            event_types: value.event_types.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveAuditEntryDto {
    pub id: String,
    pub step_id: String,
    pub sequence: u32,
    pub outcome_class: String,
    pub decision_kind: String,
    pub preflight_decision_kind: Option<String>,
    pub accepted: bool,
    pub rejection_code: Option<String>,
    pub event_count: u32,
    pub trace_count: u32,
    pub state_before_fingerprint: LiveStateFingerprintDto,
    pub state_after_fingerprint: LiveStateFingerprintDto,
}

impl From<&CommandAuditEntry> for LiveAuditEntryDto {
    fn from(value: &CommandAuditEntry) -> Self {
        Self {
            id: value.id.clone(),
            step_id: value.step_id.clone(),
            sequence: value.sequence,
            outcome_class: value.outcome_class.code().to_string(),
            decision_kind: value.decision_kind.code().to_string(),
            preflight_decision_kind: value
                .preflight_decision_kind
                .map(|kind| kind.code().to_string()),
            accepted: value.accepted,
            rejection_code: value
                .rejection
                .map(|rejection| rejection.code().to_string()),
            event_count: value.event_count,
            trace_count: value.trace_count,
            state_before_fingerprint: LiveStateFingerprintDto::from(
                &value.state_before_fingerprint,
            ),
            state_after_fingerprint: LiveStateFingerprintDto::from(&value.state_after_fingerprint),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCombatEndDto {
    pub should_end: bool,
    pub condition_kind: String,
    pub outcome_kind: String,
    pub active_sides: Vec<String>,
    pub defeated_sides: Vec<String>,
    pub winning_sides: Vec<String>,
    pub reason: String,
}

impl From<&CombatEndConditionReadout> for LiveCombatEndDto {
    fn from(value: &CombatEndConditionReadout) -> Self {
        Self {
            should_end: value.combat_should_end,
            condition_kind: value.condition_kind.code().to_string(),
            outcome_kind: value.outcome_kind.code().to_string(),
            active_sides: value.active_sides.clone(),
            defeated_sides: value.defeated_sides.clone(),
            winning_sides: value.winning_sides.clone(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveFinalizationDto {
    pub trigger: String,
    pub finalized_at_step: u32,
    pub outcome_kind: String,
    pub winning_sides: Vec<String>,
    pub remaining_sides: Vec<String>,
    pub final_state_fingerprint: LiveStateFingerprintDto,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveGameplayDecisionEvidenceDto {
    pub decision_id: String,
    pub status: String,
    pub receipt_hash: String,
    pub initial_workspace_hash: String,
    pub final_workspace_hash: String,
    pub declared_read_hashes: Vec<String>,
    pub invocation_output_hashes: Vec<String>,
    pub routing_hash: Option<String>,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveGameplayFabricDto {
    pub registry_digest: String,
    pub binding_registry_hash: String,
    pub module_state_hash: String,
    pub runtime_host_hash: String,
    pub reaction_frame_hashes: Vec<String>,
    pub decisions: Vec<LiveGameplayDecisionEvidenceDto>,
    pub pending_decision_count: u32,
}

impl From<&CombatFinalizationReadout> for LiveFinalizationDto {
    fn from(value: &CombatFinalizationReadout) -> Self {
        Self {
            trigger: value.trigger.code().to_string(),
            finalized_at_step: value.finalized_at_step,
            outcome_kind: value.outcome_kind.code().to_string(),
            winning_sides: value.winning_sides.clone(),
            remaining_sides: value.remaining_sides.clone(),
            final_state_fingerprint: LiveStateFingerprintDto::from(&value.final_state_fingerprint),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveSessionSnapshotDto {
    pub session_id: String,
    pub next_step_index: u32,
    pub lifecycle_phase: String,
    pub started_at_step: Option<u32>,
    pub ended_at_step: Option<u32>,
    pub round_number: u32,
    pub turn_index: u32,
    pub participant_order: Vec<String>,
    pub current_actor_id: Option<String>,
    pub participants: Vec<LiveParticipantDto>,
    pub board: LiveBoardDto,
    pub options: LiveCurrentActorOptionsDto,
    pub combat_end: LiveCombatEndDto,
    pub gameplay_fabric: LiveGameplayFabricDto,
    pub current_reaction_window: Option<ReactionWindowDto>,
    pub reaction_window_lifecycle_log: Vec<ReactionWindowLifecycleEntryDto>,
    pub reaction_audit_log: Vec<ReactionAuditEntryDto>,
    pub finalization: Option<LiveFinalizationDto>,
    pub combat_log: Vec<LiveCombatLogEntryDto>,
    pub audit_log: Vec<LiveAuditEntryDto>,
    pub state_fingerprint: LiveStateFingerprintDto,
    pub action_resource_fingerprint: LiveStateFingerprintDto,
}

impl From<&CombatSessionSnapshot> for LiveSessionSnapshotDto {
    fn from(value: &CombatSessionSnapshot) -> Self {
        Self {
            session_id: value.session_id.clone(),
            next_step_index: value.next_step_index,
            lifecycle_phase: value.lifecycle.phase.code().to_string(),
            started_at_step: value.lifecycle.started_at_step,
            ended_at_step: value.lifecycle.ended_at_step,
            round_number: value.turn_order.round_number,
            turn_index: value.turn_order.current_turn_index,
            participant_order: value.turn_order.participant_order.clone(),
            current_actor_id: value.turn_order.current_actor_id.clone(),
            participants: value
                .current_state
                .combatants
                .iter()
                .map(|combatant| participant(combatant, value))
                .collect(),
            board: LiveBoardDto::from(&value.current_state.board),
            options: LiveCurrentActorOptionsDto::from(&value.current_actor_options),
            combat_end: LiveCombatEndDto::from(&value.combat_end_condition),
            gameplay_fabric: LiveGameplayFabricDto {
                registry_digest: value.gameplay_fabric.registry_digest.clone(),
                binding_registry_hash: value.gameplay_fabric.binding_registry_hash.clone(),
                module_state_hash: value.gameplay_fabric.module_state_hash.clone(),
                runtime_host_hash: value.gameplay_fabric.runtime_host_hash.clone(),
                reaction_frame_hashes: value.gameplay_fabric.reaction_frame_hashes.clone(),
                decisions: value
                    .gameplay_fabric
                    .decisions
                    .iter()
                    .map(|decision| LiveGameplayDecisionEvidenceDto {
                        decision_id: decision.decision_id.clone(),
                        status: decision.status.clone(),
                        receipt_hash: decision.receipt_hash.clone(),
                        initial_workspace_hash: decision.initial_workspace_hash.clone(),
                        final_workspace_hash: decision.final_workspace_hash.clone(),
                        declared_read_hashes: decision.declared_read_hashes.clone(),
                        invocation_output_hashes: decision.invocation_output_hashes.clone(),
                        routing_hash: decision.routing_hash.clone(),
                        diagnostic_codes: decision.diagnostic_codes.clone(),
                    })
                    .collect(),
                pending_decision_count: value.gameplay_fabric.pending_decision_count,
            },
            current_reaction_window: value
                .current_reaction_window
                .as_ref()
                .map(ReactionWindowDto::from),
            reaction_window_lifecycle_log: value
                .reaction_window_lifecycle_log
                .iter()
                .map(ReactionWindowLifecycleEntryDto::from)
                .collect(),
            reaction_audit_log: value
                .reaction_audit_log
                .iter()
                .map(ReactionAuditEntryDto::from)
                .collect(),
            finalization: value.finalization.as_ref().map(LiveFinalizationDto::from),
            combat_log: value
                .combat_log
                .iter()
                .map(LiveCombatLogEntryDto::from)
                .collect(),
            audit_log: value
                .audit_log
                .iter()
                .map(LiveAuditEntryDto::from)
                .collect(),
            state_fingerprint: LiveStateFingerprintDto::from(&value.current_state_fingerprint),
            action_resource_fingerprint: LiveStateFingerprintDto::from(
                &value.action_resource_fingerprint,
            ),
        }
    }
}

fn participant(
    combatant: &FinalCombatantState,
    snapshot: &CombatSessionSnapshot,
) -> LiveParticipantDto {
    let defeated = snapshot
        .combatant_vitality
        .combatants
        .iter()
        .find(|entry| entry.combatant_id == combatant.id)
        .is_some_and(|entry| entry.defeated);
    LiveParticipantDto {
        id: combatant.id.clone(),
        name: combatant.name.clone(),
        current_hit_points: combatant.hit_points.current,
        max_hit_points: combatant.hit_points.max,
        temporary_vitality: combatant.temporary_vitality,
        defeated,
        conditions: combatant.conditions.clone(),
        position: LiveGridPositionDto {
            x: combatant.position.x,
            y: combatant.position.y,
        },
        movement_remaining: combatant.movement_remaining,
        movement_maximum: combatant.movement_maximum,
    }
}
