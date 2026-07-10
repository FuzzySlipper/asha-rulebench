//! Manual lifecycle and turn-control handling.

use super::*;

impl CombatSessionState {
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

        let active_combatant_ids = self.state.active_combatant_ids();
        if !self
            .turn_order
            .advance_to_next_active(&active_combatant_ids)
        {
            return rejected_turn_advance_readout(
                TurnAdvanceDecisionKind::RejectedByNoActiveParticipants,
                previous_turn_order,
                self.turn_order.clone(),
                state_before_fingerprint,
                "Turn order has no active participants.",
            );
        }
        let transition = turn_transition_entry(
            self.turn_transition_log.len() as u32,
            &previous_turn_order,
            &self.turn_order,
        );
        self.turn_transition_log.push(transition.clone());
        if let Some(current_actor_id) = self.turn_order.current_actor_id.clone() {
            let refreshes = self
                .state
                .advance_action_resources_for_turn_start(&current_actor_id);
            for refresh in &refreshes {
                self.record_action_resource_refresh_transition(&transition, refresh);
            }
        }
        if let Some(previous_actor_id) = transition.previous_actor_id.as_deref() {
            let expirations = self
                .state
                .advance_turn_counted_modifiers_for(previous_actor_id);
            self.record_modifier_duration_expiration_transitions(&transition, &expirations);
        }
        if transition.wrapped_round {
            let expirations = self.state.advance_all_round_counted_modifiers();
            self.record_modifier_round_duration_transitions(&transition, &expirations);
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

    pub(super) fn start_lifecycle(&mut self, trigger: LifecycleTransitionTrigger) {
        let previous_lifecycle = self.lifecycle.clone();
        let should_refresh_combat_start_resources =
            previous_lifecycle.phase == CombatLifecyclePhase::Ready;
        self.lifecycle.start_at_step(self.next_step_index);
        self.record_lifecycle_transition(trigger, self.next_step_index, previous_lifecycle);
        if should_refresh_combat_start_resources {
            self.state.refresh_action_resources_for_combat_start();
        }
    }

    fn end_lifecycle(&mut self, trigger: LifecycleTransitionTrigger) {
        let previous_lifecycle = self.lifecycle.clone();
        self.lifecycle.end_at_step(self.next_step_index);
        self.record_lifecycle_transition(trigger, self.next_step_index, previous_lifecycle);
        let expirations = self.state.expire_all_modifiers_for_event("combatEnd");
        self.record_modifier_event_expiration_transitions("combatEnd", &expirations);
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
        TurnAdvanceDecisionKind::RejectedByNoActiveParticipants => {
            CombatControlDecisionKind::RejectedByEmptyTurnOrder
        }
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
