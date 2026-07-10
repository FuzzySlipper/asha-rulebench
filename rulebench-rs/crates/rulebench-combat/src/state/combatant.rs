//! Internal mutable combatant state.

use crate::model::{
    ActionResourceKind, ActionResourceRefreshDecisionKind, ActionResourceRefreshReadout,
    ActionResourceSpendDecisionKind, ActionResourceSpendReadout, ActionResourceState,
    ActiveModifier, BoundedValue, Combatant, CombatantActionResourceReadout, FinalCombatantState,
    ModifierDurationExpirationDecisionKind, ModifierDurationExpirationReadout, ModifierOutcome,
    ModifierTenure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CombatantState {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) hit_points: BoundedValue,
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
            conditions: self.condition_labels(),
        }
    }

    pub(super) fn from_final_state(combatant: &FinalCombatantState) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            active_modifiers: Vec::new(),
            conditions: combatant.conditions.clone(),
            action_resources: default_action_resources(),
        }
    }

    pub(super) fn apply_modifier(&mut self, modifier: &ModifierOutcome) {
        if self.active_modifiers.iter().any(|active| {
            active.modifier_id == modifier.modifier_id || active.label == modifier.label
        }) {
            return;
        }

        self.active_modifiers.push(ActiveModifier::temporary(
            modifier.modifier_id.clone(),
            modifier.label.clone(),
            modifier.duration.clone(),
        ));
    }

    pub(super) fn apply_projection(&mut self, combatant: &FinalCombatantState) {
        self.name = combatant.name.clone();
        self.hit_points = combatant.hit_points;
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

    pub(super) fn expire_temporary_modifiers(&mut self) -> Vec<ModifierDurationExpirationReadout> {
        let mut retained_modifiers = Vec::with_capacity(self.active_modifiers.len());
        let mut expiration_readouts = Vec::new();

        for modifier in self.active_modifiers.drain(..) {
            if modifier.tenure == ModifierTenure::Temporary {
                expiration_readouts.push(ModifierDurationExpirationReadout {
                    combatant_id: self.id.clone(),
                    modifier_id: modifier.modifier_id.clone(),
                    accepted: true,
                    decision_kind: ModifierDurationExpirationDecisionKind::Expired,
                    previous_modifier: modifier,
                    next_modifier: None,
                    reason: "Temporary modifier expired at turn boundary.".to_string(),
                });
            } else {
                retained_modifiers.push(modifier);
            }
        }

        self.active_modifiers = retained_modifiers;
        expiration_readouts
    }
}

fn default_action_resources() -> Vec<ActionResourceState> {
    vec![ActionResourceState::standard_action_available()]
}
