use rulebench_rules::{ActionResourceKind, ActionResourcePool, ActionResourceRefreshPolicy};
use rulebench_rules::{
    AuthoredScenarioActionGrant, AuthoredScenarioControl, AuthoredScenarioControlMode,
    AuthoredScenarioDefinition, AuthoredScenarioParticipant, ClassDefinition, ClassLevelGrant,
    ClassLevelInput, DerivedStatFormula, Grid, GridCell, ItemDefinition, StatBlock, StatDefinition,
    StatDefinitionKind, StatRequirement, Team,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredClassDefinitionDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<AuthoredStatRequirementDto>,
    #[serde(default)]
    pub level_grants: Vec<AuthoredClassLevelGrantDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredClassLevelGrantDto {
    pub level: u32,
    #[serde(default)]
    pub granted_modifier_ids: Vec<String>,
    #[serde(default)]
    pub granted_ability_ids: Vec<String>,
    #[serde(default)]
    pub granted_resource_pools: Vec<AuthoredActionResourcePoolDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredStatRequirementDto {
    pub stat_id: String,
    pub minimum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredStatDefinitionDto {
    pub id: String,
    pub label: String,
    pub kind: AuthoredStatDefinitionKindDto,
    #[serde(default)]
    pub formula: Option<AuthoredDerivedStatFormulaDto>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredStatDefinitionKindDto {
    Base,
    Derived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AuthoredDerivedStatFormulaDto {
    Constant {
        value: i32,
    },
    StatReference {
        stat_id: String,
    },
    Sum {
        operands: Vec<AuthoredDerivedStatFormulaDto>,
    },
    Product {
        operands: Vec<AuthoredDerivedStatFormulaDto>,
    },
    Difference {
        minuend: Box<AuthoredDerivedStatFormulaDto>,
        subtrahend: Box<AuthoredDerivedStatFormulaDto>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredItemDefinitionDto {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub equipment_slot: String,
    #[serde(default)]
    pub requirements: Vec<AuthoredStatRequirementDto>,
    #[serde(default)]
    pub granted_modifier_ids: Vec<String>,
    #[serde(default)]
    pub granted_ability_ids: Vec<String>,
    #[serde(default)]
    pub granted_resource_pools: Vec<AuthoredActionResourcePoolDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionResourcePoolDto {
    pub id: String,
    pub kind: AuthoredActionResourceKindDto,
    pub initial: u32,
    pub maximum: u32,
    pub refresh_policy: AuthoredActionResourceRefreshPolicyDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredActionResourceKindDto {
    StandardAction,
    SpellSlot,
    Charge,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AuthoredActionResourceRefreshPolicyDto {
    Never,
    CombatStart,
    TurnStart,
    Turns { turns: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioDefinitionDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub ruleset_id: String,
    pub grid: AuthoredGridDto,
    pub participants: Vec<AuthoredScenarioParticipantDto>,
    pub selected_action_id: String,
    pub control: AuthoredScenarioControlDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredGridDto {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<AuthoredGridCellDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredGridCellDto {
    pub position: AuthoredGridPositionDto,
    #[serde(default)]
    pub terrain_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredGridPositionDto {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioParticipantDto {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub team: AuthoredTeamDto,
    pub side_id: String,
    pub initiative: i32,
    pub position: AuthoredGridPositionDto,
    pub hit_points: AuthoredBoundedValueDto,
    #[serde(default)]
    pub temporary_vitality: i32,
    #[serde(default)]
    pub class_inputs: Vec<AuthoredClassLevelInputDto>,
    pub stats: AuthoredStatBlockDto,
    #[serde(default)]
    pub defenses: Vec<AuthoredNamedNumberDto>,
    #[serde(default)]
    pub resource_pools: Vec<AuthoredActionResourcePoolDto>,
    #[serde(default)]
    pub inventory_item_ids: Vec<String>,
    #[serde(default)]
    pub equipped_item_ids: Vec<String>,
    #[serde(default)]
    pub base_ability_ids: Vec<String>,
    pub action_grants: Vec<AuthoredScenarioActionGrantDto>,
    #[serde(default)]
    pub visible_target_ids: Vec<String>,
    pub is_actor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTeamDto {
    Ally,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredBoundedValueDto {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredNamedNumberDto {
    pub id: String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredStatBlockDto {
    pub base_stats: Vec<AuthoredNamedNumberDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredClassLevelInputDto {
    pub class_id: String,
    pub version: String,
    pub level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioActionGrantDto {
    pub action_id: String,
    pub runtime_action_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioControlDto {
    pub mode: AuthoredScenarioControlModeDto,
    #[serde(default)]
    pub automation_policy_id: Option<String>,
    #[serde(default)]
    pub automation_policy_version: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredScenarioControlModeDto {
    Manual,
    Automatic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioBindingReceiptDto {
    pub binding_version: u32,
    pub content_pack_root: crate::ContentPackReferenceDto,
    pub content_pack_references: Vec<crate::ContentPackReferenceDto>,
    pub content_pack_set_fingerprint: crate::ContentFingerprintDto,
    pub scenario_id: String,
    pub participants: Vec<AuthoredScenarioParticipantReceiptDto>,
    pub control: AuthoredScenarioControlDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredScenarioParticipantReceiptDto {
    pub participant_id: String,
    pub archetypes: Vec<AuthoredClassLevelInputDto>,
    pub loadout_item_ids: Vec<String>,
    pub action_grants: Vec<AuthoredScenarioActionGrantDto>,
}

impl AuthoredClassDefinitionDto {
    pub(crate) fn to_authority(&self) -> ClassDefinition {
        ClassDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            prerequisites: self
                .prerequisites
                .iter()
                .map(AuthoredStatRequirementDto::to_authority)
                .collect(),
            level_grants: self
                .level_grants
                .iter()
                .map(AuthoredClassLevelGrantDto::to_authority)
                .collect(),
        }
    }
}

impl AuthoredClassLevelGrantDto {
    fn to_authority(&self) -> ClassLevelGrant {
        ClassLevelGrant {
            level: self.level,
            granted_modifier_ids: self.granted_modifier_ids.clone(),
            granted_ability_ids: self.granted_ability_ids.clone(),
            granted_resource_pools: self
                .granted_resource_pools
                .iter()
                .map(AuthoredActionResourcePoolDto::to_authority)
                .collect(),
        }
    }
}

impl AuthoredStatRequirementDto {
    fn to_authority(&self) -> StatRequirement {
        StatRequirement {
            stat_id: self.stat_id.clone(),
            minimum: self.minimum,
        }
    }
}

impl AuthoredStatDefinitionDto {
    pub(crate) fn to_authority(&self) -> StatDefinition {
        StatDefinition {
            id: self.id.clone(),
            label: self.label.clone(),
            kind: match self.kind {
                AuthoredStatDefinitionKindDto::Base => StatDefinitionKind::Base,
                AuthoredStatDefinitionKindDto::Derived => StatDefinitionKind::Derived,
            },
            formula: self
                .formula
                .as_ref()
                .map(AuthoredDerivedStatFormulaDto::to_authority),
            summary: self.summary.clone(),
        }
    }
}

impl AuthoredDerivedStatFormulaDto {
    fn to_authority(&self) -> DerivedStatFormula {
        match self {
            Self::Constant { value } => DerivedStatFormula::Constant { value: *value },
            Self::StatReference { stat_id } => DerivedStatFormula::StatReference {
                stat_id: stat_id.clone(),
            },
            Self::Sum { operands } => DerivedStatFormula::Sum {
                operands: operands.iter().map(Self::to_authority).collect(),
            },
            Self::Product { operands } => DerivedStatFormula::Product {
                operands: operands.iter().map(Self::to_authority).collect(),
            },
            Self::Difference {
                minuend,
                subtrahend,
            } => DerivedStatFormula::Difference {
                minuend: Box::new(minuend.to_authority()),
                subtrahend: Box::new(subtrahend.to_authority()),
            },
        }
    }
}

impl AuthoredItemDefinitionDto {
    pub(crate) fn to_authority(&self) -> ItemDefinition {
        ItemDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            equipment_slot: self.equipment_slot.clone(),
            requirements: self
                .requirements
                .iter()
                .map(AuthoredStatRequirementDto::to_authority)
                .collect(),
            granted_modifier_ids: self.granted_modifier_ids.clone(),
            granted_ability_ids: self.granted_ability_ids.clone(),
            granted_resource_pools: self
                .granted_resource_pools
                .iter()
                .map(AuthoredActionResourcePoolDto::to_authority)
                .collect(),
        }
    }
}

impl AuthoredActionResourcePoolDto {
    fn to_authority(&self) -> ActionResourcePool {
        ActionResourcePool {
            id: self.id.clone(),
            kind: match self.kind {
                AuthoredActionResourceKindDto::StandardAction => ActionResourceKind::StandardAction,
                AuthoredActionResourceKindDto::SpellSlot => ActionResourceKind::SpellSlot,
                AuthoredActionResourceKindDto::Charge => ActionResourceKind::Charge,
                AuthoredActionResourceKindDto::Cooldown => ActionResourceKind::Cooldown,
            },
            initial: self.initial,
            maximum: self.maximum,
            refresh_policy: match self.refresh_policy {
                AuthoredActionResourceRefreshPolicyDto::Never => ActionResourceRefreshPolicy::Never,
                AuthoredActionResourceRefreshPolicyDto::CombatStart => {
                    ActionResourceRefreshPolicy::CombatStart
                }
                AuthoredActionResourceRefreshPolicyDto::TurnStart => {
                    ActionResourceRefreshPolicy::TurnStart
                }
                AuthoredActionResourceRefreshPolicyDto::Turns { turns } => {
                    ActionResourceRefreshPolicy::Turns(turns)
                }
            },
        }
    }
}

impl AuthoredScenarioDefinitionDto {
    pub(crate) fn to_authority(&self) -> AuthoredScenarioDefinition {
        AuthoredScenarioDefinition {
            id: self.id.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            seed_label: self.seed_label.clone(),
            ruleset_id: self.ruleset_id.clone(),
            grid: self.grid.to_authority(),
            participants: self
                .participants
                .iter()
                .map(AuthoredScenarioParticipantDto::to_authority)
                .collect(),
            selected_action_id: self.selected_action_id.clone(),
            control: self.control.to_authority(),
        }
    }
}

impl AuthoredGridDto {
    fn to_authority(&self) -> Grid {
        Grid {
            width: self.width,
            height: self.height,
            cells: self
                .cells
                .iter()
                .map(|cell| GridCell {
                    position: cell.position.to_authority(),
                    terrain_tags: cell.terrain_tags.clone(),
                })
                .collect(),
        }
    }
}

impl AuthoredGridPositionDto {
    fn to_authority(self) -> rulebench_rules::GridPosition {
        rulebench_rules::GridPosition {
            x: self.x,
            y: self.y,
        }
    }
}

impl AuthoredScenarioParticipantDto {
    fn to_authority(&self) -> AuthoredScenarioParticipant {
        AuthoredScenarioParticipant {
            id: self.id.clone(),
            entity_id: self.entity_id.clone(),
            name: self.name.clone(),
            team: match self.team {
                AuthoredTeamDto::Ally => Team::Ally,
                AuthoredTeamDto::Enemy => Team::Enemy,
            },
            side_id: self.side_id.clone(),
            initiative: self.initiative,
            position: self.position.to_authority(),
            hit_points: rulebench_rules::BoundedValue {
                current: self.hit_points.current,
                max: self.hit_points.max,
            },
            temporary_vitality: self.temporary_vitality,
            class_inputs: self
                .class_inputs
                .iter()
                .map(|input| ClassLevelInput {
                    class_id: input.class_id.clone(),
                    version: input.version.clone(),
                    level: input.level,
                })
                .collect(),
            stats: StatBlock {
                base_stats: self
                    .stats
                    .base_stats
                    .iter()
                    .map(AuthoredNamedNumberDto::to_authority)
                    .collect(),
                derived_stats: Vec::new(),
            },
            defenses: self
                .defenses
                .iter()
                .map(AuthoredNamedNumberDto::to_authority)
                .collect(),
            resource_pools: self
                .resource_pools
                .iter()
                .map(AuthoredActionResourcePoolDto::to_authority)
                .collect(),
            inventory_item_ids: self.inventory_item_ids.clone(),
            equipped_item_ids: self.equipped_item_ids.clone(),
            base_ability_ids: self.base_ability_ids.clone(),
            action_grants: self
                .action_grants
                .iter()
                .map(|grant| AuthoredScenarioActionGrant {
                    action_id: grant.action_id.clone(),
                    runtime_action_id: grant.runtime_action_id.clone(),
                })
                .collect(),
            visible_target_ids: self.visible_target_ids.clone(),
            is_actor: self.is_actor,
        }
    }
}

impl AuthoredNamedNumberDto {
    fn to_authority(&self) -> rulebench_rules::NamedNumber {
        rulebench_rules::NamedNumber {
            id: self.id.clone(),
            label: self.label.clone(),
            value: self.value,
        }
    }
}

impl AuthoredScenarioControlDto {
    fn to_authority(&self) -> AuthoredScenarioControl {
        AuthoredScenarioControl {
            mode: match self.mode {
                AuthoredScenarioControlModeDto::Manual => AuthoredScenarioControlMode::Manual,
                AuthoredScenarioControlModeDto::Automatic => AuthoredScenarioControlMode::Automatic,
            },
            automation_policy_id: self.automation_policy_id.clone(),
            automation_policy_version: self.automation_policy_version.clone(),
        }
    }
}

impl From<&rulebench_rules::AuthoredScenarioBindingReceipt> for AuthoredScenarioBindingReceiptDto {
    fn from(value: &rulebench_rules::AuthoredScenarioBindingReceipt) -> Self {
        Self {
            binding_version: value.binding_version,
            content_pack_root: crate::ContentPackReferenceDto::from(&value.content_pack_set.root),
            content_pack_references: value
                .content_pack_set
                .packs
                .iter()
                .map(crate::ContentPackReferenceDto::from)
                .collect(),
            content_pack_set_fingerprint: crate::ContentFingerprintDto {
                algorithm: value.content_pack_set.fingerprint.algorithm.clone(),
                value: value.content_pack_set.fingerprint.value.clone(),
            },
            scenario_id: value.scenario_id.clone(),
            participants: value
                .participants
                .iter()
                .map(|participant| AuthoredScenarioParticipantReceiptDto {
                    participant_id: participant.participant_id.clone(),
                    archetypes: participant
                        .archetypes
                        .iter()
                        .map(|archetype| AuthoredClassLevelInputDto {
                            class_id: archetype.class_id.clone(),
                            version: archetype.version.clone(),
                            level: archetype.level,
                        })
                        .collect(),
                    loadout_item_ids: participant.loadout_item_ids.clone(),
                    action_grants: participant
                        .action_grants
                        .iter()
                        .map(|grant| AuthoredScenarioActionGrantDto {
                            action_id: grant.action_id.clone(),
                            runtime_action_id: grant.runtime_action_id.clone(),
                        })
                        .collect(),
                })
                .collect(),
            control: AuthoredScenarioControlDto {
                mode: match value.control.mode {
                    AuthoredScenarioControlMode::Manual => AuthoredScenarioControlModeDto::Manual,
                    AuthoredScenarioControlMode::Automatic => {
                        AuthoredScenarioControlModeDto::Automatic
                    }
                },
                automation_policy_id: value.control.automation_policy_id.clone(),
                automation_policy_version: value.control.automation_policy_version,
            },
        }
    }
}
