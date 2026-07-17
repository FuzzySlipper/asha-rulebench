/// Combat action-resource state and readbacks.
use super::ActiveModifier;
pub use rulebench_ruleset::{
    ActionResourceCost, ActionResourceKind, ActionResourcePool, ActionResourceRefreshPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceState {
    pub resource_id: String,
    pub source_id: String,
    pub kind: ActionResourceKind,
    pub current: i32,
    pub max: i32,
    pub available: bool,
    pub refresh_policy: ActionResourceRefreshPolicy,
    pub remaining_refresh_turns: Option<u32>,
}

impl ActionResourceState {
    pub fn new(kind: ActionResourceKind, current: i32, max: i32) -> Self {
        let resource_id = match kind {
            ActionResourceKind::StandardAction => "standard-action",
            ActionResourceKind::SpellSlot => "spell-slot",
            ActionResourceKind::Charge => "charge",
            ActionResourceKind::Cooldown => "cooldown",
        };
        Self {
            resource_id: resource_id.to_string(),
            source_id: "base".to_string(),
            kind,
            current,
            max,
            available: current > 0,
            refresh_policy: match kind {
                ActionResourceKind::StandardAction => ActionResourceRefreshPolicy::TurnStart,
                ActionResourceKind::SpellSlot
                | ActionResourceKind::Charge
                | ActionResourceKind::Cooldown => ActionResourceRefreshPolicy::Never,
            },
            remaining_refresh_turns: None,
        }
    }

    pub fn from_pool(pool: &ActionResourcePool) -> Self {
        Self::from_pool_with_source(pool, "base")
    }

    pub fn from_pool_with_source(pool: &ActionResourcePool, source_id: impl Into<String>) -> Self {
        let max = i32::try_from(pool.maximum).unwrap_or_default();
        let current = i32::try_from(pool.initial).unwrap_or_default();
        Self {
            resource_id: pool.id.clone(),
            source_id: source_id.into(),
            kind: pool.kind,
            current,
            max,
            available: current > 0,
            refresh_policy: pool.refresh_policy.clone(),
            remaining_refresh_turns: None,
        }
    }

    pub fn standard_action_available() -> Self {
        Self::from_pool(&ActionResourcePool::standard_action())
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
    pub resource_id: String,
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
    CooldownAdvanced,
    RejectedByMissingCombatant,
    RejectedByMissingResource,
}

impl ActionResourceRefreshDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceRefreshDecisionKind::Refreshed => "refreshed",
            ActionResourceRefreshDecisionKind::CooldownAdvanced => "cooldownAdvanced",
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
    pub resource_id: String,
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
    ChangedByEffect,
    Refreshed,
    CooldownAdvanced,
}

impl ActionResourceTransitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResourceTransitionKind::Spent => "spent",
            ActionResourceTransitionKind::ChangedByEffect => "changedByEffect",
            ActionResourceTransitionKind::Refreshed => "refreshed",
            ActionResourceTransitionKind::CooldownAdvanced => "cooldownAdvanced",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResourceTransitionEntry {
    pub sequence: u32,
    pub transition_kind: ActionResourceTransitionKind,
    pub combatant_id: String,
    pub resource_id: String,
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
