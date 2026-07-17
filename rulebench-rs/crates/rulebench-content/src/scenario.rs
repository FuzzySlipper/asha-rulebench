use crate::{
    ActiveModifier, AuthoredActionBindingReceipt, ContentPackSetReference, ScenarioMetadata,
    StatBlock, StatDefinition,
};
use rpg_core::{BoundedValue, GridPosition, NamedNumber, Team};
use rpg_ir::{
    AbilityDefinition, ActionDefinition, ActionResourcePool, ModifierTenure, RulesetMetadata,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<GridCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCell {
    pub position: GridPosition,
    pub terrain_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub team: Team,
    pub side_id: String,
    pub initiative: i32,
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub temporary_vitality: i32,
    pub class_inputs: Vec<ClassLevelInput>,
    pub stats: StatBlock,
    pub defenses: Vec<NamedNumber>,
    pub resource_pools: Vec<ActionResourcePool>,
    pub inventory_item_ids: Vec<String>,
    pub equipped_item_ids: Vec<String>,
    pub base_ability_ids: Vec<String>,
    pub active_modifiers: Vec<ActiveModifier>,
    pub conditions: Vec<String>,
    pub is_actor: bool,
}

impl Combatant {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.stats.stat_by_id(stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchScenario {
    pub metadata: ScenarioMetadata,
    pub content_pack_set: Option<ContentPackSetReference>,
    pub authored_action_binding: Option<AuthoredActionBindingReceipt>,
    pub authored_scenario_binding: Option<crate::AuthoredScenarioBindingReceipt>,
    pub rulesets: Vec<RulesetMetadata>,
    pub selected_ruleset_id: String,
    pub grid: Grid,
    pub combatants: Vec<Combatant>,
    pub entities: Vec<EntityDefinition>,
    pub abilities: Vec<AbilityDefinition>,
    pub selected_ability_id: Option<String>,
    pub classes: Vec<ClassDefinition>,
    pub selected_class_id: Option<String>,
    pub stat_definitions: Vec<StatDefinition>,
    pub modifiers: Vec<ModifierDefinition>,
    pub items: Vec<ItemDefinition>,
    pub selected_item_id: Option<String>,
    pub actions: Vec<ActionDefinition>,
    pub selected_action: ActionDefinition,
}

impl RulebenchScenario {
    pub fn ruleset_by_id(&self, ruleset_id: &str) -> Option<&RulesetMetadata> {
        self.rulesets
            .iter()
            .find(|ruleset| ruleset.id == ruleset_id)
    }

    pub fn selected_ruleset(&self) -> Option<&RulesetMetadata> {
        self.ruleset_by_id(&self.selected_ruleset_id)
    }

    pub fn entity_by_id(&self, entity_id: &str) -> Option<&EntityDefinition> {
        self.entities.iter().find(|entity| entity.id == entity_id)
    }

    pub fn ability_by_id(&self, ability_id: &str) -> Option<&AbilityDefinition> {
        self.abilities
            .iter()
            .find(|ability| ability.id == ability_id)
    }

    pub fn action_by_id(&self, action_id: &str) -> Option<&ActionDefinition> {
        self.actions.iter().find(|action| action.id == action_id)
    }

    pub fn class_by_id(&self, class_id: &str) -> Option<&ClassDefinition> {
        self.classes.iter().find(|class| class.id == class_id)
    }

    pub fn item_by_id(&self, item_id: &str) -> Option<&ItemDefinition> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn modifier_by_id(&self, modifier_id: &str) -> Option<&ModifierDefinition> {
        self.modifiers
            .iter()
            .find(|modifier| modifier.id == modifier_id)
    }

    pub fn stat_definition_by_id(&self, stat_id: &str) -> Option<&StatDefinition> {
        self.stat_definitions
            .iter()
            .find(|definition| definition.id == stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseActionIntent {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    pub target_ids: Vec<String>,
    pub target_cell: Option<GridPosition>,
    pub destination_cell: Option<GridPosition>,
    pub observed_origin: Option<GridPosition>,
}

impl UseActionIntent {
    pub fn new(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id: target_id.into(),
            target_ids: Vec::new(),
            target_cell: None,
            destination_cell: None,
            observed_origin: None,
        }
    }

    pub fn for_cell(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        destination_cell: GridPosition,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id: String::new(),
            target_ids: Vec::new(),
            target_cell: None,
            destination_cell: Some(destination_cell),
            observed_origin: None,
        }
    }

    pub fn for_targets(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        target_ids: Vec<String>,
    ) -> Self {
        let target_id = target_ids.first().cloned().unwrap_or_default();
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id,
            target_ids,
            target_cell: None,
            destination_cell: None,
            observed_origin: None,
        }
    }

    pub fn for_area(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        target_cell: GridPosition,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id: String::new(),
            target_ids: Vec::new(),
            target_cell: Some(target_cell),
            destination_cell: None,
            observed_origin: None,
        }
    }

    pub fn with_observed_origin(mut self, observed_origin: GridPosition) -> Self {
        self.observed_origin = Some(observed_origin);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub damage_adjustments: Vec<DamageAdjustment>,
}

/// A typed response an entity applies to a matching damage type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageAdjustment {
    pub damage_type: String,
    pub policy: DamageAdjustmentPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageAdjustmentPolicy {
    Resistance,
    Vulnerability,
    Immunity,
}

impl DamageAdjustmentPolicy {
    pub const fn code(self) -> &'static str {
        match self {
            DamageAdjustmentPolicy::Resistance => "resistance",
            DamageAdjustmentPolicy::Vulnerability => "vulnerability",
            DamageAdjustmentPolicy::Immunity => "immunity",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub equipment_slot: String,
    pub requirements: Vec<StatRequirement>,
    pub granted_modifier_ids: Vec<String>,
    pub granted_ability_ids: Vec<String>,
    pub granted_resource_pools: Vec<ActionResourcePool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatRequirement {
    pub stat_id: String,
    pub minimum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub prerequisites: Vec<StatRequirement>,
    pub level_grants: Vec<ClassLevelGrant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassLevelGrant {
    pub level: u32,
    pub granted_modifier_ids: Vec<String>,
    pub granted_ability_ids: Vec<String>,
    pub granted_resource_pools: Vec<ActionResourcePool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassLevelInput {
    pub class_id: String,
    pub version: String,
    pub level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDefinition {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub default_tenure: ModifierTenure,
    pub stacking_group: String,
    pub stacking_policy: ModifierStackingPolicy,
    pub duration_policy: ModifierDurationPolicy,
    pub stat_adjustments: Vec<ModifierStatAdjustment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierStackingPolicy {
    Stack,
    Replace,
    Refresh,
}

impl ModifierStackingPolicy {
    pub const fn code(self) -> &'static str {
        match self {
            ModifierStackingPolicy::Stack => "stack",
            ModifierStackingPolicy::Replace => "replace",
            ModifierStackingPolicy::Refresh => "refresh",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierDurationPolicy {
    Permanent,
    Turns(u32),
    Rounds(u32),
    UntilEvent(String),
}

impl ModifierDurationPolicy {
    pub fn display_label(&self) -> String {
        match self {
            ModifierDurationPolicy::Permanent => "permanent".to_string(),
            ModifierDurationPolicy::Turns(turns) => format!("{} turns", turns),
            ModifierDurationPolicy::Rounds(rounds) => format!("{} rounds", rounds),
            ModifierDurationPolicy::UntilEvent(event) => format!("until {}", event),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierStatAdjustment {
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantModifierStatAdjustmentReadout {
    pub combatant_id: String,
    pub contributions: Vec<ModifierStatAdjustmentContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantEffectiveStatReadout {
    pub combatant_id: String,
    pub stats: Vec<EffectiveStatReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveStatReadout {
    pub stat_id: String,
    pub stat_label: String,
    pub kind: crate::StatDefinitionKind,
    pub formula: Option<crate::DerivedStatFormula>,
    pub base_value: i32,
    pub total_modifier_delta: i32,
    pub effective_value: i32,
    pub contributions: Vec<ModifierStatAdjustmentContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierStatAdjustmentContribution {
    pub modifier_id: String,
    pub source_id: String,
    pub modifier_label: String,
    pub tenure: ModifierTenure,
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}
