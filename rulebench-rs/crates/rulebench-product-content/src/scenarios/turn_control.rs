//! Independently registered second ruleset and package proof.

use crate::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogSummary,
    ScenarioOutcomeClass, ScenarioPackage, ScenarioPackageContentReference,
    ScenarioPackageDisplayMetadata, ScenarioPackageIdentity, ScenarioPackageInitialState,
    ScenarioPackageReadbackFactories, ScenarioPackageRegistration, ScenarioPackageRulesetReference,
};
use rpg_core::*;
use rpg_ir::*;
use rulebench_combat::*;
use rulebench_replay::*;

const PACKAGE_ID: &str = "asha-rulebench.objective-turn-control";
const PACKAGE_VERSION: &str = "0.1.0";
const FAILED_SAVE_CASE_ID: &str = "binding-glyph-failed-save";
const SAVED_CASE_ID: &str = "binding-glyph-saved";
const SESSION_ID: &str = "objective-turn-control-opening";
const AUTOMATIC_RUN_ID: &str = "objective-turn-control-automatic-run";
const AUTOMATIC_REPLAY_ID: &str = "objective-turn-control-automatic-replay";

pub fn registration() -> ScenarioPackageRegistration {
    ScenarioPackageRegistration::new(
        scenario_package(),
        ScenarioPackageReadbackFactories {
            catalog_cases: scenario_catalog_cases,
            ruleset_catalog_readout,
            content_validation_readouts,
            session_transcripts,
            control_history_readouts,
            script_readouts,
            automatic_run_readouts,
            automatic_run_replay_readouts,
        },
    )
}

pub fn scenario_package() -> ScenarioPackage {
    let scenario = scenario();
    ScenarioPackage {
        identity: ScenarioPackageIdentity {
            id: PACKAGE_ID.to_string(),
            version: PACKAGE_VERSION.to_string(),
        },
        display: ScenarioPackageDisplayMetadata {
            title: "Objective Turn Control Package".to_string(),
            summary:
                "A three-participant saving-throw scenario owned by the second compiled provider."
                    .to_string(),
            tags: vec![
                "combat".to_string(),
                "saving-throw".to_string(),
                "turn-control".to_string(),
            ],
        },
        ruleset: ScenarioPackageRulesetReference {
            id: scenario.selected_ruleset_id.clone(),
            version: scenario
                .selected_ruleset()
                .expect("objective package selects its provider ruleset")
                .version
                .clone(),
        },
        content_references: vec![ScenarioPackageContentReference {
            id: "asha-rulebench.objective-turn-control.content".to_string(),
            version: PACKAGE_VERSION.to_string(),
        }],
        initial_state: ScenarioPackageInitialState {
            participant_ids: scenario
                .combatants
                .iter()
                .map(|combatant| combatant.id.clone())
                .collect(),
            scenario,
        },
        scripts: Vec::new(),
    }
}

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    vec![
        catalog_case(
            FAILED_SAVE_CASE_ID,
            "Binding Glyph Failed Save",
            "Saboteur fails a Body saving throw; damage and Anchored commit.",
            ScenarioOutcomeClass::AcceptedHit,
            vec![5, 4],
        ),
        catalog_case(
            SAVED_CASE_ID,
            "Binding Glyph Saved",
            "Saboteur succeeds on the Body saving throw and avoids both effects.",
            ScenarioOutcomeClass::AcceptedMiss,
            vec![18],
        ),
    ]
}

pub fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    let scenario = scenario();
    RulesetCatalogReadout {
        selected_ruleset_id: scenario.selected_ruleset_id,
        rulesets: scenario.rulesets,
    }
}

pub fn content_validation_readouts() -> Vec<ContentValidationReadout> {
    let scenario = scenario();
    vec![ContentValidationReadout {
        scenario_id: scenario.metadata.id.clone(),
        scenario_title: scenario.metadata.title.clone(),
        report: validate_scenario_content_report(&scenario),
    }]
}

pub fn session_transcripts() -> Vec<CombatSessionTranscript> {
    let mut state = CombatSessionState::new(SESSION_ID, scenario());
    let readout = state.submit_command(CombatSessionCommandSpec::new(
        "warden-binding-glyph",
        "Warden binds Saboteur",
        "The second provider resolves a failed saving throw and commits its effects.",
        CommandOutcomeClass::AcceptedHit,
        intent(),
        vec![5, 4],
    ));
    vec![CombatSessionTranscript {
        summary: CombatSessionSummary {
            id: SESSION_ID.to_string(),
            title: "Objective Turn Control Opening".to_string(),
            summary: "A deterministic second-provider opening transcript.".to_string(),
            seed_label: "roll-stream:5,4".to_string(),
            steps: vec![readout.step.clone()],
        },
        steps: vec![readout],
    }]
}

pub fn control_history_readouts() -> Vec<CombatControlHistoryReadout> {
    Vec::new()
}

pub fn script_readouts() -> Vec<CombatSessionScriptReadout> {
    Vec::new()
}

pub fn automatic_run_readouts() -> Vec<CombatSessionAutomaticRunReadout> {
    vec![
        automatic_run_readout(
            AUTOMATIC_RUN_ID,
            CombatAutomationPolicySpec::first_accepted_candidate(),
        ),
        automatic_run_readout(
            "objective-turn-control-lowest-vitality-run",
            CombatAutomationPolicySpec::lowest_vitality_target(),
        ),
        automatic_run_readout(
            "objective-turn-control-objective-pressure-run",
            CombatAutomationPolicySpec::objective_side_pressure(),
        ),
    ]
}

pub fn automatic_run_replay_readouts() -> Vec<CombatSessionAutomaticRunReplayReadout> {
    automatic_run_readouts()
        .into_iter()
        .enumerate()
        .map(|(index, run)| {
            let spec = CombatSessionAutomaticRunSpec::new(
                run.id.clone(),
                run.title.clone(),
                run.summary.clone(),
                run.max_steps,
                vec![5, 4],
            )
            .with_policy(run.policy.clone());
            verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
                format!("{AUTOMATIC_REPLAY_ID}-{index}"),
                "Objective Turn Control Automatic Replay",
                "Replay verification for the second compiled provider's automatic path.",
                "objective-turn-control-automatic-replay-session",
                scenario(),
                spec,
                run.final_snapshot.current_state_fingerprint,
                run.final_snapshot.finalization,
                run.decision_kind,
                run.executed_step_count,
                run.policy_decisions,
                run.final_snapshot.action_resource_transition_log,
                run.final_snapshot.equipment_ledger,
                run.final_snapshot.class_build_ledger,
                run.final_snapshot.equipment_transition_log,
                run.final_snapshot.reaction_window_lifecycle_log,
                run.final_snapshot.reaction_audit_log,
                run.final_snapshot.modifier_duration_expiration_log,
            ))
        })
        .collect()
}

pub fn accepted_receipt() -> RulebenchReceipt {
    preview_use_action(&scenario(), intent(), &[5, 4])
}

pub fn replay_package() -> ReplayPackage {
    let scenario = scenario();
    let ruleset = scenario
        .selected_ruleset()
        .expect("second-provider replay selects a ruleset")
        .artifact_provenance();
    record_replay_package(
        "objective-turn-control-replay",
        CombatSessionCreateRequest::new("objective-turn-control-replay-session", scenario),
        ruleset,
        vec![
            ReplayCommandRecordingSpec::new(
                "binding-glyph",
                ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
                    "binding-glyph",
                    "Warden binds Saboteur",
                    "Replay executes the second provider's saving-throw action.",
                    intent(),
                    vec![5, 4],
                )),
            ),
            ReplayCommandRecordingSpec::new(
                "explicit-end",
                ReplayCommand::Control(CombatControlCommandSpec::explicit_end()),
            ),
        ],
    )
    .with_narration(ReplayNarration {
        title: "Objective Turn Control Replay".to_string(),
        summary: "Deterministic archived evidence for the second compiled provider.".to_string(),
        command_summaries: vec![
            "Warden resolves Binding Glyph through a failed saving throw.".to_string(),
            "Combat ends explicitly for archival review.".to_string(),
        ],
    })
}

pub fn scenario() -> RulebenchScenario {
    let mut scenario = super::hexing_bolt::hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: "three-participant-objective-control".to_string(),
        title: "Objective Turn Control".to_string(),
        summary: "Warden and Scout contest a Saboteur under the second compiled ruleset."
            .to_string(),
        seed_label: "roll-stream:5,4".to_string(),
    };
    scenario.content_pack_set = Some(content_pack_set_reference());
    let ruleset = crate::turn_control_ruleset();
    scenario.selected_ruleset_id = ruleset.id.clone();
    scenario.rulesets = vec![ruleset];

    scenario.entities[0].id = "entity.warden".to_string();
    scenario.entities[0].name = "Warden".to_string();
    scenario.entities[0].summary = "The objective-side controller.".to_string();
    scenario.entities[1].id = "entity.saboteur".to_string();
    scenario.entities[1].name = "Saboteur".to_string();
    scenario.entities[1].summary = "An invader resisting the binding glyph.".to_string();
    let mut scout_entity = scenario.entities[0].clone();
    scout_entity.id = "entity.scout".to_string();
    scout_entity.name = "Scout".to_string();
    scout_entity.summary = "A second objective-side participant.".to_string();
    scenario.entities.push(scout_entity);

    scenario.combatants[0].id = "entity-warden".to_string();
    scenario.combatants[0].entity_id = "entity.warden".to_string();
    scenario.combatants[0].name = "Warden".to_string();
    scenario.combatants[0].side_id = "wardens".to_string();
    scenario.combatants[0].initiative = 20;
    scenario.combatants[0].base_ability_ids = vec!["ability.binding-glyph".to_string()];
    scenario.combatants[1].id = "entity-saboteur".to_string();
    scenario.combatants[1].entity_id = "entity.saboteur".to_string();
    scenario.combatants[1].name = "Saboteur".to_string();
    scenario.combatants[1].side_id = "invaders".to_string();
    scenario.combatants[1].initiative = 10;
    let mut scout = scenario.combatants[0].clone();
    scout.id = "entity-scout".to_string();
    scout.entity_id = "entity.scout".to_string();
    scout.name = "Scout".to_string();
    scout.position = GridPosition { x: 2, y: 2 };
    scout.initiative = 15;
    scout.is_actor = false;
    scout.base_ability_ids.clear();
    scenario.combatants.push(scout);

    scenario.abilities.push(AbilityDefinition {
        id: "ability.binding-glyph".to_string(),
        name: "Binding Glyph".to_string(),
        kind: AbilityDefinitionKind::Spell,
        summary: "Forces a Body save, then damages and anchors on failure.".to_string(),
        tags: vec!["save".to_string(), "control".to_string()],
    });
    scenario.modifiers.push(ModifierDefinition {
        id: "anchored".to_string(),
        label: "Anchored".to_string(),
        summary: "A temporary control marker applied by Binding Glyph.".to_string(),
        default_tenure: ModifierTenure::Temporary,
        stacking_group: "anchored".to_string(),
        stacking_policy: ModifierStackingPolicy::Refresh,
        duration_policy: ModifierDurationPolicy::Turns(1),
        stat_adjustments: Vec::new(),
    });
    scenario.selected_ability_id = Some("ability.binding-glyph".to_string());
    let action = binding_glyph_action();
    scenario.actions = vec![action.clone()];
    scenario.selected_action = action;
    scenario
}

fn binding_glyph_action() -> ActionDefinition {
    ActionDefinition {
        id: "binding_glyph".to_string(),
        ruleset_id: crate::TURN_CONTROL_RULESET_ID.to_string(),
        ability_id: "ability.binding-glyph".to_string(),
        name: "Binding Glyph".to_string(),
        actor_id: "entity-warden".to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 6,
            visibility_requirement: VisibilityRequirement::Required,
            target_ids: vec!["entity-saboteur".to_string()],
            visible_target_ids: vec!["entity-saboteur".to_string()],
            operation_pipeline: None,
        },
        check: CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
            save_stat_id: "body".to_string(),
            difficulty_class: 14,
        }),
        hit: HitEffect {
            damage_bonus: 2,
            damage_type: "force".to_string(),
            modifier_id: "anchored".to_string(),
            modifier_label: "Anchored".to_string(),
            modifier_duration: "one turn".to_string(),
            operations: vec![
                HitEffectOperation::Damage(DamageEffectOperation {
                    damage_bonus: 2,
                    damage_type: "force".to_string(),
                }),
                HitEffectOperation::ApplyModifier(ModifierEffectOperation {
                    modifier_id: "anchored".to_string(),
                    modifier_label: "Anchored".to_string(),
                    modifier_duration: "one turn".to_string(),
                }),
            ],
        },
        resource_costs: vec![ActionResourceCost::standard_action()],
        movement: None,
        action_text: "Body saving throw against DC 14 at range 6.".to_string(),
        effect_text: "1d8 + 2 force damage and Anchored for one turn on a failed save.".to_string(),
    }
}

fn content_pack_set_reference() -> ContentPackSetReference {
    let root = ContentPackReference {
        id: "asha-rulebench.objective-turn-control.content".to_string(),
        version: PACKAGE_VERSION.to_string(),
        fingerprint: ContentFingerprint {
            algorithm: CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
            value: "bc0a7e7d9ec45120".to_string(),
        },
    };
    let packs = vec![root.clone()];
    let fingerprint = fingerprint_content_pack_set(&root, &packs);
    ContentPackSetReference {
        root,
        packs,
        fingerprint,
    }
}

fn catalog_case(
    id: &str,
    title: &str,
    summary: &str,
    outcome_class: ScenarioOutcomeClass,
    rolls: Vec<i32>,
) -> ScenarioCatalogCase {
    let mut scenario = scenario();
    scenario.metadata.id = id.to_string();
    scenario.metadata.title = title.to_string();
    scenario.metadata.summary = summary.to_string();
    scenario.metadata.seed_label = format!(
        "roll-stream:{}",
        rolls
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            seed_label: scenario.metadata.seed_label.clone(),
            outcome_class,
        },
        scenario,
        intent: intent(),
        roll_stream: rolls,
    }
}

fn intent() -> UseActionIntent {
    UseActionIntent::new("entity-warden", "binding_glyph", "entity-saboteur")
}

fn automatic_run_readout(
    run_id: &str,
    policy: CombatAutomationPolicySpec,
) -> CombatSessionAutomaticRunReadout {
    let mut state = CombatSessionState::new(run_id, scenario());
    state.run_automatic_combat(
        CombatSessionAutomaticRunSpec::new(
            run_id,
            "Objective Turn Control Automatic Run",
            "The generic policy executes the second provider's first accepted command.",
            16,
            vec![5, 4],
        )
        .with_policy(policy),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_is_independent_multi_participant_provider_evidence() {
        let package = scenario_package();

        assert!(package.validate().is_ok());
        assert_eq!(package.initial_state.participant_ids.len(), 3);
        assert_eq!(package.ruleset.id, crate::TURN_CONTROL_RULESET_ID);
        assert_eq!(package.content_references.len(), 1);
    }

    #[test]
    fn saving_throw_action_differs_structurally_and_replays() {
        let receipt = accepted_receipt();

        assert!(receipt.accepted);
        assert!(receipt.events.iter().any(|event| matches!(
            event,
            DomainEvent::SavingThrowResolved {
                outcome: SavingThrowOutcome::Failed,
                ..
            }
        )));
        assert!(receipt.events.iter().any(|event| matches!(
            event,
            DomainEvent::ModifierApplied { modifier_id, .. } if modifier_id == "anchored"
        )));
        assert!(automatic_run_replay_readouts()[0].accepted);
        assert!(verify_replay_package(&replay_package()).accepted);
    }

    #[test]
    fn replay_under_the_wrong_provider_rejects_without_execution() {
        let mut replay = replay_package();
        replay.ruleset = crate::hexing_bolt_ruleset().artifact_provenance();

        let verification = verify_replay_package(&replay);

        assert!(!verification.accepted);
        assert!(verification
            .package_validation
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code.code() == "incompatibleReplayRulesetProvenance" }));
    }
}
