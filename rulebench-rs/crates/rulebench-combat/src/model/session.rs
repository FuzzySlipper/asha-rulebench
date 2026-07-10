/// Session command, audit, and snapshot readbacks.
use super::{
    ActionResourceLedgerReadout, ActionResourceState, ActionResourceTransitionEntry,
    CombatControlHistoryEntry, CombatLifecycle, CombatLifecyclePhase, CombatTurnOrder,
    CommandOutcomeClass, LifecycleTransitionEntry, ModifierDurationExpirationEntry,
    RollConsumptionEntry, RulebenchReceipt, RulebenchRejection, RulebenchScenario,
    ScenarioProjection, StateFingerprint, TargetLegality, TurnTransitionEntry, UseActionIntent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub steps: Vec<CombatSessionStepSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionStepSummary {
    pub id: String,
    pub index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: CommandOutcomeClass,
    pub log_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAttempt {
    pub step_id: String,
    pub step_index: u32,
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    pub roll_stream: Vec<i32>,
    pub outcome_class: CommandOutcomeClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDecisionKind {
    AcceptedByResolver,
    RejectedByResolver,
    RejectedByPreflight,
    RejectedByLifecycle,
    RejectedByTurnOrder,
}

impl CommandDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CommandDecisionKind::AcceptedByResolver => "acceptedByResolver",
            CommandDecisionKind::RejectedByResolver => "rejectedByResolver",
            CommandDecisionKind::RejectedByPreflight => "rejectedByPreflight",
            CommandDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CommandDecisionKind::RejectedByTurnOrder => "rejectedByTurnOrder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPreflightDecisionKind {
    Accepted,
    RejectedByShape,
    RejectedByLifecycle,
    RejectedByTurnOrder,
    RejectedByActorLookup,
    RejectedByActionLookup,
    RejectedByActionOwnership,
    RejectedByTargetLookup,
    RejectedByTargetLegality,
    RejectedByActionResource,
}

impl CommandPreflightDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CommandPreflightDecisionKind::Accepted => "accepted",
            CommandPreflightDecisionKind::RejectedByShape => "rejectedByShape",
            CommandPreflightDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CommandPreflightDecisionKind::RejectedByTurnOrder => "rejectedByTurnOrder",
            CommandPreflightDecisionKind::RejectedByActorLookup => "rejectedByActorLookup",
            CommandPreflightDecisionKind::RejectedByActionLookup => "rejectedByActionLookup",
            CommandPreflightDecisionKind::RejectedByActionOwnership => "rejectedByActionOwnership",
            CommandPreflightDecisionKind::RejectedByTargetLookup => "rejectedByTargetLookup",
            CommandPreflightDecisionKind::RejectedByTargetLegality => "rejectedByTargetLegality",
            CommandPreflightDecisionKind::RejectedByActionResource => "rejectedByActionResource",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPreflightReadout {
    pub intent: UseActionIntent,
    pub accepted: bool,
    pub decision_kind: CommandPreflightDecisionKind,
    pub rejection: Option<RulebenchRejection>,
    pub current_actor_id: Option<String>,
    pub target_legality: Option<TargetLegality>,
    pub action_resource: Option<ActionResourceState>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAuditEntry {
    pub id: String,
    pub step_id: String,
    pub sequence: u32,
    pub outcome_class: CommandOutcomeClass,
    pub decision_kind: CommandDecisionKind,
    pub preflight_decision_kind: Option<CommandPreflightDecisionKind>,
    pub accepted: bool,
    pub rejection: Option<RulebenchRejection>,
    pub event_count: u32,
    pub trace_count: u32,
    pub roll_consumption: Vec<RollConsumptionEntry>,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionUsageEntry {
    pub id: String,
    pub step_id: String,
    pub step_index: u32,
    pub round_number: u32,
    pub turn_index: u32,
    pub actor_id: String,
    pub action_id: String,
    pub ability_id: String,
    pub target_id: String,
    pub outcome_class: CommandOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionUsageSummary {
    pub round_number: u32,
    pub turn_index: u32,
    pub current_actor_id: Option<String>,
    pub used_action_count: u32,
    pub used_action_ids: Vec<String>,
    pub used_ability_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantVitalityEntry {
    pub combatant_id: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
    pub defeated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantVitalitySummary {
    pub combatants: Vec<CombatantVitalityEntry>,
    pub active_combatant_ids: Vec<String>,
    pub defeated_combatant_ids: Vec<String>,
    pub active_count: u32,
    pub defeated_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEndConditionKind {
    Ongoing,
    NoActiveEnemies,
    NoActiveAllies,
    NoActiveCombatants,
}

impl CombatEndConditionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatEndConditionKind::Ongoing => "ongoing",
            CombatEndConditionKind::NoActiveEnemies => "noActiveEnemies",
            CombatEndConditionKind::NoActiveAllies => "noActiveAllies",
            CombatEndConditionKind::NoActiveCombatants => "noActiveCombatants",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatEndConditionReadout {
    pub combat_should_end: bool,
    pub condition_kind: CombatEndConditionKind,
    pub active_ally_count: u32,
    pub active_enemy_count: u32,
    pub defeated_ally_count: u32,
    pub defeated_enemy_count: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentActorOptionsUnavailableReason {
    CombatEnded,
    NoCurrentActor,
    CurrentActorDefeated,
    NoMatchingActions,
    NoVisibleActiveTargets,
}

impl CurrentActorOptionsUnavailableReason {
    pub const fn code(self) -> &'static str {
        match self {
            CurrentActorOptionsUnavailableReason::CombatEnded => "combatEnded",
            CurrentActorOptionsUnavailableReason::NoCurrentActor => "noCurrentActor",
            CurrentActorOptionsUnavailableReason::CurrentActorDefeated => "currentActorDefeated",
            CurrentActorOptionsUnavailableReason::NoMatchingActions => "noMatchingActions",
            CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets => {
                "noVisibleActiveTargets"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentActorTargetOption {
    pub target_id: String,
    pub target_name: String,
    pub current_hit_points: i32,
    pub max_hit_points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentActorActionOption {
    pub action_id: String,
    pub ability_id: String,
    pub action_name: String,
    pub target_options: Vec<CurrentActorTargetOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentActorOptionSummary {
    pub round_number: u32,
    pub turn_index: u32,
    pub lifecycle_phase: CombatLifecyclePhase,
    pub current_actor_id: Option<String>,
    pub current_actor_defeated: bool,
    pub available: bool,
    pub unavailable_reason: Option<CurrentActorOptionsUnavailableReason>,
    pub actions: Vec<CurrentActorActionOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCandidateEntry {
    pub intent: UseActionIntent,
    pub action_id: String,
    pub ability_id: String,
    pub target_id: String,
    pub target_name: String,
    pub target_current_hit_points: i32,
    pub target_max_hit_points: i32,
    pub accepted: bool,
    pub decision_kind: CommandPreflightDecisionKind,
    pub rejection: Option<RulebenchRejection>,
    pub target_legality: Option<TargetLegality>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCandidateSummary {
    pub round_number: u32,
    pub turn_index: u32,
    pub lifecycle_phase: CombatLifecyclePhase,
    pub current_actor_id: Option<String>,
    pub current_actor_defeated: bool,
    pub available: bool,
    pub unavailable_reason: Option<CurrentActorOptionsUnavailableReason>,
    pub candidates: Vec<CommandCandidateEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatLogEntry {
    pub id: String,
    pub step_id: String,
    pub log_index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: CommandOutcomeClass,
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionTranscript {
    pub summary: CombatSessionSummary,
    pub steps: Vec<CombatSessionStepReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatControlHistoryReadout {
    pub session_id: String,
    pub title: String,
    pub summary: String,
    pub history: Vec<CombatControlHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionStepReadout {
    pub session_id: String,
    pub step: CombatSessionStepSummary,
    pub command: CommandAttempt,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
    pub combat_log: Vec<CombatLogEntry>,
    pub action_resource_ledger: ActionResourceLedgerReadout,
    pub audit_entry: CommandAuditEntry,
    pub state_before: ScenarioProjection,
    pub state_after: ScenarioProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionSnapshot {
    pub session_id: String,
    pub next_step_index: u32,
    pub lifecycle: CombatLifecycle,
    pub lifecycle_transition_log: Vec<LifecycleTransitionEntry>,
    pub turn_order: CombatTurnOrder,
    pub combat_log: Vec<CombatLogEntry>,
    pub audit_log: Vec<CommandAuditEntry>,
    pub action_usage_log: Vec<ActionUsageEntry>,
    pub action_resource_transition_log: Vec<ActionResourceTransitionEntry>,
    pub modifier_duration_expiration_log: Vec<ModifierDurationExpirationEntry>,
    pub turn_transition_log: Vec<TurnTransitionEntry>,
    pub action_resource_ledger: ActionResourceLedgerReadout,
    pub current_turn_action_usage: ActionUsageSummary,
    pub combatant_vitality: CombatantVitalitySummary,
    pub combat_end_condition: CombatEndConditionReadout,
    pub current_actor_options: CurrentActorOptionSummary,
    pub current_state: ScenarioProjection,
    pub current_state_fingerprint: StateFingerprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionError {
    UnknownSessionId,
    UnknownStepId,
}
