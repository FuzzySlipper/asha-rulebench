use super::{
    CombatLifecycle, CombatLifecyclePhase, CombatTurnOrder, LifecycleTransitionEntry,
    StateFingerprint, TurnAdvanceReadout,
};

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
