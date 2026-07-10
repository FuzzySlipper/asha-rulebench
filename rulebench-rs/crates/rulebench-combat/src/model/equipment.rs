/// Inventory, equipment, and deterministic equipment transition readbacks.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantEquipmentReadout {
    pub combatant_id: String,
    pub inventory_item_ids: Vec<String>,
    pub equipped_item_ids: Vec<String>,
    pub available_ability_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentLedgerReadout {
    pub combatants: Vec<CombatantEquipmentReadout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentCommandKind {
    Equip,
    Unequip,
}

impl EquipmentCommandKind {
    pub const fn code(self) -> &'static str {
        match self {
            EquipmentCommandKind::Equip => "equip",
            EquipmentCommandKind::Unequip => "unequip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentCommandSpec {
    pub kind: EquipmentCommandKind,
    pub combatant_id: String,
    pub item_id: String,
}

impl EquipmentCommandSpec {
    pub fn equip(combatant_id: impl Into<String>, item_id: impl Into<String>) -> Self {
        Self {
            kind: EquipmentCommandKind::Equip,
            combatant_id: combatant_id.into(),
            item_id: item_id.into(),
        }
    }

    pub fn unequip(combatant_id: impl Into<String>, item_id: impl Into<String>) -> Self {
        Self {
            kind: EquipmentCommandKind::Unequip,
            combatant_id: combatant_id.into(),
            item_id: item_id.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentDecisionKind {
    Accepted,
    RejectedByLifecycle,
    RejectedByReactionWindow,
    RejectedByCombatant,
    RejectedByItem,
    RejectedByOwnership,
    RejectedByEquippedState,
    RejectedBySlotConflict,
    RejectedByRequirement,
    RejectedByResourceConflict,
}

impl EquipmentDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            EquipmentDecisionKind::Accepted => "accepted",
            EquipmentDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            EquipmentDecisionKind::RejectedByReactionWindow => "rejectedByReactionWindow",
            EquipmentDecisionKind::RejectedByCombatant => "rejectedByCombatant",
            EquipmentDecisionKind::RejectedByItem => "rejectedByItem",
            EquipmentDecisionKind::RejectedByOwnership => "rejectedByOwnership",
            EquipmentDecisionKind::RejectedByEquippedState => "rejectedByEquippedState",
            EquipmentDecisionKind::RejectedBySlotConflict => "rejectedBySlotConflict",
            EquipmentDecisionKind::RejectedByRequirement => "rejectedByRequirement",
            EquipmentDecisionKind::RejectedByResourceConflict => "rejectedByResourceConflict",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentCommandReadout {
    pub command: EquipmentCommandSpec,
    pub accepted: bool,
    pub decision_kind: EquipmentDecisionKind,
    pub previous_equipment: Option<CombatantEquipmentReadout>,
    pub next_equipment: Option<CombatantEquipmentReadout>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentTransitionEntry {
    pub sequence: u32,
    pub transition_kind: EquipmentCommandKind,
    pub combatant_id: String,
    pub item_id: String,
    pub equipment_slot: String,
    pub granted_modifier_ids: Vec<String>,
    pub granted_ability_ids: Vec<String>,
    pub granted_resource_ids: Vec<String>,
    pub previous_equipment: CombatantEquipmentReadout,
    pub next_equipment: CombatantEquipmentReadout,
    pub reason: String,
}
