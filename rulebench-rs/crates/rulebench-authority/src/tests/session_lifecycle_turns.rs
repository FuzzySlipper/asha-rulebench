use super::super::test_support::*;

#[test]
fn session_runtime_can_end_combat_lifecycle() {
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
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-miss",
        "Runtime miss",
        "Adept misses Raider through the command runtime.",
        CommandOutcomeClass::AcceptedMiss,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    ));

    session.end_combat();

    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.lifecycle().started_at_step, Some(0));
    assert_eq!(session.lifecycle().ended_at_step, Some(2));
    assert_eq!(session.next_step_index(), 2);
    assert_eq!(session.lifecycle_transition_log().len(), 2);
    assert_eq!(
        session.lifecycle_transition_log()[0].trigger,
        LifecycleTransitionTrigger::CommandStart
    );
    assert_eq!(
        session.lifecycle_transition_log()[1].trigger,
        LifecycleTransitionTrigger::ExplicitEnd
    );
    assert_eq!(session.lifecycle_transition_log()[1].sequence, 1);
    assert_eq!(session.lifecycle_transition_log()[1].step_index, 2);
    assert_eq!(
        session.lifecycle_transition_log()[1].previous_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(
        session.lifecycle_transition_log()[1].next_phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(
        session.lifecycle_transition_log()[1].started_at_step,
        Some(0)
    );
    assert_eq!(session.lifecycle_transition_log()[1].ended_at_step, Some(2));
}

#[test]
fn session_runtime_lifecycle_transition_history_is_empty_initially() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    assert!(session.lifecycle_transition_log().is_empty());
    assert!(session.snapshot().lifecycle_transition_log.is_empty());
}

#[test]
fn session_runtime_can_start_combat_explicitly() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_start = session.snapshot();

    session.start_combat();
    let after_start = session.snapshot();

    assert_eq!(before_start.lifecycle.phase, CombatLifecyclePhase::Ready);
    assert_eq!(
        after_start.lifecycle.phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(after_start.lifecycle.started_at_step, Some(0));
    assert_eq!(after_start.lifecycle.ended_at_step, None);
    assert_eq!(after_start.next_step_index, before_start.next_step_index);
    assert_eq!(after_start.lifecycle_transition_log.len(), 1);
    assert_eq!(after_start.lifecycle_transition_log[0].sequence, 0);
    assert_eq!(
        after_start.lifecycle_transition_log[0].trigger,
        LifecycleTransitionTrigger::ExplicitStart
    );
    assert_eq!(
        after_start.lifecycle_transition_log[0].trigger.code(),
        "explicitStart"
    );
    assert_eq!(after_start.lifecycle_transition_log[0].step_index, 0);
    assert_eq!(
        after_start.lifecycle_transition_log[0].previous_phase,
        CombatLifecyclePhase::Ready
    );
    assert_eq!(
        after_start.lifecycle_transition_log[0].next_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(
        after_start.lifecycle_transition_log[0].started_at_step,
        Some(0)
    );
    assert_eq!(after_start.lifecycle_transition_log[0].ended_at_step, None);
    assert_eq!(after_start.turn_order, before_start.turn_order);
    assert_eq!(after_start.combat_log, before_start.combat_log);
    assert_eq!(after_start.audit_log, before_start.audit_log);
    assert_eq!(
        after_start.current_state_fingerprint,
        before_start.current_state_fingerprint
    );
}

#[test]
fn session_runtime_control_command_starts_combat_with_readout() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_start = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_start.current_state);

    let readout = session.submit_control_command(CombatControlCommandSpec::explicit_start());
    let after_start = session.snapshot();

    assert!(readout.accepted);
    assert_eq!(
        readout.command_kind,
        CombatControlCommandKind::ExplicitStart
    );
    assert_eq!(readout.command_kind.code(), "explicitStart");
    assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(readout.decision_kind.code(), "accepted");
    assert_eq!(readout.previous_lifecycle, before_start.lifecycle);
    assert_eq!(readout.next_lifecycle, after_start.lifecycle);
    assert_eq!(readout.previous_turn_order, before_start.turn_order);
    assert_eq!(readout.next_turn_order, before_start.turn_order);
    assert_eq!(
        readout.lifecycle_transition,
        Some(after_start.lifecycle_transition_log[0].clone())
    );
    assert_eq!(readout.turn_advance, None);
    assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(readout.reason, "Combat explicitly started.");
    assert_eq!(
        after_start.lifecycle.phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(after_start.lifecycle_transition_log.len(), 1);
    assert_eq!(session.control_history().len(), 1);
    let history = &session.control_history()[0];
    assert_eq!(history.sequence, 0);
    assert_eq!(
        history.command_kind,
        CombatControlCommandKind::ExplicitStart
    );
    assert!(history.accepted);
    assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(
        history.previous_lifecycle_phase,
        CombatLifecyclePhase::Ready
    );
    assert_eq!(
        history.next_lifecycle_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(
        history.previous_round_number,
        before_start.turn_order.round_number
    );
    assert_eq!(
        history.previous_turn_index,
        before_start.turn_order.current_turn_index
    );
    assert_eq!(
        history.previous_actor_id,
        before_start.turn_order.current_actor_id
    );
    assert_eq!(
        history.next_round_number,
        before_start.turn_order.round_number
    );
    assert_eq!(
        history.next_turn_index,
        before_start.turn_order.current_turn_index
    );
    assert_eq!(
        history.next_actor_id,
        before_start.turn_order.current_actor_id
    );
    assert_eq!(history.lifecycle_transition_sequence, Some(0));
    assert_eq!(history.turn_transition_sequence, None);
    assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(
        history.state_after_fingerprint,
        readout.state_after_fingerprint
    );
    assert_eq!(history.reason, "Combat explicitly started.");
}

#[test]
fn session_runtime_control_command_rejects_repeated_start_without_transition() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.start_combat();
    assert!(session.control_history().is_empty());
    let before_repeat = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_repeat.current_state);

    let readout = session.submit_control_command(CombatControlCommandSpec::explicit_start());
    let after_repeat = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(
        readout.command_kind,
        CombatControlCommandKind::ExplicitStart
    );
    assert_eq!(
        readout.decision_kind,
        CombatControlDecisionKind::RejectedNoop
    );
    assert_eq!(readout.decision_kind.code(), "rejectedNoop");
    assert_eq!(readout.previous_lifecycle, before_repeat.lifecycle);
    assert_eq!(readout.next_lifecycle, before_repeat.lifecycle);
    assert_eq!(readout.previous_turn_order, before_repeat.turn_order);
    assert_eq!(readout.next_turn_order, before_repeat.turn_order);
    assert_eq!(readout.lifecycle_transition, None);
    assert_eq!(readout.turn_advance, None);
    assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(readout.reason, "Combat is already in progress.");
    assert_eq!(
        after_repeat.lifecycle_transition_log,
        before_repeat.lifecycle_transition_log
    );
    assert_eq!(session.control_history().len(), 1);
    let history = &session.control_history()[0];
    assert_eq!(history.sequence, 0);
    assert_eq!(
        history.command_kind,
        CombatControlCommandKind::ExplicitStart
    );
    assert!(!history.accepted);
    assert_eq!(
        history.decision_kind,
        CombatControlDecisionKind::RejectedNoop
    );
    assert_eq!(
        history.previous_lifecycle_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(
        history.next_lifecycle_phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(history.lifecycle_transition_sequence, None);
    assert_eq!(history.turn_transition_sequence, None);
    assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(history.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(history.reason, "Combat is already in progress.");
}

#[test]
fn session_runtime_command_start_records_lifecycle_transition() {
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

    assert_eq!(session.lifecycle_transition_log().len(), 1);
    let transition = &session.lifecycle_transition_log()[0];
    assert_eq!(transition.sequence, 0);
    assert_eq!(transition.trigger, LifecycleTransitionTrigger::CommandStart);
    assert_eq!(transition.trigger.code(), "commandStart");
    assert_eq!(transition.step_index, 0);
    assert_eq!(transition.previous_phase, CombatLifecyclePhase::Ready);
    assert_eq!(transition.next_phase, CombatLifecyclePhase::InProgress);
    assert_eq!(transition.started_at_step, Some(0));
    assert_eq!(transition.ended_at_step, None);
}

#[test]
fn session_runtime_explicit_start_is_idempotent_while_in_progress() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.start_combat();
    let before_repeat = session.snapshot();

    session.start_combat();
    let after_repeat = session.snapshot();

    assert_eq!(after_repeat.lifecycle, before_repeat.lifecycle);
    assert_eq!(after_repeat.turn_order, before_repeat.turn_order);
    assert_eq!(after_repeat.next_step_index, before_repeat.next_step_index);
    assert_eq!(
        after_repeat.current_state_fingerprint,
        before_repeat.current_state_fingerprint
    );
    assert_eq!(after_repeat.combat_log, before_repeat.combat_log);
    assert_eq!(after_repeat.audit_log, before_repeat.audit_log);
    assert_eq!(
        after_repeat.lifecycle_transition_log,
        before_repeat.lifecycle_transition_log
    );
}

#[test]
fn session_runtime_explicit_start_after_end_is_noop() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.start_combat();
    session.end_combat();
    let before_start_attempt = session.snapshot();

    session.start_combat();
    let after_start_attempt = session.snapshot();

    assert_eq!(
        before_start_attempt.lifecycle.phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(
        after_start_attempt.lifecycle,
        before_start_attempt.lifecycle
    );
    assert_eq!(
        after_start_attempt.turn_order,
        before_start_attempt.turn_order
    );
    assert_eq!(
        after_start_attempt.next_step_index,
        before_start_attempt.next_step_index
    );
    assert_eq!(
        after_start_attempt.current_state_fingerprint,
        before_start_attempt.current_state_fingerprint
    );
    assert_eq!(
        after_start_attempt.combat_log,
        before_start_attempt.combat_log
    );
    assert_eq!(
        after_start_attempt.audit_log,
        before_start_attempt.audit_log
    );
    assert_eq!(
        after_start_attempt.lifecycle_transition_log,
        before_start_attempt.lifecycle_transition_log
    );
}

#[test]
fn session_runtime_command_after_explicit_start_does_not_duplicate_lifecycle_transition() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.start_combat();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::InProgress);
    assert_eq!(session.lifecycle().started_at_step, Some(0));
    assert_eq!(session.lifecycle().ended_at_step, None);
    assert_eq!(session.next_step_index(), 1);
    assert_eq!(session.lifecycle_transition_log().len(), 1);
    assert_eq!(
        session.lifecycle_transition_log()[0].trigger,
        LifecycleTransitionTrigger::ExplicitStart
    );
    assert_eq!(session.lifecycle_transition_log()[0].step_index, 0);
}

#[test]
fn session_runtime_control_command_ends_combat_and_rejects_repeated_end() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_end = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_end.current_state);

    let accepted = session.submit_control_command(CombatControlCommandSpec::explicit_end());
    let after_end = session.snapshot();

    assert!(accepted.accepted);
    assert_eq!(accepted.command_kind, CombatControlCommandKind::ExplicitEnd);
    assert_eq!(accepted.command_kind.code(), "explicitEnd");
    assert_eq!(accepted.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(accepted.previous_lifecycle, before_end.lifecycle);
    assert_eq!(accepted.next_lifecycle, after_end.lifecycle);
    assert_eq!(accepted.previous_turn_order, before_end.turn_order);
    assert_eq!(accepted.next_turn_order, before_end.turn_order);
    assert_eq!(
        accepted.lifecycle_transition,
        Some(after_end.lifecycle_transition_log[0].clone())
    );
    assert_eq!(accepted.turn_advance, None);
    assert_eq!(accepted.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(
        accepted.state_after_fingerprint,
        fingerprint_projected_state(&after_end.current_state)
    );
    assert_ne!(accepted.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(accepted.reason, "Combat explicitly ended.");
    assert_eq!(after_end.lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(after_end.lifecycle.started_at_step, Some(0));
    assert_eq!(after_end.lifecycle.ended_at_step, Some(0));
    assert_eq!(session.control_history().len(), 1);
    assert_eq!(session.control_history()[0].sequence, 0);
    assert_eq!(
        session.control_history()[0].command_kind,
        CombatControlCommandKind::ExplicitEnd
    );
    assert_eq!(
        session.control_history()[0].lifecycle_transition_sequence,
        Some(0)
    );

    let before_repeat = session.snapshot();
    let before_repeat_state_fingerprint = fingerprint_projected_state(&before_repeat.current_state);
    let rejected = session.submit_control_command(CombatControlCommandSpec::explicit_end());
    let after_repeat = session.snapshot();

    assert!(!rejected.accepted);
    assert_eq!(rejected.command_kind, CombatControlCommandKind::ExplicitEnd);
    assert_eq!(
        rejected.decision_kind,
        CombatControlDecisionKind::RejectedByLifecycle
    );
    assert_eq!(rejected.reason, "Combat is already ended.");
    assert_eq!(rejected.lifecycle_transition, None);
    assert_eq!(rejected.turn_advance, None);
    assert_eq!(rejected.previous_lifecycle, before_repeat.lifecycle);
    assert_eq!(rejected.next_lifecycle, before_repeat.lifecycle);
    assert_eq!(
        rejected.state_before_fingerprint,
        before_repeat_state_fingerprint
    );
    assert_eq!(
        rejected.state_after_fingerprint,
        before_repeat_state_fingerprint
    );
    assert_eq!(
        after_repeat.lifecycle_transition_log,
        before_repeat.lifecycle_transition_log
    );
    assert_eq!(session.control_history().len(), 1);
    assert_eq!(after_repeat, before_repeat);
}

#[test]
fn session_runtime_control_command_conditionally_ends_when_end_condition_is_met() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let mut session = CombatSessionState::new("runtime-conditional-end", scenario);
    session.start_combat();
    let before_end = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_end.current_state);

    let readout = session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
    let after_end = session.snapshot();

    assert!(before_end.combat_end_condition.combat_should_end);
    assert!(readout.accepted);
    assert_eq!(
        readout.command_kind,
        CombatControlCommandKind::EndIfConditionMet
    );
    assert_eq!(readout.command_kind.code(), "endIfConditionMet");
    assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(readout.decision_kind.code(), "accepted");
    assert_eq!(readout.previous_lifecycle, before_end.lifecycle);
    assert_eq!(readout.next_lifecycle, after_end.lifecycle);
    assert_eq!(readout.previous_turn_order, before_end.turn_order);
    assert_eq!(readout.next_turn_order, before_end.turn_order);
    assert_eq!(
        readout.lifecycle_transition,
        Some(after_end.lifecycle_transition_log[1].clone())
    );
    assert_eq!(readout.turn_advance, None);
    assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(
        readout.state_after_fingerprint,
        fingerprint_projected_state(&after_end.current_state)
    );
    assert_ne!(readout.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(
        readout.reason,
        "Combat conditionally ended. Combat should end because no active enemies remain."
    );
    assert_eq!(after_end.lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(after_end.lifecycle.started_at_step, Some(0));
    assert_eq!(after_end.lifecycle.ended_at_step, Some(0));
    assert_eq!(after_end.lifecycle_transition_log.len(), 2);
    assert_eq!(
        after_end.lifecycle_transition_log[1].trigger,
        LifecycleTransitionTrigger::ConditionalEnd
    );
    assert_eq!(
        after_end.lifecycle_transition_log[1].trigger.code(),
        "conditionalEnd"
    );
    assert_eq!(session.control_history().len(), 1);
    let history = &session.control_history()[0];
    assert_eq!(
        history.command_kind,
        CombatControlCommandKind::EndIfConditionMet
    );
    assert!(history.accepted);
    assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(history.lifecycle_transition_sequence, Some(1));
    assert_eq!(history.turn_transition_sequence, None);
    assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(
        history.state_after_fingerprint,
        readout.state_after_fingerprint
    );
}

#[test]
fn session_runtime_control_command_rejects_conditional_end_while_combat_can_continue() {
    let mut session =
        CombatSessionState::new("runtime-conditional-end", hexing_bolt_fixture_scenario());
    session.start_combat();
    let before_attempt = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_attempt.current_state);

    let readout = session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
    let after_attempt = session.snapshot();

    assert!(!before_attempt.combat_end_condition.combat_should_end);
    assert!(!readout.accepted);
    assert_eq!(
        readout.command_kind,
        CombatControlCommandKind::EndIfConditionMet
    );
    assert_eq!(
        readout.decision_kind,
        CombatControlDecisionKind::RejectedByEndCondition
    );
    assert_eq!(readout.decision_kind.code(), "rejectedByEndCondition");
    assert_eq!(
        readout.reason,
        "Combat end condition is not met. Combat can continue because multiple configured sides have active combatants."
    );
    assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
    assert_eq!(readout.next_turn_order, before_attempt.turn_order);
    assert_eq!(readout.lifecycle_transition, None);
    assert_eq!(readout.turn_advance, None);
    assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
    assert_eq!(
        after_attempt.lifecycle_transition_log,
        before_attempt.lifecycle_transition_log
    );
    assert_eq!(session.control_history().len(), 1);
    assert_eq!(
        session.control_history()[0].decision_kind,
        CombatControlDecisionKind::RejectedByEndCondition
    );
    assert_eq!(
        session.control_history()[0].lifecycle_transition_sequence,
        None
    );
}

#[test]
fn session_runtime_control_command_rejects_conditional_end_after_combat_already_ended() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let mut session = CombatSessionState::new("runtime-conditional-end", scenario);
    session.end_combat();
    let before_attempt = session.snapshot();
    let before_state_fingerprint = fingerprint_projected_state(&before_attempt.current_state);

    let readout = session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
    let after_attempt = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(
        readout.command_kind,
        CombatControlCommandKind::EndIfConditionMet
    );
    assert_eq!(
        readout.decision_kind,
        CombatControlDecisionKind::RejectedByLifecycle
    );
    assert_eq!(readout.reason, "Combat is already ended.");
    assert_eq!(readout.lifecycle_transition, None);
    assert_eq!(readout.turn_advance, None);
    assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
    assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
    assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
    assert_eq!(
        after_attempt.lifecycle_transition_log,
        before_attempt.lifecycle_transition_log
    );
    assert!(session.control_history().is_empty());
    assert_eq!(after_attempt, before_attempt);
}

#[test]
fn session_runtime_script_runs_conditional_end_control_step() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let mut session = CombatSessionState::new("runtime-conditional-end-script", scenario);

    let readout = session.run_script(CombatSessionScriptSpec::new(
        "conditional-end-script",
        "Conditional end script",
        "Script runs the Rust conditional end control.",
        vec![CombatSessionScriptStepSpec::control(
            "script-conditional-end",
            "Conditionally end combat",
            "Ends only because the Rust end condition is met.",
            CombatControlCommandSpec::end_if_condition_met(),
        )],
    ));

    assert_eq!(readout.steps.len(), 1);
    let step = &readout.steps[0];
    assert_eq!(step.command_kind, CombatSessionScriptCommandKind::Control);
    assert!(step.accepted);
    assert_eq!(
        step.decision_kind,
        CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
    );
    assert_eq!(
        step.reason,
        "Combat conditionally ended. Combat should end because no active enemies remain."
    );
    assert_eq!(step.control_history_sequence, Some(0));
    assert_eq!(step.command_audit_sequence, None);
    assert_eq!(step.runtime_step_id, None);
    assert_eq!(
        readout.final_snapshot.lifecycle.phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(
        session.lifecycle_transition_log()[0].trigger,
        LifecycleTransitionTrigger::ConditionalEnd
    );
}

#[test]
fn session_runtime_direct_control_methods_do_not_record_control_history() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.start_combat();
    session.advance_turn();
    session.end_combat();

    assert!(session.control_history().is_empty());
    assert_eq!(session.lifecycle_transition_log().len(), 2);
    assert_eq!(session.turn_transition_log().len(), 1);
}

#[test]
fn combat_lifecycle_preserves_first_end_marker() {
    let mut lifecycle = CombatLifecycle::ready();

    lifecycle.end_at_step(3);
    lifecycle.end_at_step(9);

    assert_eq!(lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(lifecycle.started_at_step, Some(3));
    assert_eq!(lifecycle.ended_at_step, Some(3));

    let mut in_progress_lifecycle = CombatLifecycle::ready();
    in_progress_lifecycle.start_at_step(1);
    in_progress_lifecycle.end_at_step(4);
    in_progress_lifecycle.end_at_step(9);

    assert_eq!(in_progress_lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(in_progress_lifecycle.started_at_step, Some(1));
    assert_eq!(in_progress_lifecycle.ended_at_step, Some(4));
}

#[test]
fn session_runtime_end_from_ready_records_lifecycle_transition() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.end_combat();

    assert_eq!(session.lifecycle_transition_log().len(), 1);
    let transition = &session.lifecycle_transition_log()[0];
    assert_eq!(transition.sequence, 0);
    assert_eq!(transition.trigger, LifecycleTransitionTrigger::ExplicitEnd);
    assert_eq!(transition.trigger.code(), "explicitEnd");
    assert_eq!(transition.step_index, 0);
    assert_eq!(transition.previous_phase, CombatLifecyclePhase::Ready);
    assert_eq!(transition.next_phase, CombatLifecyclePhase::Ended);
    assert_eq!(transition.started_at_step, Some(0));
    assert_eq!(transition.ended_at_step, Some(0));
}

#[test]
fn session_runtime_repeated_end_combat_preserves_first_end_snapshot() {
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
    session.end_combat();
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-post-end",
        "Runtime post-end command",
        "A command submitted after combat ended.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let before_repeat = session.snapshot();

    session.end_combat();
    let after_repeat = session.snapshot();

    assert_eq!(before_repeat.lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(before_repeat.lifecycle.started_at_step, Some(0));
    assert_eq!(before_repeat.lifecycle.ended_at_step, Some(1));
    assert_eq!(before_repeat.next_step_index, 1);
    assert_eq!(after_repeat.lifecycle, before_repeat.lifecycle);
    assert_eq!(after_repeat.turn_order, before_repeat.turn_order);
    assert_eq!(after_repeat.next_step_index, before_repeat.next_step_index);
    assert_eq!(
        after_repeat.current_state_fingerprint,
        before_repeat.current_state_fingerprint
    );
    assert_eq!(after_repeat.combat_log, before_repeat.combat_log);
    assert_eq!(after_repeat.audit_log, before_repeat.audit_log);
    assert_eq!(
        after_repeat.lifecycle_transition_log,
        before_repeat.lifecycle_transition_log
    );
}

#[test]
fn session_runtime_rejects_commands_after_combat_end() {
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
    session.end_combat();
    let ended_at_step = session.lifecycle().ended_at_step;
    let state_before_attempt = session.snapshot().current_state_fingerprint;

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-post-end",
        "Runtime post-end command",
        "A command submitted after combat ended.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let state_after_attempt = session.snapshot().current_state_fingerprint;

    assert_eq!(readout.step.index, 1);
    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert!(readout.receipt.events.is_empty());
    assert!(readout.receipt.attack_roll.is_none());
    assert!(readout.receipt.damage.is_none());
    assert!(readout.receipt.modifier.is_none());
    assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
    assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
    assert_eq!(state_before_attempt, state_after_attempt);
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.lifecycle().ended_at_step, ended_at_step);
    assert_eq!(session.next_step_index(), 1);
}

#[test]
fn session_runtime_returns_post_end_rejection_without_mutating_logs_or_audit() {
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
    session.end_combat();

    let before_attempt = session.snapshot();
    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-post-end",
        "Runtime post-end command",
        "A command submitted after combat ended.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(readout.combat_log[0].id, "log-runtime-post-end");
    assert!(readout.combat_log[0].event_types.is_empty());
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.audit_log().len(), 1);
    assert_eq!(readout.audit_entry.id, "audit-runtime-post-end");
    assert_eq!(readout.audit_entry.sequence, 1);
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByLifecycle
    );
    assert_eq!(
        readout.audit_entry.decision_kind.code(),
        "rejectedByLifecycle"
    );
    assert!(!readout.audit_entry.accepted);
    assert_eq!(
        readout.audit_entry.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(readout.audit_entry.event_count, 0);
    assert_eq!(readout.audit_entry.trace_count, 2);
    assert_eq!(session.snapshot(), before_attempt);
}

#[test]
fn session_runtime_post_end_command_does_not_record_action_usage() {
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
    session.end_combat();
    let before_post_end = session.action_usage_log().to_vec();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-post-end",
        "Runtime post-end command",
        "A command submitted after combat ended.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(session.action_usage_log(), before_post_end.as_slice());
}

#[test]
fn session_runtime_rejects_commands_for_non_current_actor() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let state_before_attempt = session.snapshot().current_state_fingerprint;

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-wrong-actor",
        "Runtime wrong actor",
        "Adept attempts to act during Raider's turn.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let snapshot = session.snapshot();

    assert_eq!(readout.step.index, 0);
    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert!(readout.receipt.events.is_empty());
    assert!(readout.receipt.target_legality.is_none());
    assert!(readout.receipt.attack_roll.is_none());
    assert!(readout.receipt.damage.is_none());
    assert!(readout.receipt.modifier.is_none());
    assert_eq!(
        readout.receipt.trace[1].message,
        "Command rejected by turn order."
    );
    assert_eq!(readout.state_before.combatants[1].hit_points.current, 18);
    assert_eq!(readout.state_after.combatants[1].hit_points.current, 18);
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
    assert_eq!(snapshot.current_state_fingerprint, state_before_attempt);
    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
    assert_eq!(
        snapshot.turn_order.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(snapshot.next_step_index, 1);
}

#[test]
fn session_runtime_records_non_current_actor_attempt_in_log_and_audit() {
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
    session.advance_turn();
    let after_hit_fingerprint = session.snapshot().current_state_fingerprint;

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-wrong-actor",
        "Runtime wrong actor",
        "Adept attempts to act during Raider's turn.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let snapshot = session.snapshot();

    assert_eq!(readout.combat_log[0].id, "log-runtime-wrong-actor");
    assert!(readout.combat_log[0].event_types.is_empty());
    assert_eq!(session.combat_log().len(), 2);
    assert_eq!(session.audit_log().len(), 2);
    assert_eq!(readout.audit_entry.id, "audit-runtime-wrong-actor");
    assert_eq!(readout.audit_entry.sequence, 1);
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByTurnOrder
    );
    assert_eq!(
        readout.audit_entry.decision_kind.code(),
        "rejectedByTurnOrder"
    );
    assert!(!readout.audit_entry.accepted);
    assert_eq!(
        readout.audit_entry.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(readout.audit_entry.event_count, 0);
    assert_eq!(readout.audit_entry.trace_count, 2);
    assert_eq!(session.audit_log()[1], readout.audit_entry);
    assert_eq!(snapshot.current_state_fingerprint, after_hit_fingerprint);
    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::InProgress);
    assert_eq!(snapshot.lifecycle.started_at_step, Some(0));
    assert_eq!(snapshot.lifecycle.ended_at_step, None);
    assert_eq!(snapshot.turn_order.round_number, 1);
    assert_eq!(snapshot.turn_order.current_turn_index, 1);
    assert_eq!(
        snapshot.turn_order.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(snapshot.turn_transition_log.len(), 1);
    assert_eq!(snapshot.turn_transition_log[0].previous_turn_index, 0);
    assert_eq!(snapshot.turn_transition_log[0].next_turn_index, 1);
    assert!(!snapshot.turn_transition_log[0].wrapped_round);
}

#[test]
fn session_runtime_non_current_actor_command_does_not_record_action_usage() {
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
    session.advance_turn();
    let before_wrong_actor = session.action_usage_log().to_vec();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-wrong-actor",
        "Runtime wrong actor",
        "Adept attempts to act during Raider's turn.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(session.action_usage_log(), before_wrong_actor.as_slice());
}

#[test]
fn session_runtime_current_turn_action_usage_filters_after_turn_advance() {
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

    session.advance_turn();
    let summary = session.current_turn_action_usage();

    assert_eq!(summary.round_number, 1);
    assert_eq!(summary.turn_index, 1);
    assert_eq!(summary.current_actor_id, Some("entity-raider".to_string()));
    assert_eq!(summary.used_action_count, 0);
    assert!(summary.used_action_ids.is_empty());
    assert!(summary.used_ability_ids.is_empty());
}

#[test]
fn session_runtime_turn_transition_history_is_empty_initially() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    assert!(session.turn_transition_log().is_empty());
    assert!(session.snapshot().turn_transition_log.is_empty());
}

#[test]
fn session_runtime_ended_combat_gate_takes_precedence_over_actor_gate() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    session.end_combat();

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-post-end-wrong-actor",
        "Runtime post-end wrong actor",
        "Adept attempts to act during Raider's turn after combat ended.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(
        readout.receipt.trace[1].message,
        "Command rejected by lifecycle."
    );
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.lifecycle().ended_at_step, Some(0));
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-raider".to_string())
    );
}

#[test]
fn session_runtime_initializes_turn_order_from_scenario_combatants() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    assert_eq!(session.turn_order().round_number, 1);
    assert_eq!(session.turn_order().current_turn_index, 0);
    assert_eq!(
        session.turn_order().participant_order,
        vec!["entity-adept".to_string(), "entity-raider".to_string()]
    );
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-adept".to_string())
    );
}

#[test]
fn session_runtime_orders_arbitrary_participants_by_initiative_then_id() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].initiative = 15;
    let mut third = scenario.combatants[1].clone();
    third.id = "entity-zed".to_string();
    third.name = "Zed".to_string();
    third.initiative = 20;
    scenario.combatants.push(third);

    let session = CombatSessionState::new("runtime-initiative-order", scenario);

    assert_eq!(
        session.turn_order().participant_order,
        vec![
            "entity-zed".to_string(),
            "entity-adept".to_string(),
            "entity-raider".to_string(),
        ]
    );
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-zed".to_string())
    );
}

#[test]
fn session_runtime_advances_turns_and_rounds() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.advance_turn();

    assert_eq!(session.turn_order().round_number, 1);
    assert_eq!(session.turn_order().current_turn_index, 1);
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-raider".to_string())
    );

    session.advance_turn();

    assert_eq!(session.turn_order().round_number, 2);
    assert_eq!(session.turn_order().current_turn_index, 0);
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-adept".to_string())
    );
}

#[test]
fn session_runtime_skips_a_combatant_defeated_between_turns() {
    let mut scenario = turn_control_fixture_scenario();
    scenario.rulesets[0].modules[1] = RuleModuleDeclaration::turn_control(
        TurnControlModuleConfiguration::explicit_turn_order_with_end_policy(
            CombatEndPolicy::ExplicitOnly,
        ),
    );
    let mut session = CombatSessionState::new("runtime-defeated-turn-skip", scenario);
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "defeat-raider",
        "Defeat Raider",
        "Adept defeats Raider before the next turn.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 30],
    ));

    let readout = session.advance_turn();

    assert!(readout.accepted);
    assert_eq!(readout.decision_kind, TurnAdvanceDecisionKind::Advanced);
    assert_eq!(session.turn_order().round_number, 2);
    assert_eq!(session.turn_order().current_turn_index, 0);
    assert_eq!(
        session.turn_order().current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(
        readout
            .transition
            .as_ref()
            .and_then(|transition| transition.next_actor_id.clone()),
        Some("entity-adept".to_string())
    );
}

#[test]
fn session_runtime_records_successful_turn_transition() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let previous_turn_order = session.turn_order().clone();

    let readout = session.advance_turn();

    assert_eq!(session.turn_transition_log().len(), 1);
    let transition = &session.turn_transition_log()[0];
    assert!(readout.accepted);
    assert_eq!(readout.decision_kind, TurnAdvanceDecisionKind::Advanced);
    assert_eq!(readout.previous_turn_order, previous_turn_order);
    assert_eq!(readout.next_turn_order, session.turn_order().clone());
    assert_eq!(readout.transition, Some(transition.clone()));
    assert_eq!(
        readout.state_before_fingerprint,
        readout.state_after_fingerprint
    );
    assert_eq!(readout.reason, "Turn advanced to the next participant.");
    assert_eq!(transition.sequence, 0);
    assert_eq!(transition.previous_round_number, 1);
    assert_eq!(transition.previous_turn_index, 0);
    assert_eq!(
        transition.previous_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(transition.next_round_number, 1);
    assert_eq!(transition.next_turn_index, 1);
    assert_eq!(transition.next_actor_id, Some("entity-raider".to_string()));
    assert!(!transition.wrapped_round);
}

#[test]
fn session_runtime_control_command_advances_turn_with_readout() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_advance = session.snapshot();

    let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
    let after_advance = session.snapshot();

    assert!(readout.accepted);
    assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert_eq!(readout.command_kind.code(), "advanceTurn");
    assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(readout.lifecycle_transition, None);
    let turn_advance = readout
        .turn_advance
        .as_ref()
        .expect("advance turn control returns turn readout");
    assert!(turn_advance.accepted);
    assert_eq!(
        turn_advance.decision_kind,
        TurnAdvanceDecisionKind::Advanced
    );
    assert_eq!(
        turn_advance.transition,
        Some(after_advance.turn_transition_log[0].clone())
    );
    assert_eq!(readout.previous_lifecycle, before_advance.lifecycle);
    assert_eq!(readout.next_lifecycle, before_advance.lifecycle);
    assert_eq!(readout.previous_turn_order, before_advance.turn_order);
    assert_eq!(readout.next_turn_order, after_advance.turn_order);
    assert_eq!(
        readout.state_before_fingerprint,
        turn_advance.state_before_fingerprint
    );
    assert_eq!(
        readout.state_after_fingerprint,
        turn_advance.state_after_fingerprint
    );
    assert_eq!(readout.reason, "Turn advanced to the next participant.");
    assert_eq!(session.control_history().len(), 1);
    let history = &session.control_history()[0];
    assert_eq!(history.sequence, 0);
    assert_eq!(history.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert!(history.accepted);
    assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(
        history.previous_lifecycle_phase,
        CombatLifecyclePhase::Ready
    );
    assert_eq!(history.next_lifecycle_phase, CombatLifecyclePhase::Ready);
    assert_eq!(
        history.previous_round_number,
        before_advance.turn_order.round_number
    );
    assert_eq!(
        history.previous_turn_index,
        before_advance.turn_order.current_turn_index
    );
    assert_eq!(
        history.previous_actor_id,
        before_advance.turn_order.current_actor_id
    );
    assert_eq!(
        history.next_round_number,
        after_advance.turn_order.round_number
    );
    assert_eq!(
        history.next_turn_index,
        after_advance.turn_order.current_turn_index
    );
    assert_eq!(
        history.next_actor_id,
        after_advance.turn_order.current_actor_id
    );
    assert_eq!(history.lifecycle_transition_sequence, None);
    assert_eq!(history.turn_transition_sequence, Some(0));
    assert_eq!(
        history.state_before_fingerprint,
        turn_advance.state_before_fingerprint
    );
    assert_eq!(
        history.state_after_fingerprint,
        turn_advance.state_after_fingerprint
    );
    assert_eq!(history.reason, "Turn advanced to the next participant.");
}

#[test]
fn session_runtime_records_turn_transition_round_wrap() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    session.advance_turn();
    let readout = session.advance_turn();

    assert_eq!(session.turn_transition_log().len(), 2);
    let transition = &session.turn_transition_log()[1];
    assert!(readout.accepted);
    assert_eq!(readout.decision_kind, TurnAdvanceDecisionKind::Advanced);
    assert_eq!(readout.next_turn_order, session.turn_order().clone());
    assert_eq!(readout.transition, Some(transition.clone()));
    assert_eq!(
        readout.state_before_fingerprint,
        readout.state_after_fingerprint
    );
    assert_eq!(transition.sequence, 1);
    assert_eq!(transition.previous_round_number, 1);
    assert_eq!(transition.previous_turn_index, 1);
    assert_eq!(
        transition.previous_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(transition.next_round_number, 2);
    assert_eq!(transition.next_turn_index, 0);
    assert_eq!(transition.next_actor_id, Some("entity-adept".to_string()));
    assert!(transition.wrapped_round);
}

#[test]
fn session_runtime_turn_wrap_refreshes_spent_current_actor_action_resource() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-round-one-action",
        "Runtime round one action",
        "Adept spends the standard action in round one.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    assert_eq!(
        session
            .action_resource_ledger()
            .combatants
            .iter()
            .find(|combatant| combatant.combatant_id == "entity-adept")
            .and_then(|combatant| combatant.resources.first())
            .cloned(),
        Some(ActionResourceState::new(
            ActionResourceKind::StandardAction,
            0,
            1
        ))
    );
    assert!(
        !session
            .preflight_command(UseActionIntent::new(
                "entity-adept",
                "hexing_bolt",
                "entity-raider"
            ))
            .accepted
    );

    session.advance_turn();
    let readout = session.advance_turn();

    assert!(readout.accepted);
    assert_eq!(
        readout.next_turn_order.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(
        session
            .action_resource_ledger()
            .combatants
            .iter()
            .find(|combatant| combatant.combatant_id == "entity-adept")
            .and_then(|combatant| combatant.resources.first())
            .cloned(),
        Some(ActionResourceState::standard_action_available())
    );
    assert_eq!(
        session.snapshot().action_resource_ledger,
        session.action_resource_ledger()
    );
    assert!(
        session
            .preflight_command(UseActionIntent::new(
                "entity-adept",
                "hexing_bolt",
                "entity-raider"
            ))
            .accepted
    );
    assert!(session
        .current_actor_command_candidates()
        .candidates
        .iter()
        .any(|candidate| candidate.accepted));
    assert_eq!(session.action_resource_transition_log().len(), 3);
    let spend_transition = &session.action_resource_transition_log()[0];
    assert_eq!(spend_transition.sequence, 0);
    assert_eq!(
        spend_transition.transition_kind,
        ActionResourceTransitionKind::Spent
    );
    assert_eq!(spend_transition.combatant_id, "entity-adept");
    assert_eq!(
        spend_transition.resource_kind,
        ActionResourceKind::StandardAction
    );
    assert_eq!(
        spend_transition.previous_resource,
        ActionResourceState::standard_action_available()
    );
    assert_eq!(
        spend_transition.next_resource,
        ActionResourceState::new(ActionResourceKind::StandardAction, 0, 1)
    );
    assert_eq!(
        spend_transition.command_step_id,
        Some("runtime-round-one-action".to_string())
    );
    assert_eq!(spend_transition.command_step_index, Some(0));
    assert_eq!(spend_transition.turn_transition_sequence, None);
    assert_eq!(spend_transition.round_number, Some(1));
    assert_eq!(spend_transition.turn_index, Some(0));
    assert_eq!(
        spend_transition.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(spend_transition.reason, "Action resource spent.");

    let raider_refresh_transition = &session.action_resource_transition_log()[1];
    assert_eq!(raider_refresh_transition.sequence, 1);
    assert_eq!(
        raider_refresh_transition.transition_kind,
        ActionResourceTransitionKind::Refreshed
    );
    assert_eq!(raider_refresh_transition.combatant_id, "entity-raider");
    assert_eq!(
        raider_refresh_transition.resource_kind,
        ActionResourceKind::StandardAction
    );
    assert_eq!(
        raider_refresh_transition.previous_resource,
        ActionResourceState::standard_action_available()
    );
    assert_eq!(
        raider_refresh_transition.next_resource,
        ActionResourceState::standard_action_available()
    );
    assert_eq!(raider_refresh_transition.command_step_id, None);
    assert_eq!(raider_refresh_transition.command_step_index, None);
    assert_eq!(raider_refresh_transition.turn_transition_sequence, Some(0));
    assert_eq!(raider_refresh_transition.round_number, Some(1));
    assert_eq!(raider_refresh_transition.turn_index, Some(1));
    assert_eq!(
        raider_refresh_transition.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(
        raider_refresh_transition.reason,
        "Action resource refreshed at turn start."
    );

    let adept_refresh_transition = &session.action_resource_transition_log()[2];
    assert_eq!(adept_refresh_transition.sequence, 2);
    assert_eq!(
        adept_refresh_transition.transition_kind,
        ActionResourceTransitionKind::Refreshed
    );
    assert_eq!(adept_refresh_transition.combatant_id, "entity-adept");
    assert_eq!(
        adept_refresh_transition.resource_kind,
        ActionResourceKind::StandardAction
    );
    assert_eq!(
        adept_refresh_transition.previous_resource,
        ActionResourceState::new(ActionResourceKind::StandardAction, 0, 1)
    );
    assert_eq!(
        adept_refresh_transition.next_resource,
        ActionResourceState::standard_action_available()
    );
    assert_eq!(adept_refresh_transition.command_step_id, None);
    assert_eq!(adept_refresh_transition.command_step_index, None);
    assert_eq!(adept_refresh_transition.turn_transition_sequence, Some(1));
    assert_eq!(adept_refresh_transition.round_number, Some(2));
    assert_eq!(adept_refresh_transition.turn_index, Some(0));
    assert_eq!(
        adept_refresh_transition.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(
        adept_refresh_transition.reason,
        "Action resource refreshed at turn start."
    );
}

#[test]
fn session_runtime_turn_wrap_expires_previous_actor_temporary_modifier() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-rattled-before-turns",
        "Runtime rattled before turns",
        "Adept applies rattled to Raider before turn advancement.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    let first_advance = session.advance_turn();
    assert!(first_advance.accepted);
    assert_eq!(
        first_advance.next_turn_order.current_actor_id,
        Some("entity-raider".to_string())
    );
    assert_eq!(
        first_advance.state_before_fingerprint,
        first_advance.state_after_fingerprint
    );
    assert!(session.modifier_duration_expiration_log().is_empty());
    assert_eq!(
        session
            .snapshot()
            .current_state
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider remains present")
            .conditions,
        vec!["rattled".to_string()]
    );

    let second_advance = session.advance_turn();

    assert!(second_advance.accepted);
    assert_eq!(
        second_advance.next_turn_order.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_ne!(
        second_advance.state_before_fingerprint,
        second_advance.state_after_fingerprint
    );
    assert!(session
        .snapshot()
        .current_state
        .combatants
        .iter()
        .find(|combatant| combatant.id == "entity-raider")
        .expect("raider remains present")
        .conditions
        .is_empty());
    assert_eq!(session.modifier_duration_expiration_log().len(), 1);
    let expiration = &session.modifier_duration_expiration_log()[0];
    assert_eq!(expiration.sequence, 0);
    assert_eq!(expiration.combatant_id, "entity-raider");
    assert_eq!(expiration.modifier_id, "rattled");
    assert_eq!(
        expiration.previous_modifier,
        ActiveModifier {
            modifier_id: "rattled".to_string(),
            source_id: "hexing_bolt".to_string(),
            label: "rattled".to_string(),
            duration: "until end of next turn".to_string(),
            tenure: ModifierTenure::Temporary,
            stacking_group: "rattled".to_string(),
            stacking_policy: ModifierStackingPolicy::Refresh,
            duration_policy: ModifierDurationPolicy::Turns(1),
            remaining_turns: Some(1),
            remaining_rounds: None,
        }
    );
    assert_eq!(expiration.next_modifier, None);
    assert_eq!(
        expiration.trigger,
        ModifierDurationTransitionTrigger::TurnBoundary
    );
    assert_eq!(expiration.turn_transition_sequence, Some(1));
    assert_eq!(expiration.round_number, Some(2));
    assert_eq!(expiration.turn_index, Some(0));
    assert_eq!(
        expiration.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(
        expiration.reason,
        "Turn-counted modifier expired at turn boundary."
    );
    assert_eq!(
        session.snapshot().modifier_duration_expiration_log,
        session.modifier_duration_expiration_log()
    );
}

#[test]
fn session_runtime_turn_wrap_preserves_permanent_modifier() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1]
        .active_modifiers
        .push(ActiveModifier::permanent(
            "battle-drilled",
            "battle-drilled",
        ));
    let mut session = CombatSessionState::new("runtime-permanent-modifier", scenario);

    session.advance_turn();
    session.advance_turn();

    assert!(session.modifier_duration_expiration_log().is_empty());
    assert_eq!(
        session
            .snapshot()
            .current_state
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider remains present")
            .conditions,
        vec!["battle-drilled".to_string()]
    );
}

#[test]
fn session_runtime_combat_end_expires_event_counted_modifier_with_audit_evidence() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1]
        .active_modifiers
        .push(ActiveModifier {
            modifier_id: "rattled".to_string(),
            source_id: "event-test".to_string(),
            label: "rattled".to_string(),
            duration: "until combat end".to_string(),
            tenure: ModifierTenure::Temporary,
            stacking_group: "event-rattled".to_string(),
            stacking_policy: ModifierStackingPolicy::Replace,
            duration_policy: ModifierDurationPolicy::UntilEvent("combatEnd".to_string()),
            remaining_turns: None,
            remaining_rounds: None,
        });
    let mut session = CombatSessionState::new("runtime-event-modifier", scenario);

    session.end_combat();

    assert!(session
        .snapshot()
        .current_state
        .combatants
        .iter()
        .find(|combatant| combatant.id == "entity-raider")
        .expect("raider remains present")
        .conditions
        .is_empty());
    assert_eq!(session.modifier_duration_expiration_log().len(), 1);
    let expiration = &session.modifier_duration_expiration_log()[0];
    assert_eq!(
        expiration.trigger,
        ModifierDurationTransitionTrigger::Event("combatEnd".to_string())
    );
    assert_eq!(expiration.turn_transition_sequence, None);
    assert_eq!(expiration.round_number, None);
    assert_eq!(expiration.turn_index, None);
    assert_eq!(
        expiration.reason,
        "Modifier expired when event combatEnd occurred."
    );
    assert_eq!(
        session.snapshot().modifier_duration_expiration_log,
        session.modifier_duration_expiration_log()
    );
}

#[test]
fn session_runtime_rejected_turn_advance_does_not_refresh_action_resource() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-spend-before-end",
        "Runtime spend before end",
        "Adept spends the standard action before combat ends.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    session.end_combat();
    let before = session.action_resource_ledger();
    let before_transition_log = session.action_resource_transition_log().to_vec();
    let before_expiration_log = session.modifier_duration_expiration_log().to_vec();

    let readout = session.advance_turn();
    let after = session.action_resource_ledger();

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        TurnAdvanceDecisionKind::RejectedByLifecycle
    );
    assert_eq!(after, before);
    assert_eq!(
        session.action_resource_transition_log(),
        before_transition_log
    );
    assert_eq!(
        session.modifier_duration_expiration_log(),
        before_expiration_log
    );
    assert_eq!(
        session
            .snapshot()
            .current_state
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider remains present")
            .conditions,
        vec!["rattled".to_string()]
    );
    assert_eq!(
        after
            .combatants
            .iter()
            .find(|combatant| combatant.combatant_id == "entity-adept")
            .and_then(|combatant| combatant.resources.first())
            .cloned(),
        Some(ActionResourceState::new(
            ActionResourceKind::StandardAction,
            0,
            1
        ))
    );
}

#[test]
fn session_runtime_control_turn_advance_refreshes_action_resource_on_round_wrap() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-spend-before-control-wrap",
        "Runtime spend before control wrap",
        "Adept spends the standard action before control turn advancement.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    session.submit_control_command(CombatControlCommandSpec::advance_turn());
    let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());

    assert!(readout.accepted);
    assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
    assert_eq!(session.control_history().len(), 2);
    assert_eq!(
        session
            .action_resource_ledger()
            .combatants
            .iter()
            .find(|combatant| combatant.combatant_id == "entity-adept")
            .and_then(|combatant| combatant.resources.first())
            .cloned(),
        Some(ActionResourceState::standard_action_available())
    );
}

#[test]
fn session_runtime_action_usage_records_current_turn_context() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    session.advance_turn();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-round-two-hit",
        "Runtime round two hit",
        "Adept acts after turn order wraps into round two.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(session.action_usage_log().len(), 1);
    let usage = &session.action_usage_log()[0];
    assert_eq!(usage.step_id, "runtime-round-two-hit");
    assert_eq!(usage.step_index, 0);
    assert_eq!(usage.round_number, 2);
    assert_eq!(usage.turn_index, 0);
    assert_eq!(usage.actor_id, "entity-adept");
    assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedHit);
}

#[test]
fn session_runtime_current_turn_action_usage_filters_after_round_wrap() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-round-one-hit",
        "Runtime round one hit",
        "Adept acts in round one.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    session.advance_turn();
    session.advance_turn();

    let before_round_two_action = session.current_turn_action_usage();
    assert_eq!(before_round_two_action.round_number, 2);
    assert_eq!(before_round_two_action.turn_index, 0);
    assert_eq!(
        before_round_two_action.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(before_round_two_action.used_action_count, 0);

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-round-two-hit",
        "Runtime round two hit",
        "Adept acts in round two.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let after_round_two_action = session.current_turn_action_usage();

    assert_eq!(after_round_two_action.used_action_count, 1);
    assert_eq!(
        after_round_two_action.used_action_ids,
        vec!["hexing_bolt".to_string()]
    );
}

#[test]
fn session_runtime_does_not_advance_turn_after_combat_end() {
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
    session.advance_turn();
    session.end_combat();
    let before_attempt = session.snapshot();
    let before_attempt_state_fingerprint =
        fingerprint_projected_state(&before_attempt.current_state);

    let readout = session.advance_turn();
    let after_attempt = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        TurnAdvanceDecisionKind::RejectedByLifecycle
    );
    assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
    assert_eq!(readout.next_turn_order, before_attempt.turn_order);
    assert_eq!(readout.transition, None);
    assert_eq!(
        readout.state_before_fingerprint,
        before_attempt_state_fingerprint
    );
    assert_eq!(
        readout.state_after_fingerprint,
        before_attempt_state_fingerprint
    );
    assert_eq!(readout.reason, "Combat is already ended.");
    assert_eq!(before_attempt.lifecycle.phase, CombatLifecyclePhase::Ended);
    assert_eq!(before_attempt.lifecycle.ended_at_step, Some(1));
    assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
    assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
    assert_eq!(
        after_attempt.turn_transition_log,
        before_attempt.turn_transition_log
    );
    assert_eq!(
        after_attempt.next_step_index,
        before_attempt.next_step_index
    );
    assert_eq!(
        after_attempt.current_state_fingerprint,
        before_attempt.current_state_fingerprint
    );
    assert_eq!(after_attempt.combat_log, before_attempt.combat_log);
    assert_eq!(after_attempt.audit_log, before_attempt.audit_log);
}

#[test]
fn session_runtime_control_command_rejects_turn_advance_after_end() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();
    let before_attempt = session.snapshot();

    let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
    let after_attempt = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert_eq!(
        readout.decision_kind,
        CombatControlDecisionKind::RejectedByLifecycle
    );
    assert_eq!(readout.lifecycle_transition, None);
    let turn_advance = readout
        .turn_advance
        .as_ref()
        .expect("advance turn control returns turn readout");
    assert!(!turn_advance.accepted);
    assert_eq!(
        turn_advance.decision_kind,
        TurnAdvanceDecisionKind::RejectedByLifecycle
    );
    assert_eq!(turn_advance.transition, None);
    assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
    assert_eq!(readout.next_turn_order, before_attempt.turn_order);
    assert_eq!(readout.reason, "Combat is already ended.");
    assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
    assert_eq!(
        after_attempt.turn_transition_log,
        before_attempt.turn_transition_log
    );
    assert!(session.control_history().is_empty());
    assert_eq!(after_attempt, before_attempt);
}

#[test]
fn session_runtime_turn_order_represents_empty_participants() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants.clear();
    let mut session = CombatSessionState::new("runtime-empty", scenario);

    assert_eq!(session.turn_order().round_number, 0);
    assert_eq!(session.turn_order().current_turn_index, 0);
    assert!(session.turn_order().participant_order.is_empty());
    assert_eq!(session.turn_order().current_actor_id, None);
    assert_eq!(
        session.current_turn_action_usage(),
        ActionUsageSummary {
            round_number: 0,
            turn_index: 0,
            current_actor_id: None,
            used_action_count: 0,
            used_action_ids: Vec::new(),
            used_ability_ids: Vec::new(),
        }
    );
    assert_eq!(
        session.combatant_vitality(),
        CombatantVitalitySummary {
            combatants: Vec::new(),
            active_combatant_ids: Vec::new(),
            defeated_combatant_ids: Vec::new(),
            active_count: 0,
            defeated_count: 0,
        }
    );
    assert_eq!(
        session.current_actor_options(),
        CurrentActorOptionSummary {
            round_number: 0,
            turn_index: 0,
            lifecycle_phase: CombatLifecyclePhase::Ready,
            current_actor_id: None,
            current_actor_defeated: false,
            available: false,
            unavailable_reason: Some(CurrentActorOptionsUnavailableReason::NoCurrentActor),
            actions: Vec::new(),
        }
    );
    assert!(session.turn_transition_log().is_empty());
    let before_attempt = session.snapshot();
    let before_attempt_state_fingerprint =
        fingerprint_projected_state(&before_attempt.current_state);

    let readout = session.advance_turn();
    let after_attempt = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder
    );
    assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
    assert_eq!(readout.next_turn_order, before_attempt.turn_order);
    assert_eq!(readout.transition, None);
    assert_eq!(
        readout.state_before_fingerprint,
        before_attempt_state_fingerprint
    );
    assert_eq!(
        readout.state_after_fingerprint,
        before_attempt_state_fingerprint
    );
    assert_eq!(readout.reason, "Turn order has no participants.");
    assert_eq!(session.turn_order().round_number, 0);
    assert_eq!(session.turn_order().current_turn_index, 0);
    assert_eq!(session.turn_order().current_actor_id, None);
    assert!(session.turn_transition_log().is_empty());
    assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
    assert_eq!(
        after_attempt.turn_transition_log,
        before_attempt.turn_transition_log
    );
    assert_eq!(
        after_attempt.current_state_fingerprint,
        before_attempt.current_state_fingerprint
    );
}

#[test]
fn session_runtime_control_command_rejects_turn_advance_with_empty_participants() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants.clear();
    let mut session = CombatSessionState::new("runtime-empty", scenario);
    let before_attempt = session.snapshot();

    let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
    let after_attempt = session.snapshot();

    assert!(!readout.accepted);
    assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert_eq!(
        readout.decision_kind,
        CombatControlDecisionKind::RejectedByEmptyTurnOrder
    );
    assert_eq!(readout.lifecycle_transition, None);
    let turn_advance = readout
        .turn_advance
        .as_ref()
        .expect("advance turn control returns turn readout");
    assert!(!turn_advance.accepted);
    assert_eq!(
        turn_advance.decision_kind,
        TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder
    );
    assert_eq!(turn_advance.transition, None);
    assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
    assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
    assert_eq!(readout.next_turn_order, before_attempt.turn_order);
    assert_eq!(readout.reason, "Turn order has no participants.");
    assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
    assert!(after_attempt.turn_transition_log.is_empty());
    assert_eq!(session.control_history().len(), 1);
    let history = &session.control_history()[0];
    assert_eq!(history.sequence, 0);
    assert_eq!(history.command_kind, CombatControlCommandKind::AdvanceTurn);
    assert!(!history.accepted);
    assert_eq!(
        history.decision_kind,
        CombatControlDecisionKind::RejectedByEmptyTurnOrder
    );
    assert_eq!(history.previous_round_number, 0);
    assert_eq!(history.next_round_number, 0);
    assert_eq!(history.previous_actor_id, None);
    assert_eq!(history.next_actor_id, None);
    assert_eq!(history.lifecycle_transition_sequence, None);
    assert_eq!(history.turn_transition_sequence, None);
    assert_eq!(history.reason, "Turn order has no participants.");
}
