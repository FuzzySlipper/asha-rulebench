use super::super::test_support::*;

#[test]
fn session_runtime_accepts_hit_command_and_advances_state() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ready);
    assert_eq!(session.lifecycle().started_at_step, None);
    assert_eq!(session.lifecycle().ended_at_step, None);

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(readout.session_id, "runtime-hexing-bolt");
    assert_eq!(readout.step.index, 0);
    assert_eq!(readout.step.log_index, 1);
    assert_eq!(readout.command.step_index, 0);
    assert!(readout.receipt.accepted);
    assert_eq!(readout.state_before.combatants[1].hit_points.current, 18);
    assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
    assert_eq!(
        readout.state_after.combatants[1].conditions,
        vec!["rattled".to_string()]
    );
    assert_eq!(readout.combat_log.len(), 1);
    assert_eq!(session.next_step_index(), 1);
    assert_eq!(session.combat_log().len(), 1);
    assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::InProgress);
    assert_eq!(session.lifecycle().started_at_step, Some(0));
    assert_eq!(session.lifecycle().ended_at_step, None);
}

#[test]
fn session_runtime_intent_command_derives_accepted_hit_outcome() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

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

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-hit",
        "Runtime derived hit",
        "Rust derives accepted hit outcome from receipt evidence.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert!(readout.receipt.accepted);
    assert_eq!(readout.step.outcome_class, CommandOutcomeClass::AcceptedHit);
    assert_eq!(
        readout.command.outcome_class,
        CommandOutcomeClass::AcceptedHit
    );
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::AcceptedHit
    );
    assert_eq!(
        readout.combat_log[0].outcome_class,
        CommandOutcomeClass::AcceptedHit
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(
        readout
            .audit_entry
            .preflight_decision_kind
            .map(CommandPreflightDecisionKind::code),
        Some("accepted")
    );
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
    assert_eq!(session.action_resource_transition_log().len(), 1);
    let transition = &session.action_resource_transition_log()[0];
    assert_eq!(transition.sequence, 0);
    assert_eq!(
        transition.transition_kind,
        ActionResourceTransitionKind::Spent
    );
    assert_eq!(transition.transition_kind.code(), "spent");
    assert_eq!(transition.combatant_id, "entity-adept");
    assert_eq!(transition.resource_kind, ActionResourceKind::StandardAction);
    assert_eq!(
        transition.previous_resource,
        ActionResourceState::standard_action_available()
    );
    assert_eq!(
        transition.next_resource,
        ActionResourceState::new(ActionResourceKind::StandardAction, 0, 1)
    );
    assert_eq!(
        transition.command_step_id,
        Some("runtime-derived-hit".to_string())
    );
    assert_eq!(transition.command_step_index, Some(0));
    assert_eq!(transition.turn_transition_sequence, None);
    assert_eq!(transition.round_number, Some(1));
    assert_eq!(transition.turn_index, Some(0));
    assert_eq!(
        transition.current_actor_id,
        Some("entity-adept".to_string())
    );
    assert_eq!(transition.reason, "Action resource spent.");
    assert_eq!(
        session.snapshot().action_resource_transition_log,
        session.action_resource_transition_log()
    );
}

#[test]
fn session_runtime_intent_command_rejects_spent_action_resource() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-first-action",
        "Runtime first action",
        "Adept spends the standard action resource.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let before = session.snapshot();

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-spent-resource-rejected",
        "Runtime spent resource rejection",
        "Rust rejects repeated use-action after the standard action is spent.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let after = session.snapshot();

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByPreflight
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByActionResource)
    );
    assert_eq!(
        readout
            .audit_entry
            .preflight_decision_kind
            .map(CommandPreflightDecisionKind::code),
        Some("rejectedByActionResource")
    );
    assert!(readout.receipt.events.is_empty());
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
    assert_eq!(
        after.current_state_fingerprint,
        before.current_state_fingerprint
    );
    assert_eq!(after.action_usage_log.len(), before.action_usage_log.len());
    assert_eq!(
        after.action_resource_transition_log,
        before.action_resource_transition_log
    );
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
}

#[test]
fn session_runtime_intent_command_resolver_rejection_does_not_spend_action_resource() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before = session.action_resource_ledger();

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-roll-missing-rejected",
        "Runtime missing roll rejection",
        "Rust rejects missing attack rolls without spending action resources.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![],
    ));
    let after = session.action_resource_ledger();

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::MissingAttackRoll)
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByResolver
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(before, after);
    assert!(session.action_usage_log().is_empty());
    assert!(session.action_resource_transition_log().is_empty());
}

#[test]
fn session_runtime_intent_command_derives_accepted_miss_outcome() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-miss",
        "Runtime derived miss",
        "Rust derives accepted miss outcome from receipt evidence.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    ));

    assert!(readout.receipt.accepted);
    assert_eq!(
        readout
            .receipt
            .attack_roll
            .as_ref()
            .map(|roll| roll.outcome),
        Some(AttackOutcome::Miss)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
    assert_eq!(
        readout.command.outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::Accepted)
    );
    assert_eq!(readout.audit_entry.event_count, 2);
}

#[test]
fn session_runtime_intent_command_derives_target_legality_rejection_outcome() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-target-rejected",
        "Runtime derived target rejection",
        "Rust derives target legality rejection outcome from receipt evidence.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::TargetLegalityFailed)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::RejectedTargetLegality
    );
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::RejectedTargetLegality
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByPreflight
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByTargetLegality)
    );
}

#[test]
fn session_runtime_intent_command_records_shape_preflight_rejection() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_fingerprint = session.snapshot().current_state_fingerprint;

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-shape-rejected",
        "Runtime derived shape rejection",
        "Rust rejects malformed commands before roll resolution.",
        UseActionIntent::new("", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));
    let after_snapshot = session.snapshot();

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::EmptyActorId)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::RejectedInvalidCommand
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByPreflight
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByShape)
    );
    assert_eq!(readout.audit_entry.event_count, 0);
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
    assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
    assert!(after_snapshot.action_usage_log.is_empty());
    assert_eq!(after_snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
}

#[test]
fn session_runtime_intent_command_records_action_ownership_preflight_rejection() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let before_fingerprint = session.snapshot().current_state_fingerprint;

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-action-owner-rejected",
        "Runtime derived action ownership rejection",
        "Rust rejects action ownership before roll resolution.",
        UseActionIntent::new("entity-raider", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));
    let after_snapshot = session.snapshot();

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByPreflight
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByActionOwnership)
    );
    assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
    assert!(after_snapshot.action_usage_log.is_empty());
}

#[test]
fn session_runtime_intent_command_records_target_lookup_preflight_rejection() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_fingerprint = session.snapshot().current_state_fingerprint;

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-target-lookup-rejected",
        "Runtime derived target lookup rejection",
        "Rust rejects missing targets before roll resolution.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-missing"),
        vec![17, 5],
    ));
    let after_snapshot = session.snapshot();

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidTarget)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::RejectedTargetLegality
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByPreflight
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByTargetLookup)
    );
    assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
    assert!(after_snapshot.action_usage_log.is_empty());
}

#[test]
fn session_runtime_intent_command_derives_lifecycle_invalid_outcome() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-ended-rejected",
        "Runtime derived ended rejection",
        "Rust derives invalid command outcome after combat end.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::RejectedInvalidCommand
    );
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::RejectedInvalidCommand
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByLifecycle
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByLifecycle)
    );
}

#[test]
fn session_runtime_intent_command_derives_turn_order_invalid_outcome() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();

    let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "runtime-derived-turn-rejected",
        "Runtime derived turn rejection",
        "Rust derives invalid command outcome for the wrong turn actor.",
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::InvalidAction)
    );
    assert_eq!(
        readout.step.outcome_class,
        CommandOutcomeClass::RejectedInvalidCommand
    );
    assert_eq!(
        readout.command.outcome_class,
        CommandOutcomeClass::RejectedInvalidCommand
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByTurnOrder
    );
    assert_eq!(
        readout.audit_entry.preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByTurnOrder)
    );
}

#[test]
fn session_runtime_runs_mixed_combat_script_with_reviewable_step_readback() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.run_script(CombatSessionScriptSpec::new(
        "opening-control-script",
        "Opening control script",
        "Explicit control and intent commands run through Rust authority.",
        vec![
            CombatSessionScriptStepSpec::control(
                "script-start",
                "Start combat",
                "Explicitly start combat before action resolution.",
                CombatControlCommandSpec::explicit_start(),
            ),
            CombatSessionScriptStepSpec::control(
                "script-repeat-start",
                "Repeat start",
                "Repeated start records rejected no-op control evidence.",
                CombatControlCommandSpec::explicit_start(),
            ),
            CombatSessionScriptStepSpec::intent(
                "script-hit-step",
                "Adept hit",
                "Adept uses Hexing Bolt against Raider.",
                CombatSessionIntentCommandSpec::new(
                    "script-runtime-hit",
                    "Script runtime hit",
                    "Scripted accepted hit command.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-advance-turn",
                "Advance turn",
                "Advance from Adept to Raider.",
                CombatControlCommandSpec::advance_turn(),
            ),
            CombatSessionScriptStepSpec::intent(
                "script-wrong-actor-step",
                "Wrong actor attempt",
                "Adept attempts another action on Raider turn.",
                CombatSessionIntentCommandSpec::new(
                    "script-runtime-wrong-actor",
                    "Script runtime wrong actor",
                    "Scripted turn-order rejection.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-end",
                "End combat",
                "Explicitly end combat after scripted commands.",
                CombatControlCommandSpec::explicit_end(),
            ),
        ],
    ));

    assert_eq!(readout.session_id, "runtime-hexing-bolt");
    assert_eq!(readout.script_id, "opening-control-script");
    assert_eq!(readout.steps.len(), 6);
    assert_eq!(
        readout
            .steps
            .iter()
            .map(|step| step.sequence)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );

    let start = &readout.steps[0];
    assert_eq!(start.command_kind, CombatSessionScriptCommandKind::Control);
    assert_eq!(start.command_kind.code(), "control");
    assert!(start.accepted);
    assert_eq!(
        start.decision_kind,
        CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
    );
    assert_eq!(start.decision_kind.code(), "accepted");
    assert_eq!(start.control_history_sequence, Some(0));
    assert_eq!(start.command_audit_sequence, None);
    assert_eq!(start.runtime_step_id, None);
    assert_eq!(start.reason, "Combat explicitly started.");

    let repeated_start = &readout.steps[1];
    assert!(!repeated_start.accepted);
    assert_eq!(
        repeated_start.decision_kind,
        CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::RejectedNoop)
    );
    assert_eq!(repeated_start.decision_kind.code(), "rejectedNoop");
    assert_eq!(repeated_start.control_history_sequence, Some(1));
    assert_eq!(
        repeated_start.state_before_fingerprint,
        repeated_start.state_after_fingerprint
    );
    assert_eq!(repeated_start.reason, "Combat is already in progress.");

    let hit = &readout.steps[2];
    assert_eq!(hit.command_kind, CombatSessionScriptCommandKind::Intent);
    assert_eq!(hit.command_kind.code(), "intent");
    assert!(hit.accepted);
    assert_eq!(
        hit.decision_kind,
        CombatSessionScriptDecisionKind::Intent(CommandDecisionKind::AcceptedByResolver)
    );
    assert_eq!(hit.decision_kind.code(), "acceptedByResolver");
    assert_eq!(hit.runtime_step_id, Some("script-runtime-hit".to_string()));
    assert_eq!(hit.command_audit_sequence, Some(0));
    assert_eq!(hit.control_history_sequence, None);
    assert_ne!(hit.state_before_fingerprint, hit.state_after_fingerprint);
    assert_eq!(hit.reason, "Intent command accepted by resolver.");

    let advance_turn = &readout.steps[3];
    assert!(advance_turn.accepted);
    assert_eq!(
        advance_turn.decision_kind,
        CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
    );
    assert_eq!(advance_turn.control_history_sequence, Some(2));
    assert_eq!(
        advance_turn.state_before_fingerprint,
        advance_turn.state_after_fingerprint
    );

    let wrong_actor = &readout.steps[4];
    assert!(!wrong_actor.accepted);
    assert_eq!(
        wrong_actor.decision_kind,
        CombatSessionScriptDecisionKind::Intent(CommandDecisionKind::RejectedByTurnOrder)
    );
    assert_eq!(wrong_actor.decision_kind.code(), "rejectedByTurnOrder");
    assert_eq!(
        wrong_actor.runtime_step_id,
        Some("script-runtime-wrong-actor".to_string())
    );
    assert_eq!(wrong_actor.command_audit_sequence, Some(1));
    assert_eq!(
        wrong_actor.state_before_fingerprint,
        wrong_actor.state_after_fingerprint
    );
    assert_eq!(wrong_actor.reason, "Intent command rejected by turn order.");

    let end = &readout.steps[5];
    assert!(end.accepted);
    assert_eq!(
        end.decision_kind,
        CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
    );
    assert_eq!(end.control_history_sequence, Some(3));
    assert_eq!(end.reason, "Combat explicitly ended.");

    assert_eq!(session.control_history().len(), 4);
    assert_eq!(session.audit_log().len(), 2);
    assert_eq!(session.combat_log().len(), 2);
    assert_eq!(session.next_step_index(), 2);
    assert_eq!(
        session.audit_log()[1].preflight_decision_kind,
        Some(CommandPreflightDecisionKind::RejectedByTurnOrder)
    );
    assert_eq!(
        readout.final_snapshot.lifecycle.phase,
        CombatLifecyclePhase::Ended
    );
    assert_eq!(readout.final_snapshot.lifecycle.ended_at_step, Some(2));
    assert_eq!(readout.final_snapshot.audit_log.len(), 2);
}

#[test]
fn session_runtime_empty_combat_script_is_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    let before_script = session.snapshot();

    let readout = session.run_script(CombatSessionScriptSpec::new(
        "empty-script",
        "Empty script",
        "No commands are submitted.",
        Vec::new(),
    ));

    assert_eq!(readout.session_id, "runtime-hexing-bolt");
    assert_eq!(readout.script_id, "empty-script");
    assert!(readout.steps.is_empty());
    assert_eq!(readout.final_snapshot, before_script);
    assert!(session.combat_log().is_empty());
    assert!(session.audit_log().is_empty());
    assert!(session.control_history().is_empty());
}

#[test]
fn session_runtime_script_selected_candidate_accepts_hit() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.run_script(CombatSessionScriptSpec::new(
        "selected-candidate-script",
        "Selected candidate script",
        "Script selects a Rust-visible command candidate for submission.",
        vec![CombatSessionScriptStepSpec::selected_candidate(
            "script-selected-hit-step",
            "Selected Hexing Bolt hit",
            "The current actor selected Hexing Bolt against Raider.",
            CombatSessionCandidateSelectionSpec::new(
                "script-selected-runtime-hit",
                "Script selected runtime hit",
                "Selected-candidate script command resolves as a hit.",
                "hexing_bolt",
                "entity-raider",
                vec![17, 5],
            ),
        )],
    ));

    assert_eq!(readout.steps.len(), 1);
    let step = &readout.steps[0];
    assert_eq!(
        step.command_kind,
        CombatSessionScriptCommandKind::SelectedCandidate
    );
    assert_eq!(step.command_kind.code(), "selectedCandidate");
    assert!(step.accepted);
    assert_eq!(
        step.decision_kind,
        CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(
            CommandDecisionKind::AcceptedByResolver
        )
    );
    assert_eq!(step.decision_kind.code(), "acceptedByResolver");
    assert_eq!(
        step.runtime_step_id,
        Some("script-selected-runtime-hit".to_string())
    );
    assert_eq!(step.command_audit_sequence, Some(0));
    assert_eq!(step.control_history_sequence, None);
    assert_ne!(step.state_before_fingerprint, step.state_after_fingerprint);
    assert_eq!(
        step.reason,
        "Selected candidate command accepted by resolver."
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
    assert_eq!(readout.final_snapshot, snapshot);
}

#[test]
fn session_runtime_script_selected_candidate_rejection_is_read_only() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.advance_turn();
    let before_script = session.snapshot();

    let readout = session.run_script(CombatSessionScriptSpec::new(
        "selected-candidate-rejected-script",
        "Selected candidate rejected script",
        "Script selects a candidate when Raider has no available action.",
        vec![CombatSessionScriptStepSpec::selected_candidate(
            "script-selected-unavailable-step",
            "Selected unavailable candidate",
            "The current actor has no matching command candidate.",
            CombatSessionCandidateSelectionSpec::new(
                "script-selected-unavailable",
                "Script selected unavailable",
                "Raider has no command candidates in this fixture.",
                "hexing_bolt",
                "entity-raider",
                vec![17, 5],
            ),
        )],
    ));

    assert_eq!(readout.steps.len(), 1);
    let step = &readout.steps[0];
    assert_eq!(
        step.command_kind,
        CombatSessionScriptCommandKind::SelectedCandidate
    );
    assert!(!step.accepted);
    assert_eq!(
        step.decision_kind,
        CombatSessionScriptDecisionKind::SelectedCandidateSelection(
            CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
        )
    );
    assert_eq!(step.decision_kind.code(), "rejectedByUnavailableCandidates");
    assert_eq!(step.runtime_step_id, None);
    assert_eq!(step.command_audit_sequence, None);
    assert_eq!(step.control_history_sequence, None);
    assert_eq!(step.state_before_fingerprint, step.state_after_fingerprint);
    assert_eq!(step.reason, "No command candidates are available.");

    let after_script = session.snapshot();
    assert_eq!(after_script, before_script);
    assert_eq!(readout.final_snapshot, before_script);
    assert!(session.combat_log().is_empty());
    assert!(session.audit_log().is_empty());
    assert!(session.action_usage_log().is_empty());
    assert_eq!(session.turn_transition_log().len(), 1);
}

#[test]
fn session_runtime_existing_command_spec_preserves_supplied_outcome_class() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
    session.end_combat();

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-compat-ended-rejected",
        "Runtime compatibility ended rejection",
        "Existing transcript spec preserves caller-supplied outcome class.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert!(!readout.receipt.accepted);
    assert_eq!(readout.step.outcome_class, CommandOutcomeClass::AcceptedHit);
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::AcceptedHit
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByLifecycle
    );
    assert_eq!(readout.audit_entry.preflight_decision_kind, None);
}

#[test]
fn session_runtime_records_accepted_hit_audit_entry() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-hit",
        "Runtime hit",
        "Adept hits Raider through the command runtime.",
        CommandOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    ));

    assert_eq!(readout.audit_entry.id, "audit-runtime-hit");
    assert_eq!(readout.audit_entry.step_id, "runtime-hit");
    assert_eq!(readout.audit_entry.sequence, 0);
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::AcceptedHit
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert_eq!(
        readout.audit_entry.decision_kind.code(),
        "acceptedByResolver"
    );
    assert!(readout.audit_entry.accepted);
    assert_eq!(readout.audit_entry.rejection, None);
    assert_eq!(readout.audit_entry.event_count, 4);
    assert_eq!(
        readout.audit_entry.trace_count,
        readout.receipt.trace.len() as u32
    );
    assert_eq!(
        readout.audit_entry.roll_consumption,
        readout.receipt.roll_consumption
    );
    assert_eq!(
        readout.audit_entry.state_before_fingerprint.algorithm,
        STATE_FINGERPRINT_ALGORITHM
    );
    assert_ne!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
    assert_eq!(session.audit_log(), &[readout.audit_entry]);
    assert_eq!(
        session.snapshot().audit_log[0].roll_consumption,
        readout.receipt.roll_consumption
    );
}

#[test]
fn session_runtime_records_accepted_hit_action_usage() {
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

    assert_eq!(session.action_usage_log().len(), 1);
    let usage = &session.action_usage_log()[0];
    assert_eq!(usage.id, "action-usage-runtime-hit");
    assert_eq!(usage.step_id, "runtime-hit");
    assert_eq!(usage.step_index, 0);
    assert_eq!(usage.round_number, 1);
    assert_eq!(usage.turn_index, 0);
    assert_eq!(usage.actor_id, "entity-adept");
    assert_eq!(usage.action_id, "hexing_bolt");
    assert_eq!(usage.ability_id, "ability.hexing-bolt");
    assert_eq!(usage.target_id, "entity-raider");
    assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedHit);
}
