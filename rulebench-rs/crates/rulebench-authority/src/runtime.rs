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
pub struct CombatSessionState {
    session_id: String,
    scenario: RulebenchScenario,
    state: CombatState,
    combat_log: Vec<CombatLogEntry>,
    audit_log: Vec<CommandAuditEntry>,
    action_usage_log: Vec<ActionUsageEntry>,
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

    pub fn lifecycle_transition_log(&self) -> &[LifecycleTransitionEntry] {
        &self.lifecycle_transition_log
    }

    pub fn current_turn_action_usage(&self) -> ActionUsageSummary {
        current_turn_action_usage(&self.turn_order, &self.action_usage_log)
    }

    pub fn combatant_vitality(&self) -> CombatantVitalitySummary {
        let current_state = self.state.project("Current session state.");
        combatant_vitality_summary(&current_state)
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

        current_actor_command_candidates(&self.lifecycle, &current_scenario, current_actor_options)
    }

    pub fn preflight_command(&self, intent: UseActionIntent) -> CommandPreflightReadout {
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        command_preflight_readout(
            &self.lifecycle,
            self.turn_order.current_actor_id.clone(),
            &current_scenario,
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
            lifecycle_transition_log: self.lifecycle_transition_log.clone(),
            turn_order: self.turn_order.clone(),
            combat_log: self.combat_log.clone(),
            audit_log: self.audit_log.clone(),
            action_usage_log: self.action_usage_log.clone(),
            turn_transition_log: self.turn_transition_log.clone(),
            current_turn_action_usage: self.current_turn_action_usage(),
            combatant_vitality: combatant_vitality_summary(&current_state),
            current_actor_options: current_actor_option_summary(
                &self.lifecycle,
                &self.turn_order,
                &self.scenario,
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
    intent: UseActionIntent,
) -> CommandPreflightReadout {
    if intent.actor_id.is_empty() {
        return rejected_command_preflight(
            intent,
            CommandPreflightDecisionKind::RejectedByShape,
            Some(RulebenchRejection::EmptyActorId),
            current_actor_id,
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
            reason,
        );
    }

    CommandPreflightReadout {
        intent,
        accepted: true,
        decision_kind: CommandPreflightDecisionKind::Accepted,
        rejection: None,
        current_actor_id,
        target_legality: Some(target_legality),
        reason: "Command is admissible before roll resolution.".to_string(),
    }
}

fn rejected_command_preflight(
    intent: UseActionIntent,
    decision_kind: CommandPreflightDecisionKind,
    rejection: Option<RulebenchRejection>,
    current_actor_id: Option<String>,
    target_legality: Option<TargetLegality>,
    reason: impl Into<String>,
) -> CommandPreflightReadout {
    CommandPreflightReadout {
        intent,
        accepted: false,
        decision_kind,
        rejection,
        current_actor_id,
        target_legality,
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
    options: CurrentActorOptionSummary,
) -> CommandCandidateSummary {
    let candidates = if options.available {
        current_actor_id_command_candidates(lifecycle, scenario, &options)
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

fn current_actor_id_command_candidates(
    lifecycle: &CombatLifecycle,
    scenario: &RulebenchScenario,
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
