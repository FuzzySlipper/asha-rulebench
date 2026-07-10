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
    pub modules: Vec<RuleModuleDeclaration>,
}

impl RulesetMetadata {
    pub fn validate_modules(
        &self,
    ) -> Result<ValidatedRuleModuleRegistry, RuleModuleValidationError> {
        validate_rule_modules(&self.modules)
    }
}

/// A closed identifier for a Rust behavior module selected by a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleModuleId {
    ActionResolution,
    TurnControl,
}

impl RuleModuleId {
    pub const fn code(self) -> &'static str {
        match self {
            RuleModuleId::ActionResolution => "actionResolution",
            RuleModuleId::TurnControl => "turnControl",
        }
    }

    pub const fn supported_version(self) -> &'static str {
        match self {
            RuleModuleId::ActionResolution => "1",
            RuleModuleId::TurnControl => "1",
        }
    }

    pub fn from_code(code: &str) -> Result<Self, RuleModuleValidationError> {
        match code {
            "actionResolution" => Ok(RuleModuleId::ActionResolution),
            "turnControl" => Ok(RuleModuleId::TurnControl),
            _ => Err(RuleModuleValidationError::UnknownModuleCode {
                code: code.to_string(),
            }),
        }
    }
}

/// A versioned declaration of a Rust behavior module used by a ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleModuleDeclaration {
    pub module: RuleModuleId,
    pub version: String,
    pub configuration: RuleModuleConfiguration,
}

impl RuleModuleDeclaration {
    pub fn action_resolution(configuration: ActionResolutionModuleConfiguration) -> Self {
        Self {
            module: RuleModuleId::ActionResolution,
            version: RuleModuleId::ActionResolution
                .supported_version()
                .to_string(),
            configuration: RuleModuleConfiguration::ActionResolution(configuration),
        }
    }

    pub fn turn_control(configuration: TurnControlModuleConfiguration) -> Self {
        Self {
            module: RuleModuleId::TurnControl,
            version: RuleModuleId::TurnControl.supported_version().to_string(),
            configuration: RuleModuleConfiguration::TurnControl(configuration),
        }
    }
}

/// Closed configuration schemas for supported behavior modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleModuleConfiguration {
    ActionResolution(ActionResolutionModuleConfiguration),
    TurnControl(TurnControlModuleConfiguration),
}

impl RuleModuleConfiguration {
    pub const fn module(self: &Self) -> RuleModuleId {
        match self {
            RuleModuleConfiguration::ActionResolution(_) => RuleModuleId::ActionResolution,
            RuleModuleConfiguration::TurnControl(_) => RuleModuleId::TurnControl,
        }
    }
}

/// Static action-resolution options supported by the current authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResolutionModuleConfiguration {
    pub targeting_policy: ActionResolutionTargetingPolicy,
}

impl ActionResolutionModuleConfiguration {
    pub const fn declared_targets_and_line_of_sight() -> Self {
        Self {
            targeting_policy: ActionResolutionTargetingPolicy::DeclaredTargetsAndLineOfSight,
        }
    }
}

/// The closed targeting policy vocabulary for action resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResolutionTargetingPolicy {
    DeclaredTargetsAndLineOfSight,
}

impl ActionResolutionTargetingPolicy {
    pub const fn code(self) -> &'static str {
        match self {
            ActionResolutionTargetingPolicy::DeclaredTargetsAndLineOfSight => {
                "declaredTargetsAndLineOfSight"
            }
        }
    }
}

/// Static turn-control options recognized by the current authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnControlModuleConfiguration {
    pub turn_order_policy: TurnOrderPolicy,
}

impl TurnControlModuleConfiguration {
    pub const fn explicit_turn_order() -> Self {
        Self {
            turn_order_policy: TurnOrderPolicy::Explicit,
        }
    }
}

/// The closed turn-order vocabulary for turn control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnOrderPolicy {
    Explicit,
}

impl TurnOrderPolicy {
    pub const fn code(self) -> &'static str {
        match self {
            TurnOrderPolicy::Explicit => "explicit",
        }
    }
}

/// A validated, static ruleset module registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedRuleModuleRegistry {
    declarations: Vec<ValidatedRuleModuleDeclaration>,
    action_resolution: ActionResolutionModuleConfiguration,
    turn_control: Option<TurnControlModuleConfiguration>,
}

impl ValidatedRuleModuleRegistry {
    pub fn declarations(&self) -> &[ValidatedRuleModuleDeclaration] {
        &self.declarations
    }

    pub const fn action_resolution(&self) -> &ActionResolutionModuleConfiguration {
        &self.action_resolution
    }

    pub const fn turn_control(&self) -> Option<&TurnControlModuleConfiguration> {
        self.turn_control.as_ref()
    }
}

/// A module declaration that passed version and configuration validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedRuleModuleDeclaration {
    pub module: RuleModuleId,
    pub version: String,
}

/// Stable errors emitted while validating ruleset behavior declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleModuleValidationError {
    UnknownModuleCode {
        code: String,
    },
    MissingRequiredModule {
        module: RuleModuleId,
    },
    DuplicateModuleDeclaration {
        module: RuleModuleId,
    },
    IncompatibleModuleVersion {
        module: RuleModuleId,
        expected_version: String,
        actual_version: String,
    },
    ConfigurationDoesNotMatchModule {
        module: RuleModuleId,
        configuration_module: RuleModuleId,
    },
}

impl RuleModuleValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            RuleModuleValidationError::UnknownModuleCode { .. } => "unknownRuleModule",
            RuleModuleValidationError::MissingRequiredModule { .. } => "missingRequiredRuleModule",
            RuleModuleValidationError::DuplicateModuleDeclaration { .. } => {
                "duplicateRuleModuleDeclaration"
            }
            RuleModuleValidationError::IncompatibleModuleVersion { .. } => {
                "incompatibleRuleModuleVersion"
            }
            RuleModuleValidationError::ConfigurationDoesNotMatchModule { .. } => {
                "ruleModuleConfigurationMismatch"
            }
        }
    }
}

/// Validate a ruleset's module declarations into the static registry consumed by authority code.
pub fn validate_rule_modules(
    declarations: &[RuleModuleDeclaration],
) -> Result<ValidatedRuleModuleRegistry, RuleModuleValidationError> {
    let mut validated_declarations = Vec::with_capacity(declarations.len());
    let mut action_resolution = None;
    let mut turn_control = None;

    for declaration in declarations {
        let configuration_module = declaration.configuration.module();
        if declaration.module != configuration_module {
            return Err(RuleModuleValidationError::ConfigurationDoesNotMatchModule {
                module: declaration.module,
                configuration_module,
            });
        }

        let expected_version = declaration.module.supported_version();
        if declaration.version != expected_version {
            return Err(RuleModuleValidationError::IncompatibleModuleVersion {
                module: declaration.module,
                expected_version: expected_version.to_string(),
                actual_version: declaration.version.clone(),
            });
        }

        match (&declaration.module, &declaration.configuration) {
            (RuleModuleId::ActionResolution, RuleModuleConfiguration::ActionResolution(config)) => {
                if action_resolution.replace(config.clone()).is_some() {
                    return Err(RuleModuleValidationError::DuplicateModuleDeclaration {
                        module: RuleModuleId::ActionResolution,
                    });
                }
            }
            (RuleModuleId::TurnControl, RuleModuleConfiguration::TurnControl(config)) => {
                if turn_control.replace(config.clone()).is_some() {
                    return Err(RuleModuleValidationError::DuplicateModuleDeclaration {
                        module: RuleModuleId::TurnControl,
                    });
                }
            }
            _ => {
                return Err(RuleModuleValidationError::ConfigurationDoesNotMatchModule {
                    module: declaration.module,
                    configuration_module,
                });
            }
        }

        validated_declarations.push(ValidatedRuleModuleDeclaration {
            module: declaration.module,
            version: declaration.version.clone(),
        });
    }

    let Some(action_resolution) = action_resolution else {
        return Err(RuleModuleValidationError::MissingRequiredModule {
            module: RuleModuleId::ActionResolution,
        });
    };

    Ok(ValidatedRuleModuleRegistry {
        declarations: validated_declarations,
        action_resolution,
        turn_control,
    })
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
        validate_rule_modules, AbilityDefinitionKind, ActionResolutionModuleConfiguration,
        DamageEffectOperation, HitEffect, HitEffectOperation, ModifierEffectOperation,
        ModifierTenure, RuleModuleConfiguration, RuleModuleDeclaration, RuleModuleId,
        TurnControlModuleConfiguration,
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
        assert_eq!(RuleModuleId::ActionResolution.code(), "actionResolution");
    }

    #[test]
    fn module_registry_accepts_static_supported_declarations() {
        let declarations = vec![
            RuleModuleDeclaration::action_resolution(
                ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
            ),
            RuleModuleDeclaration::turn_control(
                TurnControlModuleConfiguration::explicit_turn_order(),
            ),
        ];

        let registry = validate_rule_modules(&declarations).expect("supported modules validate");

        assert_eq!(registry.declarations().len(), 2);
        assert!(registry.turn_control().is_some());
        assert_eq!(
            registry.action_resolution().targeting_policy.code(),
            "declaredTargetsAndLineOfSight"
        );
    }

    #[test]
    fn module_registry_reports_stable_invalid_declaration_codes() {
        assert_eq!(
            RuleModuleId::from_code("not-installed").unwrap_err().code(),
            "unknownRuleModule"
        );

        let duplicate = RuleModuleDeclaration::action_resolution(
            ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
        );
        assert_eq!(
            validate_rule_modules(&[duplicate.clone(), duplicate])
                .unwrap_err()
                .code(),
            "duplicateRuleModuleDeclaration"
        );

        assert_eq!(
            validate_rule_modules(&[]).unwrap_err().code(),
            "missingRequiredRuleModule"
        );

        let incompatible_version = RuleModuleDeclaration {
            module: RuleModuleId::ActionResolution,
            version: "2".to_string(),
            configuration: RuleModuleConfiguration::ActionResolution(
                ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
            ),
        };
        assert_eq!(
            validate_rule_modules(&[incompatible_version])
                .unwrap_err()
                .code(),
            "incompatibleRuleModuleVersion"
        );

        let mismatched_configuration = RuleModuleDeclaration {
            module: RuleModuleId::ActionResolution,
            version: "1".to_string(),
            configuration: RuleModuleConfiguration::TurnControl(
                TurnControlModuleConfiguration::explicit_turn_order(),
            ),
        };
        assert_eq!(
            validate_rule_modules(&[mismatched_configuration])
                .unwrap_err()
                .code(),
            "ruleModuleConfigurationMismatch"
        );
    }
}
