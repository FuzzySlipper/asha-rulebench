/// Resolution outcomes, events, and trace records.
use super::BoundedValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidRulesetModules,
    InvalidTarget,
    TargetLegalityFailed,
    TargetOutOfRange,
    TargetNotVisible,
    MissingAttackRoll,
    MissingCheckRoll,
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
            RulebenchRejection::InvalidRulesetModules => "invalidRulesetModules",
            RulebenchRejection::InvalidTarget => "invalidTarget",
            RulebenchRejection::TargetLegalityFailed => "targetLegalityFailed",
            RulebenchRejection::TargetOutOfRange => "targetOutOfRange",
            RulebenchRejection::TargetNotVisible => "targetNotVisible",
            RulebenchRejection::MissingAttackRoll => "missingAttackRoll",
            RulebenchRejection::MissingCheckRoll => "missingCheckRoll",
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
}
