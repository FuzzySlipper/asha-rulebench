use rulebench_rules::{CapabilityIdentity, CapabilityRegistryInput, RulesetProviderManifestEntry};

use crate::{run_capability_conformance, CapabilityConformanceFilter, ScenarioPackageRegistry};

pub fn capability_registry_input(registry: &ScenarioPackageRegistry) -> CapabilityRegistryInput {
    let packages = registry
        .registrations()
        .iter()
        .map(|registration| CapabilityIdentity {
            id: registration.package.identity.id.clone(),
            version: registration.package.identity.version.clone(),
        })
        .collect();
    let providers = registry
        .provider_catalog()
        .providers()
        .iter()
        .map(|provider| RulesetProviderManifestEntry {
            provider: CapabilityIdentity {
                id: provider.provider_id.clone(),
                version: provider.provider_version.clone(),
            },
            ruleset: CapabilityIdentity {
                id: provider.ruleset.id.clone(),
                version: provider.ruleset.version.clone(),
            },
            operation_vocabulary_version: provider.operation_vocabulary_version.clone(),
            effect_operation_vocabulary_version: provider
                .effect_operation_vocabulary_version
                .clone(),
            capabilities: provider
                .capabilities
                .iter()
                .map(|capability| CapabilityIdentity {
                    id: capability.id.clone(),
                    version: capability.version.clone(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let rulesets = providers
        .iter()
        .map(|provider| provider.ruleset.clone())
        .collect();
    let scenarios = registry
        .scenario_catalog_cases()
        .into_iter()
        .map(|case| CapabilityIdentity {
            id: case.summary.id,
            version: "registered".to_string(),
        })
        .collect();

    let conformance = run_capability_conformance(registry, &CapabilityConformanceFilter::default());

    CapabilityRegistryInput {
        providers,
        rulesets,
        packages,
        scenarios,
        regression_capability_ids: conformance
            .covered_capabilities
            .into_iter()
            .map(|identity| identity.id)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_input_is_derived_from_registered_packages_and_regression_cases() {
        let input = capability_registry_input(&crate::scenario_package_registry());

        assert_eq!(input.packages.len(), 4);
        assert_eq!(input.scenarios.len(), 11);
        assert!(input
            .regression_capability_ids
            .contains(&"targeting.manhattanBurstArea".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"operation.move".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"operation.openReactionWindow".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"targeting.multipleCombatants".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"operation.heal".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"operation.grantTemporaryVitality".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"policy.firstAcceptedCandidate".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"policy.lowestVitalityTarget".to_string()));
        assert!(input
            .regression_capability_ids
            .contains(&"policy.objectiveSidePressure".to_string()));
    }
}
