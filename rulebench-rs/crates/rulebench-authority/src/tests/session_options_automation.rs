use super::super::*;

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
    assert_eq!(options.actions.len(), 1);
    assert_eq!(options.actions[0].action_id, "hexing_bolt");
    assert_eq!(options.actions[0].ability_id, "ability.hexing-bolt");
    assert_eq!(options.actions[0].action_name, "Hexing Bolt");
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
    assert_eq!(
        snapshot.current_actor_options.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(snapshot.current_actor_options.actions.len(), 1);
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
fn session_runtime_current_actor_options_report_no_actions_after_turn_advance() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();

    let options = session.current_actor_options();

    assert!(!options.available);
    assert_eq!(options.current_actor_id, Some("entity-raider".to_string()));
    assert_eq!(
        options.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
    );
    assert_eq!(
        options.unavailable_reason.map(|reason| reason.code()),
        Some("noMatchingActions")
    );
    assert!(options.actions.is_empty());
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
fn session_runtime_current_actor_options_report_defeated_current_actor_unavailable() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-defeated-actor", scenario);

    let options = session.current_actor_options();

    assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
    assert!(options.current_actor_defeated);
    assert!(!options.available);
    assert_eq!(
        options.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::CurrentActorDefeated)
    );
    assert_eq!(
        options.unavailable_reason.map(|reason| reason.code()),
        Some("currentActorDefeated")
    );
    assert!(options.actions.is_empty());
}

#[test]
fn session_runtime_current_actor_options_filter_defeated_visible_targets() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-defeated-target", scenario);

    let options = session.current_actor_options();

    assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
    assert!(!options.current_actor_defeated);
    assert!(!options.available);
    assert_eq!(
        options.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets)
    );
    assert_eq!(
        options.unavailable_reason.map(|reason| reason.code()),
        Some("noVisibleActiveTargets")
    );
    assert_eq!(options.actions.len(), 1);
    assert_eq!(options.actions[0].action_id, "hexing_bolt");
    assert!(options.actions[0].target_options.is_empty());
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
    assert_eq!(candidates.candidates.len(), 1);

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

    assert!(candidates.available);
    assert_eq!(candidates.candidates.len(), 1);
    assert_eq!(
        candidates.candidates[0].intent,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
    );
    assert_eq!(candidates.candidates[0].target_current_hit_points, 9);
    assert_eq!(candidates.candidates[0].target_max_hit_points, 18);
    assert!(!candidates.candidates[0].accepted);
    assert_eq!(
        candidates.candidates[0].decision_kind,
        CommandPreflightDecisionKind::RejectedByActionResource
    );
    assert_eq!(
        candidates.candidates[0].reason,
        "Actor has no available standard action resource."
    );
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
    assert_eq!(
        candidates.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
    );
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

    assert!(candidates.available);
    assert_eq!(candidates.candidates.len(), 1);
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
    assert_eq!(
        plan.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
    );
    assert_eq!(
        plan.reason,
        "No command candidates are available because the current actor has no matching actions."
    );
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
fn session_runtime_candidate_selection_rejects_candidate_failed_by_preflight() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].target_ids = vec!["entity-adept".to_string()];
    scenario.actions[0].visible_target_ids = vec!["entity-adept".to_string()];
    scenario.selected_action = scenario.actions[0].clone();
    let session = CombatSessionState::new("runtime-self-target-candidate", scenario);
    let candidates = session.current_actor_command_candidates();

    assert!(candidates.available);
    assert_eq!(candidates.candidates.len(), 1);
    assert!(!candidates.candidates[0].accepted);
    assert_eq!(
        candidates.candidates[0].decision_kind,
        CommandPreflightDecisionKind::RejectedByTargetLegality
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
        CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByPreflight");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(
        plan.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByTargetLegality)
    );
    assert_eq!(
        plan.rejection,
        Some(RulebenchRejection::TargetLegalityFailed)
    );
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
    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.accepted_candidate_count, 1);
    assert_eq!(plan.selected_action_id, Some("hexing_bolt".to_string()));
    assert_eq!(plan.selected_target_id, Some("entity-raider".to_string()));
    assert_eq!(plan.unavailable_reason, None);
    assert_eq!(
        plan.reason,
        "First accepted command candidate planned for deterministic auto submission."
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
        CombatSessionAutoCandidateDecisionKind::RejectedByNoAcceptedCandidate
    );
    assert_eq!(plan.decision_kind.code(), "rejectedByNoAcceptedCandidate");
    assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.accepted_candidate_count, 0);
    assert_eq!(plan.selected_action_id, None);
    assert_eq!(plan.selected_target_id, None);
    assert_eq!(plan.selection, None);
    assert_eq!(
        plan.reason,
        "No accepted command candidates are available for deterministic auto submission."
    );
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
    assert_eq!(
        execution.plan.unavailable_reason,
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
    );
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
        Some(CombatSessionAutoCandidateDecisionKind::RejectedByNoAcceptedCandidate)
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
fn session_runtime_automatic_step_prioritizes_conditional_end_when_met() {
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

    assert!(plan.accepted);
    assert_eq!(
        plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::ConditionalEnd
    );
    assert_eq!(
        plan.operation_kind,
        Some(CombatSessionAutomaticStepOperationKind::ConditionalEnd)
    );
    assert_eq!(
        plan.combat_end_condition.condition_kind,
        CombatEndConditionKind::NoActiveEnemies
    );
    assert_eq!(plan.auto_candidate_plan, None);
    assert_eq!(after_plan, before_plan);

    let execution = session.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
        "auto-step-end",
        "Auto step end",
        "Rust conditionally ends combat when the end condition is met.",
        vec![17, 5],
    ));

    assert_eq!(
        execution.plan.decision_kind,
        CombatSessionAutomaticStepDecisionKind::ConditionalEnd
    );
    assert_eq!(execution.auto_candidate, None);
    let control = execution
        .control
        .as_ref()
        .expect("conditional end step has control readout");
    assert!(control.accepted);
    assert_eq!(
        control.command_kind,
        CombatControlCommandKind::EndIfConditionMet
    );
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.lifecycle_transition_log().len(), 2);
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

    for index in 0..5 {
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
            CombatSessionAutomaticStepDecisionKind::ConditionalEnd,
        ]
    );
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
    assert_eq!(session.combat_log().len(), 2);
    assert_eq!(session.audit_log().len(), 2);
    assert_eq!(session.control_history().len(), 3);
    assert_eq!(
        session.snapshot().combat_end_condition.condition_kind,
        CombatEndConditionKind::NoActiveEnemies
    );
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
    assert_eq!(readout.executed_step_count, 5);
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
            CombatSessionAutomaticStepDecisionKind::ConditionalEnd,
        ]
    );
    assert_eq!(
        readout.final_snapshot.lifecycle.phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(readout.final_snapshot.combat_log.len(), 2);
    assert_eq!(readout.final_snapshot.audit_log.len(), 2);
    assert_eq!(session.control_history().len(), 3);
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
        expected_run.decision_kind,
        expected_run.executed_step_count,
    ));

    assert!(readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunReplayDecisionKind::Verified
    );
    assert_eq!(readout.decision_kind.code(), "verified");
    assert!(readout.final_state_fingerprint_matches);
    assert!(readout.run_decision_kind_matches);
    assert!(readout.executed_step_count_matches);
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
        expected_run.decision_kind,
        expected_run.executed_step_count + 1,
    ));

    assert!(!readout.accepted);
    assert_eq!(
        readout.decision_kind,
        CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence
    );
    assert_eq!(readout.decision_kind.code(), "mismatchedEvidence");
    assert!(readout.final_state_fingerprint_matches);
    assert!(readout.run_decision_kind_matches);
    assert!(!readout.executed_step_count_matches);
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
fn session_runtime_selected_candidate_submission_accepts_miss_noop() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

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
    assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 18);
    assert!(snapshot.current_state.combatants[1].conditions.is_empty());
    assert_eq!(snapshot.action_usage_log.len(), 1);
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
