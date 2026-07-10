//! Internal mutable combatant state.

use crate::model::{
    ActionResourceKind, ActionResourceRefreshDecisionKind, ActionResourceRefreshReadout,
    ActionResourceSpendDecisionKind, ActionResourceSpendReadout, ActionResourceState,
    ActiveModifier, BoundedValue, Combatant, CombatantActionResourceReadout, FinalCombatantState,
    ModifierDurationExpirationDecisionKind, ModifierDurationExpirationReadout,
    ModifierDurationPolicy, ModifierOutcome, ModifierStackingPolicy, ModifierTenure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CombatantState {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) hit_points: BoundedValue,
    pub(super) temporary_vitality: i32,
    pub(super) active_modifiers: Vec<ActiveModifier>,
    pub(super) conditions: Vec<String>,
    pub(super) action_resources: Vec<ActionResourceState>,
}

impl CombatantState {
    pub(super) fn from_combatant(combatant: &Combatant) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            temporary_vitality: combatant.temporary_vitality,
            active_modifiers: combatant.active_modifiers.clone(),
            conditions: combatant.conditions.clone(),
            action_resources: default_action_resources(),
        }
    }

    pub(super) fn to_final_state(&self) -> FinalCombatantState {
        FinalCombatantState {
            id: self.id.clone(),
            name: self.name.clone(),
            hit_points: self.hit_points,
            temporary_vitality: self.temporary_vitality,
            conditions: self.condition_labels(),
        }
    }

    pub(super) fn from_final_state(combatant: &FinalCombatantState) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            temporary_vitality: combatant.temporary_vitality,
            active_modifiers: Vec::new(),
            conditions: combatant.conditions.clone(),
            action_resources: default_action_resources(),
        }
    }

    pub(super) fn apply_modifier(&mut self, modifier: &ModifierOutcome) {
        let active_modifier = ActiveModifier {
            modifier_id: modifier.modifier_id.clone(),
            source_id: modifier.source_id.clone(),
            label: modifier.label.clone(),
            duration: modifier.duration.clone(),
            tenure: match modifier.duration_policy {
                ModifierDurationPolicy::Permanent => ModifierTenure::Permanent,
                ModifierDurationPolicy::Turns(_)
                | ModifierDurationPolicy::Rounds(_)
                | ModifierDurationPolicy::UntilEvent(_) => ModifierTenure::Temporary,
            },
            stacking_group: modifier.stacking_group.clone(),
            stacking_policy: modifier.stacking_policy,
            duration_policy: modifier.duration_policy.clone(),
            remaining_turns: modifier.remaining_turns,
            remaining_rounds: modifier.remaining_rounds,
        };
        match modifier.stacking_policy {
            ModifierStackingPolicy::Stack => self.active_modifiers.push(active_modifier),
            ModifierStackingPolicy::Replace => {
                self.active_modifiers
                    .retain(|active| active.stacking_group != modifier.stacking_group);
                self.active_modifiers.push(active_modifier);
            }
            ModifierStackingPolicy::Refresh => {
                if let Some(existing) = self
                    .active_modifiers
                    .iter_mut()
                    .find(|active| active.stacking_group == modifier.stacking_group)
                {
                    *existing = active_modifier;
                } else {
                    self.active_modifiers.push(active_modifier);
                }
            }
        }
    }

    pub(super) fn apply_projection(&mut self, combatant: &FinalCombatantState) {
        self.name = combatant.name.clone();
        self.hit_points = combatant.hit_points;
        self.temporary_vitality = combatant.temporary_vitality;
        self.active_modifiers = Vec::new();
        self.conditions = combatant.conditions.clone();
    }

    pub(super) fn condition_labels(&self) -> Vec<String> {
        let mut labels = self.conditions.clone();
        for modifier in &self.active_modifiers {
            if !labels.contains(&modifier.label) {
                labels.push(modifier.label.clone());
            }
        }
        labels
    }

    pub(super) fn action_resource_readout(&self) -> CombatantActionResourceReadout {
        CombatantActionResourceReadout {
            combatant_id: self.id.clone(),
            resources: self.action_resources.clone(),
        }
    }

    pub(super) fn spend_action_resource(
        &mut self,
        resource_kind: ActionResourceKind,
    ) -> ActionResourceSpendReadout {
        let Some(resource) = self
            .action_resources
            .iter_mut()
            .find(|resource| resource.kind == resource_kind)
        else {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_kind,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByMissingResource,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant does not have the requested action resource.".to_string(),
            };
        };

        let previous_resource = resource.clone();
        if !resource.available {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_kind,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByUnavailableResource,
                previous_resource: Some(previous_resource.clone()),
                next_resource: Some(previous_resource),
                reason: "Action resource is not available.".to_string(),
            };
        }

        resource.current -= 1;
        resource.available = resource.current > 0;
        let next_resource = resource.clone();

        ActionResourceSpendReadout {
            combatant_id: self.id.clone(),
            resource_kind,
            accepted: true,
            decision_kind: ActionResourceSpendDecisionKind::Spent,
            previous_resource: Some(previous_resource),
            next_resource: Some(next_resource),
            reason: "Action resource spent.".to_string(),
        }
    }

    pub(super) fn refresh_action_resource(
        &mut self,
        resource_kind: ActionResourceKind,
    ) -> ActionResourceRefreshReadout {
        let Some(resource) = self
            .action_resources
            .iter_mut()
            .find(|resource| resource.kind == resource_kind)
        else {
            return ActionResourceRefreshReadout {
                combatant_id: self.id.clone(),
                resource_kind,
                accepted: false,
                decision_kind: ActionResourceRefreshDecisionKind::RejectedByMissingResource,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant does not have the requested action resource.".to_string(),
            };
        };

        let previous_resource = resource.clone();
        resource.current = resource.max;
        resource.available = resource.current > 0;
        let next_resource = resource.clone();

        ActionResourceRefreshReadout {
            combatant_id: self.id.clone(),
            resource_kind,
            accepted: true,
            decision_kind: ActionResourceRefreshDecisionKind::Refreshed,
            previous_resource: Some(previous_resource),
            next_resource: Some(next_resource),
            reason: "Action resource refreshed.".to_string(),
        }
    }

    pub(super) fn advance_turn_counted_modifiers(
        &mut self,
    ) -> Vec<ModifierDurationExpirationReadout> {
        self.advance_counted_modifiers(ModifierDurationBoundary::Turn)
    }

    pub(super) fn advance_round_counted_modifiers(
        &mut self,
    ) -> Vec<ModifierDurationExpirationReadout> {
        self.advance_counted_modifiers(ModifierDurationBoundary::Round)
    }

    fn advance_counted_modifiers(
        &mut self,
        boundary: ModifierDurationBoundary,
    ) -> Vec<ModifierDurationExpirationReadout> {
        let mut retained_modifiers = Vec::with_capacity(self.active_modifiers.len());
        let mut expiration_readouts = Vec::new();

        for mut modifier in self.active_modifiers.drain(..) {
            let remaining_count = match boundary {
                ModifierDurationBoundary::Turn => match modifier.duration_policy {
                    ModifierDurationPolicy::Turns(_) => modifier.remaining_turns,
                    _ => {
                        retained_modifiers.push(modifier);
                        continue;
                    }
                },
                ModifierDurationBoundary::Round => match modifier.duration_policy {
                    ModifierDurationPolicy::Rounds(_) => modifier.remaining_rounds,
                    _ => {
                        retained_modifiers.push(modifier);
                        continue;
                    }
                },
            };
            let boundary_label = boundary.label();
            let boundary_display_label = boundary.display_label();

            if remaining_count.is_some_and(|count| count <= 1) {
                expiration_readouts.push(ModifierDurationExpirationReadout {
                    combatant_id: self.id.clone(),
                    modifier_id: modifier.modifier_id.clone(),
                    accepted: true,
                    decision_kind: ModifierDurationExpirationDecisionKind::Expired,
                    previous_modifier: modifier,
                    next_modifier: None,
                    reason: format!(
                        "{}-counted modifier expired at {} boundary.",
                        boundary_display_label, boundary_label
                    ),
                });
            } else {
                if let Some(count) = remaining_count {
                    let previous_modifier = modifier.clone();
                    match boundary {
                        ModifierDurationBoundary::Turn => {
                            modifier.remaining_turns = Some(count - 1)
                        }
                        ModifierDurationBoundary::Round => {
                            modifier.remaining_rounds = Some(count - 1)
                        }
                    }
                    expiration_readouts.push(ModifierDurationExpirationReadout {
                        combatant_id: self.id.clone(),
                        modifier_id: modifier.modifier_id.clone(),
                        accepted: true,
                        decision_kind: ModifierDurationExpirationDecisionKind::Advanced,
                        previous_modifier,
                        next_modifier: Some(modifier.clone()),
                        reason: format!(
                            "{}-counted modifier duration advanced at {} boundary.",
                            boundary_display_label, boundary_label
                        ),
                    });
                }
                retained_modifiers.push(modifier);
            }
        }

        self.active_modifiers = retained_modifiers;
        expiration_readouts
    }

    pub(super) fn expire_modifiers_for_event(
        &mut self,
        event: &str,
    ) -> Vec<ModifierDurationExpirationReadout> {
        let mut retained_modifiers = Vec::with_capacity(self.active_modifiers.len());
        let mut expiration_readouts = Vec::new();
        for modifier in self.active_modifiers.drain(..) {
            if matches!(&modifier.duration_policy, ModifierDurationPolicy::UntilEvent(trigger) if trigger == event)
            {
                expiration_readouts.push(ModifierDurationExpirationReadout {
                    combatant_id: self.id.clone(),
                    modifier_id: modifier.modifier_id.clone(),
                    accepted: true,
                    decision_kind: ModifierDurationExpirationDecisionKind::Expired,
                    previous_modifier: modifier,
                    next_modifier: None,
                    reason: format!("Modifier expired when event {} occurred.", event),
                });
            } else {
                retained_modifiers.push(modifier);
            }
        }
        self.active_modifiers = retained_modifiers;
        expiration_readouts
    }
}

#[derive(Debug, Clone, Copy)]
enum ModifierDurationBoundary {
    Turn,
    Round,
}

impl ModifierDurationBoundary {
    const fn label(self) -> &'static str {
        match self {
            ModifierDurationBoundary::Turn => "turn",
            ModifierDurationBoundary::Round => "round",
        }
    }

    const fn display_label(self) -> &'static str {
        match self {
            ModifierDurationBoundary::Turn => "Turn",
            ModifierDurationBoundary::Round => "Round",
        }
    }
}

fn default_action_resources() -> Vec<ActionResourceState> {
    vec![ActionResourceState::standard_action_available()]
}
