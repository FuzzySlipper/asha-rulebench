use super::{
    AttackRollResult, BoundedValue, DamageOutcome, DomainEvent, ModifierOutcome,
    RulebenchRejection, TargetLegality, TraceEntry, UseActionIntent,
};

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
