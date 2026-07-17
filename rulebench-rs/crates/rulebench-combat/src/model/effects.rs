/// Resolution outcomes, events, and trace records.
use super::{BoundedValue, ModifierDurationPolicy, ModifierStackingPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidRulesetModules,
    InvalidTarget,
    DuplicateTarget,
    TargetLimitExceeded,
    TargetDefeated,
    AreaTargetMissing,
    AreaOutOfBounds,
    AreaOutOfRange,
    TargetLegalityFailed,
    TargetOutOfRange,
    TargetNotVisible,
    MissingAttackRoll,
    MissingCheckRoll,
    MissingDamageRoll,
    InvalidRollValue,
    MovementDestinationMissing,
    MovementActorDefeated,
    MovementOutOfBounds,
    MovementDestinationOccupied,
    MovementDestinationBlocked,
    MovementStaleDestination,
    MovementOutOfRange,
    MovementBudgetExhausted,
    EffectMovementOutOfBounds,
    EffectMovementDestinationOccupied,
    EffectMovementDestinationBlocked,
    EffectResourceMissing,
    EffectResourceOutOfBounds,
}

impl RulebenchRejection {
    pub const fn code(self) -> &'static str {
        match self {
            RulebenchRejection::EmptyActorId => "emptyActorId",
            RulebenchRejection::EmptyActionId => "emptyActionId",
            RulebenchRejection::EmptyTargetId => "emptyTargetId",
            RulebenchRejection::InvalidActor => "invalidActor",
            RulebenchRejection::InvalidAction => "invalidAction",
            RulebenchRejection::InvalidRulesetModules => "invalidRulesetModules",
            RulebenchRejection::InvalidTarget => "invalidTarget",
            RulebenchRejection::DuplicateTarget => "duplicateTarget",
            RulebenchRejection::TargetLimitExceeded => "targetLimitExceeded",
            RulebenchRejection::TargetDefeated => "targetDefeated",
            RulebenchRejection::AreaTargetMissing => "areaTargetMissing",
            RulebenchRejection::AreaOutOfBounds => "areaOutOfBounds",
            RulebenchRejection::AreaOutOfRange => "areaOutOfRange",
            RulebenchRejection::TargetLegalityFailed => "targetLegalityFailed",
            RulebenchRejection::TargetOutOfRange => "targetOutOfRange",
            RulebenchRejection::TargetNotVisible => "targetNotVisible",
            RulebenchRejection::MissingAttackRoll => "missingAttackRoll",
            RulebenchRejection::MissingCheckRoll => "missingCheckRoll",
            RulebenchRejection::MissingDamageRoll => "missingDamageRoll",
            RulebenchRejection::InvalidRollValue => "invalidRollValue",
            RulebenchRejection::MovementDestinationMissing => "movementDestinationMissing",
            RulebenchRejection::MovementActorDefeated => "movementActorDefeated",
            RulebenchRejection::MovementOutOfBounds => "movementOutOfBounds",
            RulebenchRejection::MovementDestinationOccupied => "movementDestinationOccupied",
            RulebenchRejection::MovementDestinationBlocked => "movementDestinationBlocked",
            RulebenchRejection::MovementStaleDestination => "movementStaleDestination",
            RulebenchRejection::MovementOutOfRange => "movementOutOfRange",
            RulebenchRejection::MovementBudgetExhausted => "movementBudgetExhausted",
            RulebenchRejection::EffectMovementOutOfBounds => "effectMovementOutOfBounds",
            RulebenchRejection::EffectMovementDestinationOccupied => {
                "effectMovementDestinationOccupied"
            }
            RulebenchRejection::EffectMovementDestinationBlocked => {
                "effectMovementDestinationBlocked"
            }
            RulebenchRejection::EffectResourceMissing => "effectResourceMissing",
            RulebenchRejection::EffectResourceOutOfBounds => "effectResourceOutOfBounds",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavingThrowOutcome {
    Saved,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestedCheckOutcome {
    ActorWins,
    TargetWins,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageOutcome {
    pub target_id: String,
    pub damage_type: String,
    pub requested_amount: i32,
    pub amount: i32,
    pub temporary_vitality_absorbed: i32,
    pub temporary_vitality_after: i32,
    pub before: BoundedValue,
    pub after: BoundedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealingOutcome {
    pub target_id: String,
    pub healing_type: String,
    pub requested_amount: i32,
    pub amount: i32,
    pub before: BoundedValue,
    pub after: BoundedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryVitalityOutcome {
    pub target_id: String,
    pub requested_amount: i32,
    pub before: i32,
    pub after: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierOutcome {
    pub target_id: String,
    pub modifier_id: String,
    pub source_id: String,
    pub label: String,
    pub duration: String,
    pub stacking_group: String,
    pub stacking_policy: ModifierStackingPolicy,
    pub duration_policy: ModifierDurationPolicy,
    pub remaining_turns: Option<u32>,
    pub remaining_rounds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectMovementOutcome {
    pub target_id: String,
    pub movement_kind: rpg_ir::MovementKind,
    pub from: super::GridPosition,
    pub to: super::GridPosition,
    pub distance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceChangeOutcome {
    pub target_id: String,
    pub resource_id: String,
    pub requested_delta: i32,
    pub before: i32,
    pub after: i32,
    pub maximum: i32,
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
    SavingThrowResolved {
        actor_id: String,
        target_id: String,
        total: i32,
        difficulty_class: i32,
        outcome: SavingThrowOutcome,
    },
    ContestedCheckResolved {
        actor_id: String,
        target_id: String,
        actor_total: i32,
        target_total: i32,
        outcome: ContestedCheckOutcome,
    },
    DamageApplied {
        target_id: String,
        amount: i32,
        damage_type: String,
    },
    HealingApplied {
        target_id: String,
        amount: i32,
        healing_type: String,
    },
    TemporaryVitalityGranted {
        target_id: String,
        amount: i32,
    },
    ModifierApplied {
        target_id: String,
        modifier_id: String,
        duration: String,
    },
    EffectMovementApplied {
        target_id: String,
        movement_kind: rpg_ir::MovementKind,
        from: super::GridPosition,
        to: super::GridPosition,
    },
    ResourceChanged {
        target_id: String,
        resource_id: String,
        delta: i32,
        before: i32,
        after: i32,
    },
    PositionChanged {
        actor_id: String,
        from: super::GridPosition,
        to: super::GridPosition,
    },
    MovementSpent {
        actor_id: String,
        amount: u32,
        remaining: u32,
    },
}
