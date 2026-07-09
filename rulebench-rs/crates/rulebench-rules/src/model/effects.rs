use super::BoundedValue;
use rulebench_ruleset::ModifierTenure;

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
