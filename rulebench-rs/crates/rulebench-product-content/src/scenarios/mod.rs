pub mod hexing_bolt;
pub mod skirmish;
pub mod turn_control;
pub mod veteran_hexing;

pub fn registry() -> crate::ScenarioPackageRegistry {
    crate::ScenarioPackageRegistry::new_with_providers(
        vec![
            hexing_bolt::registration(),
            skirmish::registration(),
            turn_control::registration(),
            veteran_hexing::registration(),
        ],
        crate::compiled_ruleset_provider_catalog(),
    )
    .expect("built-in Rulebench scenario packages are valid and unique")
}
