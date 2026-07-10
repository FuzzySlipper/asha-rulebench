//! Immutable combat finalization and terminal-condition handling.

use super::*;

impl CombatSessionState {
    pub fn finalization(&self) -> Option<&CombatFinalizationReadout> {
        self.finalization.as_ref()
    }

    pub(super) fn finalize_if_condition_met(&mut self) -> Option<CombatFinalizationReadout> {
        if self.finalization.is_some() || self.current_reaction_window().is_some() {
            return self.finalization.clone();
        }

        let end_condition = self.combat_end_condition();
        if !end_condition.combat_should_end {
            return None;
        }

        Some(self.finalize_combat(
            LifecycleTransitionTrigger::ConditionalEnd,
            end_condition,
            false,
        ))
    }

    pub(super) fn finalize_explicitly(&mut self) -> CombatFinalizationReadout {
        let end_condition = self.combat_end_condition();
        self.finalize_combat(LifecycleTransitionTrigger::ExplicitEnd, end_condition, true)
    }

    pub(super) fn finalize_conditionally(
        &mut self,
        end_condition: CombatEndConditionReadout,
    ) -> CombatFinalizationReadout {
        self.finalize_combat(
            LifecycleTransitionTrigger::ConditionalEnd,
            end_condition,
            false,
        )
    }

    fn finalize_combat(
        &mut self,
        trigger: LifecycleTransitionTrigger,
        mut end_condition: CombatEndConditionReadout,
        explicit: bool,
    ) -> CombatFinalizationReadout {
        if let Some(finalization) = &self.finalization {
            return finalization.clone();
        }

        let previous_lifecycle = self.lifecycle.clone();
        self.lifecycle.end_at_step(self.next_step_index);
        self.record_lifecycle_transition(trigger, self.next_step_index, previous_lifecycle);
        let expirations = self.state.expire_all_modifiers_for_event("combatEnd");
        self.record_modifier_event_expiration_transitions("combatEnd", &expirations);

        if explicit {
            end_condition.combat_should_end = true;
            end_condition.condition_kind = CombatEndConditionKind::ExplicitEnd;
            end_condition.outcome_kind = CombatOutcomeKind::ExplicitEnd;
            end_condition.winning_sides.clear();
            end_condition.reason = "Combat ended by explicit authority command.".to_string();
        }

        let finalization = CombatFinalizationReadout {
            trigger,
            finalized_at_step: self.next_step_index,
            outcome_kind: end_condition.outcome_kind,
            winning_sides: end_condition.winning_sides.clone(),
            remaining_sides: end_condition.active_sides.clone(),
            final_state_fingerprint: fingerprint_projected_state(
                &self.state.project("Finalized combat state."),
            ),
            combat_log_entry_count: self.combat_log.len() as u32,
            command_audit_entry_count: self.audit_log.len() as u32,
            reason: end_condition.reason.clone(),
            end_condition,
        };
        self.finalization = Some(finalization.clone());
        finalization
    }
}
