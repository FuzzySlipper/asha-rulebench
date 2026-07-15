use super::fixture::hexing_bolt_fixture_scenario;
use crate::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogSummary,
    ScenarioOutcomeClass,
};
use rulebench_rules::{
    validate_scenario_content_report, HitEffectOperation, ReactionHookEffectOperation,
    ReactionOptionDeclaration, ReactionWindow, RulebenchScenario, ScenarioMetadata,
    UseActionIntent,
};

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    vec![
        accepted_hit_catalog_case(),
        reaction_window_catalog_case(),
        accepted_miss_catalog_case(),
        rejected_target_legality_catalog_case(),
    ]
}

fn reaction_window_catalog_case() -> ScenarioCatalogCase {
    let mut case = catalog_case(
        "hexing-bolt-reaction",
        "Hexing Bolt Reaction",
        "Adept hits Raider, suspending before effects while the authored reaction window is open.",
        "roll-stream:17,5",
        ScenarioOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    );
    let hook = ReactionHookEffectOperation {
        hook_id: "hexing-bolt.pre-effect".to_string(),
        window: ReactionWindow::BeforeEffect,
        eligible_reactor_ids: vec!["entity-adept".to_string(), "entity-raider".to_string()],
        options: vec![
            ReactionOptionDeclaration {
                id: "adept-counter".to_string(),
                reactor_id: "entity-adept".to_string(),
                opens_nested_window: true,
            },
            ReactionOptionDeclaration {
                id: "raider-ward".to_string(),
                reactor_id: "entity-raider".to_string(),
                opens_nested_window: false,
            },
        ],
        maximum_nested_depth: 1,
    };
    case.scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::OpenReactionWindow(hook.clone()));
    case.scenario
        .selected_action
        .hit
        .operations
        .push(HitEffectOperation::OpenReactionWindow(hook));
    case
}

pub fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    let hexing_bolt_scenario = hexing_bolt_fixture_scenario();
    RulesetCatalogReadout {
        selected_ruleset_id: hexing_bolt_scenario.selected_ruleset_id,
        rulesets: vec![
            hexing_bolt_scenario.rulesets[0].clone(),
            crate::turn_control_ruleset(),
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
