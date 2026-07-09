use super::StateFingerprint;

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
