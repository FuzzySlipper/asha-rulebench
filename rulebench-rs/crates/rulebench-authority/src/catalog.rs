use crate::model::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogError,
    ScenarioCatalogResolution, ScenarioCatalogSummary,
};
use crate::resolver::resolve_use_action;
use crate::scenarios::hexing_bolt;

pub fn scenario_catalog_summaries() -> Vec<ScenarioCatalogSummary> {
    scenario_catalog_cases()
        .into_iter()
        .map(|case| case.summary)
        .collect()
}

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    hexing_bolt::catalog::scenario_catalog_cases()
}

pub fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    hexing_bolt::catalog::ruleset_catalog_readout()
}

pub fn content_validation_readouts() -> Vec<ContentValidationReadout> {
    hexing_bolt::catalog::content_validation_readouts()
}

pub fn resolve_catalog_scenario(
    id: &str,
) -> Result<ScenarioCatalogResolution, ScenarioCatalogError> {
    let Some(case) = scenario_catalog_cases()
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
