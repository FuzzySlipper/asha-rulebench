use super::super::test_support::*;

#[test]
fn session_runtime_current_turn_action_usage_is_empty_initially() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let summary = session.current_turn_action_usage();

    assert_eq!(summary.round_number, 1);
    assert_eq!(summary.turn_index, 0);
    assert_eq!(summary.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(summary.used_action_count, 0);
    assert!(summary.used_action_ids.is_empty());
    assert!(summary.used_ability_ids.is_empty());
}

#[test]
fn session_runtime_current_actor_options_read_initial_action_and_target() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let options = session.current_actor_options();

    assert_eq!(options.round_number, 1);
    assert_eq!(options.turn_index, 0);
    assert_eq!(options.lifecycle_phase, CombatLifecyclePhase::Ready);
    assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
    assert!(!options.current_actor_defeated);
    assert!(options.available);
    assert_eq!(options.unavailable_reason, None);
    assert_eq!(options.actions.len(), 3);
    assert_eq!(options.actions[0].action_id, "hexing_bolt");
    assert_eq!(options.actions[0].ability_id, "ability.hexing-bolt");
    assert_eq!(options.actions[0].action_name, "Hexing Bolt");
    assert_eq!(options.actions[0].target_mode, ActionTargetMode::Entity);
    assert!(options.actions[0].destination_options.is_empty());
    assert_eq!(options.actions[0].target_options.len(), 1);
    assert_eq!(
        options.actions[0].target_options[0].target_id,
        "entity-raider"
    );
    assert_eq!(options.actions[0].target_options[0].target_name, "Raider");
    assert_eq!(options.actions[0].target_options[0].current_hit_points, 18);
    assert_eq!(options.actions[0].target_options[0].max_hit_points, 18);
}

#[test]
fn session_runtime_moves_across_multiple_commands_and_refreshes_on_next_turn() {
    let mut session = CombatSessionState::new("runtime-movement", hexing_bolt_fixture_scenario());

    let first = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "move-one",
        "Move one",
        "Move one cell.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 2, y: 1 },
        ),
        Vec::new(),
    ));
    let second = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "move-two",
        "Move two",
        "Move a second cell.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 3, y: 1 },
        ),
        Vec::new(),
    ));

    assert!(first.receipt.accepted);
    let stale = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "move-stale",
        "Move stale",
        "Reject a destination selected before the actor moved.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 3, y: 1 },
        )
        .with_observed_origin(GridPosition { x: 1, y: 1 }),
        Vec::new(),
    ));
    assert_eq!(
        stale.receipt.rejection,
        Some(RulebenchRejection::MovementStaleDestination)
    );
    assert!(second.receipt.accepted);
    assert_eq!(
        second.state_after.combatants[0].position,
        GridPosition { x: 3, y: 1 }
    );
    assert_eq!(second.state_after.combatants[0].movement_remaining, 4);
    assert!(second
        .receipt
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::PositionChanged { .. })));
    assert_ne!(
        second.audit_entry.state_before_fingerprint,
        second.audit_entry.state_after_fingerprint
    );

    let exhausted = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "move-exhausted",
        "Move exhausted",
        "Movement budget is insufficient.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 0, y: 3 },
        ),
        Vec::new(),
    ));
    assert_eq!(
        exhausted.receipt.rejection,
        Some(RulebenchRejection::MovementBudgetExhausted)
    );

    session.advance_turn();
    session.advance_turn();
    assert_eq!(
        session.snapshot().current_state.combatants[0].movement_remaining,
        6
    );
}

#[test]
fn session_runtime_rejects_blocked_occupied_out_of_bounds_and_exhausted_movement() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.grid.cells.push(GridCell {
        position: GridPosition { x: 2, y: 1 },
        terrain_tags: vec!["wall".to_string()],
    });
    let mut session = CombatSessionState::new("runtime-movement-rejections", scenario);

    let blocked = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "blocked",
        "Blocked",
        "Blocked destination.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 2, y: 1 },
        ),
        Vec::new(),
    ));
    let occupied = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "occupied",
        "Occupied",
        "Occupied destination.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 4, y: 1 },
        ),
        Vec::new(),
    ));
    let outside = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "outside",
        "Outside",
        "Outside destination.",
        UseActionIntent::for_cell(
            "entity-adept",
            "move.entity-adept",
            GridPosition { x: 99, y: 99 },
        ),
        Vec::new(),
    ));

    assert_eq!(
        blocked.receipt.rejection,
        Some(RulebenchRejection::MovementDestinationBlocked)
    );
    assert_eq!(
        occupied.receipt.rejection,
        Some(RulebenchRejection::MovementDestinationOccupied)
    );
    assert_eq!(
        outside.receipt.rejection,
        Some(RulebenchRejection::MovementOutOfBounds)
    );
    assert_eq!(
        session.snapshot().current_state.combatants[0].position,
        GridPosition { x: 1, y: 1 }
    );
}

#[test]
fn replay_reproduces_movement_and_basic_attack_evidence() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.content_pack_set = Some(
        content_import_examples()
            .into_iter()
            .find_map(|example| match example.outcome {
                ContentImportExampleOutcome::Accepted(imported) => {
                    Some(imported.resolved_set.reference)
                }
                ContentImportExampleOutcome::Rejected { .. } => None,
            })
            .expect("fixture content import includes an accepted pack set"),
    );
    let ruleset = scenario
        .selected_ruleset()
        .expect("fixture selects a ruleset")
        .artifact_provenance();
    let move_command = |id: &str, destination: GridPosition| {
        ReplayCommandRecordingSpec::new(
            id,
            ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
                id,
                "Replay movement",
                "Replay movement authority evidence.",
                UseActionIntent::for_cell("entity-adept", "move.entity-adept", destination),
                Vec::new(),
            )),
        )
    };
    let basic_attack = ReplayCommandRecordingSpec::new(
        "basic-attack-replay",
        ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
            "basic-attack-replay",
            "Focus Shot",
            "Replay a content-defined basic attack.",
            UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
            vec![14, 5],
        )),
    );
    let package = record_replay_package(
        "movement-replay",
        CombatSessionCreateRequest::new("movement-replay-session", scenario),
        ruleset,
        vec![
            move_command("move-accepted", GridPosition { x: 2, y: 1 }),
            move_command("move-rejected", GridPosition { x: 4, y: 1 }),
            basic_attack,
        ],
    );

    let verification = verify_replay_package(&package);
    let inspection = inspect_replay_package(&package);

    assert!(verification.accepted, "{verification:?}");
    assert_eq!(
        inspection.commands[0].expected,
        inspection.commands[0].actual
    );
    assert_eq!(
        inspection.commands[1].expected,
        inspection.commands[1].actual
    );
    assert_eq!(
        inspection.commands[2].expected,
        inspection.commands[2].actual
    );
    assert_eq!(
        inspection.commands[1].snapshot.current_state.combatants[0].position,
        GridPosition { x: 2, y: 1 }
    );
    assert_eq!(
        inspection.commands[2].snapshot.current_state.combatants[1]
            .hit_points
            .current,
        13
    );
}

#[test]
fn basic_attack_resolves_ranged_hit_miss_and_defeat_without_a_modifier() {
    let submit = |scenario: RulebenchScenario, id: &str, rolls: Vec<i32>| {
        let mut session = CombatSessionState::new(id, scenario);
        session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            id,
            "Focus Shot",
            "Resolve the content-defined ranged basic attack.",
            UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
            rolls,
        ))
    };

    let hit = submit(
        hexing_bolt_fixture_scenario(),
        "basic-ranged-hit",
        vec![14, 5],
    );
    let miss = submit(
        hexing_bolt_fixture_scenario(),
        "basic-ranged-miss",
        vec![1, 8],
    );
    let mut lethal_scenario = hexing_bolt_fixture_scenario();
    lethal_scenario.combatants[1].hit_points.current = 3;
    let lethal = submit(lethal_scenario, "basic-ranged-lethal", vec![14, 5]);

    assert!(hit.receipt.accepted);
    assert_eq!(hit.receipt.modifier, None);
    assert_eq!(hit.state_after.combatants[1].hit_points.current, 13);
    assert_eq!(
        miss.receipt.attack_roll.as_ref().map(|roll| roll.outcome),
        Some(AttackOutcome::Miss)
    );
    assert_eq!(miss.state_after.combatants[1].hit_points.current, 18);
    assert_eq!(lethal.state_after.combatants[1].hit_points.current, 0);
}

#[test]
fn basic_attack_resolves_melee_and_rejects_range_and_line_of_sight() {
    let mut melee_scenario = hexing_bolt_fixture_scenario();
    melee_scenario.combatants[1].position = GridPosition { x: 2, y: 1 };
    let mut melee_session = CombatSessionState::new("basic-melee", melee_scenario);
    melee_session.advance_turn();
    let melee = melee_session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "basic-melee-hit",
        "Raider Blade",
        "Resolve the content-defined melee basic attack.",
        UseActionIntent::new(
            "entity-raider",
            "basic-attack.entity-raider",
            "entity-adept",
        ),
        vec![14, 4],
    ));
    assert!(melee.receipt.accepted);
    assert_eq!(melee.state_after.combatants[0].hit_points.current, 20);

    let mut ranged_session =
        CombatSessionState::new("basic-out-of-range", hexing_bolt_fixture_scenario());
    ranged_session.advance_turn();
    let out_of_range = ranged_session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "basic-out-of-range",
        "Raider Blade",
        "Reject a melee target outside declared range.",
        UseActionIntent::new(
            "entity-raider",
            "basic-attack.entity-raider",
            "entity-adept",
        ),
        vec![14, 4],
    ));
    assert_eq!(
        out_of_range.receipt.rejection,
        Some(RulebenchRejection::TargetOutOfRange)
    );

    let mut blocked_scenario = hexing_bolt_fixture_scenario();
    blocked_scenario.actions[2]
        .targeting
        .visible_target_ids
        .clear();
    let mut blocked_session = CombatSessionState::new("basic-blocked", blocked_scenario);
    let blocked = blocked_session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "basic-blocked",
        "Focus Shot",
        "Reject a target outside declared line of sight.",
        UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
        vec![14, 5],
    ));
    assert_eq!(
        blocked.receipt.rejection,
        Some(RulebenchRejection::TargetNotVisible)
    );
}

#[test]
fn authority_generated_rolls_are_lazy_bounded_and_seed_reproducible() {
    let command = || {
        CombatSessionIntentCommandSpec::new(
            "generated-basic-attack",
            "Generated Focus Shot",
            "Authority materializes only rolls requested by deterministic resolution.",
            UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
            Vec::new(),
        )
        .with_generated_rolls(42)
    };
    let mut first = CombatSessionState::new("generated-first", hexing_bolt_fixture_scenario());
    let mut second = CombatSessionState::new("generated-second", hexing_bolt_fixture_scenario());

    let first_readout = first.submit_intent_command(command());
    let second_readout = second.submit_intent_command(command());

    assert_eq!(first_readout.roll_mode.code(), "authorityGenerated");
    assert_eq!(
        first_readout.generated_rolls,
        second_readout.generated_rolls
    );
    assert!(!first_readout.generated_rolls.is_empty());
    assert!(first_readout.generated_rolls.len() <= 2);
    assert!(first_readout
        .generated_rolls
        .iter()
        .all(|roll| match roll.request_kind {
            RollRequestKind::DamageRoll => (1..=8).contains(&roll.value),
            _ => (1..=20).contains(&roll.value),
        }));
    assert_eq!(
        first_readout.command.roll_stream,
        first_readout
            .generated_rolls
            .iter()
            .map(|roll| roll.value)
            .collect::<Vec<_>>()
    );
    assert!(first_readout
        .receipt
        .trace
        .iter()
        .any(|entry| entry.message == "Authority roll materialized."));
}

#[test]
fn supplied_rolls_classify_invalid_and_excess_values() {
    let command = |id: &str, rolls: Vec<i32>| {
        CombatSessionIntentCommandSpec::new(
            id,
            "Supplied Focus Shot",
            "Classify caller-supplied roll values.",
            UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
            rolls,
        )
    };
    let mut invalid_session =
        CombatSessionState::new("invalid-roll", hexing_bolt_fixture_scenario());
    let invalid = invalid_session.submit_intent_command(command("invalid-roll", vec![21, 5]));
    assert_eq!(
        invalid.receipt.rejection,
        Some(RulebenchRejection::InvalidRollValue)
    );

    let mut excess_session = CombatSessionState::new("excess-roll", hexing_bolt_fixture_scenario());
    let excess = excess_session.submit_intent_command(command("excess-roll", vec![20, 5, 7]));
    assert!(excess.receipt.accepted);
    assert_eq!(excess.receipt.roll_consumption.len(), 3);
    assert!(!excess.receipt.roll_consumption[2].consumed);
    assert_eq!(
        excess.receipt.roll_consumption[2].reason,
        "Excess roll value was not requested by resolution."
    );
}

#[test]
fn bounded_automatic_run_can_materialize_each_command_roll_stream() {
    let mut session = CombatSessionState::new("generated-run", hexing_bolt_fixture_scenario());

    let readout = session.run_automatic_combat(
        CombatSessionAutomaticRunSpec::new(
            "generated-run",
            "Generated run",
            "Run bounded combat without a pre-authored roll stream.",
            6,
            Vec::new(),
        )
        .with_generated_rolls(91),
    );

    assert!(matches!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded
            | CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps
    ));
    let submitted_steps = readout
        .steps
        .iter()
        .filter_map(|step| step.auto_candidate.as_ref())
        .filter_map(|execution| execution.submitted_step.as_ref())
        .collect::<Vec<_>>();
    assert!(!submitted_steps.is_empty());
    assert!(submitted_steps.iter().all(|step| {
        step.roll_mode.code() == "authorityGenerated" && !step.generated_rolls.is_empty()
    }));
}

#[test]
fn replay_records_generated_values_and_verifies_without_rerolling() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.content_pack_set = Some(
        content_import_examples()
            .into_iter()
            .find_map(|example| match example.outcome {
                ContentImportExampleOutcome::Accepted(imported) => {
                    Some(imported.resolved_set.reference)
                }
                ContentImportExampleOutcome::Rejected { .. } => None,
            })
            .expect("fixture content import includes an accepted pack set"),
    );
    let ruleset = scenario
        .selected_ruleset()
        .expect("fixture selects a ruleset")
        .artifact_provenance();
    let generated = CombatSessionIntentCommandSpec::new(
        "generated-replay-command",
        "Generated replay attack",
        "Record concrete generated values for replay.",
        UseActionIntent::new("entity-adept", "basic-attack.entity-adept", "entity-raider"),
        Vec::new(),
    )
    .with_generated_rolls(77);

    let package = record_replay_package(
        "generated-replay",
        CombatSessionCreateRequest::new("generated-replay-session", scenario),
        ruleset,
        vec![ReplayCommandRecordingSpec::new(
            "generated-replay-command",
            ReplayCommand::Intent(generated),
        )],
    );

    let ReplayCommand::Intent(recorded) = &package.commands[0].command else {
        panic!("recorded command remains an intent");
    };
    assert!(!recorded.roll_stream.is_empty());
    assert!(matches!(
        recorded.roll_mode,
        CommandRollMode::RecordedGenerated { seed: 77 }
    ));
    assert_eq!(package.evidence.randomness.len(), 1);
    assert!(matches!(
        package.evidence.randomness[0].source,
        ReplayRandomnessSource::Generated { seed: 77, .. }
    ));
    assert!(verify_replay_package(&package).accepted);
}

#[test]
fn session_runtime_projects_authoritative_legal_cell_affordances() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.grid.cells.push(GridCell {
        position: GridPosition { x: 2, y: 1 },
        terrain_tags: vec!["wall".to_string()],
    });
    let session = CombatSessionState::new("runtime-cell-options", scenario);

    let action = &session.current_actor_options().actions[1];

    assert_eq!(action.target_mode, ActionTargetMode::Cell);
    assert_eq!(action.destination_options.len(), 21);
    assert!(!action
        .destination_options
        .iter()
        .any(|option| option.position == GridPosition { x: 1, y: 1 }));
    assert!(!action
        .destination_options
        .iter()
        .any(|option| option.position == GridPosition { x: 2, y: 1 }));
    assert!(action
        .destination_options
        .iter()
        .all(|option| !option.reason.is_empty()));
}

#[test]
fn session_runtime_current_actor_options_snapshot_readback_uses_current_state() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    let snapshot = session.snapshot();

    assert!(snapshot.current_actor_options.available);
    assert_eq!(snapshot.current_actor_options.unavailable_reason, None);
    assert_eq!(
        snapshot.current_actor_options.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(snapshot.current_actor_options.actions.len(), 3);
    assert_eq!(
        snapshot.current_actor_options.actions[0].target_options[0].target_id,
        "entity-raider"
    );
    assert_eq!(
        snapshot.current_actor_options.actions[0].target_options[0].current_hit_points,
        9
    );
}

#[test]
fn session_runtime_current_actor_options_report_raider_actions_after_turn_advance() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();

    let options = session.current_actor_options();

    assert!(options.available);
    assert_eq!(options.current_actor_id, Some("entity-raider".to_string()));
    assert_eq!(options.unavailable_reason, None);
    assert_eq!(options.actions.len(), 2);
    assert_eq!(options.actions[0].action_id, "move.entity-raider");
    assert_eq!(options.actions[1].action_id, "basic-attack.entity-raider");
}

#[test]
fn session_runtime_current_actor_options_report_ended_combat_unavailable() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();

    let options = session.current_actor_options();

    assert_eq!(options.lifecycle_phase, CombatLifecyclePhase::Ended);
    assert!(!options.available);
    assert_eq!(
        options.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::CombatEnded)
    );
    assert_eq!(
        options.unavailable_reason.map(|reason| reason.code()),
        Some("combatEnded")
    );
    assert!(options.actions.is_empty());
}

#[test]
fn session_runtime_initial_turn_order_skips_defeated_combatants() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-defeated-actor", scenario);

    let options = session.current_actor_options();

    assert_eq!(options.current_actor_id, Some("entity-raider".to_string()));
    assert!(!options.current_actor_defeated);
    assert!(options.available);
    assert_eq!(options.unavailable_reason, None);
    assert_eq!(options.actions.len(), 2);
}

#[test]
fn session_runtime_current_actor_options_filter_defeated_visible_targets() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-defeated-target", scenario);

    let options = session.current_actor_options();

    assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
    assert!(!options.current_actor_defeated);
    assert!(options.available);
    assert_eq!(options.unavailable_reason, None);
    assert_eq!(options.actions.len(), 3);
    assert_eq!(options.actions[0].action_id, "hexing_bolt");
    assert!(options.actions[0].target_options.is_empty());
    assert!(!options.actions[1].destination_options.is_empty());
}

#[test]
fn session_runtime_command_candidates_read_initial_current_actor_intents() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let candidates = session.current_actor_command_candidates();

    assert!(candidates.available);
    assert_eq!(candidates.round_number, 1);
    assert_eq!(candidates.turn_index, 0);
    assert_eq!(candidates.lifecycle_phase, CombatLifecyclePhase::Ready);
    assert_eq!(
        candidates.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert!(!candidates.current_actor_defeated);
    assert_eq!(candidates.unavailable_reason, None);
    assert_eq!(candidates.candidates.len(), 2);

    let candidate = &candidates.candidates[0];
    assert_eq!(
        candidate.intent,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
    );
    assert_eq!(candidate.action_id, "hexing_bolt");
    assert_eq!(candidate.ability_id, "ability.hexing-bolt");
    assert_eq!(candidate.target_id, "entity-raider");
    assert_eq!(candidate.target_name, "Raider");
    assert_eq!(candidate.target_current_hit_points, 18);
    assert_eq!(candidate.target_max_hit_points, 18);
    assert!(candidate.accepted);
    assert_eq!(
        candidate.decision_kind,
        CommandPreflightDecisionKind::Accepted
    );
    assert_eq!(candidate.decision_kind.code(), "accepted");
    assert_eq!(candidate.rejection, None);
    assert_eq!(
        candidate
            .target_legality
            .as_ref()
            .map(|legality| legality.accepted),
        Some(true)
    );
    assert_eq!(
        candidate.reason,
        "Command is admissible before roll resolution."
    );
}

#[test]
fn session_runtime_command_candidates_read_current_state_after_hit() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    let candidates = session.current_actor_command_candidates();

    assert!(!candidates.available);
    assert!(candidates.candidates.is_empty());
    assert_eq!(candidates.unavailable_reason, None);
}

#[test]
fn session_runtime_command_candidates_report_no_candidates_when_unavailable() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();

    let candidates = session.current_actor_command_candidates();

    assert!(!candidates.available);
    assert_eq!(
        candidates.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(candidates.unavailable_reason, None);
    assert!(candidates.candidates.is_empty());
}

#[test]
fn session_runtime_command_candidates_report_ended_combat_unavailable() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();

    let candidates = session.current_actor_command_candidates();

    assert!(!candidates.available);
    assert_eq!(candidates.lifecycle_phase, CombatLifecyclePhase::Ended);
    assert_eq!(
        candidates.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::CombatEnded)
    );
    assert!(candidates.candidates.is_empty());
}

#[test]
fn session_runtime_command_candidates_are_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let before_candidates = session.snapshot();

    let candidates = session.current_actor_command_candidates();
    let after_candidates = session.snapshot();

    assert!(!candidates.available);
    assert!(candidates.candidates.is_empty());
    assert_eq!(after_candidates, before_candidates);
    assert_eq!(session.next_step_index(), 1);
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(session.action_usage_log().len(), 1);
    assert_eq!(session.turn_transition_log().len(), 0);
    assert_eq!(session.lifecycle_transition_log().len(), 1);
}

#[test]
fn session_runtime_candidate_selection_plans_current_actor_command() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_plan = session.snapshot();

    let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "planned-hit",
        "Planned hit",
        "Caller selected the Hexing Bolt candidate.",
        "hexing_bolt",
        "entity-raider",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::Accepted
    );
    assert_eq!(plan.decision_kind.code(), "accepted");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(
        plan.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(plan.rejection, None);
    assert_eq!(
        plan.reason,
        "Selected command candidate planned for deterministic submission."
    );

    let command = plan.command.as_ref().expect("accepted plan has command");
    assert_eq!(command.id, "planned-hit");
    assert_eq!(command.title, "Planned hit");
    assert_eq!(
        command.summary,
        "Caller selected the Hexing Bolt candidate."
    );
    assert_eq!(
        command.intent,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
    );
    assert_eq!(command.roll_stream, vec![17, 5]);
    assert_eq!(after_plan, before_plan);
}

#[test]
fn session_runtime_candidate_selection_rejects_unavailable_candidates() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let before_plan = session.snapshot();

    let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "planned-hit",
        "Planned hit",
        "Raider has no command candidates in this fixture.",
        "hexing_bolt",
        "entity-raider",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(!plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByUnavailableCandidates");
    assert_eq!(plan.current_actor_id, Some("entity-raider".to_string()));
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(plan.reason, "No command candidates are available.");
    assert_eq!(plan.command, None);
    assert_eq!(after_plan, before_plan);
}

#[test]
fn session_runtime_candidate_selection_rejects_missing_candidate() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_plan = session.snapshot();

    let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "planned-missing",
        "Planned missing",
        "Caller selected a target that is not in current candidates.",
        "hexing_bolt",
        "missing-target",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(!plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByMissingCandidate");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(plan.preflight_decision_kind, None);
    assert_eq!(plan.rejection, None);
    assert_eq!(
        plan.reason,
        "Selected command candidate is not available for the current actor."
    );
    assert_eq!(plan.command, None);
    assert_eq!(after_plan, before_plan);
}

#[test]
fn session_runtime_candidate_selection_omits_illegal_self_target() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].targeting.target_ids = vec!["entity-adept".to_string()];
    scenario.actions[0].targeting.visible_target_ids = vec!["entity-adept".to_string()];
    scenario.selected_action = scenario.actions[0].clone();
    let session = CombatSessionState::new("runtime-self-target-candidate", scenario);
    let candidates = session.current_actor_command_candidates();

    assert!(candidates.available);
    assert_eq!(candidates.candidates.len(), 1);
    assert_eq!(
        candidates.candidates[0].action_id,
        "basic-attack.entity-adept"
    );
    let before_plan = session.snapshot();

    let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "planned-self-target",
        "Planned self target",
        "Caller selected a visible but illegal self target.",
        "hexing_bolt",
        "entity-adept",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(!plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByMissingCandidate");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.preflight_decision_kind, None);
    assert_eq!(plan.rejection, None);
    assert_eq!(plan.command, None);
    assert_eq!(after_plan, before_plan);
}

#[test]
fn session_runtime_candidate_selection_plan_can_be_submitted() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "planned-hit",
        "Planned hit",
        "Caller selected the Hexing Bolt candidate.",
        "hexing_bolt",
        "entity-raider",
        vec![17, 5],
    ));
    let command = plan.command.expect("accepted plan has command");

    let readout = session.submit_intent_command(command);

    assert!(readout.receipt.accepted);
    assert_eq!(readout.step.id, "planned-hit");
    assert_eq!(
        readout.command,
        CommandAttempt {
            step_id: "planned-hit".to_string(),
            step_index: 0,
            actor_id: "entity-adept".to_string(),
            action_id: "hexing_bolt".to_string(),
            target_id: "entity-raider".to_string(),
            roll_stream: vec![17, 5],
            outcome_class: CommandOutcomeClass::AcceptedHit,
        }
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(session.action_usage_log().len(), 1);
    assert_eq!(
        session.snapshot().current_state.combatants[1]
            .hit_points
            .current,
        9
    );
}

#[test]
fn session_runtime_auto_candidate_plan_selects_first_accepted_candidate_read_only() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_plan = session.snapshot();

    let plan = session.plan_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
        "auto-hit",
        "Auto hit",
        "Rust selects the first accepted current actor command candidate.",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutoCandidateDecisionKind::Accepted
    );
    assert_eq!(plan.decision_kind.code(), "accepted");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.candidate_count, 2);
    assert_eq!(plan.accepted_candidate_count, 2);
    assert_eq!(plan.selected_action_id, Some("hexing_bolt".to_string()));
    assert_eq!(plan.selected_target_id, Some("entity-raider".to_string()));
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(
        plan.reason,
        "Policy firstAcceptedCandidate selected candidate 0 with deterministic score 0."
    );

    let selection = plan
        .selection
        .as_ref()
        .expect("accepted auto plan carries selection");
    assert_eq!(
        selection.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::Accepted
    );
    assert_eq!(
        selection
            .command
            .as_ref()
            .map(|command| command.intent.clone()),
        Some(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider"
        ))
    );
    assert_eq!(after_plan, before_plan);
    assert!(session.combat_log().is_empty());
    assert!(session.audit_log().is_empty());
}

#[test]
fn session_runtime_auto_candidate_submission_accepts_hit() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let execution =
        session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
            "auto-hit",
            "Auto hit",
            "Rust selects and submits the first accepted candidate.",
            vec![17, 5],
        ));

    assert!(execution.plan.accepted);
    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutoCandidateDecisionKind::Accepted
    );
    let submitted_step = execution
        .submitted_step
        .as_ref()
        .expect("accepted auto plan submits command");
    assert!(submitted_step.receipt.accepted);
    assert_eq!(submitted_step.step.id, "auto-hit");
    assert_eq!(
        submitted_step.command,
        CommandAttempt {
            step_id: "auto-hit".to_string(),
            step_index: 0,
            actor_id: "entity-adept".to_string(),
            action_id: "hexing_bolt".to_string(),
            target_id: "entity-raider".to_string(),
            roll_stream: vec![17, 5],
            outcome_class: CommandOutcomeClass::AcceptedHit,
        }
    );
    assert_eq!(
        submitted_step.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        submitted_step.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(session.action_usage_log().len(), 1);
    assert_eq!(
        session.snapshot().current_state.combatants[1]
            .hit_points
            .current,
        9
    );
}

#[test]
fn session_runtime_auto_candidate_rejects_when_no_candidate_is_accepted() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let first_execution =
        session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
            "auto-hit",
            "Auto hit",
            "Rust spends the current actor standard action.",
            vec![17, 5],
        ));
    assert!(first_execution.plan.accepted);
    let before_plan = session.snapshot();

    let plan = session.plan_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
        "auto-spent",
        "Auto spent",
        "Rust refuses to auto-submit when preflight rejects every candidate.",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(!plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByUnavailableCandidates");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.accepted_candidate_count, 0);
    assert_eq!(plan.selected_action_id, None);
    assert_eq!(plan.selected_target_id, None);
    assert_eq!(plan.selection, None);
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(plan.reason, "No command candidates are available.");
    assert_eq!(after_plan, before_plan);
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
}

#[test]
fn session_runtime_auto_candidate_submission_rejects_unavailable_candidates_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let before_execution = session.snapshot();

    let execution =
        session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
            "auto-unavailable",
            "Auto unavailable",
            "Rust refuses to auto-submit when no command candidates exist.",
            vec![17, 5],
        ));
    let after_execution = session.snapshot();

    assert!(!execution.plan.accepted);
    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates
    );
    assert_eq!(
        execution.plan.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(execution.plan.candidate_count, 0);
    assert_eq!(execution.plan.accepted_candidate_count, 0);
    assert_eq!(execution.plan.unavailable_reason, None);
    assert_eq!(execution.submitted_step, None);
    assert_eq!(after_execution, before_execution);
    assert!(session.combat_log().is_empty());
    assert!(session.audit_log().is_empty());
    assert_eq!(session.turn_transition_log().len(), 1);
}

#[test]
fn session_runtime_automatic_step_plans_candidate_submission_read_only() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_plan = session.snapshot();

    let plan = session.plan_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-hit",
        "Auto step hit",
        "Rust plans one automatic combat step.",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::SubmitCandidate
    );
    assert_eq!(plan.decision_kind.code(), "submitCandidate");
    assert_eq!(
        plan.operation_kind,
        Some(CombatSessionAutomaticStepOperationKind::SubmitCandidate)
    );
    assert_eq!(
        plan.operation_kind.map(|operation| operation.code()),
        Some("submitCandidate")
    );
    assert_eq!(plan.lifecycle_phase, CombatLifecyclePhase::Ready);
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(
        plan.combat_end_condition.condition_kind,
        CombatEndConditionKind::Ongoing
    );
    assert_eq!(
        plan.auto_candidate_plan
            .as_ref()
            .map(|candidate| candidate.accepted),
        Some(true)
    );
    assert_eq!(
        plan.reason,
        "Automatic combat step planned first accepted command candidate."
    );
    assert_eq!(after_plan, before_plan);
}

#[test]
fn session_runtime_automatic_step_executes_candidate_submission() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let execution = session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-hit",
        "Auto step hit",
        "Rust executes one automatic command candidate step.",
        vec![17, 5],
    ));

    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::SubmitCandidate
    );
    assert_eq!(execution.control, None);
    let auto_candidate = execution
        .auto_candidate
        .as_ref()
        .expect("candidate step has auto candidate execution");
    assert!(auto_candidate.plan.accepted);
    let submitted_step = auto_candidate
        .submitted_step
        .as_ref()
        .expect("accepted auto candidate submits command");
    assert_eq!(submitted_step.step.id, "auto-step-hit");
    assert!(submitted_step.receipt.accepted);
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(
        session.snapshot().current_state.combatants[1]
            .hit_points
            .current,
        9
    );
}

#[test]
fn session_runtime_automatic_step_advances_turn_when_no_candidate_is_accepted() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let first_hit =
        session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
            "auto-hit",
            "Auto hit",
            "Rust spends the current actor standard action.",
            vec![17, 5],
        ));
    assert!(first_hit.plan.accepted);
    let before_plan = session.snapshot();

    let plan = session.plan_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-advance",
        "Auto step advance",
        "Rust advances turn when no accepted candidate remains.",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::AdvanceTurn
    );
    assert_eq!(
        plan.operation_kind,
        Some(CombatSessionAutomaticStepOperationKind::AdvanceTurn)
    );
    assert_eq!(
        plan.auto_candidate_plan
            .as_ref()
            .map(|candidate| candidate.decision_kind),
        Some(CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates)
    );
    assert_eq!(after_plan, before_plan);

    let execution = session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-advance",
        "Auto step advance",
        "Rust advances turn when no accepted candidate remains.",
        vec![17, 5],
    ));

    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::AdvanceTurn
    );
    assert_eq!(execution.auto_candidate, None);
    let control = execution
        .control
        .as_ref()
        .expect("advance step has control readout");
    assert!(control.accepted);
    assert_eq!(control.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert_eq!(control.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(session.control_history().len(), 1);
    assert_eq!(session.combat_log().len(), 1);
}

#[test]
fn session_runtime_lethal_candidate_finalizes_without_a_follow_up_end_step() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
        "first-hit",
        "First hit",
        "Adept hits Raider once.",
        vec![17, 5],
    ));
    session.advance_turn();
    session.advance_turn();
    session.submit_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
        "second-hit",
        "Second hit",
        "Adept hits Raider a second time.",
        vec![17, 5],
    ));
    let before_plan = session.snapshot();

    let plan = session.plan_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-end",
        "Auto step end",
        "Rust conditionally ends combat when the end condition is met.",
        vec![17, 5],
    ));
    let after_plan = session.snapshot();

    assert!(!plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle
    );
    assert_eq!(plan.operation_kind, None);
    assert_eq!(
        plan.combat_end_condition.condition_kind,
        CombatEndConditionKind::NoActiveEnemies
    );
    assert_eq!(plan.auto_candidate_plan, None);
    assert_eq!(after_plan, before_plan);

    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.lifecycle_transition_log().len(), 2);
    let finalization = session.finalization().expect("lethal command finalizes");
    assert_eq!(
        finalization.trigger,
        LifecycleTransitionTrigger::ConditionalEnd
    );
    assert_eq!(finalization.outcome_kind, CombatOutcomeKind::Victory);
}

#[test]
fn session_runtime_automatic_step_rejects_ended_combat_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();
    let before_execution = session.snapshot();

    let execution = session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-ended",
        "Auto step ended",
        "Rust rejects automatic stepping after combat is ended.",
        vec![17, 5],
    ));
    let after_execution = session.snapshot();

    assert!(!execution.plan.accepted);
    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle
    );
    assert_eq!(execution.plan.operation_kind, None);
    assert_eq!(execution.control, None);
    assert_eq!(execution.auto_candidate, None);
    assert_eq!(after_execution, before_execution);
}

#[test]
fn session_runtime_automatic_step_can_be_invoked_until_combat_ends() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let mut decisions = Vec::new();

    for index in 0..4 {
        let execution = session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
            format!("auto-step-{index}"),
            format!("Auto step {index}"),
            "Rust applies one automatic combat step.",
            vec![17, 5],
        ));
        decisions.push(execution.plan.decision_kind);
    }

    assert_eq!(
        decisions,
        vec![
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn,
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn,
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
        ]
    );
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.combat_log().len(), 2);
    assert_eq!(session.audit_log().len(), 2);
    assert_eq!(session.control_history().len(), 2);
    assert_eq!(
        session.snapshot().combat_end_condition.condition_kind,
        CombatEndConditionKind::NoActiveEnemies
    );
}

#[test]
fn session_runtime_automatic_policy_is_deterministic_and_rejects_unsupported_inputs() {
    let first = CombatSessionState::new("policy-first", hexing_bolt_fixture_scenario());
    let second = CombatSessionState::new("policy-second", hexing_bolt_fixture_scenario());
    let spec = CombatSessionAutomaticStepSpec::new(
        "policy-step",
        "Policy step",
        "The same state and policy choose the same command.",
        vec![17, 5],
    );

    let first_plan = first.plan_automatic_step(spec.clone());
    let second_plan = second.plan_automatic_step(spec);
    assert_eq!(first_plan.policy_decision, second_plan.policy_decision);
    assert_eq!(
        first_plan.policy_decision.policy.id,
        FIRST_ACCEPTED_CANDIDATE_POLICY_ID
    );
    assert_eq!(
        first_plan.policy_decision.selected_action_id.as_deref(),
        Some("hexing_bolt")
    );
    assert_eq!(
        first_plan.policy_decision.selected_target_id.as_deref(),
        Some("entity-raider")
    );

    let mut multi_candidate_scenario = hexing_bolt_fixture_scenario();
    let mut second_action = multi_candidate_scenario.actions[0].clone();
    second_action.id = "second_bolt".to_string();
    second_action.name = "Second Bolt".to_string();
    multi_candidate_scenario.actions.push(second_action);
    let mut multi_candidate_session =
        CombatSessionState::new("policy-multiple", multi_candidate_scenario);
    let multi_execution =
        multi_candidate_session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
            "policy-multiple-step",
            "Multiple candidate policy step",
            "The policy records ordered candidates and submits the selected entry.",
            vec![17, 5],
        ));
    let evidence = &multi_execution.plan.policy_decision;
    assert_eq!(evidence.candidate_count, 3);
    assert_eq!(evidence.accepted_candidate_count, 3);
    assert_eq!(evidence.selected_candidate_index, Some(0));
    assert_eq!(
        evidence
            .candidates
            .iter()
            .map(|candidate| candidate.action_id.as_str())
            .collect::<Vec<_>>(),
        vec!["hexing_bolt", "basic-attack.entity-adept", "second_bolt",]
    );
    assert_eq!(
        multi_execution
            .auto_candidate
            .as_ref()
            .and_then(|execution| execution.submitted_step.as_ref())
            .map(|step| step.command.action_id.as_str()),
        Some("hexing_bolt")
    );

    let mut unsupported_id = CombatAutomationPolicySpec::first_accepted_candidate();
    unsupported_id.id = "dynamicCallback".to_string();
    let mut unsupported_id_session =
        CombatSessionState::new("unsupported-id", hexing_bolt_fixture_scenario());
    let before_id = unsupported_id_session.snapshot();
    let rejected_id = unsupported_id_session.run_automatic_combat(
        CombatSessionAutomaticRunSpec::new(
            "unsupported-id-run",
            "Unsupported id",
            "Rust rejects unknown policy ids.",
            8,
            vec![17, 5],
        )
        .with_policy(unsupported_id),
    );
    assert_eq!(
        rejected_id.decision_kind,
        CombatSessionAutomaticRunDecisionKind::RejectedByPolicy
    );
    assert!(rejected_id.policy_decisions.is_empty());
    assert_eq!(unsupported_id_session.snapshot(), before_id);

    let mut unsupported_version = CombatAutomationPolicySpec::first_accepted_candidate();
    unsupported_version.version += 1;
    let rejected_version = first.plan_automatic_step(
        CombatSessionAutomaticStepSpec::new(
            "unsupported-version-step",
            "Unsupported version",
            "Rust rejects unknown policy versions.",
            vec![17, 5],
        )
        .with_policy(unsupported_version),
    );
    assert_eq!(
        rejected_version.decision_kind,
        CombatSessionAutomaticStepDecisionKind::RejectedByPolicy
    );
    assert_eq!(
        rejected_version.policy_validation.code,
        CombatAutomationPolicyValidationCode::UnsupportedPolicyVersion
    );
}

#[test]
fn lowest_vitality_policy_selects_the_weakest_accepted_target_with_stable_evidence() {
    let mut scenario = hexing_bolt_fixture_scenario();
    let mut weaker_entity = scenario.entities[1].clone();
    weaker_entity.id = "entity-weakened-raider".to_string();
    weaker_entity.name = "Weakened Raider".to_string();
    scenario.entities.push(weaker_entity);

    let mut weaker_target = scenario.combatants[1].clone();
    weaker_target.id = "entity-weakened-raider".to_string();
    weaker_target.entity_id = "entity-weakened-raider".to_string();
    weaker_target.name = "Weakened Raider".to_string();
    weaker_target.position = GridPosition { x: 2, y: 2 };
    weaker_target.initiative = 1;
    weaker_target.hit_points.current = 3;
    scenario.combatants.push(weaker_target);

    for action in scenario.actions.iter_mut().filter(|action| {
        action.actor_id == "entity-adept" && !action.targeting.target_ids.is_empty()
    }) {
        action
            .targeting
            .target_ids
            .push("entity-weakened-raider".to_string());
        action
            .targeting
            .visible_target_ids
            .push("entity-weakened-raider".to_string());
    }

    let session = CombatSessionState::new("policy-lowest-vitality", scenario);
    let before = session.snapshot();
    let plan = session.plan_auto_candidate_command(
        CombatSessionAutoCandidateCommandSpec::new(
            "policy-lowest-vitality-step",
            "Choose weakest target",
            "Rust ranks all accepted candidates by target vitality.",
            vec![17, 5],
        )
        .with_policy(CombatAutomationPolicySpec::lowest_vitality_target()),
    );

    assert!(plan.accepted);
    assert_eq!(plan.selected_candidate_index, Some(1));
    assert_eq!(
        plan.selected_target_id.as_deref(),
        Some("entity-weakened-raider")
    );
    assert_eq!(plan.candidate_order[0].target_current_hit_points, 18);
    assert_eq!(plan.candidate_order[1].target_current_hit_points, 3);
    assert!(plan.candidate_order[1].policy_score < plan.candidate_order[0].policy_score);
    assert_eq!(plan.candidate_order[1].target_side_id, "enemy");
    assert_eq!(session.snapshot(), before);
}

#[test]
fn objective_side_policy_rejects_incompatible_rulesets_before_mutation() {
    let hexing = CombatSessionState::new("policy-objective-hexing", hexing_bolt_fixture_scenario());
    let before = hexing.snapshot();
    let rejected = hexing.plan_automatic_step(
        CombatSessionAutomaticStepSpec::new(
            "policy-objective-rejected",
            "Reject missing objective",
            "The policy requires an objective-side ruleset declaration.",
            vec![17, 5],
        )
        .with_policy(CombatAutomationPolicySpec::objective_side_pressure()),
    );

    assert_eq!(
        rejected.decision_kind,
        CombatSessionAutomaticStepDecisionKind::RejectedByPolicy
    );
    assert_eq!(
        rejected.policy_validation.code,
        CombatAutomationPolicyValidationCode::IncompatibleRulesetCapability
    );
    assert_eq!(hexing.snapshot(), before);

    let turn_control = CombatSessionState::new(
        "policy-objective-turn-control",
        turn_control_fixture_scenario(),
    );
    let accepted = turn_control.plan_automatic_step(
        CombatSessionAutomaticStepSpec::new(
            "policy-objective-accepted",
            "Use declared objective",
            "The turn-control ruleset declares an objective side.",
            vec![17, 5],
        )
        .with_policy(CombatAutomationPolicySpec::objective_side_pressure()),
    );

    assert!(accepted.accepted);
    assert_eq!(
        accepted.policy_validation.code,
        CombatAutomationPolicyValidationCode::Accepted
    );
    assert_eq!(
        accepted.policy_decision.policy.id,
        OBJECTIVE_SIDE_PRESSURE_POLICY_ID
    );
}

#[test]
fn session_runtime_automatic_policy_can_stop_when_no_candidate_is_available() {
    let mut session = CombatSessionState::new("policy-stop", hexing_bolt_fixture_scenario());
    let policy = CombatAutomationPolicySpec::first_accepted_candidate()
        .with_no_candidate_behavior(CombatAutomationNoCandidateBehavior::StopRun);

    let readout = session.run_automatic_combat(
        CombatSessionAutomaticRunSpec::new(
            "policy-stop-run",
            "Policy stop run",
            "Rust stops when the policy cannot choose another command.",
            8,
            vec![17, 5],
        )
        .with_policy(policy),
    );

    assert!(readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::StoppedNoCandidate
    );
    assert_eq!(readout.executed_step_count, 2);
    assert_eq!(
        readout.steps[1].plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::StoppedNoCandidate
    );
    assert_eq!(readout.steps[1].plan.operation_kind, None);
    assert_eq!(readout.policy_decisions.len(), 2);
    assert_eq!(readout.policy_decisions[1].candidate_count, 0);
}

#[test]
fn session_runtime_automatic_run_completes_fixture_combat_within_bound() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.run_automatic_combat(CombatSessionAutomaticRunSpec::new(
        "auto-run",
        "Auto run",
        "Rust runs bounded automatic combat.",
        8,
        vec![17, 5],
    ));

    assert!(readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded
    );
    assert_eq!(readout.decision_kind.code(), "completedCombatEnded");
    assert_eq!(readout.max_steps, 8);
    assert_eq!(
        readout.policy,
        CombatAutomationPolicySpec::first_accepted_candidate()
    );
    assert_eq!(readout.executed_step_count, 4);
    assert_eq!(readout.policy_decisions.len(), 4);
    assert_eq!(
        readout
            .steps
            .iter()
            .map(|step| step.plan.decision_kind)
            .collect::<Vec<_>>(),
        vec![
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn,
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn,
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
        ]
    );
    assert_eq!(
        readout.final_snapshot.lifecycle.phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(readout.final_snapshot.combat_log.len(), 2);
    assert_eq!(readout.final_snapshot.audit_log.len(), 2);
    assert_eq!(session.control_history().len(), 2);
    assert_eq!(
        readout.final_snapshot.current_state.combatants[1]
            .hit_points
            .current,
        0
    );
    assert_eq!(session.snapshot(), readout.final_snapshot);
}

#[test]
fn session_runtime_automatic_run_stops_at_max_steps_before_completion() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.run_automatic_combat(CombatSessionAutomaticRunSpec::new(
        "auto-run-short",
        "Auto run short",
        "Rust stops bounded automatic combat at max steps.",
        2,
        vec![17, 5],
    ));

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps
    );
    assert_eq!(readout.decision_kind.code(), "stoppedAtMaxSteps");
    assert_eq!(readout.executed_step_count, 2);
    assert_eq!(
        readout.final_snapshot.lifecycle.phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(
        readout
            .final_snapshot
            .turn_order
            .current_actor_id
            .as_deref(),
        Some("entity-raider")
    );
    assert_eq!(readout.final_snapshot.combat_log.len(), 1);
    assert_eq!(session.control_history().len(), 1);
    assert_eq!(session.snapshot(), readout.final_snapshot);
}

#[test]
fn session_runtime_automatic_run_rejects_already_ended_combat_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();
    let before_run = session.snapshot();

    let readout = session.run_automatic_combat(CombatSessionAutomaticRunSpec::new(
        "auto-run-ended",
        "Auto run ended",
        "Rust rejects bounded automatic combat after end.",
        8,
        vec![17, 5],
    ));

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::RejectedByLifecycle
    );
    assert_eq!(readout.decision_kind.code(), "rejectedByLifecycle");
    assert_eq!(readout.executed_step_count, 0);
    assert!(readout.steps.is_empty());
    assert_eq!(readout.final_snapshot, before_run);
    assert_eq!(session.snapshot(), before_run);
}

#[test]
fn session_runtime_automatic_run_rejects_zero_step_limit_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_run = session.snapshot();

    let readout = session.run_automatic_combat(CombatSessionAutomaticRunSpec::new(
        "auto-run-zero",
        "Auto run zero",
        "Rust rejects bounded automatic combat with no allowed steps.",
        0,
        vec![17, 5],
    ));

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunDecisionKind::RejectedByStepLimit
    );
    assert_eq!(readout.decision_kind.code(), "rejectedByStepLimit");
    assert_eq!(readout.max_steps, 0);
    assert_eq!(readout.executed_step_count, 0);
    assert!(readout.steps.is_empty());
    assert_eq!(readout.final_snapshot, before_run);
    assert_eq!(session.snapshot(), before_run);
}

#[test]
fn session_runtime_automatic_run_replay_verifies_expected_final_evidence() {
    let scenario = hexing_bolt_fixture_scenario();
    let run_spec = CombatSessionAutomaticRunSpec::new(
        "auto-run-replay",
        "Auto run replay",
        "Rust replays bounded automatic combat.",
        8,
        vec![17, 5],
    );
    let mut expected_session =
        CombatSessionState::new("runtime-hexing-bolt-expected", scenario.clone());
    let expected_run = expected_session.run_automatic_combat(run_spec.clone());
    let readout = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
        "auto-run-replay-verification",
        "Auto run replay verification",
        "Rust verifies that replayed automatic combat matches expected final evidence.",
        "runtime-hexing-bolt-replay",
        scenario,
        run_spec,
        expected_run
            .final_snapshot
            .current_state_fingerprint
            .clone(),
        expected_run.final_snapshot.finalization.clone(),
        expected_run.decision_kind,
        expected_run.executed_step_count,
        expected_run.policy_decisions.clone(),
        expected_run
            .final_snapshot
            .action_resource_transition_log
            .clone(),
        expected_run.final_snapshot.equipment_ledger.clone(),
        expected_run.final_snapshot.class_build_ledger.clone(),
        expected_run.final_snapshot.equipment_transition_log.clone(),
        expected_run
            .final_snapshot
            .reaction_window_lifecycle_log
            .clone(),
        expected_run.final_snapshot.reaction_audit_log.clone(),
        expected_run
            .final_snapshot
            .modifier_duration_expiration_log
            .clone(),
    ));

    assert!(readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunReplayDecisionKind::Verified
    );
    assert_eq!(readout.decision_kind.code(), "verified");
    assert!(readout.final_state_fingerprint_matches);
    assert!(readout.finalization_matches);
    assert!(readout.run_decision_kind_matches);
    assert!(readout.executed_step_count_matches);
    assert!(readout.policy_decisions_match);
    assert!(readout.action_resource_transition_log_matches);
    assert!(readout.equipment_ledger_matches);
    assert!(readout.class_build_ledger_matches);
    assert!(readout.equipment_transition_log_matches);
    assert!(readout.reaction_window_lifecycle_log_matches);
    assert!(readout.reaction_audit_log_matches);
    assert!(readout.modifier_duration_expiration_log_matches);
    assert_eq!(
        readout.actual_final_state_fingerprint,
        expected_run.final_snapshot.current_state_fingerprint
    );
    assert_eq!(
        readout.replayed_run.decision_kind,
        CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded
    );
}

#[test]
fn session_runtime_automatic_run_replay_rejects_policy_decision_drift_alone() {
    let scenario = hexing_bolt_fixture_scenario();
    let run_spec = CombatSessionAutomaticRunSpec::new(
        "auto-run-policy-drift",
        "Auto run policy drift",
        "Rust compares recorded policy decisions independently of final state.",
        8,
        vec![17, 5],
    );
    let mut expected_session =
        CombatSessionState::new("runtime-policy-drift-expected", scenario.clone());
    let expected_run = expected_session.run_automatic_combat(run_spec.clone());
    let mut drifted_policy_decisions = expected_run.policy_decisions.clone();
    drifted_policy_decisions[0].selected_candidate_index = None;

    let readout = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
        "auto-run-policy-drift-verification",
        "Auto run policy drift verification",
        "Only the expected policy decision transcript differs.",
        "runtime-policy-drift-replay",
        scenario,
        run_spec,
        expected_run
            .final_snapshot
            .current_state_fingerprint
            .clone(),
        expected_run.final_snapshot.finalization.clone(),
        expected_run.decision_kind,
        expected_run.executed_step_count,
        drifted_policy_decisions,
        expected_run
            .final_snapshot
            .action_resource_transition_log
            .clone(),
        expected_run.final_snapshot.equipment_ledger.clone(),
        expected_run.final_snapshot.class_build_ledger.clone(),
        expected_run.final_snapshot.equipment_transition_log.clone(),
        expected_run
            .final_snapshot
            .reaction_window_lifecycle_log
            .clone(),
        expected_run.final_snapshot.reaction_audit_log.clone(),
        expected_run
            .final_snapshot
            .modifier_duration_expiration_log
            .clone(),
    ));

    assert!(!readout.accepted);
    assert!(readout.final_state_fingerprint_matches);
    assert!(readout.finalization_matches);
    assert!(readout.run_decision_kind_matches);
    assert!(readout.executed_step_count_matches);
    assert!(!readout.policy_decisions_match);
    assert!(readout.action_resource_transition_log_matches);
    assert!(readout.equipment_ledger_matches);
    assert!(readout.class_build_ledger_matches);
    assert!(readout.equipment_transition_log_matches);
    assert!(readout.reaction_window_lifecycle_log_matches);
    assert!(readout.reaction_audit_log_matches);
    assert!(readout.modifier_duration_expiration_log_matches);
}

#[test]
fn session_runtime_automatic_run_replay_reports_mismatched_expected_evidence() {
    let scenario = hexing_bolt_fixture_scenario();
    let run_spec = CombatSessionAutomaticRunSpec::new(
        "auto-run-replay-mismatch",
        "Auto run replay mismatch",
        "Rust replays bounded automatic combat for mismatch evidence.",
        8,
        vec![17, 5],
    );
    let mut expected_session =
        CombatSessionState::new("runtime-hexing-bolt-expected", scenario.clone());
    let expected_run = expected_session.run_automatic_combat(run_spec.clone());
    let mut expected_modifier_duration_expiration_log = expected_run
        .final_snapshot
        .modifier_duration_expiration_log
        .clone();
    expected_modifier_duration_expiration_log.clear();
    let mut expected_action_resource_transition_log = expected_run
        .final_snapshot
        .action_resource_transition_log
        .clone();
    expected_action_resource_transition_log.clear();

    let readout = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
        "auto-run-replay-mismatch-verification",
        "Auto run replay mismatch verification",
        "Rust reports mismatched expected automatic combat evidence.",
        "runtime-hexing-bolt-replay",
        scenario,
        run_spec,
        expected_run
            .final_snapshot
            .current_state_fingerprint
            .clone(),
        expected_run.final_snapshot.finalization.clone(),
        expected_run.decision_kind,
        expected_run.executed_step_count,
        Vec::new(),
        expected_action_resource_transition_log,
        expected_run.final_snapshot.equipment_ledger.clone(),
        expected_run.final_snapshot.class_build_ledger.clone(),
        expected_run.final_snapshot.equipment_transition_log.clone(),
        expected_run
            .final_snapshot
            .reaction_window_lifecycle_log
            .clone(),
        expected_run.final_snapshot.reaction_audit_log.clone(),
        expected_modifier_duration_expiration_log,
    ));

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence
    );
    assert_eq!(readout.decision_kind.code(), "mismatchedEvidence");
    assert!(readout.final_state_fingerprint_matches);
    assert!(readout.finalization_matches);
    assert!(readout.run_decision_kind_matches);
    assert!(readout.executed_step_count_matches);
    assert!(!readout.policy_decisions_match);
    assert!(!readout.action_resource_transition_log_matches);
    assert!(readout.equipment_ledger_matches);
    assert!(readout.class_build_ledger_matches);
    assert!(readout.equipment_transition_log_matches);
    assert!(readout.reaction_window_lifecycle_log_matches);
    assert!(readout.reaction_audit_log_matches);
    assert!(!readout.modifier_duration_expiration_log_matches);
    assert_eq!(
        readout.actual_executed_step_count,
        expected_run.executed_step_count
    );
    assert_eq!(
        readout.reason,
        "Automatic run replay produced evidence that does not match expected final evidence."
    );
}

#[test]
fn session_runtime_selected_candidate_submission_accepts_hit() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "selected-hit",
        "Selected hit",
        "Caller selected Hexing Bolt through the selected-candidate submission path.",
        "hexing_bolt",
        "entity-raider",
        vec![17, 5],
    ));

    assert!(execution.selection.accepted);
    assert_eq!(
        execution.selection.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::Accepted
    );
    assert_eq!(
        execution
            .selection
            .command
            .as_ref()
            .map(|command| command.intent.clone()),
        Some(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider"
        ))
    );
    let submitted_step = execution
        .submitted_step
        .as_ref()
        .expect("accepted selection submits command");
    assert!(submitted_step.receipt.accepted);
    assert_eq!(submitted_step.step.id, "selected-hit");
    assert_eq!(
        submitted_step.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        submitted_step.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_ne!(
        submitted_step.audit_entry.state_before_fingerprint,
        submitted_step.audit_entry.state_after_fingerprint
    );

    let snapshot = session.snapshot();
    assert_eq!(snapshot.next_step_index, 1);
    assert_eq!(snapshot.combat_log.len(), 1);
    assert_eq!(snapshot.audit_log.len(), 1);
    assert_eq!(snapshot.action_usage_log.len(), 1);
    assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 9);
    assert_eq!(
        snapshot.current_state.combatants[1].conditions,
        vec!["rattled"]
    );
    assert_eq!(
        execution.selection.reason,
        "Selected command candidate planned for deterministic submission."
    );
}

#[test]
fn session_runtime_selected_candidate_submission_accepts_miss_and_spends_its_resource() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let resource_fingerprint_before = session.snapshot().action_resource_fingerprint;

    let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "selected-miss",
        "Selected miss",
        "Caller selected Hexing Bolt with deterministic miss rolls.",
        "hexing_bolt",
        "entity-raider",
        vec![2, 5],
    ));

    assert!(execution.selection.accepted);
    let submitted_step = execution
        .submitted_step
        .as_ref()
        .expect("accepted selection submits command");
    assert!(submitted_step.receipt.accepted);
    assert_eq!(
        submitted_step.step.outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
    assert_eq!(
        submitted_step.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        submitted_step.audit_entry.state_before_fingerprint,
        submitted_step.audit_entry.state_after_fingerprint
    );

    let snapshot = session.snapshot();
    assert_ne!(
        snapshot.action_resource_fingerprint,
        resource_fingerprint_before
    );
    assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 18);
    assert!(snapshot.current_state.combatants[1].conditions.is_empty());
    assert_eq!(snapshot.action_usage_log.len(), 1);
    assert_eq!(
        snapshot.action_resource_ledger.combatants[0].resources[0].current,
        0
    );
    assert_eq!(
        snapshot.action_usage_log[0].outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
}

#[test]
fn session_runtime_selected_candidate_submission_rejected_plan_is_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let before_execution = session.snapshot();

    let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
        "selected-unavailable",
        "Selected unavailable",
        "Raider has no command candidates in this fixture.",
        "hexing_bolt",
        "entity-raider",
        vec![17, 5],
    ));
    let after_execution = session.snapshot();

    assert!(!execution.selection.accepted);
    assert_eq!(
        execution.selection.decision_kind,
        CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
    );
    assert_eq!(execution.submitted_step, None);
    assert_eq!(after_execution, before_execution);
    assert!(session.combat_log().is_empty());
    assert!(session.audit_log().is_empty());
    assert!(session.action_usage_log().is_empty());
    assert_eq!(session.turn_transition_log().len(), 1);
}

#[test]
fn session_runtime_command_preflight_accepts_current_actor_action_target_without_rolls() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));

    assert!(preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::Accepted
    );
    assert_eq!(preflight.decision_kind.code(), "accepted");
    assert_eq!(preflight.rejection, None);
    assert_eq!(preflight.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(
        preflight
            .target_legality
            .as_ref()
            .map(|legality| legality.accepted),
        Some(true)
    );
    assert_eq!(
        preflight
            .target_legality
            .as_ref()
            .map(|legality| legality.reason.as_str()),
        Some("Target is hostile, within range, and line of sight is clear.")
    );
    assert_eq!(
        preflight.reason,
        "Command is admissible before roll resolution."
    );
}

#[test]
fn session_runtime_command_preflight_rejects_empty_shape() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let preflight =
        session.preflight_command(UseActionIntent::new("", "hexing_bolt", "entity-raider"));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByShape
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByShape");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::EmptyActorId));
    assert_eq!(preflight.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(preflight.target_legality, None);
}

#[test]
fn session_runtime_command_preflight_rejects_ended_combat() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByLifecycle
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByLifecycle");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
    assert_eq!(preflight.reason, "Combat is already ended.");
}

#[test]
fn session_runtime_command_preflight_rejects_wrong_turn_actor() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByTurnOrder
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByTurnOrder");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
    assert_eq!(
        preflight.current_actor_id,
        Some("entity-raider".to_string())
    );
}

#[test]
fn session_runtime_command_preflight_rejects_invalid_actor_without_current_actor() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants.clear();
    let session = CombatSessionState::new("runtime-empty", scenario);

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByActorLookup
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByActorLookup");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidActor));
    assert_eq!(preflight.current_actor_id, None);
}

#[test]
fn session_runtime_command_preflight_rejects_invalid_action() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "not_hexing_bolt",
        "entity-raider",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByActionLookup
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByActionLookup");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
}

#[test]
fn session_runtime_command_preflight_rejects_action_actor_mismatch() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].actor_id = "entity-raider".to_string();
    let session = CombatSessionState::new("runtime-action-mismatch", scenario);

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByActionOwnership
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByActionOwnership");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
}

#[test]
fn session_runtime_command_preflight_rejects_invalid_target() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-missing",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByTargetLookup
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByTargetLookup");
    assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidTarget));
}

#[test]
fn session_runtime_command_preflight_rejects_target_legality_failure() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-adept",
    ));

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByTargetLegality
    );
    assert_eq!(preflight.decision_kind.code(), "rejectedByTargetLegality");
    assert_eq!(
        preflight.rejection,
        Some(RulebenchRejection::TargetLegalityFailed)
    );
    assert_eq!(
        preflight
            .target_legality
            .as_ref()
            .map(|legality| legality.reason.as_str()),
        Some("Target is not hostile.")
    );
}

#[test]
fn session_runtime_command_preflight_is_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let before_preflight = session.snapshot();

    let preflight = session.preflight_command(UseActionIntent::new(
        "entity-adept",
        "hexing_bolt",
        "entity-raider",
    ));
    let after_preflight = session.snapshot();

    assert!(!preflight.accepted);
    assert_eq!(
        preflight.decision_kind,
        CommandPreflightDecisionKind::RejectedByActionResource
    );
    assert_eq!(
        preflight.action_resource,
        Some(ActionResourceState::new(
            ActionResourceKind::StandardAction,
            0,
            1
        ))
    );
    assert_eq!(after_preflight, before_preflight);
    assert_eq!(session.next_step_index(), 1);
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(session.action_usage_log().len(), 1);
    assert_eq!(session.turn_transition_log().len(), 0);
    assert_eq!(session.lifecycle_transition_log().len(), 1);
}
