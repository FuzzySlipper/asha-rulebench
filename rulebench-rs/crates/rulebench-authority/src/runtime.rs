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
        let (receipt, state_after, should_apply_state) = if combat_has_ended {
            let state_after = self
                .state
                .project("No authority state changed; combat already ended.");
            (
                ended_combat_receipt(spec.intent.clone(), state_after.clone()),
                state_after,
                false,
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

                    (receipt, state_after, true)
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

        self.turn_order.advance_turn();
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
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
) -> CommandAuditEntry {
    CommandAuditEntry {
        id: format!("audit-{}", step.id),
        step_id: step.id.clone(),
        sequence: step.index,
        outcome_class: step.outcome_class,
        accepted: receipt.accepted,
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
