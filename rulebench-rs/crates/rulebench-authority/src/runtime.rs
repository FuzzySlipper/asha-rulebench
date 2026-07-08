use crate::model::*;
use crate::resolver::resolve_use_action;
use crate::state::CombatState;

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
    next_step_index: u32,
    lifecycle: CombatLifecycle,
}

impl CombatSessionState {
    pub fn new(session_id: impl Into<String>, scenario: RulebenchScenario) -> Self {
        let state = CombatState::from_scenario(&scenario);
        Self {
            session_id: session_id.into(),
            scenario,
            state,
            combat_log: Vec::new(),
            next_step_index: 0,
            lifecycle: CombatLifecycle::ready(),
        }
    }

    pub fn submit_command(&mut self, spec: CombatSessionCommandSpec) -> CombatSessionStepReadout {
        self.lifecycle.start_at_step(self.next_step_index);
        self.scenario = self.state.apply_to_scenario(self.scenario.clone());
        let state_before = self.state.project("State before command resolution.");
        let receipt = resolve_use_action(&self.scenario, spec.intent.clone(), &spec.roll_stream);
        let state_after = receipt
            .projection
            .clone()
            .expect("session runtime resolver always produces projection");

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

        self.combat_log.push(log_entry.clone());
        self.next_step_index += 1;
        self.state = CombatState::from_projection(&state_after);

        CombatSessionStepReadout {
            session_id: self.session_id.clone(),
            step,
            command,
            scenario: self.scenario.clone(),
            receipt,
            combat_log: vec![log_entry],
            state_before,
            state_after,
        }
    }

    pub fn combat_log(&self) -> &[CombatLogEntry] {
        &self.combat_log
    }

    pub fn next_step_index(&self) -> u32 {
        self.next_step_index
    }

    pub fn lifecycle(&self) -> &CombatLifecycle {
        &self.lifecycle
    }

    pub fn end_combat(&mut self) {
        self.lifecycle.end_at_step(self.next_step_index);
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
