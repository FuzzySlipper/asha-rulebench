//! Ruleset declarations and operation vocabulary.
//!
//! This crate owns the declarative vocabulary that selects and configures Rust
//! authority behavior. It does not own content catalogs, combat state, or
//! effect application.

/// Identity and compatibility metadata for an authored ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
}

/// The authored category of an ability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityDefinitionKind {
    Ability,
    Spell,
}

impl AbilityDefinitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            AbilityDefinitionKind::Ability => "ability",
            AbilityDefinitionKind::Spell => "spell",
        }
    }
}

/// A named ability or spell declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityDefinition {
    pub id: String,
    pub name: String,
    pub kind: AbilityDefinitionKind,
    pub summary: String,
    pub tags: Vec<String>,
}

/// A declared action with targeting, check, and effect configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    pub id: String,
    pub ability_id: String,
    pub name: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub range: u32,
    pub line_of_sight_required: bool,
    pub visible_target_ids: Vec<String>,
    pub attack: AttackSpec,
    pub hit: HitEffect,
    pub action_text: String,
    pub effect_text: String,
}

/// The attack/check inputs declared by an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackSpec {
    pub modifier: i32,
    pub modifier_stat_id: String,
    pub defense_id: String,
    pub defense_label: String,
}

/// The operation set applied after an accepted hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitEffect {
    pub damage_bonus: i32,
    pub damage_type: String,
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
    pub operations: Vec<HitEffectOperation>,
}

impl HitEffect {
    pub fn damage_operation(&self) -> Option<&DamageEffectOperation> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::Damage(damage) => Some(damage),
                HitEffectOperation::ApplyModifier(_) => None,
            })
    }

    pub fn modifier_operation(&self) -> Option<&ModifierEffectOperation> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::Damage(_) => None,
                HitEffectOperation::ApplyModifier(modifier) => Some(modifier),
            })
    }
}

/// A typed effect operation selected by an action declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitEffectOperation {
    Damage(DamageEffectOperation),
    ApplyModifier(ModifierEffectOperation),
}

/// A damage operation declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageEffectOperation {
    pub damage_bonus: i32,
    pub damage_type: String,
}

/// A modifier application operation declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierEffectOperation {
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
}

/// Whether a modifier declaration survives beyond a temporary combat window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierTenure {
    Temporary,
    Permanent,
}

impl ModifierTenure {
    pub const fn code(self) -> &'static str {
        match self {
            ModifierTenure::Temporary => "temporary",
            ModifierTenure::Permanent => "permanent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AbilityDefinitionKind, DamageEffectOperation, HitEffect, HitEffectOperation,
        ModifierEffectOperation, ModifierTenure,
    };

    #[test]
    fn hit_effect_operation_accessors_preserve_typed_operation_selection() {
        let hit = HitEffect {
            damage_bonus: 2,
            damage_type: "psychic".to_string(),
            modifier_id: "rattled".to_string(),
            modifier_label: "Rattled".to_string(),
            modifier_duration: "until end of next turn".to_string(),
            operations: vec![
                HitEffectOperation::Damage(DamageEffectOperation {
                    damage_bonus: 2,
                    damage_type: "psychic".to_string(),
                }),
                HitEffectOperation::ApplyModifier(ModifierEffectOperation {
                    modifier_id: "rattled".to_string(),
                    modifier_label: "Rattled".to_string(),
                    modifier_duration: "until end of next turn".to_string(),
                }),
            ],
        };

        assert_eq!(
            hit.damage_operation()
                .map(|operation| operation.damage_bonus),
            Some(2)
        );
        assert_eq!(
            hit.modifier_operation()
                .map(|operation| operation.modifier_id.as_str()),
            Some("rattled")
        );
    }

    #[test]
    fn ruleset_enum_codes_are_stable() {
        assert_eq!(AbilityDefinitionKind::Spell.code(), "spell");
        assert_eq!(ModifierTenure::Permanent.code(), "permanent");
    }
}
