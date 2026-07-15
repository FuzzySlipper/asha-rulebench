use rulebench_rules::{
    CombatControlReadout, CombatSessionSnapshot, CombatSessionStepReadout, CommandCandidateEntry,
    CommandCandidateSummary, CommandPreflightReadout, DomainEvent, RollConsumptionEntry,
    TargetResolutionOutcome, TraceEntry, TracePhase, TraceStatus,
};
use serde::{Deserialize, Serialize};

use super::{
    LiveActionResourceCostDto, LiveActionResourceStateDto, LiveSessionSnapshotDto,
    LiveStateFingerprintDto,
};
use crate::{ReactionCommandReadoutDto, UseActionIntentDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveTransportErrorDto {
    pub kind: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveReactionExecutionDto {
    pub reaction: ReactionCommandReadoutDto,
    pub snapshot: LiveSessionSnapshotDto,
}

impl LiveReactionExecutionDto {
    pub fn new(
        reaction: &rulebench_rules::ReactionCommandReadout,
        snapshot: &CombatSessionSnapshot,
    ) -> Self {
        Self {
            reaction: ReactionCommandReadoutDto::from(reaction),
            snapshot: LiveSessionSnapshotDto::from(snapshot),
        }
    }
}

impl LiveTransportErrorDto {
    pub fn new(
        kind: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            kind: kind.into(),
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveRollEvidenceDto {
    pub sequence: u32,
    pub request_kind: String,
    pub supplied_value: Option<i32>,
    pub consumed: bool,
    pub reason: String,
}

impl From<&RollConsumptionEntry> for LiveRollEvidenceDto {
    fn from(value: &RollConsumptionEntry) -> Self {
        Self {
            sequence: value.sequence,
            request_kind: value.request_kind.code().to_string(),
            supplied_value: value.supplied_value,
            consumed: value.consumed,
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveTraceEntryDto {
    pub sequence: u32,
    pub phase: String,
    pub status: String,
    pub message: String,
    pub detail: String,
}

impl From<&TraceEntry> for LiveTraceEntryDto {
    fn from(value: &TraceEntry) -> Self {
        Self {
            sequence: value.sequence,
            phase: trace_phase(value.phase).to_string(),
            status: trace_status(value.status).to_string(),
            message: value.message.clone(),
            detail: value.detail.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveDomainEventDto {
    pub kind: String,
    pub summary: String,
}

impl From<&DomainEvent> for LiveDomainEventDto {
    fn from(value: &DomainEvent) -> Self {
        let (kind, summary) = domain_event(value);
        Self {
            kind: kind.to_string(),
            summary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LivePreflightDto {
    pub intent: UseActionIntentDto,
    pub accepted: bool,
    pub decision_kind: String,
    pub rejection_code: Option<String>,
    pub current_actor_id: Option<String>,
    pub target_id: Option<String>,
    pub target_accepted: Option<bool>,
    pub target_reason: Option<String>,
    pub resource_costs: Vec<LiveActionResourceCostDto>,
    pub action_resource: Option<LiveActionResourceStateDto>,
    pub reason: String,
}

impl From<&CommandPreflightReadout> for LivePreflightDto {
    fn from(value: &CommandPreflightReadout) -> Self {
        Self {
            intent: UseActionIntentDto::from(&value.intent),
            accepted: value.accepted,
            decision_kind: value.decision_kind.code().to_string(),
            rejection_code: value
                .rejection
                .map(|rejection| rejection.code().to_string()),
            current_actor_id: value.current_actor_id.clone(),
            target_id: value
                .target_legality
                .as_ref()
                .map(|legality| legality.target_id.clone()),
            target_accepted: value
                .target_legality
                .as_ref()
                .map(|legality| legality.accepted),
            target_reason: value
                .target_legality
                .as_ref()
                .map(|legality| legality.reason.clone()),
            resource_costs: value
                .resource_costs
                .iter()
                .map(LiveActionResourceCostDto::from)
                .collect(),
            action_resource: value
                .action_resource
                .as_ref()
                .map(LiveActionResourceStateDto::from),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCandidateDto {
    pub intent: UseActionIntentDto,
    pub ability_id: String,
    pub target_name: String,
    pub target_current_hit_points: i32,
    pub target_max_hit_points: i32,
    pub accepted: bool,
    pub decision_kind: String,
    pub rejection_code: Option<String>,
    pub reason: String,
}

impl From<&CommandCandidateEntry> for LiveCandidateDto {
    fn from(value: &CommandCandidateEntry) -> Self {
        Self {
            intent: UseActionIntentDto::from(&value.intent),
            ability_id: value.ability_id.clone(),
            target_name: value.target_name.clone(),
            target_current_hit_points: value.target_current_hit_points,
            target_max_hit_points: value.target_max_hit_points,
            accepted: value.accepted,
            decision_kind: value.decision_kind.code().to_string(),
            rejection_code: value
                .rejection
                .map(|rejection| rejection.code().to_string()),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCandidateSummaryDto {
    pub round_number: u32,
    pub turn_index: u32,
    pub lifecycle_phase: String,
    pub current_actor_id: Option<String>,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub candidates: Vec<LiveCandidateDto>,
}

impl From<&CommandCandidateSummary> for LiveCandidateSummaryDto {
    fn from(value: &CommandCandidateSummary) -> Self {
        Self {
            round_number: value.round_number,
            turn_index: value.turn_index,
            lifecycle_phase: value.lifecycle_phase.code().to_string(),
            current_actor_id: value.current_actor_id.clone(),
            available: value.available,
            unavailable_reason: value
                .unavailable_reason
                .map(|reason| reason.code().to_string()),
            candidates: value
                .candidates
                .iter()
                .map(LiveCandidateDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCommandStepDto {
    pub session_id: String,
    pub step_id: String,
    pub step_index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: String,
    pub accepted: bool,
    pub decision_kind: String,
    pub rejection_code: Option<String>,
    pub intent: UseActionIntentDto,
    pub rolls: Vec<LiveRollEvidenceDto>,
    pub events: Vec<LiveDomainEventDto>,
    pub target_results: Vec<LiveTargetResolutionDto>,
    pub trace: Vec<LiveTraceEntryDto>,
    pub state_before_fingerprint: LiveStateFingerprintDto,
    pub state_after_fingerprint: LiveStateFingerprintDto,
    pub roll_mode: String,
    pub generated_rolls: Vec<LiveGeneratedRollDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveTargetResolutionDto {
    pub target_id: String,
    pub accepted: bool,
    pub reason: String,
    pub attack_outcome: Option<String>,
    pub damage_amount: Option<i32>,
    pub movement_kind: Option<String>,
    pub movement_from: Option<super::LiveGridPositionDto>,
    pub movement_to: Option<super::LiveGridPositionDto>,
    pub resource_changes: Vec<LiveResourceChangeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveResourceChangeDto {
    pub resource_id: String,
    pub requested_delta: i32,
    pub before: i32,
    pub after: i32,
    pub maximum: i32,
}

impl From<&TargetResolutionOutcome> for LiveTargetResolutionDto {
    fn from(value: &TargetResolutionOutcome) -> Self {
        Self {
            target_id: value.target_id.clone(),
            accepted: value.target_legality.accepted,
            reason: value.target_legality.reason.clone(),
            attack_outcome: value.attack_roll.as_ref().map(|roll| match roll.outcome {
                rulebench_rules::AttackOutcome::Hit => "hit".to_string(),
                rulebench_rules::AttackOutcome::Miss => "miss".to_string(),
            }),
            damage_amount: value.damage.as_ref().map(|damage| damage.amount),
            movement_kind: value
                .movement
                .as_ref()
                .map(|movement| movement.movement_kind.code().to_string()),
            movement_from: value
                .movement
                .as_ref()
                .map(|movement| super::LiveGridPositionDto {
                    x: movement.from.x,
                    y: movement.from.y,
                }),
            movement_to: value
                .movement
                .as_ref()
                .map(|movement| super::LiveGridPositionDto {
                    x: movement.to.x,
                    y: movement.to.y,
                }),
            resource_changes: value
                .resource_changes
                .iter()
                .map(|change| LiveResourceChangeDto {
                    resource_id: change.resource_id.clone(),
                    requested_delta: change.requested_delta,
                    before: change.before,
                    after: change.after,
                    maximum: change.maximum,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveGeneratedRollDto {
    pub sequence: u32,
    pub command_id: String,
    pub request_kind: String,
    pub die_expression: String,
    pub value: i32,
    pub source_mode: String,
}

impl From<&CombatSessionStepReadout> for LiveCommandStepDto {
    fn from(value: &CombatSessionStepReadout) -> Self {
        Self {
            session_id: value.session_id.clone(),
            step_id: value.step.id.clone(),
            step_index: value.step.index,
            title: value.step.title.clone(),
            summary: value.step.summary.clone(),
            outcome_class: value.step.outcome_class.code().to_string(),
            accepted: value.audit_entry.accepted,
            decision_kind: value.audit_entry.decision_kind.code().to_string(),
            rejection_code: value
                .audit_entry
                .rejection
                .map(|rejection| rejection.code().to_string()),
            intent: UseActionIntentDto::from(&value.receipt.intent),
            rolls: value
                .receipt
                .roll_consumption
                .iter()
                .map(LiveRollEvidenceDto::from)
                .collect(),
            events: value
                .receipt
                .events
                .iter()
                .map(LiveDomainEventDto::from)
                .collect(),
            target_results: value
                .receipt
                .target_results
                .iter()
                .map(LiveTargetResolutionDto::from)
                .collect(),
            trace: value
                .receipt
                .trace
                .iter()
                .map(LiveTraceEntryDto::from)
                .collect(),
            state_before_fingerprint: LiveStateFingerprintDto::from(
                &value.audit_entry.state_before_fingerprint,
            ),
            state_after_fingerprint: LiveStateFingerprintDto::from(
                &value.audit_entry.state_after_fingerprint,
            ),
            roll_mode: value.roll_mode.code().to_string(),
            generated_rolls: value
                .generated_rolls
                .iter()
                .map(|roll| LiveGeneratedRollDto {
                    sequence: roll.sequence,
                    command_id: roll.command_id.clone(),
                    request_kind: roll.request_kind.code().to_string(),
                    die_expression: roll.die_expression.clone(),
                    value: roll.value,
                    source_mode: roll.source_mode.code().to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveCommandExecutionDto {
    pub step: LiveCommandStepDto,
    pub snapshot: LiveSessionSnapshotDto,
}

impl LiveCommandExecutionDto {
    pub fn new(step: &CombatSessionStepReadout, snapshot: &CombatSessionSnapshot) -> Self {
        Self {
            step: LiveCommandStepDto::from(step),
            snapshot: LiveSessionSnapshotDto::from(snapshot),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveControlExecutionDto {
    pub command_kind: String,
    pub accepted: bool,
    pub decision_kind: String,
    pub previous_lifecycle_phase: String,
    pub next_lifecycle_phase: String,
    pub state_before_fingerprint: LiveStateFingerprintDto,
    pub state_after_fingerprint: LiveStateFingerprintDto,
    pub reason: String,
    pub snapshot: LiveSessionSnapshotDto,
}

impl LiveControlExecutionDto {
    pub fn new(readout: &CombatControlReadout, snapshot: &CombatSessionSnapshot) -> Self {
        Self {
            command_kind: readout.command_kind.code().to_string(),
            accepted: readout.accepted,
            decision_kind: readout.decision_kind.code().to_string(),
            previous_lifecycle_phase: readout.previous_lifecycle.phase.code().to_string(),
            next_lifecycle_phase: readout.next_lifecycle.phase.code().to_string(),
            state_before_fingerprint: LiveStateFingerprintDto::from(
                &readout.state_before_fingerprint,
            ),
            state_after_fingerprint: LiveStateFingerprintDto::from(
                &readout.state_after_fingerprint,
            ),
            reason: readout.reason.clone(),
            snapshot: LiveSessionSnapshotDto::from(snapshot),
        }
    }
}

fn trace_phase(value: TracePhase) -> &'static str {
    match value {
        TracePhase::Proposal => "proposal",
        TracePhase::Validation => "validation",
        TracePhase::Resolution => "resolution",
        TracePhase::Commit => "commit",
    }
}

fn trace_status(value: TraceStatus) -> &'static str {
    match value {
        TraceStatus::Accepted => "accepted",
        TraceStatus::Rejected => "rejected",
        TraceStatus::Info => "info",
    }
}

fn domain_event(value: &DomainEvent) -> (&'static str, String) {
    match value {
        DomainEvent::IntentShapeAccepted {
            actor_id,
            action_id,
            target_id,
        } => (
            "intentShapeAccepted",
            format!("{actor_id} proposed {action_id} against {target_id}."),
        ),
        DomainEvent::ActionUsed {
            actor_id,
            action_id,
            target_id,
        } => (
            "actionUsed",
            format!("{actor_id} used {action_id} against {target_id}."),
        ),
        DomainEvent::AttackRolled {
            actor_id,
            target_id,
            total,
            defense_id,
            defense_value,
            ..
        } => (
            "attackRolled",
            format!("{actor_id} rolled {total} against {target_id} {defense_id} {defense_value}."),
        ),
        DomainEvent::SavingThrowResolved {
            actor_id,
            target_id,
            total,
            difficulty_class,
            ..
        } => (
            "savingThrowResolved",
            format!(
                "{actor_id} resolved a saving throw for {target_id}: {total} against {difficulty_class}."
            ),
        ),
        DomainEvent::ContestedCheckResolved {
            actor_id,
            target_id,
            actor_total,
            target_total,
            ..
        } => (
            "contestedCheckResolved",
            format!(
                "{actor_id} contested {target_id}: {actor_total} against {target_total}."
            ),
        ),
        DomainEvent::DamageApplied {
            target_id,
            amount,
            damage_type,
        } => (
            "damageApplied",
            format!("{target_id} took {amount} {damage_type} damage."),
        ),
        DomainEvent::HealingApplied {
            target_id,
            amount,
            healing_type,
        } => (
            "healingApplied",
            format!("{target_id} recovered {amount} from {healing_type}."),
        ),
        DomainEvent::TemporaryVitalityGranted { target_id, amount } => (
            "temporaryVitalityGranted",
            format!("{target_id} gained {amount} temporary vitality."),
        ),
        DomainEvent::ModifierApplied {
            target_id,
            modifier_id,
            duration,
        } => (
            "modifierApplied",
            format!("{target_id} gained {modifier_id} for {duration}."),
        ),
        DomainEvent::PositionChanged { actor_id, from, to } => (
            "positionChanged",
            format!("{actor_id} moved from {},{} to {},{}.", from.x, from.y, to.x, to.y),
        ),
        DomainEvent::MovementSpent { actor_id, amount, remaining } => (
            "movementSpent",
            format!("{actor_id} spent {amount} movement and has {remaining} remaining."),
        ),
        DomainEvent::EffectMovementApplied {
            target_id,
            movement_kind,
            from,
            to,
        } => (
            "effectMovementApplied",
            format!(
                "{target_id} resolved {} movement from {},{} to {},{}.",
                movement_kind.code(),
                from.x,
                from.y,
                to.x,
                to.y
            ),
        ),
        DomainEvent::ResourceChanged {
            target_id,
            resource_id,
            delta,
            before,
            after,
        } => (
            "resourceChanged",
            format!(
                "{target_id} changed {resource_id} by {delta} from {before} to {after}."
            ),
        ),
    }
}
