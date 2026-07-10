//! Authoritative session lifecycle, command intake, and control runtime.

use crate::model::*;
use crate::resolver::{resolve_use_action, target_legality_rejection, validate_target_legality};
use crate::state::CombatState;
use crate::{fingerprint_projected_state, fingerprint_projection};

mod automation;
mod control;
mod script;
mod status;

use automation::{
    combat_session_automatic_run_readout, plan_auto_candidate_command, plan_automatic_step,
    plan_candidate_command,
};
pub use automation::{
    CombatSessionAutoCandidateCommandSpec, CombatSessionAutoCandidateDecisionKind,
    CombatSessionAutoCandidateExecutionReadout, CombatSessionAutoCandidatePlanReadout,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepOperationKind,
    CombatSessionAutomaticStepPlanReadout, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
};
pub use script::{
    CombatSessionScriptCommandKind, CombatSessionScriptCommandSpec,
    CombatSessionScriptDecisionKind, CombatSessionScriptReadout, CombatSessionScriptSpec,
    CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
};

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
        let turn_order = CombatTurnOrder::from_combatants(&scenario.combatants);
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

    pub fn plan_auto_candidate_command(
        &self,
        spec: CombatSessionAutoCandidateCommandSpec,
    ) -> CombatSessionAutoCandidatePlanReadout {
        let candidates = self.current_actor_command_candidates();
        plan_auto_candidate_command(spec, candidates)
    }

    pub fn submit_auto_candidate_command(
        &mut self,
        spec: CombatSessionAutoCandidateCommandSpec,
    ) -> CombatSessionAutoCandidateExecutionReadout {
        let plan = self.plan_auto_candidate_command(spec);
        let submitted_step = plan
            .selection
            .as_ref()
            .and_then(|selection| selection.command.clone())
            .map(|command| self.submit_intent_command(command));

        CombatSessionAutoCandidateExecutionReadout {
            plan,
            submitted_step,
        }
    }

    pub fn plan_automatic_step(
        &self,
        spec: CombatSessionAutomaticStepSpec,
    ) -> CombatSessionAutomaticStepPlanReadout {
        let end_condition = self.combat_end_condition();
        plan_automatic_step(
            self.lifecycle.phase,
            self.turn_order.current_actor_id.clone(),
            end_condition,
            || {
                self.plan_auto_candidate_command(CombatSessionAutoCandidateCommandSpec::new(
                    spec.id,
                    spec.title,
                    spec.summary,
                    spec.roll_stream,
                ))
            },
        )
    }

    pub fn submit_automatic_step(
        &mut self,
        spec: CombatSessionAutomaticStepSpec,
    ) -> CombatSessionAutomaticStepExecutionReadout {
        let plan = self.plan_automatic_step(spec.clone());
        let (control, auto_candidate) = match plan.operation_kind {
            Some(CombatSessionAutomaticStepOperationKind::ConditionalEnd) => (
                Some(self.submit_control_command(CombatControlCommandSpec::end_if_condition_met())),
                None,
            ),
            Some(CombatSessionAutomaticStepOperationKind::SubmitCandidate) => (
                None,
                Some(self.submit_auto_candidate_command(
                    CombatSessionAutoCandidateCommandSpec::new(
                        spec.id,
                        spec.title,
                        spec.summary,
                        spec.roll_stream,
                    ),
                )),
            ),
            Some(CombatSessionAutomaticStepOperationKind::AdvanceTurn) => (
                Some(self.submit_control_command(CombatControlCommandSpec::advance_turn())),
                None,
            ),
            None => (None, None),
        };

        CombatSessionAutomaticStepExecutionReadout {
            plan,
            control,
            auto_candidate,
        }
    }

    pub fn run_automatic_combat(
        &mut self,
        spec: CombatSessionAutomaticRunSpec,
    ) -> CombatSessionAutomaticRunReadout {
        if self.lifecycle.phase == CombatLifecyclePhase::Ended {
            return combat_session_automatic_run_readout(
                spec.id,
                spec.title,
                spec.summary,
                false,
                CombatSessionAutomaticRunDecisionKind::RejectedByLifecycle,
                spec.max_steps,
                Vec::new(),
                self.snapshot(),
                "Automatic combat run rejected because combat is already ended.",
            );
        }

        if spec.max_steps == 0 {
            return combat_session_automatic_run_readout(
                spec.id,
                spec.title,
                spec.summary,
                false,
                CombatSessionAutomaticRunDecisionKind::RejectedByStepLimit,
                spec.max_steps,
                Vec::new(),
                self.snapshot(),
                "Automatic combat run rejected because max steps is zero.",
            );
        }

        let mut steps = Vec::new();
        for step_index in 0..spec.max_steps {
            if self.lifecycle.phase == CombatLifecyclePhase::Ended {
                break;
            }

            steps.push(
                self.submit_automatic_step(CombatSessionAutomaticStepSpec::new(
                    format!("{}-step-{step_index}", spec.id),
                    format!("{} step {}", spec.title, step_index + 1),
                    spec.summary.clone(),
                    spec.roll_stream.clone(),
                )),
            );
        }

        let final_snapshot = self.snapshot();
        let combat_ended = final_snapshot.lifecycle.phase == CombatLifecyclePhase::Ended;
        let (accepted, decision_kind, reason) = if combat_ended {
            (
                true,
                CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded,
                "Automatic combat run completed because combat reached ended lifecycle.",
            )
        } else {
            (
                false,
                CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps,
                "Automatic combat run stopped at the max-step guard before combat ended.",
            )
        };

        combat_session_automatic_run_readout(
            spec.id,
            spec.title,
            spec.summary,
            accepted,
            decision_kind,
            spec.max_steps,
            steps,
            final_snapshot,
            reason,
        )
    }

    pub fn next_step_index(&self) -> u32 {
        self.next_step_index
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
                    trigger: ModifierDurationTransitionTrigger::TurnBoundary,
                    turn_transition_sequence: Some(transition.sequence),
                    round_number: Some(transition.next_round_number),
                    turn_index: Some(transition.next_turn_index),
                    current_actor_id: transition.next_actor_id.clone(),
                    reason: expiration.reason.clone(),
                });
        }
    }

    fn record_modifier_event_expiration_transitions(
        &mut self,
        event: &str,
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
                    trigger: ModifierDurationTransitionTrigger::Event(event.to_string()),
                    turn_transition_sequence: None,
                    round_number: None,
                    turn_index: None,
                    current_actor_id: self.turn_order.current_actor_id.clone(),
                    reason: expiration.reason.clone(),
                });
        }
    }

    fn record_modifier_round_duration_transitions(
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
                    trigger: ModifierDurationTransitionTrigger::RoundBoundary,
                    turn_transition_sequence: Some(transition.sequence),
                    round_number: Some(transition.next_round_number),
                    turn_index: Some(transition.next_turn_index),
                    current_actor_id: transition.next_actor_id.clone(),
                    reason: expiration.reason.clone(),
                });
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
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption: Vec::new(),
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
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption: Vec::new(),
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
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption: Vec::new(),
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
        roll_consumption: receipt.roll_consumption.clone(),
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
        DomainEvent::SavingThrowResolved { .. } => "SavingThrowResolved",
        DomainEvent::ContestedCheckResolved { .. } => "ContestedCheckResolved",
        DomainEvent::DamageApplied { .. } => "DamageApplied",
        DomainEvent::HealingApplied { .. } => "HealingApplied",
        DomainEvent::TemporaryVitalityGranted { .. } => "TemporaryVitalityGranted",
        DomainEvent::ModifierApplied { .. } => "ModifierApplied",
    }
    .to_string()
}
