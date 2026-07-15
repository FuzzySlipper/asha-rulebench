use std::collections::BTreeSet;

use rulebench_rules::{
    CapabilityIdentity, CapabilityRegistryInput, HitEffectOperation, TargetKind, TargetSelection,
};

use crate::ScenarioPackageRegistry;

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

    let mut regression_ids = BTreeSet::new();
    for case in registry.scenario_catalog_cases() {
        for action in &case.scenario.actions {
            let targeting_id = if action.movement.is_some() {
                "targeting.cellMovement"
            } else {
                match (
                    action.targeting.target_kind,
                    action.targeting.selection,
                    action.targeting.operation_pipeline.as_ref(),
                ) {
                    (TargetKind::Combatant, TargetSelection::Single, _) => {
                        "targeting.singleCombatant"
                    }
                    (TargetKind::Combatant, TargetSelection::Multiple, _) => {
                        "targeting.multipleCombatants"
                    }
                    (TargetKind::Area, _, Some(_)) => "targeting.manhattanBurstArea",
                    (TargetKind::Area, _, None) => continue,
                }
            };
            regression_ids.insert(targeting_id.to_string());
            for operation in &action.hit.operations {
                regression_ids.insert(format!("operation.{}", operation_id(operation)));
            }
        }
    }
    for readout in registry.combat_session_automatic_run_readouts() {
        regression_ids.insert(format!("policy.{}", readout.policy.id));
    }

    CapabilityRegistryInput {
        rulesets,
        packages,
        scenarios,
        regression_capability_ids: regression_ids.into_iter().collect(),
    }
}

fn operation_id(operation: &HitEffectOperation) -> &'static str {
    operation.id().code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_input_is_derived_from_registered_packages_and_regression_cases() {
        let input = capability_registry_input(&crate::scenario_package_registry());

        assert_eq!(input.packages.len(), 3);
        assert_eq!(input.scenarios.len(), 7);
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
            .contains(&"policy.firstAcceptedCandidate".to_string()));
    }
}
