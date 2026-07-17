//! Named Rulebench product content and primary workflow samples.
//!
//! This crate is product composition, not portable RPG authority or exhaustive
//! certification. Downstream proof lives in `asha-rulebench-testing`.

#![forbid(unsafe_code)]

mod capabilities;
mod catalog;
mod content_import;
mod package;
mod providers;
mod registry;
mod replay_review;
pub mod scenarios;

pub use capabilities::capability_registry_input;
pub use catalog::*;
pub use content_import::{
    content_import_examples, ContentImportExample, ContentImportExampleOutcome,
};
pub use package::{
    ScenarioPackage, ScenarioPackageContentReference, ScenarioPackageDisplayMetadata,
    ScenarioPackageIdentity, ScenarioPackageInitialState, ScenarioPackageRulesetReference,
    ScenarioPackageScript, ScenarioPackageValidationError,
};
pub use providers::{
    compiled_ruleset_provider_catalog, hexing_bolt_ruleset, turn_control_ruleset,
    HEXING_BOLT_PROVIDER_ID, HEXING_BOLT_RULESET_ID, HEXING_BOLT_RULESET_VERSION,
    TURN_CONTROL_PROVIDER_ID, TURN_CONTROL_RULESET_ID, TURN_CONTROL_RULESET_VERSION,
};
pub use registry::{
    ScenarioPackageReadbackFactories, ScenarioPackageRegistration, ScenarioPackageRegistry,
    ScenarioPackageRegistryError, ScenarioPackageSelectionError,
};
pub use replay_review::replay_review_packages;
pub use scenarios::hexing_bolt::{
    accepted_hexing_bolt_fixture_receipt, combat_session_automatic_run_readouts,
    combat_session_automatic_run_replay_readouts, combat_session_control_history_readouts,
    combat_session_script_readouts, combat_session_transcripts, content_validation_readouts,
    hexing_bolt_fixture_scenario, hexing_bolt_scenario_package, rejected_target_fixture_receipt,
    ruleset_catalog_readout, scenario_catalog_cases, turn_control_fixture_scenario,
};

use rulebench_combat::preview_use_action;

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
    let receipt = preview_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
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

pub fn aggregated_combat_session_transcripts() -> Vec<rulebench_combat::CombatSessionTranscript> {
    scenario_package_registry().combat_session_transcripts()
}

pub fn aggregated_combat_session_control_history_readouts(
) -> Vec<rulebench_combat::CombatControlHistoryReadout> {
    scenario_package_registry().combat_session_control_history_readouts()
}

pub fn aggregated_combat_session_script_readouts(
) -> Vec<rulebench_combat::CombatSessionScriptReadout> {
    scenario_package_registry().combat_session_script_readouts()
}

pub fn aggregated_combat_session_automatic_run_readouts(
) -> Vec<rulebench_combat::CombatSessionAutomaticRunReadout> {
    scenario_package_registry().combat_session_automatic_run_readouts()
}

pub fn aggregated_combat_session_automatic_run_replay_readouts(
) -> Vec<rulebench_replay::CombatSessionAutomaticRunReplayReadout> {
    scenario_package_registry().combat_session_automatic_run_replay_readouts()
}
