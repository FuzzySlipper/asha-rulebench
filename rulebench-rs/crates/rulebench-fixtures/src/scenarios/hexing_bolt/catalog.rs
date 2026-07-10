use super::fixture::{hexing_bolt_fixture_scenario, turn_control_fixture_scenario};
use crate::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogSummary,
    ScenarioOutcomeClass,
};
use rulebench_rules::{
    validate_scenario_content_report, RulebenchScenario, ScenarioMetadata, UseActionIntent,
};

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    vec![
        accepted_hit_catalog_case(),
        accepted_miss_catalog_case(),
        rejected_target_legality_catalog_case(),
        turn_control_catalog_case(),
    ]
}

fn turn_control_catalog_case() -> ScenarioCatalogCase {
    let mut scenario = turn_control_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: "turn-control-hit".to_string(),
        title: "Turn Control Ruleset Hit".to_string(),
        summary: "The second ruleset resolves the same minimal action with turn control selected."
            .to_string(),
        seed_label: "roll-stream:17,5".to_string(),
    };
    ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: scenario.metadata.id.clone(),
            title: scenario.metadata.title.clone(),
            summary: scenario.metadata.summary.clone(),
            seed_label: scenario.metadata.seed_label.clone(),
            outcome_class: ScenarioOutcomeClass::AcceptedHit,
        },
        scenario,
        intent: UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        roll_stream: vec![17, 5],
    }
}

pub fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    let hexing_bolt_scenario = hexing_bolt_fixture_scenario();
    let turn_control_scenario = turn_control_fixture_scenario();
    RulesetCatalogReadout {
        selected_ruleset_id: hexing_bolt_scenario.selected_ruleset_id,
        rulesets: vec![
            hexing_bolt_scenario.rulesets[0].clone(),
            turn_control_scenario.rulesets[0].clone(),
        ],
    }
}

pub fn content_validation_readouts() -> Vec<ContentValidationReadout> {
    let mut readouts = scenario_catalog_cases()
        .into_iter()
        .map(|case| ContentValidationReadout {
            scenario_id: case.summary.id,
            scenario_title: case.summary.title,
            report: validate_scenario_content_report(&case.scenario),
        })
        .collect::<Vec<_>>();
    readouts.extend(invalid_content_validation_readouts());
    readouts
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

fn invalid_content_validation_readouts() -> Vec<ContentValidationReadout> {
    vec![
        invalid_content_validation_readout(
            "hexing-bolt-invalid-selected-ruleset",
            "Hexing Bolt Invalid Selected Ruleset",
            |scenario| {
                scenario.selected_ruleset_id = "asha-rulebench.missing.v0".to_string();
            },
        ),
        invalid_content_validation_readout(
            "hexing-bolt-invalid-selected-ability",
            "Hexing Bolt Invalid Selected Ability",
            |scenario| {
                scenario.selected_ability_id = Some("ability.missing".to_string());
            },
        ),
        invalid_content_validation_readout(
            "hexing-bolt-invalid-equipped-item",
            "Hexing Bolt Invalid Equipped Item",
            |scenario| {
                scenario.combatants[0]
                    .equipped_item_ids
                    .push("item.missing-focus".to_string());
            },
        ),
    ]
}

fn invalid_content_validation_readout(
    id: &str,
    title: &str,
    configure: impl FnOnce(&mut RulebenchScenario),
) -> ContentValidationReadout {
    let mut scenario = hexing_bolt_fixture_scenario();
    configure(&mut scenario);

    ContentValidationReadout {
        scenario_id: id.to_string(),
        scenario_title: title.to_string(),
        report: validate_scenario_content_report(&scenario),
    }
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
