pub mod hexing_bolt;
pub mod skirmish;
pub mod veteran_hexing;

pub fn registry() -> crate::ScenarioPackageRegistry {
    crate::ScenarioPackageRegistry::new(vec![
        hexing_bolt::registration(),
        skirmish::registration(),
        veteran_hexing::registration(),
    ])
    .expect("built-in Rulebench scenario packages are valid and unique")
}
