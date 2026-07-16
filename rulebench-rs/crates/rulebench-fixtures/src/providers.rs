use rulebench_rules::*;

pub const HEXING_BOLT_PROVIDER_ID: &str = "provider.asha-rulebench.hexing-bolt";
pub const TURN_CONTROL_PROVIDER_ID: &str = "provider.asha-rulebench.turn-control";
pub const HEXING_BOLT_RULESET_ID: &str = "asha-rulebench.hexing-bolt.v0";
pub const HEXING_BOLT_RULESET_VERSION: &str = "0.0.0";
pub const TURN_CONTROL_RULESET_ID: &str = "asha-rulebench.turn-control.v0";
pub const TURN_CONTROL_RULESET_VERSION: &str = "0.1.0";

pub fn compiled_ruleset_provider_catalog() -> RulesetProviderCatalog {
    RulesetProviderCatalog::new(vec![hexing_bolt_provider(), turn_control_provider()])
        .expect("built-in Rulebench providers are compatible and collision-free")
}

pub fn hexing_bolt_ruleset() -> RulesetMetadata {
    RulesetMetadata {
        id: HEXING_BOLT_RULESET_ID.to_string(),
        name: "Hexing Bolt Fixture Rules".to_string(),
        version: HEXING_BOLT_RULESET_VERSION.to_string(),
        summary: "Local action-resolution rules for the Hexing Bolt package family.".to_string(),
        modules: vec![RuleModuleDeclaration::action_resolution(
            ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
        )],
    }
}

pub fn turn_control_ruleset() -> RulesetMetadata {
    RulesetMetadata {
        id: TURN_CONTROL_RULESET_ID.to_string(),
        name: "Objective Turn Control Rules".to_string(),
        version: TURN_CONTROL_RULESET_VERSION.to_string(),
        summary: "Saving-throw actions under explicit turns and objective-side victory."
            .to_string(),
        modules: vec![
            RuleModuleDeclaration::action_resolution(
                ActionResolutionModuleConfiguration::with_supported_check_handlers(
                    ActionResolutionTargetingPolicy::DeclaredTargetsAndLineOfSight,
                    vec![
                        CheckHandlerKind::AttackVsDefense,
                        CheckHandlerKind::SavingThrow,
                    ],
                ),
            ),
            RuleModuleDeclaration::turn_control(
                TurnControlModuleConfiguration::explicit_turn_order_with_end_policy(
                    CombatEndPolicy::ObjectiveSideVictory {
                        side_id: "wardens".to_string(),
                    },
                ),
            ),
        ],
    }
}

fn hexing_bolt_provider() -> RulesetProviderDescriptor {
    RulesetProviderDescriptor {
        provider_id: HEXING_BOLT_PROVIDER_ID.to_string(),
        provider_version: "1".to_string(),
        ruleset: hexing_bolt_ruleset(),
        operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        effect_operation_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        capabilities: executable_conformance_capabilities()
            .into_iter()
            .filter(|identity| identity.id != "policy.objectiveSidePressure")
            .map(|identity| RulesetProviderCapability {
                id: identity.id,
                version: identity.version,
            })
            .chain([RulesetProviderCapability {
                id: "check.attackVsDefense".to_string(),
                version: "1".to_string(),
            }])
            .collect(),
    }
}

fn turn_control_provider() -> RulesetProviderDescriptor {
    RulesetProviderDescriptor {
        provider_id: TURN_CONTROL_PROVIDER_ID.to_string(),
        provider_version: "1".to_string(),
        ruleset: turn_control_ruleset(),
        operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        effect_operation_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        capabilities: vec![
            capability("check.attackVsDefense", "1"),
            capability("check.savingThrow", "1"),
            capability(
                "operation.applyModifier",
                EffectOperationId::VOCABULARY_VERSION,
            ),
            capability("operation.damage", EffectOperationId::VOCABULARY_VERSION),
            capability(
                "policy.firstAcceptedCandidate",
                &FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION.to_string(),
            ),
            capability(
                "policy.lowestVitalityTarget",
                &LOWEST_VITALITY_TARGET_POLICY_VERSION.to_string(),
            ),
            capability(
                "policy.objectiveSidePressure",
                &OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION.to_string(),
            ),
            capability(
                "targeting.singleCombatant",
                OperationPipelineV2::VOCABULARY_VERSION,
            ),
        ],
    }
}

fn capability(id: &str, version: &str) -> RulesetProviderCapability {
    RulesetProviderCapability {
        id: id.to_string(),
        version: version.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_catalog_is_stable_and_supports_both_rulesets() {
        let catalog = compiled_ruleset_provider_catalog();

        assert_eq!(catalog.providers().len(), 2);
        assert_eq!(
            catalog
                .select_ruleset(HEXING_BOLT_RULESET_ID, "0.0.0")
                .expect("Hexing Bolt provider")
                .provider_id,
            HEXING_BOLT_PROVIDER_ID
        );
        assert_eq!(
            catalog
                .select_ruleset(TURN_CONTROL_RULESET_ID, "0.1.0")
                .expect("turn-control provider")
                .provider_id,
            TURN_CONTROL_PROVIDER_ID
        );
    }

    #[test]
    fn provider_collisions_and_vocabulary_drift_are_classified() {
        let provider = turn_control_provider();
        let collision = RulesetProviderCatalog::new(vec![provider.clone(), provider.clone()])
            .expect_err("duplicate provider identity must fail");
        assert!(collision.iter().any(|error| {
            error.code() == "duplicateRulesetProviderIdentity"
                || error.code() == "rulesetProviderCollision"
        }));

        let mut drifted = provider;
        drifted.operation_vocabulary_version = "999".to_string();
        let drift = RulesetProviderCatalog::new(vec![drifted])
            .expect_err("operation vocabulary drift must fail");
        assert!(drift
            .iter()
            .any(|error| { error.code() == "incompatibleProviderOperationVocabulary" }));
    }

    #[test]
    fn removed_or_upgraded_provider_rejects_stored_provenance_descriptively() {
        let catalog = compiled_ruleset_provider_catalog();
        let provenance = turn_control_ruleset().artifact_provenance();
        assert!(catalog.validate_artifact_provenance(&provenance).is_ok());

        let without_turn_control = RulesetProviderCatalog::new(vec![hexing_bolt_provider()])
            .expect("remaining provider is valid");
        let missing = without_turn_control
            .validate_artifact_provenance(&provenance)
            .expect_err("removed provider must fail");
        assert_eq!(missing.code(), "rulesetProviderUnavailable");

        let mut upgraded = provenance;
        upgraded.ruleset_version = "0.2.0".to_string();
        let incompatible = catalog
            .validate_artifact_provenance(&upgraded)
            .expect_err("unknown provider version must fail");
        assert_eq!(incompatible.code(), "incompatibleProviderRulesetVersion");
    }
}
