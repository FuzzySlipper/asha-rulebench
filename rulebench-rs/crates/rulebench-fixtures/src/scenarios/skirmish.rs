//! Multi-participant human-testing fixture over the developed ruleset.

use crate::{
    ContentValidationReadout, FixtureGoldenArtifact, FixtureGoldenArtifactKind,
    FixtureGoldenManifest, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogSummary,
    ScenarioOutcomeClass, ScenarioPackage, ScenarioPackageContentReference,
    ScenarioPackageDisplayMetadata, ScenarioPackageEvidenceExpectation,
    ScenarioPackageEvidenceKind, ScenarioPackageIdentity, ScenarioPackageInitialState,
    ScenarioPackageReadbackFactories, ScenarioPackageRegistration, ScenarioPackageRulesetReference,
};
use rulebench_rules::*;

const PACKAGE_ID: &str = "asha-rulebench.watchtower-skirmish";
const SCENARIO_ID: &str = "watchtower-skirmish";

pub fn registration() -> ScenarioPackageRegistration {
    ScenarioPackageRegistration::new(
        scenario_package(),
        ScenarioPackageReadbackFactories {
            catalog_cases: scenario_catalog_cases,
            ruleset_catalog_readout,
            content_validation_readouts,
            session_transcripts: Vec::new,
            control_history_readouts: Vec::new,
            script_readouts: Vec::new,
            automatic_run_readouts: Vec::new,
            automatic_run_replay_readouts: Vec::new,
        },
    )
}

pub fn watchtower_skirmish_scenario() -> RulebenchScenario {
    let mut scenario = super::hexing_bolt::hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: SCENARIO_ID.to_string(),
        title: "Ruined Watchtower Skirmish".to_string(),
        summary: "Two allies and two enemies reposition around a blocked watchtower cell before trading content-defined attacks.".to_string(),
        seed_label: "manual-or-authority-generated".to_string(),
    };
    scenario.grid = Grid {
        width: 10,
        height: 7,
        cells: vec![
            cell(1, 2, "clear"),
            cell(1, 4, "clear"),
            cell(8, 2, "clear"),
            cell(8, 4, "clear"),
            cell(5, 3, "blocked"),
            cell(4, 3, "difficult"),
        ],
    };

    let mut scout = scenario.combatants[0].clone();
    rename_combatant(&mut scout, "entity-scout", "entity.scout", "Scout");
    scout.initiative = 13;
    scout.position = GridPosition { x: 1, y: 4 };
    scout
        .base_ability_ids
        .retain(|id| id != "ability.hexing-bolt");

    let mut bruiser = scenario.combatants[1].clone();
    rename_combatant(&mut bruiser, "entity-bruiser", "entity.bruiser", "Bruiser");
    bruiser.initiative = 8;
    bruiser.position = GridPosition { x: 8, y: 4 };

    scenario.combatants[0].position = GridPosition { x: 1, y: 2 };
    scenario.combatants[0].initiative = 16;
    scenario.combatants[1].position = GridPosition { x: 8, y: 2 };
    scenario.combatants[1].initiative = 11;
    scenario.combatants = vec![
        scenario.combatants[0].clone(),
        scout,
        scenario.combatants[1].clone(),
        bruiser,
    ];

    let mut scout_entity = scenario.entities[0].clone();
    scout_entity.id = "entity.scout".to_string();
    scout_entity.name = "Scout".to_string();
    let mut bruiser_entity = scenario.entities[1].clone();
    bruiser_entity.id = "entity.bruiser".to_string();
    bruiser_entity.name = "Bruiser".to_string();
    scenario.entities.extend([scout_entity, bruiser_entity]);

    let ally_targets = ["entity-raider", "entity-bruiser"];
    let enemy_targets = ["entity-adept", "entity-scout"];
    let base_actions = scenario.actions.clone();
    scenario.actions.clear();
    scenario
        .actions
        .push(with_targets(base_actions[0].clone(), &ally_targets));
    add_actor_actions(
        &mut scenario.actions,
        &base_actions,
        "entity-adept",
        "entity-adept",
        &ally_targets,
        "Focus Shot",
    );
    add_actor_actions(
        &mut scenario.actions,
        &base_actions,
        "entity-adept",
        "entity-scout",
        &ally_targets,
        "Scout Bow",
    );
    add_actor_actions(
        &mut scenario.actions,
        &base_actions,
        "entity-raider",
        "entity-raider",
        &enemy_targets,
        "Raider Blade",
    );
    add_actor_actions(
        &mut scenario.actions,
        &base_actions,
        "entity-raider",
        "entity-bruiser",
        &enemy_targets,
        "Bruiser Club",
    );
    scenario.selected_action = scenario.actions[0].clone();
    scenario
}

fn add_actor_actions(
    actions: &mut Vec<ActionDefinition>,
    templates: &[ActionDefinition],
    source: &str,
    actor: &str,
    targets: &[&str],
    attack_name: &str,
) {
    for template in templates
        .iter()
        .filter(|action| action.actor_id == source && action.ability_id != "ability.hexing-bolt")
    {
        let mut action = with_targets(template.clone(), targets);
        action.actor_id = actor.to_string();
        action.id = action.id.replace(source, actor);
        if action.ability_id == "ability.basic-attack" {
            action.name = attack_name.to_string();
        }
        actions.push(action);
    }
}

fn with_targets(mut action: ActionDefinition, targets: &[&str]) -> ActionDefinition {
    let targets = targets
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    action.targeting.target_ids = targets.clone();
    action.targeting.visible_target_ids = targets;
    action
}

fn rename_combatant(combatant: &mut Combatant, id: &str, entity_id: &str, name: &str) {
    combatant.id = id.to_string();
    combatant.entity_id = entity_id.to_string();
    combatant.name = name.to_string();
    combatant.is_actor = false;
}

fn cell(x: u32, y: u32, terrain: &str) -> GridCell {
    GridCell {
        position: GridPosition { x, y },
        terrain_tags: vec![terrain.to_string()],
    }
}

fn scenario_package() -> ScenarioPackage {
    let scenario = watchtower_skirmish_scenario();
    ScenarioPackage {
        identity: ScenarioPackageIdentity {
            id: PACKAGE_ID.to_string(),
            version: "0.1.0".to_string(),
        },
        display: ScenarioPackageDisplayMetadata {
            title: "Ruined Watchtower Skirmish".to_string(),
            summary: scenario.metadata.summary.clone(),
            tags: vec![
                "combat".to_string(),
                "human-testing".to_string(),
                "skirmish".to_string(),
            ],
        },
        ruleset: ScenarioPackageRulesetReference {
            id: scenario.selected_ruleset_id.clone(),
            version: scenario
                .selected_ruleset()
                .expect("skirmish selects ruleset")
                .version
                .clone(),
        },
        content_references: vec![ScenarioPackageContentReference {
            id: "asha-rulebench.watchtower.content".to_string(),
            version: "0.1.0".to_string(),
        }],
        initial_state: ScenarioPackageInitialState {
            participant_ids: scenario
                .combatants
                .iter()
                .map(|value| value.id.clone())
                .collect(),
            scenario,
        },
        scripts: Vec::new(),
        expected_evidence: vec![ScenarioPackageEvidenceExpectation {
            id: SCENARIO_ID.to_string(),
            kind: ScenarioPackageEvidenceKind::CatalogCase,
        }],
        golden_manifest: FixtureGoldenManifest {
            package_id: PACKAGE_ID.to_string(),
            artifacts: vec![FixtureGoldenArtifact {
                id: SCENARIO_ID.to_string(),
                kind: FixtureGoldenArtifactKind::ScenarioCatalog,
                check_command: "pnpm run catalog:check".to_string(),
            }],
        },
    }
}

fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    let scenario = watchtower_skirmish_scenario();
    vec![ScenarioCatalogCase {
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
    }]
}

fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    let scenario = watchtower_skirmish_scenario();
    RulesetCatalogReadout {
        selected_ruleset_id: scenario.selected_ruleset_id,
        rulesets: scenario.rulesets,
    }
}

fn content_validation_readouts() -> Vec<ContentValidationReadout> {
    let scenario = watchtower_skirmish_scenario();
    vec![ContentValidationReadout {
        scenario_id: scenario.metadata.id.clone(),
        scenario_title: scenario.metadata.title.clone(),
        report: validate_scenario_content_report(&scenario),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skirmish_is_valid_and_exposes_four_participants_with_spatial_actions() {
        let scenario = watchtower_skirmish_scenario();
        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(scenario.combatants.len(), 4);
        assert_eq!(
            scenario
                .combatants
                .iter()
                .filter(|value| value.team == Team::Ally)
                .count(),
            2
        );
        assert_eq!(
            scenario
                .combatants
                .iter()
                .filter(|value| value.team == Team::Enemy)
                .count(),
            2
        );
        assert!(scenario
            .grid
            .cells
            .iter()
            .any(|cell| cell.terrain_tags.iter().any(|tag| tag == "blocked")));
        for combatant in &scenario.combatants {
            assert!(scenario
                .actions
                .iter()
                .any(|action| action.actor_id == combatant.id && action.movement.is_some()));
            assert!(scenario
                .actions
                .iter()
                .any(|action| action.actor_id == combatant.id
                    && action.ability_id == "ability.basic-attack"));
        }
    }

    #[test]
    fn skirmish_supports_manual_and_generated_resolution() {
        let scenario = watchtower_skirmish_scenario();
        let intent = UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider");
        assert!(resolve_use_action(&scenario, intent.clone(), &[17, 5]).accepted);
        let mut session = CombatSessionState::new("generated-skirmish", scenario);
        let result = session.submit_intent_command(
            CombatSessionIntentCommandSpec::new(
                "generated-hit",
                "Generated hit",
                "Authority supplies rolls.",
                intent,
                Vec::new(),
            )
            .with_generated_rolls(41),
        );
        assert!(!result.generated_rolls.is_empty());
    }
}
