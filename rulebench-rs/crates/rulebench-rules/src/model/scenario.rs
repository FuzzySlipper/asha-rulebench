use super::{
    AbilityDefinition, ActionDefinition, ActiveModifier, BoundedValue, GridPosition,
    ModifierTenure, NamedNumber, RulesetMetadata, ScenarioMetadata, StatBlock, StatDefinition,
    Team,
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
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub class_ids: Vec<String>,
    pub stats: StatBlock,
    pub defenses: Vec<NamedNumber>,
    pub equipped_item_ids: Vec<String>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDefinition {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDefinition {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub default_tenure: ModifierTenure,
    pub stat_adjustments: Vec<ModifierStatAdjustment>,
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
    pub base_value: i32,
    pub total_modifier_delta: i32,
    pub effective_value: i32,
    pub contributions: Vec<ModifierStatAdjustmentContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierStatAdjustmentContribution {
    pub modifier_id: String,
    pub modifier_label: String,
    pub tenure: ModifierTenure,
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}
