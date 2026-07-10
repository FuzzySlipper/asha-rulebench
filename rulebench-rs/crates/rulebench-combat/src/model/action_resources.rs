/// Combat action-resource state and readbacks.
use super::ActiveModifier;
pub use rulebench_ruleset::{ActionResourceCost, ActionResourceKind};

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
    RejectedByInvalidAmount,
    RejectedByUnavailableResource,
    RejectedByInsufficientResource,
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
            ActionResourceSpendDecisionKind::RejectedByInvalidAmount => "rejectedByInvalidAmount",
            ActionResourceSpendDecisionKind::RejectedByUnavailableResource => {
                "rejectedByUnavailableResource"
            }
            ActionResourceSpendDecisionKind::RejectedByInsufficientResource => {
                "rejectedByInsufficientResource"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceSpendReadout {
    pub combatant_id: String,
    pub resource_kind: ActionResourceKind,
    pub amount: u32,
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
    pub amount: u32,
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
    Advanced,
    Expired,
}

impl ModifierDurationExpirationDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ModifierDurationExpirationDecisionKind::Advanced => "advanced",
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
    pub trigger: ModifierDurationTransitionTrigger,
    pub turn_transition_sequence: Option<u32>,
    pub round_number: Option<u32>,
    pub turn_index: Option<u32>,
    pub current_actor_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierDurationTransitionTrigger {
    TurnBoundary,
    RoundBoundary,
    Event(String),
}
