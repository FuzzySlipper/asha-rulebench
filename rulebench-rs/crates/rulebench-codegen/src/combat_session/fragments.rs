use crate::combat_session::scalars::*;
use crate::ts_emit::{ts_string, ts_string_array};

use rulebench_fixtures::{
    ActionResourceLedgerReadout, ActionResourceState, ActionResourceTransitionEntry,
    ActionUsageEntry, ActionUsageSummary, ActiveModifier, CombatControlHistoryEntry,
    CombatEndConditionReadout, CombatLogEntry, CombatSessionScriptStepReadout,
    CombatSessionStepSummary, CombatTurnOrder, CombatantVitalityEntry, CombatantVitalitySummary,
    CommandAttempt, CommandAuditEntry, CommandPreflightDecisionKind, CurrentActorActionOption,
    CurrentActorOptionSummary, CurrentActorOptionsUnavailableReason, CurrentActorTargetOption,
    LifecycleTransitionEntry, ModifierDurationExpirationEntry, ModifierDurationTransitionTrigger,
    RulebenchRejection, ScenarioProjection, TurnTransitionEntry,
};

pub(crate) fn render_action_resource_ledger(
    ledger: &ActionResourceLedgerReadout,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  combatants: [\n"));
    for combatant in &ledger.combatants {
        out.push_str(&format!("{indent}    {{\n"));
        out.push_str(&format!(
            "{indent}      combatantId: {},\n",
            ts_string(&combatant.combatant_id)
        ));
        out.push_str(&format!("{indent}      resources: [\n"));
        for resource in &combatant.resources {
            out.push_str(&render_action_resource_state(resource, indent));
        }
        out.push_str(&format!("{indent}      ],\n"));
        out.push_str(&format!("{indent}    }},\n"));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_action_resource_state(resource: &ActionResourceState, indent: &str) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}        {{\n"));
    out.push_str(&format!(
        "{indent}          kind: {},\n",
        ts_string(action_resource_kind(resource.kind))
    ));
    out.push_str(&format!(
        "{indent}          current: {},\n",
        resource.current
    ));
    out.push_str(&format!("{indent}          max: {},\n", resource.max));
    out.push_str(&format!(
        "{indent}          available: {},\n",
        resource.available
    ));
    out.push_str(&format!("{indent}        }},\n"));
    out
}

pub(crate) fn render_step_summary(step: &CombatSessionStepSummary, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  id: {},\n", ts_string(&step.id)));
    out.push_str(&format!("{indent}  index: {},\n", step.index));
    out.push_str(&format!("{indent}  title: {},\n", ts_string(&step.title)));
    out.push_str(&format!(
        "{indent}  summary: {},\n",
        ts_string(&step.summary)
    ));
    out.push_str(&format!(
        "{indent}  outcomeClass: {},\n",
        ts_string(outcome_class(step.outcome_class))
    ));
    out.push_str(&format!("{indent}  logIndex: {},\n", step.log_index));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_command(command: &CommandAttempt, indent: &str) -> String {
    let roll_stream = command
        .roll_stream
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  stepId: {},\n",
        ts_string(&command.step_id)
    ));
    out.push_str(&format!("{indent}  stepIndex: {},\n", command.step_index));
    out.push_str(&format!(
        "{indent}  actorId: {},\n",
        ts_string(&command.actor_id)
    ));
    out.push_str(&format!(
        "{indent}  actionId: {},\n",
        ts_string(&command.action_id)
    ));
    out.push_str(&format!(
        "{indent}  targetId: {},\n",
        ts_string(&command.target_id)
    ));
    out.push_str(&format!("{indent}  rollStream: [{roll_stream}],\n"));
    out.push_str(&format!(
        "{indent}  outcomeClass: {},\n",
        ts_string(outcome_class(command.outcome_class))
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_log_entry(entry: &CombatLogEntry, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  id: {},\n", ts_string(&entry.id)));
    out.push_str(&format!(
        "{indent}  stepId: {},\n",
        ts_string(&entry.step_id)
    ));
    out.push_str(&format!("{indent}  logIndex: {},\n", entry.log_index));
    out.push_str(&format!("{indent}  title: {},\n", ts_string(&entry.title)));
    out.push_str(&format!(
        "{indent}  summary: {},\n",
        ts_string(&entry.summary)
    ));
    out.push_str(&format!(
        "{indent}  outcomeClass: {},\n",
        ts_string(outcome_class(entry.outcome_class))
    ));
    out.push_str(&format!(
        "{indent}  eventTypes: {},\n",
        ts_string_array(&entry.event_types)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_control_history_entry(
    entry: &CombatControlHistoryEntry,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  commandKind: {},\n",
        ts_string(control_command_kind(entry.command_kind))
    ));
    out.push_str(&format!("{indent}  accepted: {},\n", entry.accepted));
    out.push_str(&format!(
        "{indent}  decisionKind: {},\n",
        ts_string(control_decision_kind(entry.decision_kind))
    ));
    out.push_str(&format!(
        "{indent}  previousLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(entry.previous_lifecycle_phase))
    ));
    out.push_str(&format!(
        "{indent}  nextLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(entry.next_lifecycle_phase))
    ));
    out.push_str(&format!(
        "{indent}  previousRoundNumber: {},\n",
        entry.previous_round_number
    ));
    out.push_str(&format!(
        "{indent}  previousTurnIndex: {},\n",
        entry.previous_turn_index
    ));
    out.push_str(&format!(
        "{indent}  previousActorId: {},\n",
        render_optional_string(&entry.previous_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  nextRoundNumber: {},\n",
        entry.next_round_number
    ));
    out.push_str(&format!(
        "{indent}  nextTurnIndex: {},\n",
        entry.next_turn_index
    ));
    out.push_str(&format!(
        "{indent}  nextActorId: {},\n",
        render_optional_string(&entry.next_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  lifecycleTransitionSequence: {},\n",
        render_optional_u32(entry.lifecycle_transition_sequence)
    ));
    out.push_str(&format!(
        "{indent}  turnTransitionSequence: {},\n",
        render_optional_u32(entry.turn_transition_sequence)
    ));
    out.push_str(&format!(
        "{indent}  stateBeforeFingerprint: {},\n",
        render_fingerprint(&entry.state_before_fingerprint, indent)
    ));
    out.push_str(&format!(
        "{indent}  stateAfterFingerprint: {},\n",
        render_fingerprint(&entry.state_after_fingerprint, indent)
    ));
    out.push_str(&format!(
        "{indent}  reason: {},\n",
        ts_string(&entry.reason)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}
pub(crate) fn render_command_audit_entry(entry: &CommandAuditEntry, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  id: {},\n", ts_string(&entry.id)));
    out.push_str(&format!(
        "{indent}  stepId: {},\n",
        ts_string(&entry.step_id)
    ));
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  outcomeClass: {},\n",
        ts_string(outcome_class(entry.outcome_class))
    ));
    out.push_str(&format!(
        "{indent}  decisionKind: {},\n",
        ts_string(command_decision_kind(entry.decision_kind))
    ));
    out.push_str(&format!(
        "{indent}  preflightDecisionKind: {},\n",
        render_optional_preflight_decision_kind(entry.preflight_decision_kind)
    ));
    out.push_str(&format!("{indent}  accepted: {},\n", entry.accepted));
    out.push_str(&format!(
        "{indent}  rejection: {},\n",
        render_optional_rejection(entry.rejection)
    ));
    out.push_str(&format!("{indent}  eventCount: {},\n", entry.event_count));
    out.push_str(&format!("{indent}  traceCount: {},\n", entry.trace_count));
    out.push_str(&format!("{indent}  rollConsumption: [\n"));
    for roll in &entry.roll_consumption {
        out.push_str(&format!("{indent}    {{\n"));
        out.push_str(&format!("{indent}      sequence: {},\n", roll.sequence));
        out.push_str(&format!(
            "{indent}      requestKind: {},\n",
            ts_string(roll.request_kind.code())
        ));
        out.push_str(&format!(
            "{indent}      suppliedValue: {},\n",
            render_optional_i32(roll.supplied_value)
        ));
        out.push_str(&format!("{indent}      consumed: {},\n", roll.consumed));
        out.push_str(&format!(
            "{indent}      reason: {},\n",
            ts_string(&roll.reason)
        ));
        out.push_str(&format!("{indent}    }},\n"));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!(
        "{indent}  stateBeforeFingerprint: {},\n",
        render_fingerprint(&entry.state_before_fingerprint, indent)
    ));
    out.push_str(&format!(
        "{indent}  stateAfterFingerprint: {},\n",
        render_fingerprint(&entry.state_after_fingerprint, indent)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_action_usage_entry(entry: &ActionUsageEntry, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  id: {},\n", ts_string(&entry.id)));
    out.push_str(&format!(
        "{indent}  stepId: {},\n",
        ts_string(&entry.step_id)
    ));
    out.push_str(&format!("{indent}  stepIndex: {},\n", entry.step_index));
    out.push_str(&format!("{indent}  roundNumber: {},\n", entry.round_number));
    out.push_str(&format!("{indent}  turnIndex: {},\n", entry.turn_index));
    out.push_str(&format!(
        "{indent}  actorId: {},\n",
        ts_string(&entry.actor_id)
    ));
    out.push_str(&format!(
        "{indent}  actionId: {},\n",
        ts_string(&entry.action_id)
    ));
    out.push_str(&format!(
        "{indent}  abilityId: {},\n",
        ts_string(&entry.ability_id)
    ));
    out.push_str(&format!(
        "{indent}  targetId: {},\n",
        ts_string(&entry.target_id)
    ));
    out.push_str(&format!(
        "{indent}  outcomeClass: {},\n",
        ts_string(outcome_class(entry.outcome_class))
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_turn_order(turn_order: &CombatTurnOrder, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  roundNumber: {},\n",
        turn_order.round_number
    ));
    out.push_str(&format!(
        "{indent}  currentTurnIndex: {},\n",
        turn_order.current_turn_index
    ));
    out.push_str(&format!(
        "{indent}  participantOrder: {},\n",
        ts_string_array(&turn_order.participant_order)
    ));
    out.push_str(&format!(
        "{indent}  currentActorId: {},\n",
        render_optional_string(&turn_order.current_actor_id)
    ));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_lifecycle_transition_entry(
    entry: &LifecycleTransitionEntry,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  trigger: {},\n",
        ts_string(lifecycle_transition_trigger(entry.trigger))
    ));
    out.push_str(&format!("{indent}  stepIndex: {},\n", entry.step_index));
    out.push_str(&format!(
        "{indent}  previousLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(entry.previous_phase))
    ));
    out.push_str(&format!(
        "{indent}  nextLifecyclePhase: {},\n",
        ts_string(lifecycle_phase(entry.next_phase))
    ));
    out.push_str(&format!(
        "{indent}  startedAtStep: {},\n",
        render_optional_u32(entry.started_at_step)
    ));
    out.push_str(&format!(
        "{indent}  endedAtStep: {},\n",
        render_optional_u32(entry.ended_at_step)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_turn_transition_entry(entry: &TurnTransitionEntry, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  previousRoundNumber: {},\n",
        entry.previous_round_number
    ));
    out.push_str(&format!(
        "{indent}  previousTurnIndex: {},\n",
        entry.previous_turn_index
    ));
    out.push_str(&format!(
        "{indent}  previousActorId: {},\n",
        render_optional_string(&entry.previous_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  nextRoundNumber: {},\n",
        entry.next_round_number
    ));
    out.push_str(&format!(
        "{indent}  nextTurnIndex: {},\n",
        entry.next_turn_index
    ));
    out.push_str(&format!(
        "{indent}  nextActorId: {},\n",
        render_optional_string(&entry.next_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  wrappedRound: {},\n",
        entry.wrapped_round
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_action_usage_summary(summary: &ActionUsageSummary, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  roundNumber: {},\n",
        summary.round_number
    ));
    out.push_str(&format!("{indent}  turnIndex: {},\n", summary.turn_index));
    out.push_str(&format!(
        "{indent}  currentActorId: {},\n",
        render_optional_string(&summary.current_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  usedActionCount: {},\n",
        summary.used_action_count
    ));
    out.push_str(&format!(
        "{indent}  usedActionIds: {},\n",
        ts_string_array(&summary.used_action_ids)
    ));
    out.push_str(&format!(
        "{indent}  usedAbilityIds: {},\n",
        ts_string_array(&summary.used_ability_ids)
    ));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_current_actor_options(
    options: &CurrentActorOptionSummary,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  roundNumber: {},\n",
        options.round_number
    ));
    out.push_str(&format!("{indent}  turnIndex: {},\n", options.turn_index));
    out.push_str(&format!(
        "{indent}  lifecyclePhase: {},\n",
        ts_string(lifecycle_phase(options.lifecycle_phase))
    ));
    out.push_str(&format!(
        "{indent}  currentActorId: {},\n",
        render_optional_string(&options.current_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  currentActorDefeated: {},\n",
        options.current_actor_defeated
    ));
    out.push_str(&format!("{indent}  available: {},\n", options.available));
    out.push_str(&format!(
        "{indent}  unavailableReason: {},\n",
        render_optional_current_actor_options_unavailable_reason(options.unavailable_reason)
    ));
    out.push_str(&format!("{indent}  actions: [\n"));
    for action in &options.actions {
        out.push_str(&render_current_actor_action_option(action, indent));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_current_actor_action_option(
    action: &CurrentActorActionOption,
    indent: &str,
) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}    {{\n"));
    out.push_str(&format!(
        "{indent}      actionId: {},\n",
        ts_string(&action.action_id)
    ));
    out.push_str(&format!(
        "{indent}      abilityId: {},\n",
        ts_string(&action.ability_id)
    ));
    out.push_str(&format!(
        "{indent}      actionName: {},\n",
        ts_string(&action.action_name)
    ));
    out.push_str(&format!("{indent}      targetOptions: [\n"));
    for target in &action.target_options {
        out.push_str(&render_current_actor_target_option(target, indent));
    }
    out.push_str(&format!("{indent}      ],\n"));
    out.push_str(&format!("{indent}    }},\n"));
    out
}

pub(crate) fn render_current_actor_target_option(
    target: &CurrentActorTargetOption,
    indent: &str,
) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}        {{\n"));
    out.push_str(&format!(
        "{indent}          targetId: {},\n",
        ts_string(&target.target_id)
    ));
    out.push_str(&format!(
        "{indent}          targetName: {},\n",
        ts_string(&target.target_name)
    ));
    out.push_str(&format!(
        "{indent}          currentHitPoints: {},\n",
        target.current_hit_points
    ));
    out.push_str(&format!(
        "{indent}          maxHitPoints: {},\n",
        target.max_hit_points
    ));
    out.push_str(&format!("{indent}        }},\n"));
    out
}

pub(crate) fn render_optional_current_actor_options_unavailable_reason(
    reason: Option<CurrentActorOptionsUnavailableReason>,
) -> String {
    reason
        .map(|inner| ts_string(inner.code()))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_combatant_vitality(
    summary: &CombatantVitalitySummary,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  combatants: [\n"));
    for combatant in &summary.combatants {
        out.push_str(&render_combatant_vitality_entry(combatant, indent));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!(
        "{indent}  activeCombatantIds: {},\n",
        ts_string_array(&summary.active_combatant_ids)
    ));
    out.push_str(&format!(
        "{indent}  defeatedCombatantIds: {},\n",
        ts_string_array(&summary.defeated_combatant_ids)
    ));
    out.push_str(&format!(
        "{indent}  activeCount: {},\n",
        summary.active_count
    ));
    out.push_str(&format!(
        "{indent}  defeatedCount: {},\n",
        summary.defeated_count
    ));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_combatant_vitality_entry(
    entry: &CombatantVitalityEntry,
    indent: &str,
) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}    {{\n"));
    out.push_str(&format!(
        "{indent}      combatantId: {},\n",
        ts_string(&entry.combatant_id)
    ));
    out.push_str(&format!(
        "{indent}      currentHitPoints: {},\n",
        entry.current_hit_points
    ));
    out.push_str(&format!(
        "{indent}      maxHitPoints: {},\n",
        entry.max_hit_points
    ));
    out.push_str(&format!("{indent}      defeated: {},\n", entry.defeated));
    out.push_str(&format!("{indent}    }},\n"));
    out
}

pub(crate) fn render_combat_end_condition(
    readout: &CombatEndConditionReadout,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  combatShouldEnd: {},\n",
        readout.combat_should_end
    ));
    out.push_str(&format!(
        "{indent}  conditionKind: {},\n",
        ts_string(combat_end_condition_kind(readout.condition_kind))
    ));
    out.push_str(&format!(
        "{indent}  activeAllyCount: {},\n",
        readout.active_ally_count
    ));
    out.push_str(&format!(
        "{indent}  activeEnemyCount: {},\n",
        readout.active_enemy_count
    ));
    out.push_str(&format!(
        "{indent}  defeatedAllyCount: {},\n",
        readout.defeated_ally_count
    ));
    out.push_str(&format!(
        "{indent}  defeatedEnemyCount: {},\n",
        readout.defeated_enemy_count
    ));
    out.push_str(&format!(
        "{indent}  reason: {},\n",
        ts_string(&readout.reason)
    ));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_optional_preflight_decision_kind(
    decision_kind: Option<CommandPreflightDecisionKind>,
) -> String {
    decision_kind
        .map(|inner| ts_string(preflight_decision_kind(inner)))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_optional_rejection(rejection: Option<RulebenchRejection>) -> String {
    rejection
        .map(|inner| ts_string(inner.code()))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_action_resource_transition_entry(
    entry: &ActionResourceTransitionEntry,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  transitionKind: {},\n",
        ts_string(action_resource_transition_kind(entry.transition_kind))
    ));
    out.push_str(&format!(
        "{indent}  combatantId: {},\n",
        ts_string(&entry.combatant_id)
    ));
    out.push_str(&format!(
        "{indent}  resourceKind: {},\n",
        ts_string(action_resource_kind(entry.resource_kind))
    ));
    out.push_str(&format!(
        "{indent}  previousResource: {},\n",
        render_action_resource_state_inline(&entry.previous_resource)
    ));
    out.push_str(&format!(
        "{indent}  nextResource: {},\n",
        render_action_resource_state_inline(&entry.next_resource)
    ));
    out.push_str(&format!(
        "{indent}  commandStepId: {},\n",
        render_optional_string(&entry.command_step_id)
    ));
    out.push_str(&format!(
        "{indent}  commandStepIndex: {},\n",
        render_optional_u32(entry.command_step_index)
    ));
    out.push_str(&format!(
        "{indent}  turnTransitionSequence: {},\n",
        render_optional_u32(entry.turn_transition_sequence)
    ));
    out.push_str(&format!(
        "{indent}  roundNumber: {},\n",
        render_optional_u32(entry.round_number)
    ));
    out.push_str(&format!(
        "{indent}  turnIndex: {},\n",
        render_optional_u32(entry.turn_index)
    ));
    out.push_str(&format!(
        "{indent}  currentActorId: {},\n",
        render_optional_string(&entry.current_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  reason: {},\n",
        ts_string(&entry.reason)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_action_resource_state_inline(resource: &ActionResourceState) -> String {
    format!(
        "{{ kind: {}, current: {}, max: {}, available: {} }}",
        ts_string(action_resource_kind(resource.kind)),
        resource.current,
        resource.max,
        resource.available
    )
}

pub(crate) fn render_modifier_duration_expiration_entry(
    entry: &ModifierDurationExpirationEntry,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", entry.sequence));
    out.push_str(&format!(
        "{indent}  combatantId: {},\n",
        ts_string(&entry.combatant_id)
    ));
    out.push_str(&format!(
        "{indent}  modifierId: {},\n",
        ts_string(&entry.modifier_id)
    ));
    out.push_str(&format!(
        "{indent}  previousModifier: {},\n",
        render_active_modifier(&entry.previous_modifier)
    ));
    out.push_str(&format!(
        "{indent}  nextModifier: {},\n",
        render_optional_active_modifier(&entry.next_modifier)
    ));
    out.push_str(&format!(
        "{indent}  trigger: {},\n",
        render_modifier_duration_trigger(&entry.trigger)
    ));
    out.push_str(&format!(
        "{indent}  turnTransitionSequence: {},\n",
        render_optional_u32(entry.turn_transition_sequence)
    ));
    out.push_str(&format!(
        "{indent}  roundNumber: {},\n",
        render_optional_u32(entry.round_number)
    ));
    out.push_str(&format!(
        "{indent}  turnIndex: {},\n",
        render_optional_u32(entry.turn_index)
    ));
    out.push_str(&format!(
        "{indent}  currentActorId: {},\n",
        render_optional_string(&entry.current_actor_id)
    ));
    out.push_str(&format!(
        "{indent}  reason: {},\n",
        ts_string(&entry.reason)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_modifier_duration_trigger(
    trigger: &ModifierDurationTransitionTrigger,
) -> String {
    match trigger {
        ModifierDurationTransitionTrigger::TurnBoundary => {
            "{ kind: 'turnBoundary', event: null }".to_string()
        }
        ModifierDurationTransitionTrigger::RoundBoundary => {
            "{ kind: 'roundBoundary', event: null }".to_string()
        }
        ModifierDurationTransitionTrigger::Event(event) => {
            format!("{{ kind: 'event', event: {} }}", ts_string(event))
        }
    }
}

pub(crate) fn render_active_modifier(modifier: &ActiveModifier) -> String {
    format!(
        "{{ modifierId: {}, sourceId: {}, label: {}, duration: {}, tenure: {}, stackingGroup: {}, stackingPolicy: {}, durationPolicy: {}, remainingTurns: {}, remainingRounds: {} }}",
        ts_string(&modifier.modifier_id),
        ts_string(&modifier.source_id),
        ts_string(&modifier.label),
        ts_string(&modifier.duration),
        ts_string(modifier_tenure(modifier.tenure)),
        ts_string(&modifier.stacking_group),
        ts_string(modifier.stacking_policy.code()),
        render_modifier_duration_policy(&modifier.duration_policy),
        render_optional_u32(modifier.remaining_turns),
        render_optional_u32(modifier.remaining_rounds)
    )
}

fn render_modifier_duration_policy(policy: &rulebench_fixtures::ModifierDurationPolicy) -> String {
    match policy {
        rulebench_fixtures::ModifierDurationPolicy::Permanent => {
            "{ kind: 'permanent', value: null, event: null }".to_string()
        }
        rulebench_fixtures::ModifierDurationPolicy::Turns(turns) => {
            format!("{{ kind: 'turns', value: {}, event: null }}", turns)
        }
        rulebench_fixtures::ModifierDurationPolicy::Rounds(rounds) => {
            format!("{{ kind: 'rounds', value: {}, event: null }}", rounds)
        }
        rulebench_fixtures::ModifierDurationPolicy::UntilEvent(event) => format!(
            "{{ kind: 'untilEvent', value: null, event: {} }}",
            ts_string(event)
        ),
    }
}

pub(crate) fn render_optional_active_modifier(modifier: &Option<ActiveModifier>) -> String {
    modifier
        .as_ref()
        .map(render_active_modifier)
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_script_step_readout(
    step: &CombatSessionScriptStepReadout,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("{indent}  sequence: {},\n", step.sequence));
    out.push_str(&format!("{indent}  id: {},\n", ts_string(&step.id)));
    out.push_str(&format!("{indent}  title: {},\n", ts_string(&step.title)));
    out.push_str(&format!(
        "{indent}  summary: {},\n",
        ts_string(&step.summary)
    ));
    out.push_str(&format!(
        "{indent}  commandKind: {},\n",
        ts_string(script_command_kind(step.command_kind))
    ));
    out.push_str(&format!("{indent}  accepted: {},\n", step.accepted));
    out.push_str(&format!(
        "{indent}  decisionKind: {},\n",
        ts_string(script_decision_kind(step.decision_kind))
    ));
    out.push_str(&format!("{indent}  reason: {},\n", ts_string(&step.reason)));
    out.push_str(&format!(
        "{indent}  stateBeforeFingerprint: {},\n",
        render_fingerprint(&step.state_before_fingerprint, indent)
    ));
    out.push_str(&format!(
        "{indent}  stateAfterFingerprint: {},\n",
        render_fingerprint(&step.state_after_fingerprint, indent)
    ));
    out.push_str(&format!(
        "{indent}  runtimeStepId: {},\n",
        render_optional_string(&step.runtime_step_id)
    ));
    out.push_str(&format!(
        "{indent}  commandAuditSequence: {},\n",
        render_optional_u32(step.command_audit_sequence)
    ));
    out.push_str(&format!(
        "{indent}  controlHistorySequence: {},\n",
        render_optional_u32(step.control_history_sequence)
    ));
    out.push_str(&format!("{indent}}},\n"));
    out
}

pub(crate) fn render_state(state: &ScenarioProjection, indent: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  summary: {},\n",
        ts_string(&state.summary)
    ));
    out.push_str(&format!("{indent}  combatants: [\n"));
    for combatant in &state.combatants {
        out.push_str(&format!("{indent}    {{\n"));
        out.push_str(&format!(
            "{indent}      id: {},\n",
            ts_string(&combatant.id)
        ));
        out.push_str(&format!(
            "{indent}      name: {},\n",
            ts_string(&combatant.name)
        ));
        out.push_str(&format!(
            "{indent}      hitPoints: {{ current: {}, max: {} }},\n",
            combatant.hit_points.current, combatant.hit_points.max
        ));
        out.push_str(&format!(
            "{indent}      conditions: {},\n",
            ts_string_array(&combatant.conditions)
        ));
        out.push_str(&format!("{indent}    }},\n"));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!("{indent}}}"));
    out
}
