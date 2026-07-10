use rulebench_rules::{
    ActionResolutionModuleConfiguration, ActionResolutionTargetingPolicy, CheckHandlerKind,
    RuleModuleConfiguration, RuleModuleDeclaration, RuleModuleId, RuleModuleValidationError,
    RulesetMetadata, TurnControlModuleConfiguration, TurnOrderPolicy,
};

/// Stable wire form of a ruleset definition authored outside Rust authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetDefinitionDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub modules: Vec<RuleModuleDeclarationDto>,
}

/// Stable wire form of one selected Rust behavior module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleModuleDeclarationDto {
    pub module: String,
    pub version: String,
    pub configuration: RuleModuleConfigurationDto,
}

/// Closed configuration vocabulary carried over the protocol boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleModuleConfigurationDto {
    ActionResolution {
        targeting_policy: String,
        supported_check_handlers: Vec<String>,
    },
    TurnControl {
        turn_order_policy: String,
    },
}

/// Rust-owned diagnostics for converting authored wire data into authority declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesetAuthoringError {
    RuleModuleValidation(RuleModuleValidationError),
    UnsupportedActionResolutionTargetingPolicy { policy: String },
    UnsupportedCheckHandler { handler: String },
    UnsupportedTurnOrderPolicy { policy: String },
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
        } => RuleModuleConfiguration::TurnControl(TurnControlModuleConfiguration {
            turn_order_policy: parse_turn_order_policy(configured_policy)?,
        }),
    };
    Ok(RuleModuleDeclaration {
        module,
        version: declaration.version.clone(),
        configuration,
    })
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
}
