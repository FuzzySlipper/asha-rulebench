use super::super::test_support::*;

#[test]
fn session_runtime_snapshot_reads_initial_state() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let snapshot = session.snapshot();

    assert_eq!(snapshot.session_id, "runtime-hexing-bolt");
    assert_eq!(snapshot.next_step_index, 0);
    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
    assert!(snapshot.lifecycle_transition_log.is_empty());
    assert!(snapshot.current_actor_options.available);
    assert_eq!(
        snapshot.current_actor_options.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(
        snapshot.turn_order.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert!(snapshot.combat_log.is_empty());
    assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 18);
}

#[test]
fn session_runtime_snapshot_fingerprint_is_stable_for_unchanged_state() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let first_snapshot = session.snapshot();
    let second_snapshot = session.snapshot();

    assert_eq!(
        first_snapshot.current_state_fingerprint.algorithm,
        PROJECTION_FINGERPRINT_ALGORITHM
    );
    assert_eq!(
        first_snapshot.current_state_fingerprint,
        second_snapshot.current_state_fingerprint
    );
    assert_eq!(
        first_snapshot.current_state_fingerprint,
        fingerprint_projection(&first_snapshot.current_state)
    );
}

#[test]
fn session_runtime_snapshot_reads_command_updates() {
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

    assert_eq!(snapshot.next_step_index, 1);
    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::InProgress);
    assert_eq!(snapshot.lifecycle_transition_log.len(), 1);
    assert_eq!(
        snapshot.lifecycle_transition_log[0].trigger,
        LifecycleTransitionTrigger::CommandStart
    );
    assert_eq!(snapshot.lifecycle_transition_log[0].step_index, 0);
    assert_eq!(
        snapshot.lifecycle_transition_log[0].previous_phase,
        CombatLifecyclePhase::Ready
    );
    assert_eq!(
        snapshot.lifecycle_transition_log[0].next_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(snapshot.combat_log.len(), 1);
    assert_eq!(snapshot.combat_log[0].step_id, "runtime-hit");
    assert_eq!(snapshot.audit_log.len(), 1);
    assert_eq!(snapshot.audit_log[0].step_id, "runtime-hit");
    assert_eq!(
        snapshot.audit_log[0].decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert!(snapshot.audit_log[0].accepted);
    assert_eq!(snapshot.audit_log[0].rejection, None);
    assert_eq!(snapshot.action_usage_log.len(), 1);
    assert_eq!(snapshot.action_usage_log[0].step_id, "runtime-hit");
    assert_eq!(
        snapshot.action_usage_log[0].ability_id,
        "ability.hexing-bolt"
    );
    assert!(snapshot.turn_transition_log.is_empty());
    assert_eq!(snapshot.current_turn_action_usage.used_action_count, 1);
    assert_eq!(
        snapshot.current_turn_action_usage.used_action_ids,
        vec!["hexing_bolt".to_string()]
    );
    assert_eq!(snapshot.combatant_vitality.active_count, 2);
    assert_eq!(snapshot.combatant_vitality.defeated_count, 0);
    assert_eq!(
        snapshot.combatant_vitality.active_combatant_ids,
        vec!["entity-adept".to_string(), "entity-raider".to_string()]
    );
    assert!(snapshot.current_actor_options.available);
    assert_eq!(snapshot.current_actor_options.unavailable_reason, None);
    assert_eq!(
        snapshot.current_actor_options.actions[0].target_options[0].current_hit_points,
        9
    );
    assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 9);
    assert_eq!(
        snapshot.current_state.combatants[1].conditions,
        vec!["rattled".to_string()]
    );
}

#[test]
fn session_runtime_vitality_summary_marks_zero_hp_defeated() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-zero-hp", scenario);

    let summary = session.combatant_vitality();

    assert_eq!(summary.active_count, 1);
    assert_eq!(summary.defeated_count, 1);
    assert_eq!(
        summary.active_combatant_ids,
        vec!["entity-adept".to_string()]
    );
    assert_eq!(
        summary.defeated_combatant_ids,
        vec!["entity-raider".to_string()]
    );
    assert!(summary.combatants[1].defeated);
}

#[test]
fn session_runtime_vitality_summary_marks_negative_hp_defeated() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0].hit_points.current = -3;
    let session = CombatSessionState::new("runtime-negative-hp", scenario);

    let summary = session.combatant_vitality();

    assert_eq!(summary.active_count, 1);
    assert_eq!(summary.defeated_count, 1);
    assert_eq!(
        summary.active_combatant_ids,
        vec!["entity-raider".to_string()]
    );
    assert_eq!(
        summary.defeated_combatant_ids,
        vec!["entity-adept".to_string()]
    );
    assert_eq!(summary.combatants[0].current_hit_points, -3);
    assert!(summary.combatants[0].defeated);
}

#[test]
fn session_runtime_snapshot_fingerprint_changes_after_accepted_hit() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let initial_fingerprint = session.snapshot().current_state_fingerprint;

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    let hit_fingerprint = session.snapshot().current_state_fingerprint;

    assert_ne!(initial_fingerprint, hit_fingerprint);
}

#[test]
fn session_runtime_snapshot_fingerprint_is_preserved_after_rejected_command() {
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
    let after_hit_fingerprint = session.snapshot().current_state_fingerprint;

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-rejected",
        "Runtime rejected",
        "Adept targets themself through the command runtime.",
        CommandOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));
    let after_rejection_snapshot = session.snapshot();

    assert_eq!(after_rejection_snapshot.next_step_index, 2);
    assert_eq!(
        after_rejection_snapshot.current_state_fingerprint,
        after_hit_fingerprint
    );
}

#[test]
fn session_runtime_snapshot_reads_turn_and_end_state() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.advance_turn();
    session.end_combat();

    let snapshot = session.snapshot();

    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(snapshot.lifecycle.started_at_step, Some(0));
    assert_eq!(snapshot.lifecycle.ended_at_step, Some(0));
    assert_eq!(snapshot.lifecycle_transition_log.len(), 1);
    assert_eq!(
        snapshot.lifecycle_transition_log[0].trigger,
        LifecycleTransitionTrigger::ExplicitEnd
    );
    assert_eq!(snapshot.lifecycle_transition_log[0].step_index, 0);
    assert_eq!(
        snapshot.lifecycle_transition_log[0].previous_phase,
        CombatLifecyclePhase::Ready
    );
    assert_eq!(
        snapshot.lifecycle_transition_log[0].next_phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(
        snapshot.lifecycle_transition_log[0].started_at_step,
        Some(0)
    );
    assert_eq!(snapshot.lifecycle_transition_log[0].ended_at_step, Some(0));
    assert_eq!(snapshot.turn_order.round_number, 1);
    assert_eq!(snapshot.turn_order.current_turn_index, 1);
    assert_eq!(
        snapshot.turn_order.current_actor_id,
        Some("entity-raider".to_string())
    );
}

#[test]
fn combat_session_rejects_unknown_session_id() {
    let error = resolve_combat_session_step("not-a-session", "adept-hexing-bolt-hit")
        .expect_err("unknown session fails");

    assert_eq!(error, CombatSessionError::UnknownSessionId);
}

#[test]
fn combat_session_rejects_unknown_step_id() {
    let error = resolve_combat_session_step("hexing-bolt-opening-exchange", "not-a-step")
        .expect_err("unknown step fails");

    assert_eq!(error, CombatSessionError::UnknownStepId);
}
