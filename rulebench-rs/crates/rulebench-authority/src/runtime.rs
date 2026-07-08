use crate::model::*;
use crate::resolver::resolve_use_action;
use crate::state::CombatState;
use crate::{fingerprint_projected_state, fingerprint_projection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCommandSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub outcome_class: CommandOutcomeClass,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionCommandSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        outcome_class: CommandOutcomeClass,
        intent: UseActionIntent,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            outcome_class,
            intent,
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionState {
    session_id: String,
    scenario: RulebenchScenario,
    state: CombatState,
    combat_log: Vec<CombatLogEntry>,
    audit_log: Vec<CommandAuditEntry>,
    action_usage_log: Vec<ActionUsageEntry>,
    turn_transition_log: Vec<TurnTransitionEntry>,
    next_step_index: u32,
    lifecycle: CombatLifecycle,
    turn_order: CombatTurnOrder,
}

impl CombatSessionState {
    pub fn new(session_id: impl Into<String>, scenario: RulebenchScenario) -> Self {
        let state = CombatState::from_scenario(&scenario);
        let turn_order = CombatTurnOrder::from_participant_order(
            scenario
                .combatants
                .iter()
                .map(|combatant| combatant.id.clone())
                .collect(),
        );
        Self {
            session_id: session_id.into(),
            scenario,
            state,
            combat_log: Vec::new(),
            audit_log: Vec::new(),
            action_usage_log: Vec::new(),
            turn_transition_log: Vec::new(),
            next_step_index: 0,
            lifecycle: CombatLifecycle::ready(),
            turn_order,
        }
    }

    pub fn submit_command(&mut self, spec: CombatSessionCommandSpec) -> CombatSessionStepReadout {
        self.scenario = self.state.apply_to_scenario(self.scenario.clone());
        let turn_context = self.turn_order.clone();
        let state_before = self.state.project("State before command resolution.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);
        let combat_has_ended = self.lifecycle.phase == CombatLifecyclePhase::Ended;
        let (receipt, state_after, should_apply_state, decision_kind) = if combat_has_ended {
            let state_after = self
                .state
                .project("No authority state changed; combat already ended.");
            (
                ended_combat_receipt(spec.intent.clone(), state_after.clone()),
                state_after,
                false,
                CommandDecisionKind::RejectedByLifecycle,
            )
        } else {
            match self.turn_order.current_actor_id.as_deref() {
                Some(current_actor_id) if spec.intent.actor_id != current_actor_id => {
                    let state_after = self.state.project(
                        "No authority state changed; actor is not the current turn actor.",
                    );
                    (
                        non_current_actor_receipt(
                            spec.intent.clone(),
                            current_actor_id,
                            state_after.clone(),
                        ),
                        state_after,
                        false,
                        CommandDecisionKind::RejectedByTurnOrder,
                    )
                }
                _ => {
                    self.lifecycle.start_at_step(self.next_step_index);
                    let receipt =
                        resolve_use_action(&self.scenario, spec.intent.clone(), &spec.roll_stream);
                    let state_after = receipt
                        .projection
                        .clone()
                        .expect("session runtime resolver always produces projection");
                    let decision_kind = if receipt.accepted {
                        CommandDecisionKind::AcceptedByResolver
                    } else {
                        CommandDecisionKind::RejectedByResolver
                    };

                    (receipt, state_after, true, decision_kind)
                }
            }
        };
        let state_after_fingerprint = fingerprint_projected_state(&state_after);

        let step = CombatSessionStepSummary {
            id: spec.id,
            index: self.next_step_index,
            title: spec.title,
            summary: spec.summary,
            outcome_class: spec.outcome_class,
            log_index: self.next_step_index + 1,
        };
        let command = CommandAttempt {
            step_id: step.id.clone(),
            step_index: step.index,
            actor_id: spec.intent.actor_id,
            action_id: spec.intent.action_id,
            target_id: spec.intent.target_id,
            roll_stream: spec.roll_stream,
            outcome_class: step.outcome_class,
        };
        let log_entry = combat_log_entry(&step, &receipt);
        let audit_entry = command_audit_entry(
            &step,
            &receipt,
            decision_kind,
            state_before_fingerprint,
            state_after_fingerprint,
        );
        let action_usage_entry = if receipt.accepted {
            self.scenario
                .action_by_id(&command.action_id)
                .map(|action| action_usage_entry(&step, &command, &turn_context, action))
        } else {
            None
        };

        self.combat_log.push(log_entry.clone());
        self.audit_log.push(audit_entry.clone());
        if let Some(entry) = action_usage_entry {
            self.action_usage_log.push(entry);
        }
        self.next_step_index += 1;
        if should_apply_state {
            self.state = CombatState::from_projection(&state_after);
        }

        CombatSessionStepReadout {
            session_id: self.session_id.clone(),
            step,
            command,
            scenario: self.scenario.clone(),
            receipt,
            combat_log: vec![log_entry],
            audit_entry,
            state_before,
            state_after,
        }
    }

    pub fn combat_log(&self) -> &[CombatLogEntry] {
        &self.combat_log
    }

    pub fn audit_log(&self) -> &[CommandAuditEntry] {
        &self.audit_log
    }

    pub fn action_usage_log(&self) -> &[ActionUsageEntry] {
        &self.action_usage_log
    }

    pub fn turn_transition_log(&self) -> &[TurnTransitionEntry] {
        &self.turn_transition_log
    }

    pub fn current_turn_action_usage(&self) -> ActionUsageSummary {
        current_turn_action_usage(&self.turn_order, &self.action_usage_log)
    }

    pub fn combatant_vitality(&self) -> CombatantVitalitySummary {
        let current_state = self.state.project("Current session state.");
        combatant_vitality_summary(&current_state)
    }

    pub fn next_step_index(&self) -> u32 {
        self.next_step_index
    }

    pub fn lifecycle(&self) -> &CombatLifecycle {
        &self.lifecycle
    }

    pub fn start_combat(&mut self) {
        self.lifecycle.start_at_step(self.next_step_index);
    }

    pub fn end_combat(&mut self) {
        self.lifecycle.end_at_step(self.next_step_index);
    }

    pub fn turn_order(&self) -> &CombatTurnOrder {
        &self.turn_order
    }

    pub fn advance_turn(&mut self) {
        if self.lifecycle.phase == CombatLifecyclePhase::Ended {
            return;
        }

        let previous_turn_order = self.turn_order.clone();
        self.turn_order.advance_turn();
        if self.turn_order != previous_turn_order {
            let transition = turn_transition_entry(
                self.turn_transition_log.len() as u32,
                &previous_turn_order,
                &self.turn_order,
            );
            self.turn_transition_log.push(transition);
        }
    }

    pub fn snapshot(&self) -> CombatSessionSnapshot {
        let current_state = self.state.project("Current session state.");
        let current_state_fingerprint = fingerprint_projection(&current_state);

        CombatSessionSnapshot {
            session_id: self.session_id.clone(),
            next_step_index: self.next_step_index,
            lifecycle: self.lifecycle.clone(),
            turn_order: self.turn_order.clone(),
            combat_log: self.combat_log.clone(),
            audit_log: self.audit_log.clone(),
            action_usage_log: self.action_usage_log.clone(),
            turn_transition_log: self.turn_transition_log.clone(),
            current_turn_action_usage: self.current_turn_action_usage(),
            combatant_vitality: combatant_vitality_summary(&current_state),
            current_state,
            current_state_fingerprint,
        }
    }
}

fn ended_combat_receipt(
    intent: UseActionIntent,
    projection: ScenarioProjection,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(RulebenchRejection::InvalidAction),
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        events: Vec::new(),
        trace: vec![
            TraceEntry::new(
                1,
                TracePhase::Proposal,
                TraceStatus::Info,
                "UseActionIntent received.",
                "Session command submitted after combat ended.",
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Rejected,
                "Command rejected by lifecycle.",
                "Combat is already ended.",
            ),
        ],
        projection: Some(projection),
    }
}

fn non_current_actor_receipt(
    intent: UseActionIntent,
    current_actor_id: &str,
    projection: ScenarioProjection,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(RulebenchRejection::InvalidAction),
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        events: Vec::new(),
        trace: vec![
            TraceEntry::new(
                1,
                TracePhase::Proposal,
                TraceStatus::Info,
                "UseActionIntent received.",
                "Session command submitted for actor outside the current turn.",
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Rejected,
                "Command rejected by turn order.",
                format!("Current actor is {current_actor_id}."),
            ),
        ],
        projection: Some(projection),
    }
}

fn combat_log_entry(step: &CombatSessionStepSummary, receipt: &RulebenchReceipt) -> CombatLogEntry {
    CombatLogEntry {
        id: format!("log-{}", step.id),
        step_id: step.id.clone(),
        log_index: step.log_index,
        title: step.title.clone(),
        summary: step.summary.clone(),
        outcome_class: step.outcome_class,
        event_types: receipt.events.iter().map(domain_event_type).collect(),
    }
}

fn command_audit_entry(
    step: &CombatSessionStepSummary,
    receipt: &RulebenchReceipt,
    decision_kind: CommandDecisionKind,
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
) -> CommandAuditEntry {
    CommandAuditEntry {
        id: format!("audit-{}", step.id),
        step_id: step.id.clone(),
        sequence: step.index,
        outcome_class: step.outcome_class,
        decision_kind,
        accepted: receipt.accepted,
        rejection: receipt.rejection,
        event_count: receipt.events.len() as u32,
        trace_count: receipt.trace.len() as u32,
        state_before_fingerprint,
        state_after_fingerprint,
    }
}

fn action_usage_entry(
    step: &CombatSessionStepSummary,
    command: &CommandAttempt,
    turn_context: &CombatTurnOrder,
    action: &ActionDefinition,
) -> ActionUsageEntry {
    ActionUsageEntry {
        id: format!("action-usage-{}", step.id),
        step_id: step.id.clone(),
        step_index: step.index,
        round_number: turn_context.round_number,
        turn_index: turn_context.current_turn_index,
        actor_id: command.actor_id.clone(),
        action_id: command.action_id.clone(),
        ability_id: action.ability_id.clone(),
        target_id: command.target_id.clone(),
        outcome_class: step.outcome_class,
    }
}

fn turn_transition_entry(
    sequence: u32,
    previous_turn_order: &CombatTurnOrder,
    next_turn_order: &CombatTurnOrder,
) -> TurnTransitionEntry {
    TurnTransitionEntry {
        sequence,
        previous_round_number: previous_turn_order.round_number,
        previous_turn_index: previous_turn_order.current_turn_index,
        previous_actor_id: previous_turn_order.current_actor_id.clone(),
        next_round_number: next_turn_order.round_number,
        next_turn_index: next_turn_order.current_turn_index,
        next_actor_id: next_turn_order.current_actor_id.clone(),
        wrapped_round: next_turn_order.round_number > previous_turn_order.round_number,
    }
}

fn current_turn_action_usage(
    turn_order: &CombatTurnOrder,
    action_usage_log: &[ActionUsageEntry],
) -> ActionUsageSummary {
    let current_actor_id = turn_order.current_actor_id.clone();
    let current_actor_matches = |entry: &ActionUsageEntry| {
        current_actor_id
            .as_deref()
            .is_some_and(|actor_id| entry.actor_id == actor_id)
    };
    let current_turn_entries = action_usage_log
        .iter()
        .filter(|entry| entry.round_number == turn_order.round_number)
        .filter(|entry| entry.turn_index == turn_order.current_turn_index)
        .filter(|entry| current_actor_matches(entry));

    let mut used_action_ids = Vec::new();
    let mut used_ability_ids = Vec::new();
    for entry in current_turn_entries {
        used_action_ids.push(entry.action_id.clone());
        used_ability_ids.push(entry.ability_id.clone());
    }

    ActionUsageSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        current_actor_id,
        used_action_count: used_action_ids.len() as u32,
        used_action_ids,
        used_ability_ids,
    }
}

fn combatant_vitality_summary(projection: &ScenarioProjection) -> CombatantVitalitySummary {
    let mut combatants = Vec::new();
    let mut active_combatant_ids = Vec::new();
    let mut defeated_combatant_ids = Vec::new();

    for combatant in &projection.combatants {
        let defeated = combatant.hit_points.current <= 0;
        let entry = CombatantVitalityEntry {
            combatant_id: combatant.id.clone(),
            current_hit_points: combatant.hit_points.current,
            max_hit_points: combatant.hit_points.max,
            defeated,
        };

        if defeated {
            defeated_combatant_ids.push(combatant.id.clone());
        } else {
            active_combatant_ids.push(combatant.id.clone());
        }
        combatants.push(entry);
    }

    CombatantVitalitySummary {
        active_count: active_combatant_ids.len() as u32,
        defeated_count: defeated_combatant_ids.len() as u32,
        combatants,
        active_combatant_ids,
        defeated_combatant_ids,
    }
}

fn domain_event_type(event: &DomainEvent) -> String {
    match event {
        DomainEvent::IntentShapeAccepted { .. } => "IntentShapeAccepted",
        DomainEvent::ActionUsed { .. } => "ActionUsed",
        DomainEvent::AttackRolled { .. } => "AttackRolled",
        DomainEvent::DamageApplied { .. } => "DamageApplied",
        DomainEvent::ModifierApplied { .. } => "ModifierApplied",
    }
    .to_string()
}
