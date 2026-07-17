use rulebench_rpg_adapter::{
    CombatLogEntry, CombatSessionStepReadout, CombatSessionStepSummary, CombatSessionSummary,
    CombatSessionTranscript, DomainEvent, FinalCombatantState, RulebenchReceipt, RulebenchScenario,
    ScenarioProjection, Team,
};
use serde::{Deserialize, Serialize};

use crate::{LiveBoardDto, LiveTraceEntryDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerScenarioSummaryDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub outcome_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerDefenseDto {
    pub id: String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerCombatantDto {
    pub id: String,
    pub name: String,
    pub team: String,
    pub side_id: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
    pub temporary_vitality: i32,
    pub conditions: Vec<String>,
    pub position_x: u32,
    pub position_y: u32,
    pub defenses: Vec<ViewerDefenseDto>,
    pub is_actor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerSelectedActionDto {
    pub id: String,
    pub name: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub action_text: String,
    pub effect_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerSelectedTargetDto {
    pub target_id: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerDomainEventDto {
    pub sequence: u32,
    pub kind: String,
    pub summary: String,
    pub entity_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerFinalCombatantDto {
    pub id: String,
    pub name: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
    pub temporary_vitality: i32,
    pub conditions: Vec<String>,
    pub position_x: u32,
    pub position_y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerFinalStateDto {
    pub summary: String,
    pub combatants: Vec<ViewerFinalCombatantDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerScenarioReadoutDto {
    pub identity: ViewerScenarioSummaryDto,
    pub board: LiveBoardDto,
    pub combatants: Vec<ViewerCombatantDto>,
    pub selected_action: ViewerSelectedActionDto,
    pub selected_target: Option<ViewerSelectedTargetDto>,
    pub domain_events: Vec<ViewerDomainEventDto>,
    pub trace: Vec<LiveTraceEntryDto>,
    pub final_state: ViewerFinalStateDto,
}

impl ViewerScenarioReadoutDto {
    pub fn new(
        identity: ViewerScenarioSummaryDto,
        scenario: &RulebenchScenario,
        receipt: &RulebenchReceipt,
        final_state: &ScenarioProjection,
    ) -> Self {
        Self {
            identity,
            board: LiveBoardDto::from(&final_state.board),
            combatants: scenario
                .combatants
                .iter()
                .map(|combatant| {
                    let final_combatant = final_state
                        .combatants
                        .iter()
                        .find(|candidate| candidate.id == combatant.id);
                    ViewerCombatantDto {
                        id: combatant.id.clone(),
                        name: combatant.name.clone(),
                        team: match combatant.team {
                            Team::Ally => "ally".to_string(),
                            Team::Enemy => "enemy".to_string(),
                        },
                        side_id: combatant.side_id.clone(),
                        current_hit_points: final_combatant
                            .map_or(combatant.hit_points.current, |state| {
                                state.hit_points.current
                            }),
                        max_hit_points: final_combatant
                            .map_or(combatant.hit_points.max, |state| state.hit_points.max),
                        temporary_vitality: final_combatant
                            .map_or(combatant.temporary_vitality, |state| {
                                state.temporary_vitality
                            }),
                        conditions: final_combatant.map_or_else(
                            || combatant.conditions.clone(),
                            |state| state.conditions.clone(),
                        ),
                        position_x: final_combatant
                            .map_or(combatant.position.x, |state| state.position.x),
                        position_y: final_combatant
                            .map_or(combatant.position.y, |state| state.position.y),
                        defenses: combatant
                            .defenses
                            .iter()
                            .map(|defense| ViewerDefenseDto {
                                id: defense.id.clone(),
                                label: defense.label.clone(),
                                value: defense.value,
                            })
                            .collect(),
                        is_actor: combatant.is_actor,
                    }
                })
                .collect(),
            selected_action: ViewerSelectedActionDto {
                id: scenario.selected_action.id.clone(),
                name: scenario.selected_action.name.clone(),
                actor_id: scenario.selected_action.actor_id.clone(),
                target_ids: scenario.selected_action.targeting.target_ids.clone(),
                action_text: scenario.selected_action.action_text.clone(),
                effect_text: scenario.selected_action.effect_text.clone(),
            },
            selected_target: receipt.target_legality.as_ref().map(|target| {
                ViewerSelectedTargetDto {
                    target_id: target.target_id.clone(),
                    accepted: target.accepted,
                    reason: target.reason.clone(),
                }
            }),
            domain_events: receipt
                .events
                .iter()
                .enumerate()
                .map(|(index, event)| viewer_event(index as u32 + 1, event))
                .collect(),
            trace: receipt.trace.iter().map(LiveTraceEntryDto::from).collect(),
            final_state: ViewerFinalStateDto::from(final_state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerSessionStepSummaryDto {
    pub id: String,
    pub index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: String,
    pub log_index: u32,
}

impl From<&CombatSessionStepSummary> for ViewerSessionStepSummaryDto {
    fn from(value: &CombatSessionStepSummary) -> Self {
        Self {
            id: value.id.clone(),
            index: value.index,
            title: value.title.clone(),
            summary: value.summary.clone(),
            outcome_class: value.outcome_class.code().to_string(),
            log_index: value.log_index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerSessionSummaryDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub steps: Vec<ViewerSessionStepSummaryDto>,
}

impl From<&CombatSessionSummary> for ViewerSessionSummaryDto {
    fn from(value: &CombatSessionSummary) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            summary: value.summary.clone(),
            seed_label: value.seed_label.clone(),
            steps: value
                .steps
                .iter()
                .map(ViewerSessionStepSummaryDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerCommandAttemptDto {
    pub step_id: String,
    pub step_index: u32,
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    pub roll_stream: Vec<i32>,
    pub outcome_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerCombatLogEntryDto {
    pub id: String,
    pub step_id: String,
    pub log_index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: String,
    pub event_types: Vec<String>,
}

impl From<&CombatLogEntry> for ViewerCombatLogEntryDto {
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
pub struct ViewerSessionStepReadoutDto {
    pub session_id: String,
    pub step: ViewerSessionStepSummaryDto,
    pub command: ViewerCommandAttemptDto,
    pub scenario: ViewerScenarioReadoutDto,
    pub combat_log: Vec<ViewerCombatLogEntryDto>,
    pub state_before: ViewerFinalStateDto,
    pub state_after: ViewerFinalStateDto,
}

impl From<&CombatSessionStepReadout> for ViewerSessionStepReadoutDto {
    fn from(value: &CombatSessionStepReadout) -> Self {
        let identity = ViewerScenarioSummaryDto {
            id: value.step.id.clone(),
            title: value.step.title.clone(),
            summary: value.step.summary.clone(),
            seed_label: format!(
                "roll-stream:{}",
                value
                    .command
                    .roll_stream
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            outcome_class: value.step.outcome_class.code().to_string(),
        };
        Self {
            session_id: value.session_id.clone(),
            step: ViewerSessionStepSummaryDto::from(&value.step),
            command: ViewerCommandAttemptDto {
                step_id: value.command.step_id.clone(),
                step_index: value.command.step_index,
                actor_id: value.command.actor_id.clone(),
                action_id: value.command.action_id.clone(),
                target_id: value.command.target_id.clone(),
                roll_stream: value.command.roll_stream.clone(),
                outcome_class: value.command.outcome_class.code().to_string(),
            },
            scenario: ViewerScenarioReadoutDto::new(
                identity,
                &value.scenario,
                &value.receipt,
                &value.state_after,
            ),
            combat_log: value
                .combat_log
                .iter()
                .map(ViewerCombatLogEntryDto::from)
                .collect(),
            state_before: ViewerFinalStateDto::from(&value.state_before),
            state_after: ViewerFinalStateDto::from(&value.state_after),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewerSessionTranscriptDto {
    pub summary: ViewerSessionSummaryDto,
    pub steps: Vec<ViewerSessionStepReadoutDto>,
}

impl From<&CombatSessionTranscript> for ViewerSessionTranscriptDto {
    fn from(value: &CombatSessionTranscript) -> Self {
        Self {
            summary: ViewerSessionSummaryDto::from(&value.summary),
            steps: value
                .steps
                .iter()
                .map(ViewerSessionStepReadoutDto::from)
                .collect(),
        }
    }
}

impl From<&ScenarioProjection> for ViewerFinalStateDto {
    fn from(value: &ScenarioProjection) -> Self {
        Self {
            summary: value.summary.clone(),
            combatants: value
                .combatants
                .iter()
                .map(ViewerFinalCombatantDto::from)
                .collect(),
        }
    }
}

impl From<&FinalCombatantState> for ViewerFinalCombatantDto {
    fn from(value: &FinalCombatantState) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            current_hit_points: value.hit_points.current,
            max_hit_points: value.hit_points.max,
            temporary_vitality: value.temporary_vitality,
            conditions: value.conditions.clone(),
            position_x: value.position.x,
            position_y: value.position.y,
        }
    }
}

fn viewer_event(sequence: u32, event: &DomainEvent) -> ViewerDomainEventDto {
    let live = crate::LiveDomainEventDto::from(event);
    ViewerDomainEventDto {
        sequence,
        kind: live.kind,
        summary: live.summary,
        entity_ids: event_entity_ids(event),
    }
}

fn event_entity_ids(event: &DomainEvent) -> Vec<String> {
    match event {
        DomainEvent::IntentShapeAccepted {
            actor_id,
            target_id,
            ..
        }
        | DomainEvent::ActionUsed {
            actor_id,
            target_id,
            ..
        }
        | DomainEvent::AttackRolled {
            actor_id,
            target_id,
            ..
        }
        | DomainEvent::SavingThrowResolved {
            actor_id,
            target_id,
            ..
        }
        | DomainEvent::ContestedCheckResolved {
            actor_id,
            target_id,
            ..
        } => vec![actor_id.clone(), target_id.clone()],
        DomainEvent::DamageApplied { target_id, .. }
        | DomainEvent::HealingApplied { target_id, .. }
        | DomainEvent::TemporaryVitalityGranted { target_id, .. }
        | DomainEvent::ModifierApplied { target_id, .. }
        | DomainEvent::EffectMovementApplied { target_id, .. }
        | DomainEvent::ResourceChanged { target_id, .. } => vec![target_id.clone()],
        DomainEvent::PositionChanged { actor_id, .. }
        | DomainEvent::MovementSpent { actor_id, .. } => vec![actor_id.clone()],
    }
}
