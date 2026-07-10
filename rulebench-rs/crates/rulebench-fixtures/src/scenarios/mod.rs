pub mod hexing_bolt;

pub fn registry() -> crate::ScenarioPackageRegistry {
    crate::ScenarioPackageRegistry::new(vec![hexing_bolt::registration()])
        .expect("built-in Rulebench scenario packages are valid and unique")
}
