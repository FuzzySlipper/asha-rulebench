use crate::fixtures::hexing_bolt_fixture_scenario;
use crate::model::*;
use crate::resolver::resolve_use_action;

pub fn scenario_catalog_summaries() -> Vec<ScenarioCatalogSummary> {
    scenario_catalog_cases()
        .into_iter()
        .map(|case| case.summary)
        .collect()
}

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    vec![
        accepted_hit_catalog_case(),
        accepted_miss_catalog_case(),
        rejected_target_legality_catalog_case(),
    ]
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

fn accepted_hit_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-hit",
        "Hexing Bolt Hit",
        "Adept hits Raider, applying psychic damage and rattled.",
        "roll-stream:17,5",
        ScenarioOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    )
}

fn accepted_miss_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-miss",
        "Hexing Bolt Miss",
        "Adept targets Raider but the attack misses, leaving state unchanged.",
        "roll-stream:2,5",
        ScenarioOutcomeClass::AcceptedMiss,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    )
}

fn rejected_target_legality_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-self-target-rejected",
        "Hexing Bolt Self Target Rejected",
        "Adept attempts to target themself and target legality rejects the intent.",
        "roll-stream:17,5",
        ScenarioOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    )
}

fn catalog_case(
    id: &str,
    title: &str,
    summary: &str,
    seed_label: &str,
    outcome_class: ScenarioOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
) -> ScenarioCatalogCase {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        seed_label: seed_label.to_string(),
    };
    ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            seed_label: seed_label.to_string(),
            outcome_class,
        },
        scenario,
        intent,
        roll_stream,
    }
}
