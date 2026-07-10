//! Rulebench-local scenarios, fixtures, and regression packs.
//!
//! This reserved boundary will own authored testbench content such as Hexing
//! Bolt. Game repositories should not need this crate to execute shared rules.

#![forbid(unsafe_code)]

mod catalog;
mod goldens;
mod package;
mod registry;
pub mod scenarios;

pub use catalog::*;
pub use goldens::{FixtureGoldenArtifact, FixtureGoldenArtifactKind, FixtureGoldenManifest};
pub use package::{
    ScenarioPackage, ScenarioPackageContentReference, ScenarioPackageDisplayMetadata,
    ScenarioPackageEvidenceExpectation, ScenarioPackageEvidenceKind, ScenarioPackageIdentity,
    ScenarioPackageInitialState, ScenarioPackageRulesetReference, ScenarioPackageScript,
    ScenarioPackageValidationError,
};
pub use registry::{
    ScenarioPackageReadbackFactories, ScenarioPackageRegistration, ScenarioPackageRegistry,
    ScenarioPackageRegistryError, ScenarioPackageSelectionError,
};
pub use rulebench_rules::*;
pub use scenarios::hexing_bolt::{
    accepted_hexing_bolt_fixture_receipt, combat_session_automatic_run_readouts,
    combat_session_automatic_run_replay_readouts, combat_session_control_history_readouts,
    combat_session_script_readouts, combat_session_transcripts, content_validation_readouts,
    hexing_bolt_fixture_scenario, hexing_bolt_scenario_package, rejected_target_fixture_receipt,
    ruleset_catalog_readout, scenario_catalog_cases, turn_control_fixture_scenario,
};

pub fn scenario_package_registry() -> ScenarioPackageRegistry {
    scenarios::registry()
}

pub fn registered_scenario_packages() -> Vec<ScenarioPackage> {
    scenario_package_registry()
        .registrations()
        .iter()
        .map(|registration| registration.package.clone())
        .collect()
}

pub fn aggregated_scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    scenario_package_registry().scenario_catalog_cases()
}

pub fn resolve_catalog_scenario(
    id: &str,
) -> Result<ScenarioCatalogResolution, ScenarioCatalogError> {
    let Some(case) = aggregated_scenario_catalog_cases()
        .into_iter()
        .find(|case| case.summary.id == id)
    else {
        return Err(ScenarioCatalogError::UnknownScenarioId);
    };
    let receipt = resolve_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
    Ok(ScenarioCatalogResolution {
        case: case.summary,
        scenario: case.scenario,
        receipt,
    })
}

pub fn aggregated_ruleset_catalog_readout() -> RulesetCatalogReadout {
    scenario_package_registry().ruleset_catalog_readout()
}

pub fn aggregated_content_validation_readouts() -> Vec<ContentValidationReadout> {
    scenario_package_registry().content_validation_readouts()
}

pub fn aggregated_combat_session_transcripts() -> Vec<rulebench_rules::CombatSessionTranscript> {
    scenario_package_registry().combat_session_transcripts()
}

pub fn aggregated_combat_session_control_history_readouts(
) -> Vec<rulebench_rules::CombatControlHistoryReadout> {
    scenario_package_registry().combat_session_control_history_readouts()
}

pub fn aggregated_combat_session_script_readouts(
) -> Vec<rulebench_rules::CombatSessionScriptReadout> {
    scenario_package_registry().combat_session_script_readouts()
}

pub fn aggregated_combat_session_automatic_run_readouts(
) -> Vec<rulebench_rules::CombatSessionAutomaticRunReadout> {
    scenario_package_registry().combat_session_automatic_run_readouts()
}

pub fn aggregated_combat_session_automatic_run_replay_readouts(
) -> Vec<rulebench_rules::CombatSessionAutomaticRunReplayReadout> {
    scenario_package_registry().combat_session_automatic_run_replay_readouts()
}
