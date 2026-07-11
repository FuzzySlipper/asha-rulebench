/// Authoritative action receipts and projected combat state.
use super::{
    AttackRollResult, BoundedValue, DamageOutcome, DomainEvent, GridPosition, HealingOutcome,
    ModifierOutcome, RulebenchRejection, TargetLegality, TemporaryVitalityOutcome, TraceEntry,
    UseActionIntent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialBoardState {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<SpatialCellState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialCellState {
    pub position: GridPosition,
    pub terrain_tags: Vec<String>,
    pub blocks_movement: bool,
    pub occupant_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalCombatantState {
    pub id: String,
    pub name: String,
    pub hit_points: BoundedValue,
    pub temporary_vitality: i32,
    pub conditions: Vec<String>,
    pub position: GridPosition,
    pub movement_remaining: u32,
    pub movement_maximum: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProjection {
    pub summary: String,
    pub board: SpatialBoardState,
    pub combatants: Vec<FinalCombatantState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollRequestKind {
    AttackRoll,
    SavingThrowRoll,
    ContestedActorRoll,
    ContestedTargetRoll,
    DamageRoll,
}

impl RollRequestKind {
    pub const fn code(self) -> &'static str {
        match self {
            RollRequestKind::AttackRoll => "attackRoll",
            RollRequestKind::SavingThrowRoll => "savingThrowRoll",
            RollRequestKind::ContestedActorRoll => "contestedActorRoll",
            RollRequestKind::ContestedTargetRoll => "contestedTargetRoll",
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
    pub healing: Option<HealingOutcome>,
    pub temporary_vitality: Option<TemporaryVitalityOutcome>,
    pub modifier: Option<ModifierOutcome>,
    pub roll_consumption: Vec<RollConsumptionEntry>,
    pub events: Vec<DomainEvent>,
    pub trace: Vec<TraceEntry>,
    pub projection: Option<ScenarioProjection>,
}
