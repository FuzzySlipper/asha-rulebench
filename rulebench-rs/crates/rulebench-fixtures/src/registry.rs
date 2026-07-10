use std::collections::HashSet;

use rulebench_rules::{
    CombatControlHistoryReadout, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunReplayReadout, CombatSessionScriptReadout, CombatSessionTranscript,
};

use crate::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioPackage,
};

/// Fixed Rust registration for one Rulebench scenario package.
///
/// These function pointers are compiled fixture readback constructors, not
/// authored scenario callbacks and never participate in rule resolution.
#[derive(Clone)]
pub struct ScenarioPackageRegistration {
    pub package: ScenarioPackage,
    factories: ScenarioPackageReadbackFactories,
}

#[derive(Clone)]
pub struct ScenarioPackageReadbackFactories {
    pub(crate) catalog_cases: fn() -> Vec<ScenarioCatalogCase>,
    pub(crate) ruleset_catalog_readout: fn() -> RulesetCatalogReadout,
    pub(crate) content_validation_readouts: fn() -> Vec<ContentValidationReadout>,
    pub(crate) session_transcripts: fn() -> Vec<CombatSessionTranscript>,
    pub(crate) control_history_readouts: fn() -> Vec<CombatControlHistoryReadout>,
    pub(crate) script_readouts: fn() -> Vec<CombatSessionScriptReadout>,
    pub(crate) automatic_run_readouts: fn() -> Vec<CombatSessionAutomaticRunReadout>,
    pub(crate) automatic_run_replay_readouts: fn() -> Vec<CombatSessionAutomaticRunReplayReadout>,
}

impl ScenarioPackageRegistration {
    pub fn new(package: ScenarioPackage, factories: ScenarioPackageReadbackFactories) -> Self {
        Self { package, factories }
    }
}

#[derive(Clone)]
pub struct ScenarioPackageRegistry {
    registrations: Vec<ScenarioPackageRegistration>,
}

impl ScenarioPackageRegistry {
    pub fn new(
        mut registrations: Vec<ScenarioPackageRegistration>,
    ) -> Result<Self, Vec<ScenarioPackageRegistryError>> {
        registrations.sort_by(|left, right| {
            left.package
                .identity
                .id
                .cmp(&right.package.identity.id)
                .then_with(|| {
                    left.package
                        .identity
                        .version
                        .cmp(&right.package.identity.version)
                })
        });

        let mut errors = Vec::new();
        let mut identities = HashSet::new();
        for registration in &registrations {
            let identity = format!(
                "{}@{}",
                registration.package.identity.id, registration.package.identity.version
            );
            if !identities.insert(identity.clone()) {
                errors.push(ScenarioPackageRegistryError::DuplicatePackageIdentity { identity });
            }
            if registration.package.validate().is_err() {
                errors.push(ScenarioPackageRegistryError::InvalidPackage {
                    package_id: registration.package.identity.id.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(Self { registrations })
        } else {
            Err(errors)
        }
    }

    pub fn registrations(&self) -> &[ScenarioPackageRegistration] {
        &self.registrations
    }

    pub fn select(
        &self,
        package_id: &str,
        package_version: &str,
    ) -> Result<&ScenarioPackageRegistration, ScenarioPackageSelectionError> {
        self.registrations
            .iter()
            .find(|registration| {
                registration.package.identity.id == package_id
                    && registration.package.identity.version == package_version
            })
            .ok_or_else(|| ScenarioPackageSelectionError::UnknownPackage {
                package_id: package_id.to_string(),
                package_version: package_version.to_string(),
            })
    }

    pub fn scenario_catalog_cases(&self) -> Vec<ScenarioCatalogCase> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.catalog_cases)())
            .collect()
    }

    pub fn ruleset_catalog_readout(&self) -> RulesetCatalogReadout {
        let mut readouts = self
            .registrations
            .iter()
            .map(|registration| (registration.factories.ruleset_catalog_readout)());
        let Some(first_readout) = readouts.next() else {
            return RulesetCatalogReadout {
                selected_ruleset_id: String::new(),
                rulesets: Vec::new(),
            };
        };
        let mut rulesets = first_readout.rulesets;
        let mut seen_ruleset_ids = rulesets
            .iter()
            .map(|ruleset| ruleset.id.clone())
            .collect::<HashSet<_>>();
        for readout in readouts {
            for ruleset in readout.rulesets {
                if seen_ruleset_ids.insert(ruleset.id.clone()) {
                    rulesets.push(ruleset);
                }
            }
        }
        RulesetCatalogReadout {
            selected_ruleset_id: first_readout.selected_ruleset_id,
            rulesets,
        }
    }

    pub fn content_validation_readouts(&self) -> Vec<ContentValidationReadout> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.content_validation_readouts)())
            .collect()
    }

    pub fn combat_session_transcripts(&self) -> Vec<CombatSessionTranscript> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.session_transcripts)())
            .collect()
    }

    pub fn combat_session_control_history_readouts(&self) -> Vec<CombatControlHistoryReadout> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.control_history_readouts)())
            .collect()
    }

    pub fn combat_session_script_readouts(&self) -> Vec<CombatSessionScriptReadout> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.script_readouts)())
            .collect()
    }

    pub fn combat_session_automatic_run_readouts(&self) -> Vec<CombatSessionAutomaticRunReadout> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.automatic_run_readouts)())
            .collect()
    }

    pub fn combat_session_automatic_run_replay_readouts(
        &self,
    ) -> Vec<CombatSessionAutomaticRunReplayReadout> {
        self.registrations
            .iter()
            .flat_map(|registration| (registration.factories.automatic_run_replay_readouts)())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioPackageRegistryError {
    DuplicatePackageIdentity { identity: String },
    InvalidPackage { package_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioPackageSelectionError {
    UnknownPackage {
        package_id: String,
        package_version: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registration_with_identity(id: &str, version: &str) -> ScenarioPackageRegistration {
        let mut registration = crate::scenarios::hexing_bolt::registration();
        registration.package.identity.id = id.to_string();
        registration.package.identity.version = version.to_string();
        registration.package.golden_manifest.package_id = id.to_string();
        registration
    }

    #[test]
    fn registry_orders_packages_by_stable_identity_and_selects_exact_versions() {
        let registry = ScenarioPackageRegistry::new(vec![
            registration_with_identity("zeta", "1.0.0"),
            registration_with_identity("alpha", "2.0.0"),
            registration_with_identity("alpha", "1.0.0"),
        ])
        .expect("registrations are valid and unique");

        let identities = registry
            .registrations()
            .iter()
            .map(|registration| {
                format!(
                    "{}@{}",
                    registration.package.identity.id, registration.package.identity.version
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(identities, vec!["alpha@1.0.0", "alpha@2.0.0", "zeta@1.0.0"]);
        assert_eq!(
            registry
                .select("alpha", "2.0.0")
                .expect("exact package exists")
                .package
                .identity
                .version,
            "2.0.0"
        );
    }

    #[test]
    fn registry_rejects_duplicate_id_version_and_unknown_selection() {
        let duplicate_result = ScenarioPackageRegistry::new(vec![
            registration_with_identity("duplicate", "1.0.0"),
            registration_with_identity("duplicate", "1.0.0"),
        ]);
        assert!(duplicate_result.is_err());
        let duplicate_errors = duplicate_result
            .err()
            .expect("duplicate errors are present");

        assert_eq!(
            duplicate_errors,
            vec![ScenarioPackageRegistryError::DuplicatePackageIdentity {
                identity: "duplicate@1.0.0".to_string(),
            }]
        );

        let registry =
            ScenarioPackageRegistry::new(vec![registration_with_identity("alpha", "1.0.0")])
                .expect("single package is valid");
        match registry.select("missing", "1.0.0") {
            Err(ScenarioPackageSelectionError::UnknownPackage {
                package_id,
                package_version,
            }) => {
                assert_eq!(package_id, "missing");
                assert_eq!(package_version, "1.0.0");
            }
            Ok(_) => panic!("missing package selection unexpectedly succeeded"),
        }
    }

    #[test]
    fn registry_aggregates_fixture_readbacks_without_package_specific_paths() {
        let registry = crate::scenario_package_registry();

        assert_eq!(registry.registrations().len(), 2);
        assert_eq!(registry.scenario_catalog_cases().len(), 5);
        assert_eq!(registry.combat_session_transcripts().len(), 2);
        assert_eq!(registry.combat_session_script_readouts().len(), 1);
    }
}
