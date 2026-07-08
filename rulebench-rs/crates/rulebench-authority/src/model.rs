pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
}

impl ScenarioOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            ScenarioOutcomeClass::AcceptedHit => "acceptedHit",
            ScenarioOutcomeClass::AcceptedMiss => "acceptedMiss",
            ScenarioOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub outcome_class: ScenarioOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogCase {
    pub summary: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogResolution {
    pub case: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetCatalogReadout {
    pub selected_ruleset_id: String,
    pub rulesets: Vec<RulesetMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationReadout {
    pub scenario_id: String,
    pub scenario_title: String,
    pub report: ContentValidationReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioCatalogError {
    UnknownScenarioId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
    RejectedInvalidCommand,
}

impl CommandOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            CommandOutcomeClass::AcceptedHit => "acceptedHit",
            CommandOutcomeClass::AcceptedMiss => "acceptedMiss",
            CommandOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
            CommandOutcomeClass::RejectedInvalidCommand => "rejectedInvalidCommand",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatLifecyclePhase {
    Ready,
    InProgress,
    Ended,
}

impl CombatLifecyclePhase {
    pub const fn code(self) -> &'static str {
        match self {
            CombatLifecyclePhase::Ready => "ready",
            CombatLifecyclePhase::InProgress => "inProgress",
            CombatLifecyclePhase::Ended => "ended",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatLifecycle {
    pub phase: CombatLifecyclePhase,
    pub started_at_step: Option<u32>,
    pub ended_at_step: Option<u32>,
}

impl CombatLifecycle {
    pub const fn ready() -> Self {
        Self {
            phase: CombatLifecyclePhase::Ready,
            started_at_step: None,
            ended_at_step: None,
        }
    }

    pub fn start_at_step(&mut self, step_index: u32) {
        if self.phase == CombatLifecyclePhase::Ready {
            self.phase = CombatLifecyclePhase::InProgress;
            self.started_at_step = Some(step_index);
        }
    }

    pub fn end_at_step(&mut self, step_index: u32) {
        if self.phase == CombatLifecyclePhase::Ended {
            return;
        }

        if self.started_at_step.is_none() {
            self.started_at_step = Some(step_index);
        }
        self.phase = CombatLifecyclePhase::Ended;
        self.ended_at_step = Some(step_index);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransitionTrigger {
    ExplicitStart,
    CommandStart,
    ExplicitEnd,
    ConditionalEnd,
}

impl LifecycleTransitionTrigger {
    pub const fn code(self) -> &'static str {
        match self {
            LifecycleTransitionTrigger::ExplicitStart => "explicitStart",
            LifecycleTransitionTrigger::CommandStart => "commandStart",
            LifecycleTransitionTrigger::ExplicitEnd => "explicitEnd",
            LifecycleTransitionTrigger::ConditionalEnd => "conditionalEnd",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTransitionEntry {
    pub sequence: u32,
    pub trigger: LifecycleTransitionTrigger,
    pub step_index: u32,
    pub previous_phase: CombatLifecyclePhase,
    pub next_phase: CombatLifecyclePhase,
    pub started_at_step: Option<u32>,
    pub ended_at_step: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatTurnOrder {
    pub round_number: u32,
    pub current_turn_index: u32,
    pub participant_order: Vec<String>,
    pub current_actor_id: Option<String>,
}

impl CombatTurnOrder {
    pub fn from_participant_order(participant_order: Vec<String>) -> Self {
        let current_actor_id = participant_order.first().cloned();
        let round_number = if participant_order.is_empty() { 0 } else { 1 };

        Self {
            round_number,
            current_turn_index: 0,
            participant_order,
            current_actor_id,
        }
    }

    pub fn advance_turn(&mut self) {
        if self.participant_order.is_empty() {
            return;
        }

        let next_turn_index = (self.current_turn_index + 1) % self.participant_order.len() as u32;
        if next_turn_index == 0 {
            self.round_number += 1;
        }

        self.current_turn_index = next_turn_index;
        self.current_actor_id = self
            .participant_order
            .get(next_turn_index as usize)
            .cloned();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnTransitionEntry {
    pub sequence: u32,
    pub previous_round_number: u32,
    pub previous_turn_index: u32,
    pub previous_actor_id: Option<String>,
    pub next_round_number: u32,
    pub next_turn_index: u32,
    pub next_actor_id: Option<String>,
    pub wrapped_round: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnAdvanceDecisionKind {
    Advanced,
    RejectedByLifecycle,
    RejectedByEmptyTurnOrder,
}

impl TurnAdvanceDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            TurnAdvanceDecisionKind::Advanced => "advanced",
            TurnAdvanceDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder => "rejectedByEmptyTurnOrder",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnAdvanceReadout {
    pub accepted: bool,
    pub decision_kind: TurnAdvanceDecisionKind,
    pub previous_turn_order: CombatTurnOrder,
    pub next_turn_order: CombatTurnOrder,
    pub transition: Option<TurnTransitionEntry>,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResourceKind {
    StandardAction,
}

impl ActionResourceKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceKind::StandardAction => "standardAction",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceState {
    pub kind: ActionResourceKind,
    pub current: i32,
    pub max: i32,
    pub available: bool,
}

impl ActionResourceState {
    pub const fn new(kind: ActionResourceKind, current: i32, max: i32) -> Self {
        Self {
            kind,
            current,
            max,
            available: current > 0,
        }
    }

    pub const fn standard_action_available() -> Self {
        Self::new(ActionResourceKind::StandardAction, 1, 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantActionResourceReadout {
    pub combatant_id: String,
    pub resources: Vec<ActionResourceState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceLedgerReadout {
    pub combatants: Vec<CombatantActionResourceReadout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResourceSpendDecisionKind {
    Spent,
    RejectedByMissingCombatant,
    RejectedByMissingResource,
    RejectedByUnavailableResource,
}

impl ActionResourceSpendDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceSpendDecisionKind::Spent => "spent",
            ActionResourceSpendDecisionKind::RejectedByMissingCombatant => {
                "rejectedByMissingCombatant"
            }
            ActionResourceSpendDecisionKind::RejectedByMissingResource => {
                "rejectedByMissingResource"
            }
            ActionResourceSpendDecisionKind::RejectedByUnavailableResource => {
                "rejectedByUnavailableResource"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceSpendReadout {
    pub combatant_id: String,
    pub resource_kind: ActionResourceKind,
    pub accepted: bool,
    pub decision_kind: ActionResourceSpendDecisionKind,
    pub previous_resource: Option<ActionResourceState>,
    pub next_resource: Option<ActionResourceState>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResourceRefreshDecisionKind {
    Refreshed,
    RejectedByMissingCombatant,
    RejectedByMissingResource,
}

impl ActionResourceRefreshDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceRefreshDecisionKind::Refreshed => "refreshed",
            ActionResourceRefreshDecisionKind::RejectedByMissingCombatant => {
                "rejectedByMissingCombatant"
            }
            ActionResourceRefreshDecisionKind::RejectedByMissingResource => {
                "rejectedByMissingResource"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceRefreshReadout {
    pub combatant_id: String,
    pub resource_kind: ActionResourceKind,
    pub accepted: bool,
    pub decision_kind: ActionResourceRefreshDecisionKind,
    pub previous_resource: Option<ActionResourceState>,
    pub next_resource: Option<ActionResourceState>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResourceTransitionKind {
    Spent,
    Refreshed,
}

impl ActionResourceTransitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceTransitionKind::Spent => "spent",
            ActionResourceTransitionKind::Refreshed => "refreshed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceTransitionEntry {
    pub sequence: u32,
    pub transition_kind: ActionResourceTransitionKind,
    pub combatant_id: String,
    pub resource_kind: ActionResourceKind,
    pub previous_resource: ActionResourceState,
    pub next_resource: ActionResourceState,
    pub command_step_id: Option<String>,
    pub command_step_index: Option<u32>,
    pub turn_transition_sequence: Option<u32>,
    pub round_number: Option<u32>,
    pub turn_index: Option<u32>,
    pub current_actor_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierDurationExpirationDecisionKind {
    Expired,
}

impl ModifierDurationExpirationDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ModifierDurationExpirationDecisionKind::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDurationExpirationReadout {
    pub combatant_id: String,
    pub modifier_id: String,
    pub accepted: bool,
    pub decision_kind: ModifierDurationExpirationDecisionKind,
    pub previous_modifier: ActiveModifier,
    pub next_modifier: Option<ActiveModifier>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDurationExpirationEntry {
    pub sequence: u32,
    pub combatant_id: String,
    pub modifier_id: String,
    pub previous_modifier: ActiveModifier,
    pub next_modifier: Option<ActiveModifier>,
    pub turn_transition_sequence: u32,
    pub round_number: u32,
    pub turn_index: u32,
    pub current_actor_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatControlCommandKind {
    ExplicitStart,
    ExplicitEnd,
    AdvanceTurn,
    EndIfConditionMet,
}

impl CombatControlCommandKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatControlCommandKind::ExplicitStart => "explicitStart",
            CombatControlCommandKind::ExplicitEnd => "explicitEnd",
            CombatControlCommandKind::AdvanceTurn => "advanceTurn",
            CombatControlCommandKind::EndIfConditionMet => "endIfConditionMet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatControlDecisionKind {
    Accepted,
    RejectedNoop,
    RejectedByLifecycle,
    RejectedByEmptyTurnOrder,
    RejectedByEndCondition,
}

impl CombatControlDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatControlDecisionKind::Accepted => "accepted",
            CombatControlDecisionKind::RejectedNoop => "rejectedNoop",
            CombatControlDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CombatControlDecisionKind::RejectedByEmptyTurnOrder => "rejectedByEmptyTurnOrder",
            CombatControlDecisionKind::RejectedByEndCondition => "rejectedByEndCondition",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatControlCommandSpec {
    pub kind: CombatControlCommandKind,
}

impl CombatControlCommandSpec {
    pub const fn explicit_start() -> Self {
        Self {
            kind: CombatControlCommandKind::ExplicitStart,
        }
    }

    pub const fn explicit_end() -> Self {
        Self {
            kind: CombatControlCommandKind::ExplicitEnd,
        }
    }

    pub const fn advance_turn() -> Self {
        Self {
            kind: CombatControlCommandKind::AdvanceTurn,
        }
    }

    pub const fn end_if_condition_met() -> Self {
        Self {
            kind: CombatControlCommandKind::EndIfConditionMet,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatControlReadout {
    pub command_kind: CombatControlCommandKind,
    pub accepted: bool,
    pub decision_kind: CombatControlDecisionKind,
    pub previous_lifecycle: CombatLifecycle,
    pub next_lifecycle: CombatLifecycle,
    pub previous_turn_order: CombatTurnOrder,
    pub next_turn_order: CombatTurnOrder,
    pub lifecycle_transition: Option<LifecycleTransitionEntry>,
    pub turn_advance: Option<TurnAdvanceReadout>,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatControlHistoryEntry {
    pub sequence: u32,
    pub command_kind: CombatControlCommandKind,
    pub accepted: bool,
    pub decision_kind: CombatControlDecisionKind,
    pub previous_lifecycle_phase: CombatLifecyclePhase,
    pub next_lifecycle_phase: CombatLifecyclePhase,
    pub previous_round_number: u32,
    pub previous_turn_index: u32,
    pub previous_actor_id: Option<String>,
    pub next_round_number: u32,
    pub next_turn_index: u32,
    pub next_actor_id: Option<String>,
    pub lifecycle_transition_sequence: Option<u32>,
    pub turn_transition_sequence: Option<u32>,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
    pub reason: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMetadata {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDiagnosticSeverity {
    Error,
    Warning,
}

impl ContentDiagnosticSeverity {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDiagnosticSeverity::Error => "error",
            ContentDiagnosticSeverity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDiagnosticCode {
    EmptyRulesetId,
    DuplicateRulesetId,
    SelectedRulesetMissingFromCatalog,
    EmptyAbilityId,
    DuplicateAbilityId,
    EmptyEntityId,
    DuplicateEntityId,
    EmptyActionId,
    DuplicateActionId,
    EmptyClassId,
    DuplicateClassId,
    EmptyStatDefinitionId,
    DuplicateStatDefinitionId,
    EmptyModifierId,
    DuplicateModifierId,
    EmptyItemId,
    DuplicateItemId,
    SelectedAbilityMissingFromCatalog,
    SelectedActionMissingFromCatalog,
    SelectedClassMissingFromCatalog,
    SelectedItemMissingFromCatalog,
    MissingCombatantEntity,
    MissingActionAbility,
    MissingActionActor,
    MissingActionTarget,
    VisibleTargetOutsideTargetIds,
    MissingAttackModifierStat,
    MissingTargetDefense,
    MissingCombatantClass,
    MissingCombatantStatDefinition,
    MissingHitModifierDefinition,
    MissingModifierStatAdjustmentTarget,
    MissingActiveModifierDefinition,
    MissingEquippedItem,
}

impl ContentDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDiagnosticCode::EmptyRulesetId => "emptyRulesetId",
            ContentDiagnosticCode::DuplicateRulesetId => "duplicateRulesetId",
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog => {
                "selectedRulesetMissingFromCatalog"
            }
            ContentDiagnosticCode::EmptyAbilityId => "emptyAbilityId",
            ContentDiagnosticCode::DuplicateAbilityId => "duplicateAbilityId",
            ContentDiagnosticCode::EmptyEntityId => "emptyEntityId",
            ContentDiagnosticCode::DuplicateEntityId => "duplicateEntityId",
            ContentDiagnosticCode::EmptyActionId => "emptyActionId",
            ContentDiagnosticCode::DuplicateActionId => "duplicateActionId",
            ContentDiagnosticCode::EmptyClassId => "emptyClassId",
            ContentDiagnosticCode::DuplicateClassId => "duplicateClassId",
            ContentDiagnosticCode::EmptyStatDefinitionId => "emptyStatDefinitionId",
            ContentDiagnosticCode::DuplicateStatDefinitionId => "duplicateStatDefinitionId",
            ContentDiagnosticCode::EmptyModifierId => "emptyModifierId",
            ContentDiagnosticCode::DuplicateModifierId => "duplicateModifierId",
            ContentDiagnosticCode::EmptyItemId => "emptyItemId",
            ContentDiagnosticCode::DuplicateItemId => "duplicateItemId",
            ContentDiagnosticCode::SelectedAbilityMissingFromCatalog => {
                "selectedAbilityMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedActionMissingFromCatalog => {
                "selectedActionMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedClassMissingFromCatalog => {
                "selectedClassMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedItemMissingFromCatalog => {
                "selectedItemMissingFromCatalog"
            }
            ContentDiagnosticCode::MissingCombatantEntity => "missingCombatantEntity",
            ContentDiagnosticCode::MissingActionAbility => "missingActionAbility",
            ContentDiagnosticCode::MissingActionActor => "missingActionActor",
            ContentDiagnosticCode::MissingActionTarget => "missingActionTarget",
            ContentDiagnosticCode::VisibleTargetOutsideTargetIds => "visibleTargetOutsideTargetIds",
            ContentDiagnosticCode::MissingAttackModifierStat => "missingAttackModifierStat",
            ContentDiagnosticCode::MissingTargetDefense => "missingTargetDefense",
            ContentDiagnosticCode::MissingCombatantClass => "missingCombatantClass",
            ContentDiagnosticCode::MissingCombatantStatDefinition => {
                "missingCombatantStatDefinition"
            }
            ContentDiagnosticCode::MissingHitModifierDefinition => "missingHitModifierDefinition",
            ContentDiagnosticCode::MissingModifierStatAdjustmentTarget => {
                "missingModifierStatAdjustmentTarget"
            }
            ContentDiagnosticCode::MissingActiveModifierDefinition => {
                "missingActiveModifierDefinition"
            }
            ContentDiagnosticCode::MissingEquippedItem => "missingEquippedItem",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDiagnostic {
    pub severity: ContentDiagnosticSeverity,
    pub code: ContentDiagnosticCode,
    pub content_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationReport {
    pub accepted: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<ContentDiagnostic>,
}

impl ContentValidationReport {
    pub fn from_diagnostics(diagnostics: Vec<ContentDiagnostic>) -> Self {
        let error_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ContentDiagnosticSeverity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ContentDiagnosticSeverity::Warning)
            .count();

        Self {
            accepted: error_count == 0,
            error_count,
            warning_count,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<GridCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCell {
    pub position: GridPosition,
    pub terrain_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Ally,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedValue {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedNumber {
    pub id: String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatDefinitionKind {
    Base,
    Derived,
}

impl StatDefinitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            StatDefinitionKind::Base => "base",
            StatDefinitionKind::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatDefinition {
    pub id: String,
    pub label: String,
    pub kind: StatDefinitionKind,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBlock {
    pub base_stats: Vec<NamedNumber>,
    pub derived_stats: Vec<NamedNumber>,
}

impl StatBlock {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.base_stats
            .iter()
            .chain(self.derived_stats.iter())
            .find(|stat| stat.id == stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub team: Team,
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub class_ids: Vec<String>,
    pub stats: StatBlock,
    pub defenses: Vec<NamedNumber>,
    pub equipped_item_ids: Vec<String>,
    pub active_modifiers: Vec<ActiveModifier>,
    pub conditions: Vec<String>,
    pub is_actor: bool,
}

impl Combatant {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.stats.stat_by_id(stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchScenario {
    pub metadata: ScenarioMetadata,
    pub rulesets: Vec<RulesetMetadata>,
    pub selected_ruleset_id: String,
    pub grid: Grid,
    pub combatants: Vec<Combatant>,
    pub entities: Vec<EntityDefinition>,
    pub abilities: Vec<AbilityDefinition>,
    pub selected_ability_id: Option<String>,
    pub classes: Vec<ClassDefinition>,
    pub selected_class_id: Option<String>,
    pub stat_definitions: Vec<StatDefinition>,
    pub modifiers: Vec<ModifierDefinition>,
    pub items: Vec<ItemDefinition>,
    pub selected_item_id: Option<String>,
    pub actions: Vec<ActionDefinition>,
    pub selected_action: ActionDefinition,
}

impl RulebenchScenario {
    pub fn ruleset_by_id(&self, ruleset_id: &str) -> Option<&RulesetMetadata> {
        self.rulesets
            .iter()
            .find(|ruleset| ruleset.id == ruleset_id)
    }

    pub fn selected_ruleset(&self) -> Option<&RulesetMetadata> {
        self.ruleset_by_id(&self.selected_ruleset_id)
    }

    pub fn entity_by_id(&self, entity_id: &str) -> Option<&EntityDefinition> {
        self.entities.iter().find(|entity| entity.id == entity_id)
    }

    pub fn ability_by_id(&self, ability_id: &str) -> Option<&AbilityDefinition> {
        self.abilities
            .iter()
            .find(|ability| ability.id == ability_id)
    }

    pub fn action_by_id(&self, action_id: &str) -> Option<&ActionDefinition> {
        self.actions.iter().find(|action| action.id == action_id)
    }

    pub fn class_by_id(&self, class_id: &str) -> Option<&ClassDefinition> {
        self.classes.iter().find(|class| class.id == class_id)
    }

    pub fn item_by_id(&self, item_id: &str) -> Option<&ItemDefinition> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn modifier_by_id(&self, modifier_id: &str) -> Option<&ModifierDefinition> {
        self.modifiers
            .iter()
            .find(|modifier| modifier.id == modifier_id)
    }

    pub fn stat_definition_by_id(&self, stat_id: &str) -> Option<&StatDefinition> {
        self.stat_definitions
            .iter()
            .find(|definition| definition.id == stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseActionIntent {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
}

impl UseActionIntent {
    pub fn new(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id: target_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    pub id: String,
    pub ability_id: String,
    pub name: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub range: u32,
    pub line_of_sight_required: bool,
    pub visible_target_ids: Vec<String>,
    pub attack: AttackSpec,
    pub hit: HitEffect,
    pub action_text: String,
    pub effect_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityDefinitionKind {
    Ability,
    Spell,
}

impl AbilityDefinitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            AbilityDefinitionKind::Ability => "ability",
            AbilityDefinitionKind::Spell => "spell",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityDefinition {
    pub id: String,
    pub name: String,
    pub kind: AbilityDefinitionKind,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDefinition {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub default_tenure: ModifierTenure,
    pub stat_adjustments: Vec<ModifierStatAdjustment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierStatAdjustment {
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantModifierStatAdjustmentReadout {
    pub combatant_id: String,
    pub contributions: Vec<ModifierStatAdjustmentContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantEffectiveStatReadout {
    pub combatant_id: String,
    pub stats: Vec<EffectiveStatReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveStatReadout {
    pub stat_id: String,
    pub stat_label: String,
    pub base_value: i32,
    pub total_modifier_delta: i32,
    pub effective_value: i32,
    pub contributions: Vec<ModifierStatAdjustmentContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierStatAdjustmentContribution {
    pub modifier_id: String,
    pub modifier_label: String,
    pub tenure: ModifierTenure,
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackSpec {
    pub modifier: i32,
    pub modifier_stat_id: String,
    pub defense_id: String,
    pub defense_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitEffect {
    pub damage_bonus: i32,
    pub damage_type: String,
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
    pub operations: Vec<HitEffectOperation>,
}

impl HitEffect {
    pub fn damage_operation(&self) -> Option<&DamageEffectOperation> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::Damage(damage) => Some(damage),
                HitEffectOperation::ApplyModifier(_) => None,
            })
    }

    pub fn modifier_operation(&self) -> Option<&ModifierEffectOperation> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::Damage(_) => None,
                HitEffectOperation::ApplyModifier(modifier) => Some(modifier),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitEffectOperation {
    Damage(DamageEffectOperation),
    ApplyModifier(ModifierEffectOperation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageEffectOperation {
    pub damage_bonus: i32,
    pub damage_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierEffectOperation {
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierTenure {
    Temporary,
    Permanent,
}

impl ModifierTenure {
    pub const fn code(self) -> &'static str {
        match self {
            ModifierTenure::Temporary => "temporary",
            ModifierTenure::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveModifier {
    pub modifier_id: String,
    pub label: String,
    pub duration: String,
    pub tenure: ModifierTenure,
}

impl ActiveModifier {
    pub fn temporary(
        modifier_id: impl Into<String>,
        label: impl Into<String>,
        duration: impl Into<String>,
    ) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            label: label.into(),
            duration: duration.into(),
            tenure: ModifierTenure::Temporary,
        }
    }

    pub fn permanent(modifier_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            label: label.into(),
            duration: "permanent".to_string(),
            tenure: ModifierTenure::Permanent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidTarget,
    TargetLegalityFailed,
    TargetOutOfRange,
    TargetNotVisible,
    MissingAttackRoll,
    MissingDamageRoll,
}

impl RulebenchRejection {
    pub const fn code(self) -> &'static str {
        match self {
            RulebenchRejection::EmptyActorId => "emptyActorId",
            RulebenchRejection::EmptyActionId => "emptyActionId",
            RulebenchRejection::EmptyTargetId => "emptyTargetId",
            RulebenchRejection::InvalidActor => "invalidActor",
            RulebenchRejection::InvalidAction => "invalidAction",
            RulebenchRejection::InvalidTarget => "invalidTarget",
            RulebenchRejection::TargetLegalityFailed => "targetLegalityFailed",
            RulebenchRejection::TargetOutOfRange => "targetOutOfRange",
            RulebenchRejection::TargetNotVisible => "targetNotVisible",
            RulebenchRejection::MissingAttackRoll => "missingAttackRoll",
            RulebenchRejection::MissingDamageRoll => "missingDamageRoll",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    pub sequence: u32,
    pub phase: TracePhase,
    pub status: TraceStatus,
    pub message: String,
    pub detail: String,
}

impl TraceEntry {
    pub fn new(
        sequence: u32,
        phase: TracePhase,
        status: TraceStatus,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            sequence,
            phase,
            status,
            message: message.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracePhase {
    Proposal,
    Validation,
    Resolution,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceStatus {
    Accepted,
    Rejected,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetLegality {
    pub target_id: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackOutcome {
    Hit,
    Miss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackRollResult {
    pub roll: i32,
    pub modifier: i32,
    pub total: i32,
    pub defense_id: String,
    pub defense_value: i32,
    pub outcome: AttackOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageOutcome {
    pub target_id: String,
    pub damage_type: String,
    pub amount: i32,
    pub before: BoundedValue,
    pub after: BoundedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierOutcome {
    pub target_id: String,
    pub modifier_id: String,
    pub label: String,
    pub duration: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    IntentShapeAccepted {
        actor_id: String,
        action_id: String,
        target_id: String,
    },
    ActionUsed {
        actor_id: String,
        action_id: String,
        target_id: String,
    },
    AttackRolled {
        actor_id: String,
        target_id: String,
        total: i32,
        defense_id: String,
        defense_value: i32,
        outcome: AttackOutcome,
    },
    DamageApplied {
        target_id: String,
        amount: i32,
        damage_type: String,
    },
    ModifierApplied {
        target_id: String,
        modifier_id: String,
        duration: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalCombatantState {
    pub id: String,
    pub name: String,
    pub hit_points: BoundedValue,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProjection {
    pub summary: String,
    pub combatants: Vec<FinalCombatantState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateFingerprint {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollRequestKind {
    AttackRoll,
    DamageRoll,
}

impl RollRequestKind {
    pub const fn code(self) -> &'static str {
        match self {
            RollRequestKind::AttackRoll => "attackRoll",
            RollRequestKind::DamageRoll => "damageRoll",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollConsumptionEntry {
    pub sequence: u32,
    pub request_kind: RollRequestKind,
    pub supplied_value: Option<i32>,
    pub consumed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchReceipt {
    pub accepted: bool,
    pub authority_surface: &'static str,
    pub intent: UseActionIntent,
    pub rejection: Option<RulebenchRejection>,
    pub target_legality: Option<TargetLegality>,
    pub attack_roll: Option<AttackRollResult>,
    pub damage: Option<DamageOutcome>,
    pub modifier: Option<ModifierOutcome>,
    pub roll_consumption: Vec<RollConsumptionEntry>,
    pub events: Vec<DomainEvent>,
    pub trace: Vec<TraceEntry>,
    pub projection: Option<ScenarioProjection>,
}
