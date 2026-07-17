use rpg_ir::{
    ActionResolutionModuleConfiguration, ActionResolutionTargetingPolicy, CheckHandlerKind,
    CombatEndPolicy, RuleModuleConfiguration, RuleModuleDeclaration, RuleModuleId,
    RuleModuleValidationError, RulesetMetadata, TurnControlModuleConfiguration, TurnOrderPolicy,
};
use serde::{Deserialize, Serialize};

/// Stable wire form of a ruleset definition authored outside Rust authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RulesetDefinitionDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub modules: Vec<RuleModuleDeclarationDto>,
}

/// Stable wire form of one selected Rust behavior module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleModuleDeclarationDto {
    pub module: String,
    pub version: String,
    pub configuration: RuleModuleConfigurationDto,
}

/// Closed configuration vocabulary carried over the protocol boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "module",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum RuleModuleConfigurationDto {
    ActionResolution {
        targeting_policy: String,
        supported_check_handlers: Vec<String>,
    },
    TurnControl {
        turn_order_policy: String,
        combat_end_policy: String,
        objective_side: Option<String>,
    },
}

/// Rust-owned diagnostics for converting authored wire data into authority declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesetAuthoringError {
    RuleModuleValidation(RuleModuleValidationError),
    UnsupportedActionResolutionTargetingPolicy { policy: String },
    UnsupportedCheckHandler { handler: String },
    UnsupportedTurnOrderPolicy { policy: String },
    UnsupportedCombatEndPolicy { policy: String },
    InvalidCombatEndPolicyConfiguration { policy: String },
}

impl RulesetAuthoringError {
    pub const fn code(&self) -> &'static str {
        match self {
            RulesetAuthoringError::RuleModuleValidation(error) => error.code(),
            RulesetAuthoringError::UnsupportedActionResolutionTargetingPolicy { .. } => {
                "unsupportedActionResolutionTargetingPolicy"
            }
            RulesetAuthoringError::UnsupportedCheckHandler { .. } => "unsupportedCheckHandler",
            RulesetAuthoringError::UnsupportedTurnOrderPolicy { .. } => {
                "unsupportedTurnOrderPolicy"
            }
            RulesetAuthoringError::UnsupportedCombatEndPolicy { .. } => {
                "unsupportedCombatEndPolicy"
            }
            RulesetAuthoringError::InvalidCombatEndPolicyConfiguration { .. } => {
                "invalidCombatEndPolicyConfiguration"
            }
        }
    }
}

/// Converts an authored wire DTO and lets Rust validate selected behavior modules.
pub fn validate_ruleset_definition(
    definition: &RulesetDefinitionDto,
) -> Result<RulesetMetadata, RulesetAuthoringError> {
    let modules = definition
        .modules
        .iter()
        .map(convert_module_declaration)
        .collect::<Result<Vec<_>, _>>()?;
    let ruleset = RulesetMetadata {
        id: definition.id.clone(),
        name: definition.name.clone(),
        version: definition.version.clone(),
        summary: definition.summary.clone(),
        modules,
    };
    ruleset
        .validate_modules()
        .map_err(RulesetAuthoringError::RuleModuleValidation)?;
    Ok(ruleset)
}

fn convert_module_declaration(
    declaration: &RuleModuleDeclarationDto,
) -> Result<RuleModuleDeclaration, RulesetAuthoringError> {
    let module = RuleModuleId::from_code(&declaration.module)
        .map_err(RulesetAuthoringError::RuleModuleValidation)?;
    let configuration = match &declaration.configuration {
        RuleModuleConfigurationDto::ActionResolution {
            targeting_policy,
            supported_check_handlers,
        } => RuleModuleConfiguration::ActionResolution(ActionResolutionModuleConfiguration {
            targeting_policy: action_resolution_targeting_policy(targeting_policy)?,
            supported_check_handlers: supported_check_handlers
                .iter()
                .map(|handler| parse_check_handler(handler))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        RuleModuleConfigurationDto::TurnControl {
            turn_order_policy: configured_policy,
            combat_end_policy,
            objective_side,
        } => RuleModuleConfiguration::TurnControl(TurnControlModuleConfiguration {
            turn_order_policy: parse_turn_order_policy(configured_policy)?,
            combat_end_policy: parse_combat_end_policy(
                combat_end_policy,
                objective_side.as_deref(),
            )?,
        }),
    };
    Ok(RuleModuleDeclaration {
        module,
        version: declaration.version.clone(),
        configuration,
    })
}

fn parse_combat_end_policy(
    policy: &str,
    objective_side: Option<&str>,
) -> Result<CombatEndPolicy, RulesetAuthoringError> {
    match (policy, objective_side) {
        ("lastSideStanding", None) => Ok(CombatEndPolicy::LastSideStanding),
        ("explicitOnly", None) => Ok(CombatEndPolicy::ExplicitOnly),
        ("objectiveSideVictory", Some(side_id)) if !side_id.is_empty() => {
            Ok(CombatEndPolicy::ObjectiveSideVictory {
                side_id: side_id.to_string(),
            })
        }
        ("lastSideStanding" | "explicitOnly" | "objectiveSideVictory", _) => {
            Err(RulesetAuthoringError::InvalidCombatEndPolicyConfiguration {
                policy: policy.to_string(),
            })
        }
        _ => Err(RulesetAuthoringError::UnsupportedCombatEndPolicy {
            policy: policy.to_string(),
        }),
    }
}

fn parse_check_handler(handler: &str) -> Result<CheckHandlerKind, RulesetAuthoringError> {
    match handler {
        "attackVsDefense" => Ok(CheckHandlerKind::AttackVsDefense),
        "savingThrow" => Ok(CheckHandlerKind::SavingThrow),
        "contested" => Ok(CheckHandlerKind::Contested),
        _ => Err(RulesetAuthoringError::UnsupportedCheckHandler {
            handler: handler.to_string(),
        }),
    }
}

fn action_resolution_targeting_policy(
    policy: &str,
) -> Result<ActionResolutionTargetingPolicy, RulesetAuthoringError> {
    match policy {
        "declaredTargetsAndLineOfSight" => {
            Ok(ActionResolutionTargetingPolicy::DeclaredTargetsAndLineOfSight)
        }
        _ => Err(
            RulesetAuthoringError::UnsupportedActionResolutionTargetingPolicy {
                policy: policy.to_string(),
            },
        ),
    }
}

fn parse_turn_order_policy(policy: &str) -> Result<TurnOrderPolicy, RulesetAuthoringError> {
    match policy {
        "explicit" => Ok(TurnOrderPolicy::Explicit),
        _ => Err(RulesetAuthoringError::UnsupportedTurnOrderPolicy {
            policy: policy.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_ruleset_definition, RuleModuleConfigurationDto, RuleModuleDeclarationDto,
        RulesetDefinitionDto,
    };

    fn valid_definition() -> RulesetDefinitionDto {
        RulesetDefinitionDto {
            id: "test.ruleset".to_string(),
            name: "Test Ruleset".to_string(),
            version: "1.0.0".to_string(),
            summary: "Protocol conversion test ruleset.".to_string(),
            modules: vec![RuleModuleDeclarationDto {
                module: "actionResolution".to_string(),
                version: "1".to_string(),
                configuration: RuleModuleConfigurationDto::ActionResolution {
                    targeting_policy: "declaredTargetsAndLineOfSight".to_string(),
                    supported_check_handlers: vec!["attackVsDefense".to_string()],
                },
            }],
        }
    }

    #[test]
    fn authored_ruleset_converts_to_validated_authority_metadata() {
        let definition = valid_definition();

        let ruleset = validate_ruleset_definition(&definition).expect("definition is valid");

        assert_eq!(ruleset.id, definition.id);
        assert_eq!(ruleset.modules.len(), 1);
    }

    #[test]
    fn invalid_module_version_remains_a_rust_diagnostic() {
        let mut definition = valid_definition();
        definition.modules[0].version = "2".to_string();

        let error = validate_ruleset_definition(&definition).expect_err("version is invalid");

        assert_eq!(error.code(), "incompatibleRuleModuleVersion");
    }

    #[test]
    fn unsupported_module_configuration_remains_a_rust_diagnostic() {
        let mut definition = valid_definition();
        definition.modules[0].configuration = RuleModuleConfigurationDto::ActionResolution {
            targeting_policy: "unrecognizedPolicy".to_string(),
            supported_check_handlers: vec!["attackVsDefense".to_string()],
        };

        let error = validate_ruleset_definition(&definition).expect_err("policy is invalid");

        assert_eq!(error.code(), "unsupportedActionResolutionTargetingPolicy");
    }

    #[test]
    fn authored_objective_side_policy_converts_to_authority_configuration() {
        let mut definition = valid_definition();
        definition.modules.push(RuleModuleDeclarationDto {
            module: "turnControl".to_string(),
            version: "1".to_string(),
            configuration: RuleModuleConfigurationDto::TurnControl {
                turn_order_policy: "explicit".to_string(),
                combat_end_policy: "objectiveSideVictory".to_string(),
                objective_side: Some("heroes".to_string()),
            },
        });

        let ruleset = validate_ruleset_definition(&definition).expect("policy is valid");
        let turn_control = ruleset
            .validate_modules()
            .expect("modules remain valid")
            .turn_control()
            .cloned()
            .expect("turn control is configured");

        assert_eq!(
            turn_control.combat_end_policy.objective_side_id(),
            Some("heroes")
        );
    }

    #[test]
    fn authored_combat_end_policy_rejects_unknown_or_incomplete_configuration() {
        let mut definition = valid_definition();
        definition.modules.push(RuleModuleDeclarationDto {
            module: "turnControl".to_string(),
            version: "1".to_string(),
            configuration: RuleModuleConfigurationDto::TurnControl {
                turn_order_policy: "explicit".to_string(),
                combat_end_policy: "objectiveSideVictory".to_string(),
                objective_side: None,
            },
        });
        assert_eq!(
            validate_ruleset_definition(&definition)
                .expect_err("objective side is required")
                .code(),
            "invalidCombatEndPolicyConfiguration"
        );

        if let RuleModuleConfigurationDto::TurnControl {
            combat_end_policy, ..
        } = &mut definition.modules[1].configuration
        {
            *combat_end_policy = "unknownPolicy".to_string();
        }
        assert_eq!(
            validate_ruleset_definition(&definition)
                .expect_err("unknown policy is rejected")
                .code(),
            "unsupportedCombatEndPolicy"
        );
    }
}
