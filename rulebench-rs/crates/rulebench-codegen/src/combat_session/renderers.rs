use crate::combat_session::fragments::*;
use crate::combat_session::scalars::*;
use crate::ts_emit::{render_scenario_readout, ts_string};

use rulebench_fixtures::{
    CombatControlHistoryReadout, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunReplayReadout, CombatSessionScriptReadout, CombatSessionStepReadout,
    CombatSessionSummary, ScenarioMetadata,
};

pub(crate) fn render_session_summary(summary: &CombatSessionSummary) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!("      id: {},\n", ts_string(&summary.id)));
    out.push_str(&format!("      title: {},\n", ts_string(&summary.title)));
    out.push_str(&format!(
        "      summary: {},\n",
        ts_string(&summary.summary)
    ));
    out.push_str(&format!(
        "      seedLabel: {},\n",
        ts_string(&summary.seed_label)
    ));
    out.push_str("      steps: [\n");
    for step in &summary.steps {
        out.push_str(&render_step_summary(step, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("    },\n");
    out
}

pub(crate) fn render_step_readout(readout: &CombatSessionStepReadout) -> String {
    let mut scenario = readout.scenario.clone();
    scenario.metadata = ScenarioMetadata {
        id: readout.step.id.clone(),
        title: readout.step.title.clone(),
        summary: readout.step.summary.clone(),
        seed_label: format!(
            "roll-stream:{}",
            readout
                .command
                .roll_stream
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
    };
    let target = readout
        .receipt
        .target_legality
        .as_ref()
        .expect("session readout has target legality");

    let mut out = String::from("    {\n");
    out.push_str(&format!(
        "      sessionId: {},\n",
        ts_string(&readout.session_id)
    ));
    out.push_str("      step: ");
    out.push_str(&render_step_summary(&readout.step, "      "));
    out.push_str("      command: ");
    out.push_str(&render_command(&readout.command, "      "));
    out.push_str("      scenarioReadout: ");
    out.push_str(&render_scenario_readout(
        &scenario,
        &readout.receipt.events,
        &readout.receipt.trace,
        &readout.state_after,
        target,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      combatLog: [\n");
    for entry in &readout.combat_log {
        out.push_str(&render_log_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      actionResourceLedger: ");
    out.push_str(&render_action_resource_ledger(
        &readout.action_resource_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      stateBefore: ");
    out.push_str(&render_state(&readout.state_before, "      "));
    out.push_str(",\n");
    out.push_str("      stateAfter: ");
    out.push_str(&render_state(&readout.state_after, "      "));
    out.push_str(",\n");
    out.push_str("    },\n");
    out
}

pub(crate) fn render_control_history_readout(readout: &CombatControlHistoryReadout) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!(
        "      sessionId: {},\n",
        ts_string(&readout.session_id)
    ));
    out.push_str(&format!("      title: {},\n", ts_string(&readout.title)));
    out.push_str(&format!(
        "      summary: {},\n",
        ts_string(&readout.summary)
    ));
    out.push_str("      history: [\n");
    for entry in &readout.history {
        out.push_str(&render_control_history_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("    },\n");
    out
}

pub(crate) fn render_script_readout(readout: &CombatSessionScriptReadout) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!(
        "      sessionId: {},\n",
        ts_string(&readout.session_id)
    ));
    out.push_str(&format!(
        "      scriptId: {},\n",
        ts_string(&readout.script_id)
    ));
    out.push_str(&format!("      title: {},\n", ts_string(&readout.title)));
    out.push_str(&format!(
        "      summary: {},\n",
        ts_string(&readout.summary)
    ));
    out.push_str("      steps: [\n");
    for step in &readout.steps {
        out.push_str(&render_script_step_readout(step, "        "));
    }
    out.push_str("      ],\n");
    out.push_str(&format!(
        "      finalLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(readout.final_snapshot.lifecycle.phase))
    ));
    out.push_str(&format!(
        "      finalStateFingerprint: {},\n",
        render_fingerprint(&readout.final_snapshot.current_state_fingerprint, "      ")
    ));
    out.push_str("      finalState: ");
    out.push_str(&render_state(
        &readout.final_snapshot.current_state,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalTurnOrder: ");
    out.push_str(&render_turn_order(
        &readout.final_snapshot.turn_order,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalActionResourceLedger: ");
    out.push_str(&render_action_resource_ledger(
        &readout.final_snapshot.action_resource_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalEquipmentLedger: ");
    out.push_str(&render_equipment_ledger(
        &readout.final_snapshot.equipment_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalClassBuildLedger: ");
    out.push_str(&render_class_build_ledger(
        &readout.final_snapshot.class_build_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      currentTurnActionUsage: ");
    out.push_str(&render_action_usage_summary(
        &readout.final_snapshot.current_turn_action_usage,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCurrentActorOptions: ");
    out.push_str(&render_current_actor_options(
        &readout.final_snapshot.current_actor_options,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCombatantVitality: ");
    out.push_str(&render_combatant_vitality(
        &readout.final_snapshot.combatant_vitality,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCombatEndCondition: ");
    out.push_str(&render_combat_end_condition(
        &readout.final_snapshot.combat_end_condition,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      lifecycleTransitionLog: [\n");
    for entry in &readout.final_snapshot.lifecycle_transition_log {
        out.push_str(&render_lifecycle_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      turnTransitionLog: [\n");
    for entry in &readout.final_snapshot.turn_transition_log {
        out.push_str(&render_turn_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      commandAuditLog: [\n");
    for entry in &readout.final_snapshot.audit_log {
        out.push_str(&render_command_audit_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      actionUsageLog: [\n");
    for entry in &readout.final_snapshot.action_usage_log {
        out.push_str(&render_action_usage_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      actionResourceTransitionLog: [\n");
    for entry in &readout.final_snapshot.action_resource_transition_log {
        out.push_str(&render_action_resource_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      equipmentTransitionLog: [\n");
    for entry in &readout.final_snapshot.equipment_transition_log {
        out.push_str(&render_equipment_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      currentReactionWindow: ");
    out.push_str(&render_optional_reaction_window(
        readout.final_snapshot.current_reaction_window.as_ref(),
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      reactionWindowLifecycleLog: [\n");
    for entry in &readout.final_snapshot.reaction_window_lifecycle_log {
        out.push_str(&render_reaction_window_lifecycle_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      reactionAuditLog: [\n");
    for entry in &readout.final_snapshot.reaction_audit_log {
        out.push_str(&render_reaction_audit_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      modifierDurationExpirationLog: [\n");
    for entry in &readout.final_snapshot.modifier_duration_expiration_log {
        out.push_str(&render_modifier_duration_expiration_entry(
            entry, "        ",
        ));
    }
    out.push_str("      ],\n");
    out.push_str("    },\n");
    out
}

pub(crate) fn render_automatic_run_readout(readout: &CombatSessionAutomaticRunReadout) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!("      id: {},\n", ts_string(&readout.id)));
    out.push_str(&format!("      title: {},\n", ts_string(&readout.title)));
    out.push_str(&format!(
        "      summary: {},\n",
        ts_string(&readout.summary)
    ));
    out.push_str(&format!("      accepted: {},\n", readout.accepted));
    out.push_str(&format!(
        "      decisionKind: {},\n",
        ts_string(automatic_run_decision_kind(readout.decision_kind))
    ));
    out.push_str(&format!("      maxSteps: {},\n", readout.max_steps));
    out.push_str("      policy: ");
    out.push_str(&render_automation_policy(&readout.policy, "      "));
    out.push_str(",\n");
    out.push_str(&format!(
        "      executedStepCount: {},\n",
        readout.executed_step_count
    ));
    out.push_str("      stepDecisions: [\n");
    for (sequence, step) in readout.steps.iter().enumerate() {
        out.push_str("        {\n");
        out.push_str(&format!("          sequence: {sequence},\n"));
        out.push_str(&format!("          accepted: {},\n", step.plan.accepted));
        out.push_str(&format!(
            "          decisionKind: {},\n",
            ts_string(automatic_step_decision_kind(step.plan.decision_kind))
        ));
        out.push_str(&format!(
            "          operationKind: {},\n",
            render_optional_automatic_step_operation_kind(step.plan.operation_kind)
        ));
        out.push_str(&format!(
            "          policyValidation: {},\n",
            render_automation_policy_validation(&step.plan.policy_validation, "          ")
        ));
        out.push_str(&format!(
            "          policyDecision: {},\n",
            render_automation_policy_decision(&step.plan.policy_decision, "          ")
        ));
        out.push_str(&format!(
            "          reason: {},\n",
            ts_string(&step.plan.reason)
        ));
        out.push_str("        },\n");
    }
    out.push_str("      ],\n");
    out.push_str("      policyDecisions: [\n");
    for decision in &readout.policy_decisions {
        out.push_str("        ");
        out.push_str(&render_automation_policy_decision(decision, "        "));
        out.push_str(",\n");
    }
    out.push_str("      ],\n");
    out.push_str(&format!(
        "      finalLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(readout.final_snapshot.lifecycle.phase))
    ));
    out.push_str("      finalState: ");
    out.push_str(&render_state(
        &readout.final_snapshot.current_state,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalActionResourceLedger: ");
    out.push_str(&render_action_resource_ledger(
        &readout.final_snapshot.action_resource_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalEquipmentLedger: ");
    out.push_str(&render_equipment_ledger(
        &readout.final_snapshot.equipment_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalClassBuildLedger: ");
    out.push_str(&render_class_build_ledger(
        &readout.final_snapshot.class_build_ledger,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCurrentActorOptions: ");
    out.push_str(&render_current_actor_options(
        &readout.final_snapshot.current_actor_options,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCombatantVitality: ");
    out.push_str(&render_combatant_vitality(
        &readout.final_snapshot.combatant_vitality,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      finalCombatEndCondition: ");
    out.push_str(&render_combat_end_condition(
        &readout.final_snapshot.combat_end_condition,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      combatLog: [\n");
    for entry in &readout.final_snapshot.combat_log {
        out.push_str(&render_log_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      commandAuditLog: [\n");
    for entry in &readout.final_snapshot.audit_log {
        out.push_str(&render_command_audit_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      lifecycleTransitionLog: [\n");
    for entry in &readout.final_snapshot.lifecycle_transition_log {
        out.push_str(&render_lifecycle_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      turnTransitionLog: [\n");
    for entry in &readout.final_snapshot.turn_transition_log {
        out.push_str(&render_turn_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      actionUsageLog: [\n");
    for entry in &readout.final_snapshot.action_usage_log {
        out.push_str(&render_action_usage_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      actionResourceTransitionLog: [\n");
    for entry in &readout.final_snapshot.action_resource_transition_log {
        out.push_str(&render_action_resource_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      equipmentTransitionLog: [\n");
    for entry in &readout.final_snapshot.equipment_transition_log {
        out.push_str(&render_equipment_transition_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      currentReactionWindow: ");
    out.push_str(&render_optional_reaction_window(
        readout.final_snapshot.current_reaction_window.as_ref(),
        "      ",
    ));
    out.push_str(",\n");
    out.push_str("      reactionWindowLifecycleLog: [\n");
    for entry in &readout.final_snapshot.reaction_window_lifecycle_log {
        out.push_str(&render_reaction_window_lifecycle_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      reactionAuditLog: [\n");
    for entry in &readout.final_snapshot.reaction_audit_log {
        out.push_str(&render_reaction_audit_entry(entry, "        "));
    }
    out.push_str("      ],\n");
    out.push_str("      modifierDurationExpirationLog: [\n");
    for entry in &readout.final_snapshot.modifier_duration_expiration_log {
        out.push_str(&render_modifier_duration_expiration_entry(
            entry, "        ",
        ));
    }
    out.push_str("      ],\n");
    out.push_str(&format!(
        "      combatLogEntryCount: {},\n",
        readout.final_snapshot.combat_log.len()
    ));
    out.push_str(&format!(
        "      auditEntryCount: {},\n",
        readout.final_snapshot.audit_log.len()
    ));
    out.push_str(&format!("      reason: {},\n", ts_string(&readout.reason)));
    out.push_str("    },\n");
    out
}

pub(crate) fn render_automatic_run_replay_readout(
    readout: &CombatSessionAutomaticRunReplayReadout,
) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!("      id: {},\n", ts_string(&readout.id)));
    out.push_str(&format!("      title: {},\n", ts_string(&readout.title)));
    out.push_str(&format!(
        "      summary: {},\n",
        ts_string(&readout.summary)
    ));
    out.push_str(&format!("      accepted: {},\n", readout.accepted));
    out.push_str(&format!(
        "      decisionKind: {},\n",
        ts_string(readout.decision_kind.code())
    ));
    out.push_str(&format!(
        "      expectedFinalStateFingerprint: {},\n",
        render_fingerprint(&readout.expected_final_state_fingerprint, "      ")
    ));
    out.push_str(&format!(
        "      actualFinalStateFingerprint: {},\n",
        render_fingerprint(&readout.actual_final_state_fingerprint, "      ")
    ));
    out.push_str(&format!(
        "      finalStateFingerprintMatches: {},\n",
        readout.final_state_fingerprint_matches
    ));
    out.push_str(&format!(
        "      expectedRunDecisionKind: {},\n",
        ts_string(readout.expected_run_decision_kind.code())
    ));
    out.push_str(&format!(
        "      actualRunDecisionKind: {},\n",
        ts_string(readout.actual_run_decision_kind.code())
    ));
    out.push_str(&format!(
        "      runDecisionKindMatches: {},\n",
        readout.run_decision_kind_matches
    ));
    out.push_str(&format!(
        "      expectedExecutedStepCount: {},\n",
        readout.expected_executed_step_count
    ));
    out.push_str(&format!(
        "      actualExecutedStepCount: {},\n",
        readout.actual_executed_step_count
    ));
    out.push_str(&format!(
        "      executedStepCountMatches: {},\n",
        readout.executed_step_count_matches
    ));
    out.push_str(&format!(
        "      policyDecisionsMatch: {},\n",
        readout.policy_decisions_match
    ));
    out.push_str(&format!(
        "      actionResourceTransitionLogMatches: {},\n",
        readout.action_resource_transition_log_matches
    ));
    out.push_str(&format!(
        "      equipmentLedgerMatches: {},\n",
        readout.equipment_ledger_matches
    ));
    out.push_str(&format!(
        "      classBuildLedgerMatches: {},\n",
        readout.class_build_ledger_matches
    ));
    out.push_str(&format!(
        "      equipmentTransitionLogMatches: {},\n",
        readout.equipment_transition_log_matches
    ));
    out.push_str(&format!(
        "      reactionWindowLifecycleLogMatches: {},\n",
        readout.reaction_window_lifecycle_log_matches
    ));
    out.push_str(&format!(
        "      reactionAuditLogMatches: {},\n",
        readout.reaction_audit_log_matches
    ));
    out.push_str(&format!(
        "      modifierDurationExpirationLogMatches: {},\n",
        readout.modifier_duration_expiration_log_matches
    ));
    out.push_str("      replayedRun: ");
    out.push_str(&render_nested_automatic_run_readout(
        &readout.replayed_run,
        "      ",
    ));
    out.push_str(",\n");
    out.push_str(&format!("      reason: {},\n", ts_string(&readout.reason)));
    out.push_str("    },\n");
    out
}

pub(crate) fn render_nested_automatic_run_readout(
    readout: &CombatSessionAutomaticRunReadout,
    indent: &str,
) -> String {
    let rendered = render_automatic_run_readout(readout);
    let trimmed = rendered.trim_end();
    let value = trimmed.strip_suffix(',').unwrap_or(trimmed);
    let mut lines = value.lines();
    let mut out = String::new();

    if let Some(first_line) = lines.next() {
        out.push_str(first_line.trim_start());
        out.push('\n');
    }
    for line in lines {
        out.push_str(indent);
        out.push_str(line.strip_prefix("    ").unwrap_or(line));
        out.push('\n');
    }

    if out.ends_with('\n') {
        out.pop();
    }
    out
}
