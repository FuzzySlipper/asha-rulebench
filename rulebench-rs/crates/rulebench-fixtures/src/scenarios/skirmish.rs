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
            cell(2, 2, "clear"),
            cell(1, 4, "clear"),
            cell(8, 2, "clear"),
            cell(9, 2, "clear"),
            cell(8, 3, "clear"),
            cell(8, 4, "clear"),
            cell(9, 4, "clear"),
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
    scenario
        .combatants
        .get_mut(0)
        .expect("watchtower adept exists")
        .base_ability_ids
        .push("ability.storm-pulse".to_string());
    scenario.abilities.push(AbilityDefinition {
        id: "ability.storm-pulse".to_string(),
        name: "Storm Pulse".to_string(),
        kind: AbilityDefinitionKind::Spell,
        summary: "A bounded area spell proving operation-pipeline v2.".to_string(),
        tags: vec![
            "spell".to_string(),
            "area".to_string(),
            "stateful".to_string(),
        ],
    });
    let mut storm_pulse = with_targets(base_actions[0].clone(), &ally_targets);
    storm_pulse.id = "storm-pulse".to_string();
    storm_pulse.ability_id = "ability.storm-pulse".to_string();
    storm_pulse.name = "Storm Pulse".to_string();
    storm_pulse.targeting.target_kind = TargetKind::Area;
    storm_pulse.targeting.selection = TargetSelection::Multiple;
    storm_pulse.targeting.maximum_range = 8;
    storm_pulse.targeting.operation_pipeline = Some(OperationPipelineV2 {
        maximum_targets: 2,
        area: Some(AreaTargetingDeclaration {
            shape: AreaShape::ManhattanBurst,
            radius: 1,
        }),
        roll_policy: ActionRollPolicy::Shared,
        failure_policy: TargetFailurePolicy::Atomic,
        target_order: TargetOrderPolicy::CanonicalId,
    });
    storm_pulse.hit.operations = vec![
        HitEffectOperation::Damage(DamageEffectOperation {
            damage_bonus: 2,
            damage_type: "thunder".to_string(),
        }),
        HitEffectOperation::Move(MovementEffectOperation {
            maximum_distance: 1,
            movement_kind: MovementKind::Push,
        }),
        HitEffectOperation::ChangeResource(ResourceChangeEffectOperation {
            resource_id: "standard-action".to_string(),
            delta: -1,
        }),
    ];
    storm_pulse.hit.damage_bonus = 2;
    storm_pulse.hit.damage_type = "thunder".to_string();
    storm_pulse.hit.modifier_id.clear();
    storm_pulse.hit.modifier_label.clear();
    storm_pulse.hit.modifier_duration.clear();
    storm_pulse.action_text =
        "Shared Mind attack against enemies in a radius-1 Manhattan burst.".to_string();
    storm_pulse.effect_text =
        "Thunder damage, push 1, and consume one standard action atomically.".to_string();
    scenario.actions.push(storm_pulse);
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
    let base = watchtower_skirmish_scenario();
    let area = storm_pulse_area_conformance_scenario();
    let multiple = storm_pulse_multiple_conformance_scenario();
    let vitality = vitality_operations_conformance_scenario();
    vec![
        ScenarioCatalogCase {
            summary: ScenarioCatalogSummary {
                id: base.metadata.id.clone(),
                title: base.metadata.title.clone(),
                summary: base.metadata.summary.clone(),
                seed_label: base.metadata.seed_label.clone(),
                outcome_class: ScenarioOutcomeClass::AcceptedHit,
            },
            scenario: base,
            intent: UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            roll_stream: vec![17, 5],
        },
        accepted_catalog_case(
            area,
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 }),
        ),
        accepted_catalog_case(
            multiple,
            UseActionIntent::for_targets(
                "entity-adept",
                "storm-pulse",
                vec!["entity-raider".to_string(), "entity-bruiser".to_string()],
            ),
        ),
        accepted_catalog_case(
            vitality,
            UseActionIntent::new("entity-adept", "vitality-strike", "entity-raider"),
        ),
    ]
}

fn accepted_catalog_case(
    scenario: RulebenchScenario,
    intent: UseActionIntent,
) -> ScenarioCatalogCase {
    ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: scenario.metadata.id.clone(),
            title: scenario.metadata.title.clone(),
            summary: scenario.metadata.summary.clone(),
            seed_label: scenario.metadata.seed_label.clone(),
            outcome_class: ScenarioOutcomeClass::AcceptedHit,
        },
        scenario,
        intent,
        roll_stream: vec![17, 5],
    }
}

fn storm_pulse_area_conformance_scenario() -> RulebenchScenario {
    let mut scenario = watchtower_skirmish_scenario();
    scenario.metadata.id = "watchtower-storm-pulse-area".to_string();
    scenario.metadata.title = "Storm Pulse Area Conformance".to_string();
    scenario.metadata.summary =
        "Executes canonical area selection and the atomic stateful operation pipeline.".to_string();
    scenario.selected_action = scenario
        .actions
        .iter()
        .find(|action| action.id == "storm-pulse")
        .expect("storm pulse exists")
        .clone();
    scenario
}

fn storm_pulse_multiple_conformance_scenario() -> RulebenchScenario {
    let mut scenario = watchtower_skirmish_scenario();
    scenario.metadata.id = "watchtower-storm-pulse-multiple".to_string();
    scenario.metadata.title = "Storm Pulse Multiple-target Conformance".to_string();
    scenario.metadata.summary =
        "Executes canonical explicit multi-target selection through the atomic pipeline."
            .to_string();
    let action = scenario
        .actions
        .iter_mut()
        .find(|action| action.id == "storm-pulse")
        .expect("storm pulse exists");
    action.targeting.target_kind = TargetKind::Combatant;
    action.targeting.selection = TargetSelection::Multiple;
    action.targeting.maximum_range = 10;
    action.targeting.operation_pipeline = Some(OperationPipelineV2 {
        maximum_targets: 2,
        area: None,
        roll_policy: ActionRollPolicy::Shared,
        failure_policy: TargetFailurePolicy::Atomic,
        target_order: TargetOrderPolicy::CanonicalId,
    });
    scenario.selected_action = action.clone();
    scenario
}

fn vitality_operations_conformance_scenario() -> RulebenchScenario {
    let mut scenario = watchtower_skirmish_scenario();
    scenario.metadata.id = "watchtower-vitality-operations".to_string();
    scenario.metadata.title = "Vitality Operations Conformance".to_string();
    scenario.metadata.summary =
        "Executes capped healing and replace-only temporary vitality as Rust-owned effects."
            .to_string();
    let target = scenario
        .combatants
        .iter_mut()
        .find(|combatant| combatant.id == "entity-raider")
        .expect("raider exists");
    target.hit_points.current = 16;
    target.temporary_vitality = 4;
    let action = scenario
        .actions
        .iter_mut()
        .find(|action| action.id == "hexing_bolt")
        .expect("hexing bolt exists");
    action.id = "vitality-strike".to_string();
    action.name = "Vitality Strike".to_string();
    action
        .hit
        .operations
        .push(HitEffectOperation::Heal(HealingEffectOperation {
            healing_bonus: 99,
            healing_type: "vitality".to_string(),
        }));
    action
        .hit
        .operations
        .push(HitEffectOperation::GrantTemporaryVitality(
            TemporaryVitalityEffectOperation { vitality_bonus: 10 },
        ));
    scenario.selected_action = action.clone();
    scenario
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

    fn explicit_storm_scenario(
        roll_policy: ActionRollPolicy,
        maximum_targets: u32,
    ) -> RulebenchScenario {
        let mut scenario = watchtower_skirmish_scenario();
        let action = scenario
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse action exists");
        action.targeting.target_kind = TargetKind::Combatant;
        action.targeting.selection = TargetSelection::Multiple;
        action.targeting.maximum_range = 10;
        action.targeting.operation_pipeline = Some(OperationPipelineV2 {
            maximum_targets,
            area: None,
            roll_policy,
            failure_policy: TargetFailurePolicy::Atomic,
            target_order: TargetOrderPolicy::CanonicalId,
        });
        action.hit.operations = vec![HitEffectOperation::Damage(DamageEffectOperation {
            damage_bonus: 2,
            damage_type: "thunder".to_string(),
        })];
        scenario.selected_action = action.clone();
        scenario
    }

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

    #[test]
    fn storm_pulse_resolves_canonical_targets_and_stateful_effects_atomically() {
        let scenario = watchtower_skirmish_scenario();
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 }),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(
            receipt
                .target_results
                .iter()
                .map(|result| result.target_id.as_str())
                .collect::<Vec<_>>(),
            vec!["entity-bruiser", "entity-raider"]
        );
        assert!(receipt
            .target_results
            .iter()
            .all(|result| result.movement.is_some() && result.resource_changes.len() == 1));
        let projection = receipt
            .projection
            .expect("accepted v2 receipt projects state");
        assert_eq!(
            projection
                .combatants
                .iter()
                .find(|combatant| combatant.id == "entity-bruiser")
                .expect("bruiser projected")
                .position,
            GridPosition { x: 9, y: 4 }
        );
        assert_eq!(
            projection
                .combatants
                .iter()
                .find(|combatant| combatant.id == "entity-raider")
                .expect("raider projected")
                .position,
            GridPosition { x: 9, y: 2 }
        );
    }

    #[test]
    fn storm_pulse_rolls_back_every_target_when_one_effect_destination_is_blocked() {
        let mut scenario = watchtower_skirmish_scenario();
        scenario
            .grid
            .cells
            .iter_mut()
            .find(|cell| cell.position == GridPosition { x: 9, y: 2 })
            .expect("raider push cell exists")
            .terrain_tags = vec!["blocked".to_string()];
        let initial = CombatState::from_scenario(&scenario).project("initial");
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 }),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::EffectMovementDestinationBlocked)
        );
        assert!(receipt.events.is_empty());
        let rejected = receipt
            .projection
            .expect("rejection projects unchanged state");
        assert_eq!(rejected.combatants, initial.combatants);
    }

    #[test]
    fn storm_pulse_rejects_missing_and_out_of_bounds_resources_atomically() {
        let mut out_of_bounds = watchtower_skirmish_scenario();
        let operation = out_of_bounds
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists")
            .hit
            .operations
            .iter_mut()
            .find_map(|operation| match operation {
                HitEffectOperation::ChangeResource(operation) => Some(operation),
                _ => None,
            })
            .expect("storm pulse resource operation exists");
        operation.delta = 1;
        let intent =
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 });
        let receipt = resolve_use_action(&out_of_bounds, intent.clone(), &[17, 5]);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::EffectResourceOutOfBounds)
        );
        assert!(receipt.events.is_empty());

        out_of_bounds
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists")
            .hit
            .operations
            .iter_mut()
            .find_map(|operation| match operation {
                HitEffectOperation::ChangeResource(operation) => Some(operation),
                _ => None,
            })
            .expect("storm pulse resource operation exists")
            .resource_id = "missing-resource".to_string();
        let receipt = resolve_use_action(&out_of_bounds, intent, &[17, 5]);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::EffectResourceMissing)
        );
        assert!(receipt.events.is_empty());
    }

    #[test]
    fn storm_pulse_session_snapshot_retains_effect_resources_and_replayable_intent() {
        let scenario = watchtower_skirmish_scenario();
        let mut session = CombatSessionState::new("storm-pulse-session", scenario);
        let options = session.current_actor_options();
        let storm_option = options
            .actions
            .iter()
            .find(|action| action.action_id == "storm-pulse")
            .expect("storm pulse affordance exists");
        assert!(storm_option.target_set_options.iter().any(|target_set| {
            target_set.target_cell == Some(GridPosition { x: 8, y: 3 })
                && target_set.target_ids
                    == vec!["entity-bruiser".to_string(), "entity-raider".to_string()]
        }));
        let candidates = session.current_actor_command_candidates();
        assert!(candidates.candidates.iter().any(|candidate| {
            candidate.action_id == "storm-pulse"
                && candidate.accepted
                && candidate.intent.target_cell.is_some()
        }));
        let result = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "storm-pulse-command",
            "Storm Pulse",
            "Resolve the bounded area operation through the session owner.",
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 }),
            vec![17, 5],
        ));

        assert!(result.receipt.accepted);
        assert_eq!(result.command.target_id, "entity-bruiser");
        assert_eq!(
            result.receipt.intent.target_cell,
            Some(GridPosition { x: 8, y: 3 })
        );
        for target_id in ["entity-bruiser", "entity-raider"] {
            let resources = result
                .action_resource_ledger
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == target_id)
                .expect("target resource ledger exists");
            assert_eq!(
                resources
                    .resources
                    .iter()
                    .find(|resource| resource.resource_id == "standard-action")
                    .expect("standard action exists")
                    .current,
                0
            );
        }
        let actor_resources = result
            .action_resource_ledger
            .combatants
            .iter()
            .find(|combatant| combatant.combatant_id == "entity-adept")
            .expect("actor resource ledger exists");
        assert_eq!(
            actor_resources
                .resources
                .iter()
                .find(|resource| resource.resource_id == "standard-action")
                .expect("actor standard action exists")
                .current,
            0
        );
        assert!(
            result
                .receipt
                .events
                .iter()
                .filter(|event| matches!(event, DomainEvent::ResourceChanged { .. }))
                .count()
                == 2
        );
        let snapshot = session.snapshot();
        let transitions = &snapshot.action_resource_transition_log;
        assert_eq!(
            transitions
                .iter()
                .filter(|entry| {
                    entry.transition_kind == ActionResourceTransitionKind::ChangedByEffect
                })
                .count(),
            2
        );
        assert_eq!(
            transitions
                .iter()
                .filter(|entry| entry.transition_kind == ActionResourceTransitionKind::Spent)
                .count(),
            1
        );
    }

    #[test]
    fn storm_pulse_reaction_window_suspends_and_resumes_the_whole_owner_transaction() {
        let mut scenario = watchtower_skirmish_scenario();
        let action = scenario
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists");
        action
            .hit
            .operations
            .push(HitEffectOperation::OpenReactionWindow(
                ReactionHookEffectOperation {
                    hook_id: "storm-pulse.before-effect".to_string(),
                    window: ReactionWindow::BeforeEffect,
                    eligible_reactor_ids: vec![
                        "entity-bruiser".to_string(),
                        "entity-raider".to_string(),
                    ],
                    options: vec![
                        ReactionOptionDeclaration {
                            id: "bruiser-ward".to_string(),
                            reactor_id: "entity-bruiser".to_string(),
                            opens_nested_window: false,
                        },
                        ReactionOptionDeclaration {
                            id: "raider-ward".to_string(),
                            reactor_id: "entity-raider".to_string(),
                            opens_nested_window: false,
                        },
                    ],
                    maximum_nested_depth: 0,
                },
            ));
        scenario.selected_action = action.clone();
        assert!(validate_scenario_content(&scenario).is_empty());
        let initial = CombatState::from_scenario(&scenario).project("initial");
        let mut session = CombatSessionState::new("storm-reaction", scenario);
        let submitted = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "storm-reaction-command",
            "Storm Pulse reaction",
            "Suspend the complete v2 effect owner before commit.",
            UseActionIntent::for_area("entity-adept", "storm-pulse", GridPosition { x: 8, y: 3 }),
            vec![17, 5],
        ));
        assert!(submitted.receipt.accepted);
        assert_eq!(submitted.state_after.combatants, initial.combatants);
        let pending = session.snapshot();
        assert_eq!(pending.current_state.combatants, initial.combatants);
        assert_eq!(
            session
                .preflight_command(UseActionIntent::for_area(
                    "entity-adept",
                    "storm-pulse",
                    GridPosition { x: 8, y: 3 },
                ))
                .decision_kind,
            CommandPreflightDecisionKind::RejectedByReactionWindow
        );
        let root = pending
            .current_reaction_window
            .expect("storm reaction window opens");
        let first = session
            .submit_reaction_command(ReactionCommandSpec::pass(root.id.clone(), "entity-raider"));
        assert!(
            first.accepted,
            "reaction rejected: {first:?}; root: {root:?}"
        );
        let resumed =
            session.submit_reaction_command(ReactionCommandSpec::pass(root.id, "entity-bruiser"));
        assert!(resumed.resumed_pending_resolution);

        let committed = session.snapshot();
        assert!(committed.current_reaction_window.is_none());
        assert_eq!(
            committed
                .current_state
                .combatants
                .iter()
                .find(|combatant| combatant.id == "entity-bruiser")
                .expect("bruiser remains projected")
                .position,
            GridPosition { x: 9, y: 4 }
        );
        assert_eq!(
            committed
                .action_resource_transition_log
                .iter()
                .filter(|entry| {
                    entry.transition_kind == ActionResourceTransitionKind::ChangedByEffect
                })
                .count(),
            2
        );
    }

    #[test]
    fn storm_pulse_replay_verifies_and_classifies_target_cell_drift() {
        let mut scenario = watchtower_skirmish_scenario();
        scenario.content_pack_set = Some(
            crate::content_import_examples()
                .into_iter()
                .find_map(|example| match example.outcome {
                    crate::ContentImportExampleOutcome::Accepted(imported) => {
                        Some(imported.resolved_set.reference)
                    }
                    crate::ContentImportExampleOutcome::Rejected { .. } => None,
                })
                .expect("fixture content import includes an accepted pack set"),
        );
        let ruleset = scenario
            .selected_ruleset()
            .expect("watchtower ruleset exists")
            .artifact_provenance();
        let package = record_replay_package(
            "storm-pulse-replay",
            CombatSessionCreateRequest::new("storm-pulse-replay-session", scenario),
            ruleset,
            vec![ReplayCommandRecordingSpec::new(
                "storm-pulse",
                ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
                    "storm-pulse",
                    "Storm Pulse",
                    "Replay the canonical area target set.",
                    UseActionIntent::for_area(
                        "entity-adept",
                        "storm-pulse",
                        GridPosition { x: 8, y: 3 },
                    ),
                    vec![17, 5],
                )),
            )],
        );

        let initial_verification = verify_replay_package(&package);
        assert!(
            initial_verification.accepted,
            "initial replay mismatch: {:?}; diagnostics: {:?}",
            initial_verification.mismatch, initial_verification.package_validation.diagnostics
        );
        let mut drifted = package;
        let ReplayCommand::Intent(command) = &mut drifted.commands[0].command else {
            panic!("storm pulse command is an intent");
        };
        command.intent.target_cell = Some(GridPosition { x: 8, y: 2 });
        let verification = verify_replay_package(&drifted);
        assert!(!verification.accepted);
        assert!(verification.mismatch.is_some());
    }

    #[test]
    fn explicit_multi_target_pipeline_classifies_duplicate_limit_defeat_and_range() {
        let target_ids = vec!["entity-raider".to_string(), "entity-bruiser".to_string()];
        let scenario = explicit_storm_scenario(ActionRollPolicy::Shared, 2);
        let duplicate = resolve_use_action(
            &scenario,
            UseActionIntent::for_targets(
                "entity-adept",
                "storm-pulse",
                vec!["entity-raider".to_string(), "entity-raider".to_string()],
            ),
            &[17, 5],
        );
        assert_eq!(
            duplicate.rejection,
            Some(RulebenchRejection::DuplicateTarget)
        );

        let limited = explicit_storm_scenario(ActionRollPolicy::Shared, 1);
        assert_eq!(
            resolve_use_action(
                &limited,
                UseActionIntent::for_targets("entity-adept", "storm-pulse", target_ids.clone()),
                &[17, 5],
            )
            .rejection,
            Some(RulebenchRejection::TargetLimitExceeded)
        );

        let mut defeated = scenario.clone();
        defeated
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider exists")
            .hit_points
            .current = 0;
        assert_eq!(
            resolve_use_action(
                &defeated,
                UseActionIntent::for_targets(
                    "entity-adept",
                    "storm-pulse",
                    vec!["entity-raider".to_string()],
                ),
                &[17, 5],
            )
            .rejection,
            Some(RulebenchRejection::TargetDefeated)
        );

        let mut ranged = scenario;
        ranged
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists")
            .targeting
            .maximum_range = 1;
        assert_eq!(
            resolve_use_action(
                &ranged,
                UseActionIntent::for_targets(
                    "entity-adept",
                    "storm-pulse",
                    vec!["entity-raider".to_string()],
                ),
                &[17, 5],
            )
            .rejection,
            Some(RulebenchRejection::TargetOutOfRange)
        );
    }

    #[test]
    fn per_target_rolls_preserve_miss_and_hit_evidence_in_canonical_order() {
        let scenario = explicit_storm_scenario(ActionRollPolicy::PerTarget, 2);
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::for_targets(
                "entity-adept",
                "storm-pulse",
                vec!["entity-raider".to_string(), "entity-bruiser".to_string()],
            ),
            &[2, 5, 17, 4],
        );

        assert!(
            receipt.accepted,
            "per-target rejection: {:?}",
            receipt.rejection
        );
        assert_eq!(receipt.target_results[0].target_id, "entity-bruiser");
        assert!(receipt.target_results[0].damage.is_none());
        assert_eq!(receipt.target_results[1].target_id, "entity-raider");
        assert!(receipt.target_results[1].damage.is_some());
        assert_eq!(receipt.roll_consumption.len(), 4);
        assert!(!receipt.roll_consumption[1].consumed);
        assert!(receipt.roll_consumption[3].consumed);
    }

    #[test]
    fn no_roll_shift_and_pull_use_typed_deterministic_destinations() {
        let mut shifted = explicit_storm_scenario(ActionRollPolicy::NoRoll, 2);
        shifted
            .grid
            .cells
            .extend([cell(8, 1, "clear"), cell(8, 3, "clear")]);
        let shifted_action = shifted
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists");
        shifted_action
            .hit
            .operations
            .push(HitEffectOperation::Move(MovementEffectOperation {
                maximum_distance: 1,
                movement_kind: MovementKind::Shift,
            }));
        let shifted_receipt = resolve_use_action(
            &shifted,
            UseActionIntent::for_targets(
                "entity-adept",
                "storm-pulse",
                vec!["entity-raider".to_string(), "entity-bruiser".to_string()],
            ),
            &[],
        );
        assert!(
            shifted_receipt.accepted,
            "no-roll rejection: {:?}",
            shifted_receipt.rejection
        );
        assert!(shifted_receipt.roll_consumption.is_empty());
        assert_eq!(
            shifted_receipt.target_results[0]
                .movement
                .as_ref()
                .expect("bruiser shifted")
                .to,
            GridPosition { x: 8, y: 3 }
        );
        assert_eq!(
            shifted_receipt.target_results[1]
                .movement
                .as_ref()
                .expect("raider shifted")
                .to,
            GridPosition { x: 8, y: 1 }
        );

        let mut pulled = explicit_storm_scenario(ActionRollPolicy::Shared, 2);
        pulled
            .grid
            .cells
            .extend([cell(7, 2, "clear"), cell(7, 4, "clear")]);
        pulled
            .actions
            .iter_mut()
            .find(|action| action.id == "storm-pulse")
            .expect("storm pulse exists")
            .hit
            .operations
            .push(HitEffectOperation::Move(MovementEffectOperation {
                maximum_distance: 1,
                movement_kind: MovementKind::Pull,
            }));
        let pulled_receipt = resolve_use_action(
            &pulled,
            UseActionIntent::for_targets(
                "entity-adept",
                "storm-pulse",
                vec!["entity-raider".to_string(), "entity-bruiser".to_string()],
            ),
            &[17, 5],
        );
        assert!(pulled_receipt.accepted);
        assert!(pulled_receipt.target_results.iter().all(|target| {
            target
                .movement
                .as_ref()
                .is_some_and(|movement| movement.to.x == 7)
        }));
    }
}
