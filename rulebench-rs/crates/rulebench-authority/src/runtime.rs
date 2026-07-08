use crate::model::*;
use crate::resolver::{resolve_use_action, target_legality_rejection, validate_target_legality};
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
pub struct CombatSessionIntentCommandSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionIntentCommandSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        intent: UseActionIntent,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            intent,
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateSelectionSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub action_id: String,
    pub target_id: String,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionCandidateSelectionSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        action_id: impl Into<String>,
        target_id: impl Into<String>,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            action_id: action_id.into(),
            target_id: target_id.into(),
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionCandidateSelectionDecisionKind {
    Accepted,
    RejectedByUnavailableCandidates,
    RejectedByMissingCandidate,
    RejectedByPreflight,
}

impl CombatSessionCandidateSelectionDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionCandidateSelectionDecisionKind::Accepted => "accepted",
            CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates => {
                "rejectedByUnavailableCandidates"
            }
            CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate => {
                "rejectedByMissingCandidate"
            }
            CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight => {
                "rejectedByPreflight"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateSelectionReadout {
    pub action_id: String,
    pub target_id: String,
    pub accepted: bool,
    pub decision_kind: CombatSessionCandidateSelectionDecisionKind,
    pub current_actor_id: Option<String>,
    pub unavailable_reason: Option<CurrentActorOptionsUnavailableReason>,
    pub preflight_decision_kind: Option<CommandPreflightDecisionKind>,
    pub rejection: Option<RulebenchRejection>,
    pub reason: String,
    pub command: Option<CombatSessionIntentCommandSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateExecutionReadout {
    pub selection: CombatSessionCandidateSelectionReadout,
    pub submitted_step: Option<CombatSessionStepReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub steps: Vec<CombatSessionScriptStepSpec>,
}

impl CombatSessionScriptSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<CombatSessionScriptStepSpec>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            steps,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptStepSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub command: CombatSessionScriptCommandSpec,
}

impl CombatSessionScriptStepSpec {
    pub fn intent(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatSessionIntentCommandSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::Intent(command),
        }
    }

    pub fn control(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatControlCommandSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::Control(command),
        }
    }

    pub fn selected_candidate(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatSessionCandidateSelectionSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::SelectedCandidate(command),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatSessionScriptCommandSpec {
    Intent(CombatSessionIntentCommandSpec),
    Control(CombatControlCommandSpec),
    SelectedCandidate(CombatSessionCandidateSelectionSpec),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionScriptCommandKind {
    Intent,
    Control,
    SelectedCandidate,
}

impl CombatSessionScriptCommandKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionScriptCommandKind::Intent => "intent",
            CombatSessionScriptCommandKind::Control => "control",
            CombatSessionScriptCommandKind::SelectedCandidate => "selectedCandidate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionScriptDecisionKind {
    Intent(CommandDecisionKind),
    Control(CombatControlDecisionKind),
    SelectedCandidateSubmitted(CommandDecisionKind),
    SelectedCandidateSelection(CombatSessionCandidateSelectionDecisionKind),
}

impl CombatSessionScriptDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionScriptDecisionKind::Intent(decision_kind) => decision_kind.code(),
            CombatSessionScriptDecisionKind::Control(decision_kind) => decision_kind.code(),
            CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(decision_kind) => {
                decision_kind.code()
            }
            CombatSessionScriptDecisionKind::SelectedCandidateSelection(decision_kind) => {
                decision_kind.code()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptStepReadout {
    pub sequence: u32,
    pub id: String,
    pub title: String,
    pub summary: String,
    pub command_kind: CombatSessionScriptCommandKind,
    pub accepted: bool,
    pub decision_kind: CombatSessionScriptDecisionKind,
    pub reason: String,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
    pub runtime_step_id: Option<String>,
    pub command_audit_sequence: Option<u32>,
    pub control_history_sequence: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptReadout {
    pub session_id: String,
    pub script_id: String,
    pub title: String,
    pub summary: String,
    pub steps: Vec<CombatSessionScriptStepReadout>,
    pub final_snapshot: CombatSessionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionState {
    session_id: String,
    scenario: RulebenchScenario,
    state: CombatState,
    combat_log: Vec<CombatLogEntry>,
    audit_log: Vec<CommandAuditEntry>,
    action_usage_log: Vec<ActionUsageEntry>,
    action_resource_transition_log: Vec<ActionResourceTransitionEntry>,
    modifier_duration_expiration_log: Vec<ModifierDurationExpirationEntry>,
    control_history: Vec<CombatControlHistoryEntry>,
    turn_transition_log: Vec<TurnTransitionEntry>,
    lifecycle_transition_log: Vec<LifecycleTransitionEntry>,
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
            action_resource_transition_log: Vec::new(),
            modifier_duration_expiration_log: Vec::new(),
            control_history: Vec::new(),
            turn_transition_log: Vec::new(),
            lifecycle_transition_log: Vec::new(),
            next_step_index: 0,
            lifecycle: CombatLifecycle::ready(),
            turn_order,
        }
    }

    pub fn submit_command(&mut self, spec: CombatSessionCommandSpec) -> CombatSessionStepReadout {
        self.submit_command_parts(
            spec.id,
            spec.title,
            spec.summary,
            Some(spec.outcome_class),
            spec.intent,
            spec.roll_stream,
            false,
        )
    }

    pub fn submit_intent_command(
        &mut self,
        spec: CombatSessionIntentCommandSpec,
    ) -> CombatSessionStepReadout {
        self.submit_command_parts(
            spec.id,
            spec.title,
            spec.summary,
            None,
            spec.intent,
            spec.roll_stream,
            true,
        )
    }

    pub fn run_script(&mut self, spec: CombatSessionScriptSpec) -> CombatSessionScriptReadout {
        let mut steps = Vec::with_capacity(spec.steps.len());
        for (index, step) in spec.steps.into_iter().enumerate() {
            steps.push(self.run_script_step(index as u32, step));
        }

        CombatSessionScriptReadout {
            session_id: self.session_id.clone(),
            script_id: spec.id,
            title: spec.title,
            summary: spec.summary,
            steps,
            final_snapshot: self.snapshot(),
        }
    }

    fn run_script_step(
        &mut self,
        sequence: u32,
        spec: CombatSessionScriptStepSpec,
    ) -> CombatSessionScriptStepReadout {
        match spec.command.clone() {
            CombatSessionScriptCommandSpec::Intent(command) => {
                let readout = self.submit_intent_command(command);
                combat_session_script_intent_step_readout(sequence, spec, &readout)
            }
            CombatSessionScriptCommandSpec::Control(command) => {
                let previous_control_history_len = self.control_history.len();
                let readout = self.submit_control_command(command);
                let control_history_sequence = self
                    .control_history
                    .get(previous_control_history_len)
                    .map(|entry| entry.sequence);
                combat_session_script_control_step_readout(
                    sequence,
                    spec,
                    &readout,
                    control_history_sequence,
                )
            }
            CombatSessionScriptCommandSpec::SelectedCandidate(command) => {
                let state_before_fingerprint = self.snapshot().current_state_fingerprint;
                let execution = self.submit_candidate_command(command);
                let state_after_fingerprint = self.snapshot().current_state_fingerprint;
                combat_session_script_selected_candidate_step_readout(
                    sequence,
                    spec,
                    &execution,
                    state_before_fingerprint,
                    state_after_fingerprint,
                )
            }
        }
    }

    fn submit_command_parts(
        &mut self,
        id: String,
        title: String,
        summary: String,
        outcome_class: Option<CommandOutcomeClass>,
        intent: UseActionIntent,
        roll_stream: Vec<i32>,
        preflight_enabled: bool,
    ) -> CombatSessionStepReadout {
        self.scenario = self.state.apply_to_scenario(self.scenario.clone());
        let turn_context = self.turn_order.clone();
        let state_before = self.state.project("State before command resolution.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);
        let preflight = if preflight_enabled {
            Some(command_preflight_readout(
                &self.lifecycle,
                self.turn_order.current_actor_id.clone(),
                &self.scenario,
                &self.state.action_resource_ledger(),
                intent.clone(),
            ))
        } else {
            None
        };
        let rejected_preflight = preflight.as_ref().filter(|readout| !readout.accepted);
        let combat_has_ended = self.lifecycle.phase == CombatLifecyclePhase::Ended;
        let (receipt, state_after, should_apply_state, decision_kind) = if let Some(preflight) =
            rejected_preflight
        {
            let state_after = self
                .state
                .project("No authority state changed; command preflight rejected.");
            (
                preflight_rejected_receipt(preflight, state_after.clone()),
                state_after,
                false,
                command_decision_kind_for_preflight(preflight.decision_kind),
            )
        } else if combat_has_ended {
            let state_after = self
                .state
                .project("No authority state changed; combat already ended.");
            (
                ended_combat_receipt(intent.clone(), state_after.clone()),
                state_after,
                false,
                CommandDecisionKind::RejectedByLifecycle,
            )
        } else {
            match self.turn_order.current_actor_id.as_deref() {
                Some(current_actor_id) if intent.actor_id != current_actor_id => {
                    let state_after = self.state.project(
                        "No authority state changed; actor is not the current turn actor.",
                    );
                    (
                        non_current_actor_receipt(
                            intent.clone(),
                            current_actor_id,
                            state_after.clone(),
                        ),
                        state_after,
                        false,
                        CommandDecisionKind::RejectedByTurnOrder,
                    )
                }
                _ => {
                    self.start_lifecycle(LifecycleTransitionTrigger::CommandStart);
                    let receipt = resolve_use_action(&self.scenario, intent.clone(), &roll_stream);
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
        let outcome_class = outcome_class.unwrap_or_else(|| derive_command_outcome_class(&receipt));
        let preflight_decision_kind = preflight.as_ref().map(|readout| readout.decision_kind);

        let step = CombatSessionStepSummary {
            id,
            index: self.next_step_index,
            title,
            summary,
            outcome_class,
            log_index: self.next_step_index + 1,
        };
        let command = CommandAttempt {
            step_id: step.id.clone(),
            step_index: step.index,
            actor_id: intent.actor_id,
            action_id: intent.action_id,
            target_id: intent.target_id,
            roll_stream,
            outcome_class: step.outcome_class,
        };
        let log_entry = combat_log_entry(&step, &receipt);
        let audit_entry = command_audit_entry(
            &step,
            &receipt,
            decision_kind,
            preflight_decision_kind,
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
        if receipt.accepted {
            let spend = self
                .state
                .spend_action_resource(&command.actor_id, ActionResourceKind::StandardAction);
            self.record_action_resource_spend_transition(&step, &spend);
        }
        self.next_step_index += 1;
        if should_apply_state {
            self.apply_receipt_effects_to_state(&receipt);
        }

        CombatSessionStepReadout {
            session_id: self.session_id.clone(),
            step,
            command,
            scenario: self.scenario.clone(),
            receipt,
            combat_log: vec![log_entry],
            action_resource_ledger: self.state.action_resource_ledger(),
            audit_entry,
            state_before,
            state_after,
        }
    }

    fn apply_receipt_effects_to_state(&mut self, receipt: &RulebenchReceipt) {
        if !receipt.accepted {
            return;
        }

        let (Some(damage), Some(modifier)) = (receipt.damage.as_ref(), receipt.modifier.as_ref())
        else {
            return;
        };

        self.state.apply_hit(damage, modifier);
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

    pub fn action_resource_transition_log(&self) -> &[ActionResourceTransitionEntry] {
        &self.action_resource_transition_log
    }

    pub fn modifier_duration_expiration_log(&self) -> &[ModifierDurationExpirationEntry] {
        &self.modifier_duration_expiration_log
    }

    pub fn control_history(&self) -> &[CombatControlHistoryEntry] {
        &self.control_history
    }

    pub fn turn_transition_log(&self) -> &[TurnTransitionEntry] {
        &self.turn_transition_log
    }

    pub fn lifecycle_transition_log(&self) -> &[LifecycleTransitionEntry] {
        &self.lifecycle_transition_log
    }

    pub fn current_turn_action_usage(&self) -> ActionUsageSummary {
        current_turn_action_usage(&self.turn_order, &self.action_usage_log)
    }

    pub fn action_resource_ledger(&self) -> ActionResourceLedgerReadout {
        self.state.action_resource_ledger()
    }

    pub fn combatant_vitality(&self) -> CombatantVitalitySummary {
        let current_state = self.state.project("Current session state.");
        combatant_vitality_summary(&current_state)
    }

    pub fn combat_end_condition(&self) -> CombatEndConditionReadout {
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        combat_end_condition_readout(&current_scenario)
    }

    pub fn current_actor_options(&self) -> CurrentActorOptionSummary {
        let current_state = self.state.project("Current session state.");
        current_actor_option_summary(
            &self.lifecycle,
            &self.turn_order,
            &self.scenario,
            &current_state,
        )
    }

    pub fn current_actor_command_candidates(&self) -> CommandCandidateSummary {
        let current_state = self.state.project("Current session state.");
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        let current_actor_options = current_actor_option_summary(
            &self.lifecycle,
            &self.turn_order,
            &current_scenario,
            &current_state,
        );

        current_actor_command_candidates(
            &self.lifecycle,
            &current_scenario,
            &self.state.action_resource_ledger(),
            current_actor_options,
        )
    }

    pub fn plan_candidate_command(
        &self,
        spec: CombatSessionCandidateSelectionSpec,
    ) -> CombatSessionCandidateSelectionReadout {
        let candidates = self.current_actor_command_candidates();
        plan_candidate_command(spec, candidates)
    }

    pub fn submit_candidate_command(
        &mut self,
        spec: CombatSessionCandidateSelectionSpec,
    ) -> CombatSessionCandidateExecutionReadout {
        let selection = self.plan_candidate_command(spec);
        let submitted_step = selection
            .command
            .clone()
            .map(|command| self.submit_intent_command(command));

        CombatSessionCandidateExecutionReadout {
            selection,
            submitted_step,
        }
    }

    pub fn preflight_command(&self, intent: UseActionIntent) -> CommandPreflightReadout {
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        command_preflight_readout(
            &self.lifecycle,
            self.turn_order.current_actor_id.clone(),
            &current_scenario,
            &self.state.action_resource_ledger(),
            intent,
        )
    }

    pub fn next_step_index(&self) -> u32 {
        self.next_step_index
    }

    pub fn lifecycle(&self) -> &CombatLifecycle {
        &self.lifecycle
    }

    pub fn start_combat(&mut self) {
        self.start_lifecycle(LifecycleTransitionTrigger::ExplicitStart);
    }

    pub fn end_combat(&mut self) {
        self.end_lifecycle(LifecycleTransitionTrigger::ExplicitEnd);
    }

    pub fn submit_control_command(
        &mut self,
        spec: CombatControlCommandSpec,
    ) -> CombatControlReadout {
        let readout = match spec.kind {
            CombatControlCommandKind::ExplicitStart => self.submit_explicit_start_control(),
            CombatControlCommandKind::ExplicitEnd => self.submit_explicit_end_control(),
            CombatControlCommandKind::AdvanceTurn => self.submit_advance_turn_control(),
            CombatControlCommandKind::EndIfConditionMet => self.submit_conditional_end_control(),
        };
        let history_entry =
            combat_control_history_entry(self.control_history.len() as u32, &readout);
        self.control_history.push(history_entry);
        readout
    }

    pub fn turn_order(&self) -> &CombatTurnOrder {
        &self.turn_order
    }

    pub fn advance_turn(&mut self) -> TurnAdvanceReadout {
        let previous_turn_order = self.turn_order.clone();
        let state_before = self.state.project("State before turn advancement.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);

        if self.lifecycle.phase == CombatLifecyclePhase::Ended {
            return rejected_turn_advance_readout(
                TurnAdvanceDecisionKind::RejectedByLifecycle,
                previous_turn_order,
                self.turn_order.clone(),
                state_before_fingerprint,
                "Combat is already ended.",
            );
        }

        if self.turn_order.participant_order.is_empty() {
            return rejected_turn_advance_readout(
                TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder,
                previous_turn_order,
                self.turn_order.clone(),
                state_before_fingerprint,
                "Turn order has no participants.",
            );
        }

        self.turn_order.advance_turn();
        let transition = turn_transition_entry(
            self.turn_transition_log.len() as u32,
            &previous_turn_order,
            &self.turn_order,
        );
        self.turn_transition_log.push(transition.clone());
        if let Some(current_actor_id) = self.turn_order.current_actor_id.clone() {
            let refresh = self
                .state
                .refresh_action_resource(&current_actor_id, ActionResourceKind::StandardAction);
            self.record_action_resource_refresh_transition(&transition, &refresh);
        }
        if let Some(previous_actor_id) = transition.previous_actor_id.as_deref() {
            let expirations = self.state.expire_temporary_modifiers_for(previous_actor_id);
            self.record_modifier_duration_expiration_transitions(&transition, &expirations);
        }
        let state_after = self.state.project("State after turn advancement.");
        let state_after_fingerprint = fingerprint_projected_state(&state_after);

        TurnAdvanceReadout {
            accepted: true,
            decision_kind: TurnAdvanceDecisionKind::Advanced,
            previous_turn_order,
            next_turn_order: self.turn_order.clone(),
            transition: Some(transition),
            state_before_fingerprint,
            state_after_fingerprint,
            reason: "Turn advanced to the next participant.".to_string(),
        }
    }

    pub fn snapshot(&self) -> CombatSessionSnapshot {
        let current_state = self.state.project("Current session state.");
        let current_state_fingerprint = fingerprint_projection(&current_state);
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());

        CombatSessionSnapshot {
            session_id: self.session_id.clone(),
            next_step_index: self.next_step_index,
            lifecycle: self.lifecycle.clone(),
            lifecycle_transition_log: self.lifecycle_transition_log.clone(),
            turn_order: self.turn_order.clone(),
            combat_log: self.combat_log.clone(),
            audit_log: self.audit_log.clone(),
            action_usage_log: self.action_usage_log.clone(),
            action_resource_transition_log: self.action_resource_transition_log.clone(),
            modifier_duration_expiration_log: self.modifier_duration_expiration_log.clone(),
            turn_transition_log: self.turn_transition_log.clone(),
            action_resource_ledger: self.state.action_resource_ledger(),
            current_turn_action_usage: self.current_turn_action_usage(),
            combatant_vitality: combatant_vitality_summary(&current_state),
            combat_end_condition: combat_end_condition_readout(&current_scenario),
            current_actor_options: current_actor_option_summary(
                &self.lifecycle,
                &self.turn_order,
                &current_scenario,
                &current_state,
            ),
            current_state,
            current_state_fingerprint,
        }
    }

    fn start_lifecycle(&mut self, trigger: LifecycleTransitionTrigger) {
        let previous_lifecycle = self.lifecycle.clone();
        self.lifecycle.start_at_step(self.next_step_index);
        self.record_lifecycle_transition(trigger, self.next_step_index, previous_lifecycle);
    }

    fn end_lifecycle(&mut self, trigger: LifecycleTransitionTrigger) {
        let previous_lifecycle = self.lifecycle.clone();
        self.lifecycle.end_at_step(self.next_step_index);
        self.record_lifecycle_transition(trigger, self.next_step_index, previous_lifecycle);
    }

    fn record_lifecycle_transition(
        &mut self,
        trigger: LifecycleTransitionTrigger,
        step_index: u32,
        previous_lifecycle: CombatLifecycle,
    ) {
        if self.lifecycle == previous_lifecycle {
            return;
        }

        self.lifecycle_transition_log
            .push(LifecycleTransitionEntry {
                sequence: self.lifecycle_transition_log.len() as u32,
                trigger,
                step_index,
                previous_phase: previous_lifecycle.phase,
                next_phase: self.lifecycle.phase,
                started_at_step: self.lifecycle.started_at_step,
                ended_at_step: self.lifecycle.ended_at_step,
            });
    }

    fn record_action_resource_spend_transition(
        &mut self,
        step: &CombatSessionStepSummary,
        spend: &ActionResourceSpendReadout,
    ) {
        let (Some(previous_resource), Some(next_resource)) =
            (spend.previous_resource.clone(), spend.next_resource.clone())
        else {
            return;
        };
        if !spend.accepted {
            return;
        }

        self.action_resource_transition_log
            .push(ActionResourceTransitionEntry {
                sequence: self.action_resource_transition_log.len() as u32,
                transition_kind: ActionResourceTransitionKind::Spent,
                combatant_id: spend.combatant_id.clone(),
                resource_kind: spend.resource_kind,
                previous_resource,
                next_resource,
                command_step_id: Some(step.id.clone()),
                command_step_index: Some(step.index),
                turn_transition_sequence: None,
                round_number: Some(self.turn_order.round_number),
                turn_index: Some(self.turn_order.current_turn_index),
                current_actor_id: self.turn_order.current_actor_id.clone(),
                reason: spend.reason.clone(),
            });
    }

    fn record_action_resource_refresh_transition(
        &mut self,
        transition: &TurnTransitionEntry,
        refresh: &ActionResourceRefreshReadout,
    ) {
        let (Some(previous_resource), Some(next_resource)) = (
            refresh.previous_resource.clone(),
            refresh.next_resource.clone(),
        ) else {
            return;
        };
        if !refresh.accepted {
            return;
        }

        self.action_resource_transition_log
            .push(ActionResourceTransitionEntry {
                sequence: self.action_resource_transition_log.len() as u32,
                transition_kind: ActionResourceTransitionKind::Refreshed,
                combatant_id: refresh.combatant_id.clone(),
                resource_kind: refresh.resource_kind,
                previous_resource,
                next_resource,
                command_step_id: None,
                command_step_index: None,
                turn_transition_sequence: Some(transition.sequence),
                round_number: Some(transition.next_round_number),
                turn_index: Some(transition.next_turn_index),
                current_actor_id: transition.next_actor_id.clone(),
                reason: refresh.reason.clone(),
            });
    }

    fn record_modifier_duration_expiration_transitions(
        &mut self,
        transition: &TurnTransitionEntry,
        expirations: &[ModifierDurationExpirationReadout],
    ) {
        for expiration in expirations.iter().filter(|expiration| expiration.accepted) {
            self.modifier_duration_expiration_log
                .push(ModifierDurationExpirationEntry {
                    sequence: self.modifier_duration_expiration_log.len() as u32,
                    combatant_id: expiration.combatant_id.clone(),
                    modifier_id: expiration.modifier_id.clone(),
                    previous_modifier: expiration.previous_modifier.clone(),
                    next_modifier: expiration.next_modifier.clone(),
                    turn_transition_sequence: transition.sequence,
                    round_number: transition.next_round_number,
                    turn_index: transition.next_turn_index,
                    current_actor_id: transition.next_actor_id.clone(),
                    reason: expiration.reason.clone(),
                });
        }
    }

    fn submit_explicit_start_control(&mut self) -> CombatControlReadout {
        let previous_lifecycle = self.lifecycle.clone();
        let previous_turn_order = self.turn_order.clone();
        let state_before = self.state.project("State before explicit start control.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);
        let lifecycle_transition_count = self.lifecycle_transition_log.len();

        let (accepted, decision_kind, reason) = match self.lifecycle.phase {
            CombatLifecyclePhase::Ready => {
                self.start_lifecycle(LifecycleTransitionTrigger::ExplicitStart);
                (
                    true,
                    CombatControlDecisionKind::Accepted,
                    "Combat explicitly started.",
                )
            }
            CombatLifecyclePhase::InProgress => (
                false,
                CombatControlDecisionKind::RejectedNoop,
                "Combat is already in progress.",
            ),
            CombatLifecyclePhase::Ended => (
                false,
                CombatControlDecisionKind::RejectedByLifecycle,
                "Combat is already ended.",
            ),
        };

        combat_control_readout(
            CombatControlCommandKind::ExplicitStart,
            accepted,
            decision_kind,
            previous_lifecycle,
            self.lifecycle.clone(),
            previous_turn_order,
            self.turn_order.clone(),
            lifecycle_transition_since(&self.lifecycle_transition_log, lifecycle_transition_count),
            None,
            state_before_fingerprint,
            fingerprint_projected_state(&self.state.project("State after explicit start control.")),
            reason,
        )
    }

    fn submit_explicit_end_control(&mut self) -> CombatControlReadout {
        let previous_lifecycle = self.lifecycle.clone();
        let previous_turn_order = self.turn_order.clone();
        let state_before = self.state.project("State before explicit end control.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);
        let lifecycle_transition_count = self.lifecycle_transition_log.len();

        let (accepted, decision_kind, reason) =
            if self.lifecycle.phase == CombatLifecyclePhase::Ended {
                (
                    false,
                    CombatControlDecisionKind::RejectedByLifecycle,
                    "Combat is already ended.",
                )
            } else {
                self.end_lifecycle(LifecycleTransitionTrigger::ExplicitEnd);
                (
                    true,
                    CombatControlDecisionKind::Accepted,
                    "Combat explicitly ended.",
                )
            };

        combat_control_readout(
            CombatControlCommandKind::ExplicitEnd,
            accepted,
            decision_kind,
            previous_lifecycle,
            self.lifecycle.clone(),
            previous_turn_order,
            self.turn_order.clone(),
            lifecycle_transition_since(&self.lifecycle_transition_log, lifecycle_transition_count),
            None,
            state_before_fingerprint,
            fingerprint_projected_state(&self.state.project("State after explicit end control.")),
            reason,
        )
    }

    fn submit_conditional_end_control(&mut self) -> CombatControlReadout {
        let previous_lifecycle = self.lifecycle.clone();
        let previous_turn_order = self.turn_order.clone();
        let state_before = self.state.project("State before conditional end control.");
        let state_before_fingerprint = fingerprint_projected_state(&state_before);
        let lifecycle_transition_count = self.lifecycle_transition_log.len();
        let end_condition = self.combat_end_condition();

        let (accepted, decision_kind, reason) =
            if self.lifecycle.phase == CombatLifecyclePhase::Ended {
                (
                    false,
                    CombatControlDecisionKind::RejectedByLifecycle,
                    "Combat is already ended.".to_string(),
                )
            } else if end_condition.combat_should_end {
                self.end_lifecycle(LifecycleTransitionTrigger::ConditionalEnd);
                (
                    true,
                    CombatControlDecisionKind::Accepted,
                    format!("Combat conditionally ended. {}", end_condition.reason),
                )
            } else {
                (
                    false,
                    CombatControlDecisionKind::RejectedByEndCondition,
                    format!("Combat end condition is not met. {}", end_condition.reason),
                )
            };

        combat_control_readout(
            CombatControlCommandKind::EndIfConditionMet,
            accepted,
            decision_kind,
            previous_lifecycle,
            self.lifecycle.clone(),
            previous_turn_order,
            self.turn_order.clone(),
            lifecycle_transition_since(&self.lifecycle_transition_log, lifecycle_transition_count),
            None,
            state_before_fingerprint,
            fingerprint_projected_state(
                &self.state.project("State after conditional end control."),
            ),
            reason,
        )
    }

    fn submit_advance_turn_control(&mut self) -> CombatControlReadout {
        let previous_lifecycle = self.lifecycle.clone();
        let previous_turn_order = self.turn_order.clone();
        let turn_advance = self.advance_turn();
        let decision_kind = combat_control_decision_kind_for_turn_advance(&turn_advance);

        combat_control_readout(
            CombatControlCommandKind::AdvanceTurn,
            turn_advance.accepted,
            decision_kind,
            previous_lifecycle,
            self.lifecycle.clone(),
            previous_turn_order,
            self.turn_order.clone(),
            None,
            Some(turn_advance.clone()),
            turn_advance.state_before_fingerprint,
            turn_advance.state_after_fingerprint,
            turn_advance.reason,
        )
    }
}

fn combat_control_readout(
    command_kind: CombatControlCommandKind,
    accepted: bool,
    decision_kind: CombatControlDecisionKind,
    previous_lifecycle: CombatLifecycle,
    next_lifecycle: CombatLifecycle,
    previous_turn_order: CombatTurnOrder,
    next_turn_order: CombatTurnOrder,
    lifecycle_transition: Option<LifecycleTransitionEntry>,
    turn_advance: Option<TurnAdvanceReadout>,
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
    reason: impl Into<String>,
) -> CombatControlReadout {
    CombatControlReadout {
        command_kind,
        accepted,
        decision_kind,
        previous_lifecycle,
        next_lifecycle,
        previous_turn_order,
        next_turn_order,
        lifecycle_transition,
        turn_advance,
        state_before_fingerprint,
        state_after_fingerprint,
        reason: reason.into(),
    }
}

fn combat_control_history_entry(
    sequence: u32,
    readout: &CombatControlReadout,
) -> CombatControlHistoryEntry {
    CombatControlHistoryEntry {
        sequence,
        command_kind: readout.command_kind,
        accepted: readout.accepted,
        decision_kind: readout.decision_kind,
        previous_lifecycle_phase: readout.previous_lifecycle.phase,
        next_lifecycle_phase: readout.next_lifecycle.phase,
        previous_round_number: readout.previous_turn_order.round_number,
        previous_turn_index: readout.previous_turn_order.current_turn_index,
        previous_actor_id: readout.previous_turn_order.current_actor_id.clone(),
        next_round_number: readout.next_turn_order.round_number,
        next_turn_index: readout.next_turn_order.current_turn_index,
        next_actor_id: readout.next_turn_order.current_actor_id.clone(),
        lifecycle_transition_sequence: readout
            .lifecycle_transition
            .as_ref()
            .map(|transition| transition.sequence),
        turn_transition_sequence: readout
            .turn_advance
            .as_ref()
            .and_then(|turn_advance| turn_advance.transition.as_ref())
            .map(|transition| transition.sequence),
        state_before_fingerprint: readout.state_before_fingerprint.clone(),
        state_after_fingerprint: readout.state_after_fingerprint.clone(),
        reason: readout.reason.clone(),
    }
}

fn lifecycle_transition_since(
    lifecycle_transition_log: &[LifecycleTransitionEntry],
    previous_len: usize,
) -> Option<LifecycleTransitionEntry> {
    lifecycle_transition_log.get(previous_len).cloned()
}

fn combat_control_decision_kind_for_turn_advance(
    turn_advance: &TurnAdvanceReadout,
) -> CombatControlDecisionKind {
    match turn_advance.decision_kind {
        TurnAdvanceDecisionKind::Advanced => CombatControlDecisionKind::Accepted,
        TurnAdvanceDecisionKind::RejectedByLifecycle => {
            CombatControlDecisionKind::RejectedByLifecycle
        }
        TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder => {
            CombatControlDecisionKind::RejectedByEmptyTurnOrder
        }
    }
}

fn combat_session_script_intent_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatSessionStepReadout,
) -> CombatSessionScriptStepReadout {
    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::Intent,
        accepted: readout.audit_entry.accepted,
        decision_kind: CombatSessionScriptDecisionKind::Intent(readout.audit_entry.decision_kind),
        reason: intent_script_step_reason(&readout.audit_entry),
        state_before_fingerprint: readout.audit_entry.state_before_fingerprint.clone(),
        state_after_fingerprint: readout.audit_entry.state_after_fingerprint.clone(),
        runtime_step_id: Some(readout.step.id.clone()),
        command_audit_sequence: Some(readout.audit_entry.sequence),
        control_history_sequence: None,
    }
}

fn combat_session_script_control_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatControlReadout,
    control_history_sequence: Option<u32>,
) -> CombatSessionScriptStepReadout {
    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::Control,
        accepted: readout.accepted,
        decision_kind: CombatSessionScriptDecisionKind::Control(readout.decision_kind),
        reason: readout.reason.clone(),
        state_before_fingerprint: readout.state_before_fingerprint.clone(),
        state_after_fingerprint: readout.state_after_fingerprint.clone(),
        runtime_step_id: None,
        command_audit_sequence: None,
        control_history_sequence,
    }
}

fn combat_session_script_selected_candidate_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatSessionCandidateExecutionReadout,
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
) -> CombatSessionScriptStepReadout {
    if let Some(submitted_step) = &readout.submitted_step {
        return CombatSessionScriptStepReadout {
            sequence,
            id: spec.id,
            title: spec.title,
            summary: spec.summary,
            command_kind: CombatSessionScriptCommandKind::SelectedCandidate,
            accepted: submitted_step.audit_entry.accepted,
            decision_kind: CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(
                submitted_step.audit_entry.decision_kind,
            ),
            reason: selected_candidate_submitted_script_step_reason(&submitted_step.audit_entry),
            state_before_fingerprint: submitted_step.audit_entry.state_before_fingerprint.clone(),
            state_after_fingerprint: submitted_step.audit_entry.state_after_fingerprint.clone(),
            runtime_step_id: Some(submitted_step.step.id.clone()),
            command_audit_sequence: Some(submitted_step.audit_entry.sequence),
            control_history_sequence: None,
        };
    }

    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::SelectedCandidate,
        accepted: false,
        decision_kind: CombatSessionScriptDecisionKind::SelectedCandidateSelection(
            readout.selection.decision_kind,
        ),
        reason: readout.selection.reason.clone(),
        state_before_fingerprint,
        state_after_fingerprint,
        runtime_step_id: None,
        command_audit_sequence: None,
        control_history_sequence: None,
    }
}

fn intent_script_step_reason(audit_entry: &CommandAuditEntry) -> String {
    match audit_entry.decision_kind {
        CommandDecisionKind::AcceptedByResolver => "Intent command accepted by resolver.",
        CommandDecisionKind::RejectedByResolver => "Intent command rejected by resolver.",
        CommandDecisionKind::RejectedByPreflight => "Intent command rejected by preflight.",
        CommandDecisionKind::RejectedByLifecycle => "Intent command rejected by lifecycle.",
        CommandDecisionKind::RejectedByTurnOrder => "Intent command rejected by turn order.",
    }
    .to_string()
}

fn selected_candidate_submitted_script_step_reason(audit_entry: &CommandAuditEntry) -> String {
    match audit_entry.decision_kind {
        CommandDecisionKind::AcceptedByResolver => {
            "Selected candidate command accepted by resolver."
        }
        CommandDecisionKind::RejectedByResolver => {
            "Selected candidate command rejected by resolver."
        }
        CommandDecisionKind::RejectedByPreflight => {
            "Selected candidate command rejected by preflight."
        }
        CommandDecisionKind::RejectedByLifecycle => {
            "Selected candidate command rejected by lifecycle."
        }
        CommandDecisionKind::RejectedByTurnOrder => {
            "Selected candidate command rejected by turn order."
        }
    }
    .to_string()
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

fn command_preflight_readout(
    lifecycle: &CombatLifecycle,
    current_actor_id: Option<String>,
    scenario: &RulebenchScenario,
    action_resources: &ActionResourceLedgerReadout,
    intent: UseActionIntent,
) -> CommandPreflightReadout {
    if intent.actor_id.is_empty() {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByShape,
            Some(RulebenchRejection::EmptyActorId),
            current_actor_id,
            None,
            None,
            "Actor id is empty.",
        );
    }
    if intent.action_id.is_empty() {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByShape,
            Some(RulebenchRejection::EmptyActionId),
            current_actor_id,
            None,
            None,
            "Action id is empty.",
        );
    }
    if intent.target_id.is_empty() {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByShape,
            Some(RulebenchRejection::EmptyTargetId),
            current_actor_id,
            None,
            None,
            "Target id is empty.",
        );
    }

    if lifecycle.phase == CombatLifecyclePhase::Ended {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByLifecycle,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            None,
            None,
            "Combat is already ended.",
        );
    }

    if current_actor_id
        .as_deref()
        .is_some_and(|actor_id| intent.actor_id != actor_id)
    {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByTurnOrder,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            None,
            None,
            "Actor is not the current turn actor.",
        );
    }

    let Some(actor) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.actor_id)
    else {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByActorLookup,
            Some(RulebenchRejection::InvalidActor),
            current_actor_id,
            None,
            None,
            "Actor is not present in the current scenario.",
        );
    };

    let Some(action) = scenario.action_by_id(&intent.action_id) else {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByActionLookup,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            None,
            None,
            "Action is not present in the current scenario.",
        );
    };

    if action.actor_id != intent.actor_id {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByActionOwnership,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            None,
            None,
            "Action does not belong to the proposed actor.",
        );
    }

    let Some(target) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.target_id)
    else {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByTargetLookup,
            Some(RulebenchRejection::InvalidTarget),
            current_actor_id,
            None,
            None,
            "Target is not present in the current scenario.",
        );
    };

    let target_legality = validate_target_legality(actor, target, action);
    if !target_legality.accepted {
        let rejection = target_legality_rejection(&target_legality);
        let reason = target_legality.reason.clone();
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByTargetLegality,
            Some(rejection),
            current_actor_id,
            Some(target_legality),
            None,
            reason,
        );
    }

    let action_resource = standard_action_resource_for(action_resources, &intent.actor_id);
    let Some(action_resource) = action_resource else {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByActionResource,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            Some(target_legality),
            None,
            "Actor has no standard action resource in the ledger.",
        );
    };

    if !action_resource.available {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByActionResource,
            Some(RulebenchRejection::InvalidAction),
            current_actor_id,
            Some(target_legality),
            Some(action_resource),
            "Actor has no available standard action resource.",
        );
    }

    CommandPreflightReadout {
        intent,
        accepted: true,
        decision_kind: CommandPreflightDecisionKind::Accepted,
        rejection: None,
        current_actor_id,
        target_legality: Some(target_legality),
        action_resource: Some(action_resource),
        reason: "Command is admissible before roll resolution.".to_string(),
    }
}

fn standard_action_resource_for(
    action_resources: &ActionResourceLedgerReadout,
    combatant_id: &str,
) -> Option<ActionResourceState> {
    action_resources
        .combatants
        .iter()
        .find(|combatant| combatant.combatant_id == combatant_id)
        .and_then(|combatant| {
            combatant
                .resources
                .iter()
                .find(|resource| resource.kind == ActionResourceKind::StandardAction)
                .cloned()
        })
}

fn rejected_command_preflight(
    intent: UseActionIntent,
    decision_kind: CommandPreflightDecisionKind,
    rejection: Option<RulebenchRejection>,
    current_actor_id: Option<String>,
    target_legality: Option<TargetLegality>,
    action_resource: Option<ActionResourceState>,
    reason: impl Into<String>,
) -> CommandPreflightReadout {
    CommandPreflightReadout {
        intent,
        accepted: false,
        decision_kind,
        rejection,
        current_actor_id,
        target_legality,
        action_resource,
        reason: reason.into(),
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

fn preflight_rejected_receipt(
    preflight: &CommandPreflightReadout,
    projection: ScenarioProjection,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent: preflight.intent.clone(),
        rejection: preflight.rejection,
        target_legality: preflight.target_legality.clone(),
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
                "Session command submitted through preflight-gated intent path.",
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Rejected,
                "Command rejected by preflight.",
                preflight.reason.clone(),
            ),
        ],
        projection: Some(projection),
    }
}

fn command_decision_kind_for_preflight(
    decision_kind: CommandPreflightDecisionKind,
) -> CommandDecisionKind {
    match decision_kind {
        CommandPreflightDecisionKind::RejectedByLifecycle => {
            CommandDecisionKind::RejectedByLifecycle
        }
        CommandPreflightDecisionKind::RejectedByTurnOrder => {
            CommandDecisionKind::RejectedByTurnOrder
        }
        _ => CommandDecisionKind::RejectedByPreflight,
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

fn derive_command_outcome_class(receipt: &RulebenchReceipt) -> CommandOutcomeClass {
    if receipt.accepted {
        if receipt.events.iter().any(domain_event_is_damage_applied) {
            CommandOutcomeClass::AcceptedHit
        } else {
            CommandOutcomeClass::AcceptedMiss
        }
    } else if receipt.rejection.is_some_and(is_target_legality_rejection) {
        CommandOutcomeClass::RejectedTargetLegality
    } else {
        CommandOutcomeClass::RejectedInvalidCommand
    }
}

fn domain_event_is_damage_applied(event: &DomainEvent) -> bool {
    matches!(event, DomainEvent::DamageApplied { .. })
}

fn is_target_legality_rejection(rejection: RulebenchRejection) -> bool {
    matches!(
        rejection,
        RulebenchRejection::TargetLegalityFailed
            | RulebenchRejection::TargetOutOfRange
            | RulebenchRejection::TargetNotVisible
            | RulebenchRejection::InvalidTarget
    )
}

fn command_audit_entry(
    step: &CombatSessionStepSummary,
    receipt: &RulebenchReceipt,
    decision_kind: CommandDecisionKind,
    preflight_decision_kind: Option<CommandPreflightDecisionKind>,
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
) -> CommandAuditEntry {
    CommandAuditEntry {
        id: format!("audit-{}", step.id),
        step_id: step.id.clone(),
        sequence: step.index,
        outcome_class: step.outcome_class,
        decision_kind,
        preflight_decision_kind,
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

fn rejected_turn_advance_readout(
    decision_kind: TurnAdvanceDecisionKind,
    previous_turn_order: CombatTurnOrder,
    next_turn_order: CombatTurnOrder,
    state_fingerprint: StateFingerprint,
    reason: impl Into<String>,
) -> TurnAdvanceReadout {
    TurnAdvanceReadout {
        accepted: false,
        decision_kind,
        previous_turn_order,
        next_turn_order,
        transition: None,
        state_before_fingerprint: state_fingerprint.clone(),
        state_after_fingerprint: state_fingerprint,
        reason: reason.into(),
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

fn combat_end_condition_readout(scenario: &RulebenchScenario) -> CombatEndConditionReadout {
    let mut active_ally_count = 0;
    let mut active_enemy_count = 0;
    let mut defeated_ally_count = 0;
    let mut defeated_enemy_count = 0;

    for combatant in &scenario.combatants {
        let defeated = combatant.hit_points.current <= 0;
        match (combatant.team, defeated) {
            (Team::Ally, false) => active_ally_count += 1,
            (Team::Ally, true) => defeated_ally_count += 1,
            (Team::Enemy, false) => active_enemy_count += 1,
            (Team::Enemy, true) => defeated_enemy_count += 1,
        }
    }

    let condition_kind = if active_ally_count == 0 && active_enemy_count == 0 {
        CombatEndConditionKind::NoActiveCombatants
    } else if active_enemy_count == 0 {
        CombatEndConditionKind::NoActiveEnemies
    } else if active_ally_count == 0 {
        CombatEndConditionKind::NoActiveAllies
    } else {
        CombatEndConditionKind::Ongoing
    };

    CombatEndConditionReadout {
        combat_should_end: condition_kind != CombatEndConditionKind::Ongoing,
        condition_kind,
        active_ally_count,
        active_enemy_count,
        defeated_ally_count,
        defeated_enemy_count,
        reason: combat_end_condition_reason(condition_kind),
    }
}

fn combat_end_condition_reason(kind: CombatEndConditionKind) -> String {
    match kind {
        CombatEndConditionKind::Ongoing => {
            "Combat can continue because both sides have active combatants."
        }
        CombatEndConditionKind::NoActiveEnemies => {
            "Combat should end because no active enemies remain."
        }
        CombatEndConditionKind::NoActiveAllies => {
            "Combat should end because no active allies remain."
        }
        CombatEndConditionKind::NoActiveCombatants => {
            "Combat should end because no active combatants remain."
        }
    }
    .to_string()
}

fn current_actor_option_summary(
    lifecycle: &CombatLifecycle,
    turn_order: &CombatTurnOrder,
    scenario: &RulebenchScenario,
    projection: &ScenarioProjection,
) -> CurrentActorOptionSummary {
    let current_actor_id = turn_order.current_actor_id.clone();
    let current_actor_defeated = current_actor_id
        .as_deref()
        .and_then(|actor_id| projected_combatant_by_id(projection, actor_id))
        .is_some_and(|actor| actor.hit_points.current <= 0);

    if lifecycle.phase == CombatLifecyclePhase::Ended {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::CombatEnded,
            Vec::new(),
        );
    }

    let Some(actor_id) = current_actor_id.as_deref() else {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::NoCurrentActor,
            Vec::new(),
        );
    };

    if current_actor_defeated {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::CurrentActorDefeated,
            Vec::new(),
        );
    }

    let actions = scenario
        .actions
        .iter()
        .filter(|action| action.actor_id == actor_id)
        .map(|action| current_actor_action_option(action, projection))
        .collect::<Vec<_>>();

    if actions.is_empty() {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::NoMatchingActions,
            actions,
        );
    }

    let available = actions
        .iter()
        .any(|action| !action.target_options.is_empty());
    let unavailable_reason = if available {
        None
    } else {
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets)
    };

    CurrentActorOptionSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        lifecycle_phase: lifecycle.phase,
        current_actor_id,
        current_actor_defeated,
        available,
        unavailable_reason,
        actions,
    }
}

fn current_actor_command_candidates(
    lifecycle: &CombatLifecycle,
    scenario: &RulebenchScenario,
    action_resources: &ActionResourceLedgerReadout,
    options: CurrentActorOptionSummary,
) -> CommandCandidateSummary {
    let candidates = if options.available {
        current_actor_id_command_candidates(lifecycle, scenario, action_resources, &options)
    } else {
        Vec::new()
    };

    CommandCandidateSummary {
        round_number: options.round_number,
        turn_index: options.turn_index,
        lifecycle_phase: options.lifecycle_phase,
        current_actor_id: options.current_actor_id,
        current_actor_defeated: options.current_actor_defeated,
        available: !candidates.is_empty(),
        unavailable_reason: options.unavailable_reason,
        candidates,
    }
}

fn plan_candidate_command(
    spec: CombatSessionCandidateSelectionSpec,
    candidates: CommandCandidateSummary,
) -> CombatSessionCandidateSelectionReadout {
    if !candidates.available {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind:
                CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: candidates.unavailable_reason,
            preflight_decision_kind: None,
            rejection: None,
            reason: candidate_selection_unavailable_reason(candidates.unavailable_reason),
            command: None,
        };
    }

    let Some(candidate) = candidates.candidates.iter().find(|candidate| {
        candidate.action_id == spec.action_id && candidate.target_id == spec.target_id
    }) else {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind: CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: None,
            preflight_decision_kind: None,
            rejection: None,
            reason: "Selected command candidate is not available for the current actor."
                .to_string(),
            command: None,
        };
    };

    if !candidate.accepted {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind: CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: None,
            preflight_decision_kind: Some(candidate.decision_kind),
            rejection: candidate.rejection,
            reason: candidate.reason.clone(),
            command: None,
        };
    }

    let command = CombatSessionIntentCommandSpec::new(
        spec.id,
        spec.title,
        spec.summary,
        candidate.intent.clone(),
        spec.roll_stream,
    );

    CombatSessionCandidateSelectionReadout {
        action_id: spec.action_id,
        target_id: spec.target_id,
        accepted: true,
        decision_kind: CombatSessionCandidateSelectionDecisionKind::Accepted,
        current_actor_id: candidates.current_actor_id,
        unavailable_reason: None,
        preflight_decision_kind: Some(candidate.decision_kind),
        rejection: None,
        reason: "Selected command candidate planned for deterministic submission.".to_string(),
        command: Some(command),
    }
}

fn candidate_selection_unavailable_reason(
    reason: Option<CurrentActorOptionsUnavailableReason>,
) -> String {
    match reason {
        Some(CurrentActorOptionsUnavailableReason::CombatEnded) => {
            "No command candidates are available because combat is ended."
        }
        Some(CurrentActorOptionsUnavailableReason::NoCurrentActor) => {
            "No command candidates are available because there is no current actor."
        }
        Some(CurrentActorOptionsUnavailableReason::CurrentActorDefeated) => {
            "No command candidates are available because the current actor is defeated."
        }
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions) => {
            "No command candidates are available because the current actor has no matching actions."
        }
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets) => {
            "No command candidates are available because there are no visible active targets."
        }
        None => "No command candidates are available.",
    }
    .to_string()
}

fn current_actor_id_command_candidates(
    lifecycle: &CombatLifecycle,
    scenario: &RulebenchScenario,
    action_resources: &ActionResourceLedgerReadout,
    options: &CurrentActorOptionSummary,
) -> Vec<CommandCandidateEntry> {
    let Some(actor_id) = options.current_actor_id.as_deref() else {
        return Vec::new();
    };

    options
        .actions
        .iter()
        .flat_map(|action| {
            action.target_options.iter().map(move |target| {
                let intent = UseActionIntent::new(
                    actor_id,
                    action.action_id.clone(),
                    target.target_id.clone(),
                );
                let preflight = command_preflight_readout(
                    lifecycle,
                    options.current_actor_id.clone(),
                    scenario,
                    action_resources,
                    intent.clone(),
                );

                CommandCandidateEntry {
                    intent,
                    action_id: action.action_id.clone(),
                    ability_id: action.ability_id.clone(),
                    target_id: target.target_id.clone(),
                    target_name: target.target_name.clone(),
                    target_current_hit_points: target.current_hit_points,
                    target_max_hit_points: target.max_hit_points,
                    accepted: preflight.accepted,
                    decision_kind: preflight.decision_kind,
                    rejection: preflight.rejection,
                    target_legality: preflight.target_legality,
                    reason: preflight.reason,
                }
            })
        })
        .collect()
}

fn unavailable_current_actor_options(
    lifecycle: &CombatLifecycle,
    turn_order: &CombatTurnOrder,
    current_actor_id: Option<String>,
    current_actor_defeated: bool,
    reason: CurrentActorOptionsUnavailableReason,
    actions: Vec<CurrentActorActionOption>,
) -> CurrentActorOptionSummary {
    CurrentActorOptionSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        lifecycle_phase: lifecycle.phase,
        current_actor_id,
        current_actor_defeated,
        available: false,
        unavailable_reason: Some(reason),
        actions,
    }
}

fn current_actor_action_option(
    action: &ActionDefinition,
    projection: &ScenarioProjection,
) -> CurrentActorActionOption {
    let target_options = action
        .visible_target_ids
        .iter()
        .filter_map(|target_id| projected_combatant_by_id(projection, target_id))
        .filter(|target| target.hit_points.current > 0)
        .map(|target| CurrentActorTargetOption {
            target_id: target.id.clone(),
            target_name: target.name.clone(),
            current_hit_points: target.hit_points.current,
            max_hit_points: target.hit_points.max,
        })
        .collect();

    CurrentActorActionOption {
        action_id: action.id.clone(),
        ability_id: action.ability_id.clone(),
        action_name: action.name.clone(),
        target_options,
    }
}

fn projected_combatant_by_id<'a>(
    projection: &'a ScenarioProjection,
    combatant_id: &str,
) -> Option<&'a FinalCombatantState> {
    projection
        .combatants
        .iter()
        .find(|combatant| combatant.id == combatant_id)
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
