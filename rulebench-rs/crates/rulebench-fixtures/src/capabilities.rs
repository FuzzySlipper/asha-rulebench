use rulebench_rules::{CapabilityIdentity, CapabilityRegistryInput};

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
    let rulesets = registry
        .registrations()
        .iter()
        .map(|registration| CapabilityIdentity {
            id: registration.package.ruleset.id.clone(),
            version: registration.package.ruleset.version.clone(),
        })
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

        assert_eq!(input.packages.len(), 3);
        assert_eq!(input.scenarios.len(), 10);
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
    }
}
