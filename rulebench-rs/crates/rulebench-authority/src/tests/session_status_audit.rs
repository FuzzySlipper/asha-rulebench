use super::super::*;

#[test]
fn session_runtime_current_turn_action_usage_summarizes_accepted_hit() {
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
    let summary = session.current_turn_action_usage();

    assert_eq!(summary.round_number, 1);
    assert_eq!(summary.turn_index, 0);
    assert_eq!(summary.current_actor_id, Some("entity-adept".to_string()));
    assert_eq!(summary.used_action_count, 1);
    assert_eq!(summary.used_action_ids, vec!["hexing_bolt".to_string()]);
    assert_eq!(
        summary.used_ability_ids,
        vec!["ability.hexing-bolt".to_string()]
    );
}

#[test]
fn session_runtime_vitality_summary_reads_initial_active_combatants() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let summary = session.combatant_vitality();

    assert_eq!(summary.combatants.len(), 2);
    assert_eq!(summary.active_count, 2);
    assert_eq!(summary.defeated_count, 0);
    assert_eq!(
        summary.active_combatant_ids,
        vec!["entity-adept".to_string(), "entity-raider".to_string()]
    );
    assert!(summary.defeated_combatant_ids.is_empty());
    assert_eq!(summary.combatants[0].combatant_id, "entity-adept");
    assert_eq!(summary.combatants[0].current_hit_points, 24);
    assert_eq!(summary.combatants[0].max_hit_points, 24);
    assert!(!summary.combatants[0].defeated);
    assert_eq!(summary.combatants[1].combatant_id, "entity-raider");
    assert_eq!(summary.combatants[1].current_hit_points, 18);
    assert_eq!(summary.combatants[1].max_hit_points, 18);
    assert!(!summary.combatants[1].defeated);
}

#[test]
fn session_runtime_miss_preserves_prior_state() {
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

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-miss",
        "Runtime miss",
        "Adept misses Raider through the command runtime.",
        CommandOutcomeClass::AcceptedMiss,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    ));

    assert_eq!(readout.step.index, 1);
    assert!(readout.receipt.accepted);
    assert_eq!(
        readout
            .receipt
            .attack_roll
            .as_ref()
            .map(|roll| roll.outcome),
        Some(AttackOutcome::Miss)
    );
    assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
    assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
    assert_eq!(
        readout.state_after.combatants[1].conditions,
        vec!["rattled".to_string()]
    );
}

#[test]
fn session_runtime_vitality_summary_keeps_damaged_combatant_active_above_zero() {
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
    let summary = session.combatant_vitality();

    assert_eq!(summary.active_count, 2);
    assert_eq!(summary.defeated_count, 0);
    assert_eq!(summary.combatants[1].combatant_id, "entity-raider");
    assert_eq!(summary.combatants[1].current_hit_points, 9);
    assert_eq!(summary.combatants[1].max_hit_points, 18);
    assert!(!summary.combatants[1].defeated);
}

#[test]
fn session_runtime_combat_end_condition_reads_ongoing_combat() {
    let session = CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    let readout = session.combat_end_condition();

    assert!(!readout.combat_should_end);
    assert_eq!(readout.condition_kind, CombatEndConditionKind::Ongoing);
    assert_eq!(readout.condition_kind.code(), "ongoing");
    assert_eq!(readout.active_ally_count, 1);
    assert_eq!(readout.active_enemy_count, 1);
    assert_eq!(readout.defeated_ally_count, 0);
    assert_eq!(readout.defeated_enemy_count, 0);
    assert_eq!(
        readout.reason,
        "Combat can continue because both sides have active combatants."
    );
    assert_eq!(session.snapshot().combat_end_condition, readout);
}

#[test]
fn session_runtime_combat_end_condition_reports_no_active_enemies() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 0;
    let session = CombatSessionState::new("runtime-no-active-enemies", scenario);

    let readout = session.combat_end_condition();

    assert!(readout.combat_should_end);
    assert_eq!(
        readout.condition_kind,
        CombatEndConditionKind::NoActiveEnemies
    );
    assert_eq!(readout.condition_kind.code(), "noActiveEnemies");
    assert_eq!(readout.active_ally_count, 1);
    assert_eq!(readout.active_enemy_count, 0);
    assert_eq!(readout.defeated_ally_count, 0);
    assert_eq!(readout.defeated_enemy_count, 1);
    assert_eq!(
        readout.reason,
        "Combat should end because no active enemies remain."
    );
}

#[test]
fn session_runtime_combat_end_condition_reports_no_active_allies() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0].hit_points.current = -1;
    let session = CombatSessionState::new("runtime-no-active-allies", scenario);

    let readout = session.combat_end_condition();

    assert!(readout.combat_should_end);
    assert_eq!(
        readout.condition_kind,
        CombatEndConditionKind::NoActiveAllies
    );
    assert_eq!(readout.condition_kind.code(), "noActiveAllies");
    assert_eq!(readout.active_ally_count, 0);
    assert_eq!(readout.active_enemy_count, 1);
    assert_eq!(readout.defeated_ally_count, 1);
    assert_eq!(readout.defeated_enemy_count, 0);
    assert_eq!(
        readout.reason,
        "Combat should end because no active allies remain."
    );
}

#[test]
fn session_runtime_combat_end_condition_reports_no_active_combatants() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants.clear();
    let session = CombatSessionState::new("runtime-no-active-combatants", scenario);

    let readout = session.combat_end_condition();

    assert!(readout.combat_should_end);
    assert_eq!(
        readout.condition_kind,
        CombatEndConditionKind::NoActiveCombatants
    );
    assert_eq!(readout.condition_kind.code(), "noActiveCombatants");
    assert_eq!(readout.active_ally_count, 0);
    assert_eq!(readout.active_enemy_count, 0);
    assert_eq!(readout.defeated_ally_count, 0);
    assert_eq!(readout.defeated_enemy_count, 0);
    assert_eq!(
        readout.reason,
        "Combat should end because no active combatants remain."
    );
    assert_eq!(session.snapshot().combat_end_condition, readout);
}

#[test]
fn session_runtime_records_miss_noop_audit_entry() {
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

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-miss",
        "Runtime miss",
        "Adept misses Raider through the command runtime.",
        CommandOutcomeClass::AcceptedMiss,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    ));

    assert_eq!(readout.audit_entry.id, "audit-runtime-miss");
    assert_eq!(readout.audit_entry.sequence, 1);
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::AcceptedMiss
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::AcceptedByResolver
    );
    assert!(readout.audit_entry.accepted);
    assert_eq!(readout.audit_entry.rejection, None);
    assert_eq!(readout.audit_entry.event_count, 2);
    assert_eq!(
        readout.audit_entry.trace_count,
        readout.receipt.trace.len() as u32
    );
    assert_eq!(
        readout.audit_entry.roll_consumption,
        readout.receipt.roll_consumption
    );
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
}

#[test]
fn session_runtime_records_accepted_miss_action_usage() {
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

    assert_eq!(session.action_usage_log().len(), 2);
    let usage = &session.action_usage_log()[1];
    assert_eq!(usage.id, "action-usage-runtime-miss");
    assert_eq!(usage.step_id, "runtime-miss");
    assert_eq!(usage.step_index, 1);
    assert_eq!(usage.round_number, 1);
    assert_eq!(usage.turn_index, 0);
    assert_eq!(usage.actor_id, "entity-adept");
    assert_eq!(usage.action_id, "hexing_bolt");
    assert_eq!(usage.ability_id, "ability.hexing-bolt");
    assert_eq!(usage.target_id, "entity-raider");
    assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedMiss);
}

#[test]
fn session_runtime_current_turn_action_usage_includes_accepted_miss() {
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

    let summary = session.current_turn_action_usage();

    assert_eq!(summary.used_action_count, 2);
    assert_eq!(
        summary.used_action_ids,
        vec!["hexing_bolt".to_string(), "hexing_bolt".to_string()]
    );
    assert_eq!(
        summary.used_ability_ids,
        vec![
            "ability.hexing-bolt".to_string(),
            "ability.hexing-bolt".to_string()
        ]
    );
}

#[test]
fn session_runtime_rejection_preserves_prior_state() {
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

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-rejected",
        "Runtime rejected",
        "Adept targets themself through the command runtime.",
        CommandOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));

    assert_eq!(readout.step.index, 2);
    assert!(!readout.receipt.accepted);
    assert_eq!(
        readout.receipt.rejection,
        Some(RulebenchRejection::TargetLegalityFailed)
    );
    assert!(readout.receipt.events.is_empty());
    assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
    assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
    assert_eq!(
        readout.state_after.combatants[1].conditions,
        vec!["rattled".to_string()]
    );
}

#[test]
fn session_runtime_records_rejected_command_audit_entry() {
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

    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "runtime-rejected",
        "Runtime rejected",
        "Adept targets themself through the command runtime.",
        CommandOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));

    assert_eq!(readout.audit_entry.id, "audit-runtime-rejected");
    assert_eq!(readout.audit_entry.sequence, 1);
    assert_eq!(
        readout.audit_entry.outcome_class,
        CommandOutcomeClass::RejectedTargetLegality
    );
    assert_eq!(
        readout.audit_entry.decision_kind,
        CommandDecisionKind::RejectedByResolver
    );
    assert!(!readout.audit_entry.accepted);
    assert_eq!(
        readout.audit_entry.rejection,
        Some(RulebenchRejection::TargetLegalityFailed)
    );
    assert_eq!(readout.audit_entry.event_count, 0);
    assert_eq!(
        readout.audit_entry.trace_count,
        readout.receipt.trace.len() as u32
    );
    assert_eq!(
        readout.audit_entry.state_before_fingerprint,
        readout.audit_entry.state_after_fingerprint
    );
}

#[test]
fn session_runtime_rejected_command_does_not_record_action_usage() {
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
    let before_rejection = session.action_usage_log().to_vec();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-rejected",
        "Runtime rejected",
        "Adept targets themself through the command runtime.",
        CommandOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));

    assert_eq!(session.action_usage_log(), before_rejection.as_slice());
}

#[test]
fn session_runtime_current_turn_action_usage_ignores_rejected_commands() {
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
    let before_rejection = session.current_turn_action_usage();

    session.submit_command(CombatSessionCommandSpec::new(
        "runtime-rejected",
        "Runtime rejected",
        "Adept targets themself through the command runtime.",
        CommandOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    ));

    assert_eq!(session.current_turn_action_usage(), before_rejection);
}

#[test]
fn session_runtime_accumulates_log_entries_and_step_index() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    for (id, outcome_class, intent, rolls) in [
        (
            "runtime-hit",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ),
        (
            "runtime-miss",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ),
        (
            "runtime-rejected",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ),
    ] {
        session.submit_command(CombatSessionCommandSpec::new(
            id,
            id,
            id,
            outcome_class,
            intent,
            rolls,
        ));
    }

    assert_eq!(session.next_step_index(), 3);
    assert_eq!(session.lifecycle().started_at_step, Some(0));
    assert_eq!(
        session
            .combat_log()
            .iter()
            .map(|entry| entry.log_index)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        session
            .combat_log()
            .iter()
            .map(|entry| entry.event_types.len())
            .collect::<Vec<_>>(),
        vec![4, 2, 0]
    );
}

#[test]
fn session_runtime_accumulates_audit_entries_separately_from_combat_log() {
    let mut session =
        CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

    for (id, outcome_class, intent, rolls) in [
        (
            "runtime-hit",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ),
        (
            "runtime-miss",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ),
        (
            "runtime-rejected",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ),
    ] {
        session.submit_command(CombatSessionCommandSpec::new(
            id,
            id,
            id,
            outcome_class,
            intent,
            rolls,
        ));
    }

    assert_eq!(session.combat_log().len(), 3);
    assert_eq!(session.audit_log().len(), 3);
    assert_eq!(
        session
            .audit_log()
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "audit-runtime-hit",
            "audit-runtime-miss",
            "audit-runtime-rejected"
        ]
    );
    assert_eq!(
        session
            .audit_log()
            .iter()
            .map(|entry| entry.accepted)
            .collect::<Vec<_>>(),
        vec![true, true, false]
    );
    assert_eq!(
        session
            .audit_log()
            .iter()
            .map(|entry| entry.decision_kind)
            .collect::<Vec<_>>(),
        vec![
            CommandDecisionKind::AcceptedByResolver,
            CommandDecisionKind::AcceptedByResolver,
            CommandDecisionKind::RejectedByResolver
        ]
    );
    assert_eq!(
        session
            .audit_log()
            .iter()
            .map(|entry| entry.rejection)
            .collect::<Vec<_>>(),
        vec![None, None, Some(RulebenchRejection::TargetLegalityFailed)]
    );
    assert_eq!(
        session
            .audit_log()
            .iter()
            .map(|entry| entry.event_count)
            .collect::<Vec<_>>(),
        vec![4, 2, 0]
    );
}
