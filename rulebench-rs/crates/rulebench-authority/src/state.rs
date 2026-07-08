use crate::model::{
    ActionResourceKind, ActionResourceLedgerReadout, ActionResourceRefreshDecisionKind,
    ActionResourceRefreshReadout, ActionResourceSpendDecisionKind, ActionResourceSpendReadout,
    ActionResourceState, ActiveModifier, BoundedValue, Combatant, CombatantActionResourceReadout,
    DamageOutcome, FinalCombatantState, ModifierOutcome, RulebenchScenario, ScenarioProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatState {
    combatants: Vec<CombatantState>,
}

impl CombatState {
    pub(crate) fn from_scenario(scenario: &RulebenchScenario) -> Self {
        Self {
            combatants: scenario
                .combatants
                .iter()
                .map(CombatantState::from_combatant)
                .collect(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_projection(projection: &ScenarioProjection) -> Self {
        Self {
            combatants: projection
                .combatants
                .iter()
                .map(CombatantState::from_final_state)
                .collect(),
        }
    }

    pub(crate) fn project(&self, summary: &str) -> ScenarioProjection {
        ScenarioProjection {
            summary: summary.to_string(),
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::to_final_state)
                .collect(),
        }
    }

    pub(crate) fn apply_hit(&mut self, damage: &DamageOutcome, modifier: &ModifierOutcome) {
        for combatant in &mut self.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
            }
            if combatant.id == modifier.target_id {
                combatant.apply_modifier(modifier);
            }
        }
    }

    pub(crate) fn apply_projection(&mut self, projection: &ScenarioProjection) {
        for projected in &projection.combatants {
            if let Some(combatant) = self
                .combatants
                .iter_mut()
                .find(|combatant| combatant.id == projected.id)
            {
                combatant.apply_projection(projected);
            }
        }
    }

    pub(crate) fn action_resource_ledger(&self) -> ActionResourceLedgerReadout {
        ActionResourceLedgerReadout {
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::action_resource_readout)
                .collect(),
        }
    }

    #[cfg(test)]
    pub(crate) fn action_resources_for(
        &self,
        combatant_id: &str,
    ) -> Option<CombatantActionResourceReadout> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(CombatantState::action_resource_readout)
    }

    pub(crate) fn spend_action_resource(
        &mut self,
        combatant_id: &str,
        resource_kind: ActionResourceKind,
    ) -> ActionResourceSpendReadout {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return ActionResourceSpendReadout {
                combatant_id: combatant_id.to_string(),
                resource_kind,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByMissingCombatant,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant is not present in the action resource ledger.".to_string(),
            };
        };

        combatant.spend_action_resource(resource_kind)
    }

    pub(crate) fn refresh_action_resource(
        &mut self,
        combatant_id: &str,
        resource_kind: ActionResourceKind,
    ) -> ActionResourceRefreshReadout {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return ActionResourceRefreshReadout {
                combatant_id: combatant_id.to_string(),
                resource_kind,
                accepted: false,
                decision_kind: ActionResourceRefreshDecisionKind::RejectedByMissingCombatant,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant is not present in the action resource ledger.".to_string(),
            };
        };

        combatant.refresh_action_resource(resource_kind)
    }

    #[cfg(test)]
    pub(crate) fn active_modifiers_for(&self, combatant_id: &str) -> Option<&[ActiveModifier]> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(|combatant| combatant.active_modifiers.as_slice())
    }

    pub(crate) fn apply_to_scenario(&self, mut scenario: RulebenchScenario) -> RulebenchScenario {
        for combatant in &mut scenario.combatants {
            if let Some(state) = self
                .combatants
                .iter()
                .find(|state| state.id == combatant.id)
            {
                combatant.hit_points = state.hit_points;
                combatant.active_modifiers = state.active_modifiers.clone();
                combatant.conditions = state.condition_labels();
            }
        }
        scenario
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CombatantState {
    id: String,
    name: String,
    hit_points: BoundedValue,
    active_modifiers: Vec<ActiveModifier>,
    conditions: Vec<String>,
    action_resources: Vec<ActionResourceState>,
}

impl CombatantState {
    fn from_combatant(combatant: &Combatant) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            active_modifiers: combatant.active_modifiers.clone(),
            conditions: combatant.conditions.clone(),
            action_resources: default_action_resources(),
        }
    }

    fn to_final_state(&self) -> FinalCombatantState {
        FinalCombatantState {
            id: self.id.clone(),
            name: self.name.clone(),
            hit_points: self.hit_points,
            conditions: self.condition_labels(),
        }
    }

    #[cfg(test)]
    fn from_final_state(combatant: &FinalCombatantState) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            active_modifiers: Vec::new(),
            conditions: combatant.conditions.clone(),
            action_resources: default_action_resources(),
        }
    }

    fn apply_modifier(&mut self, modifier: &ModifierOutcome) {
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

    fn apply_projection(&mut self, combatant: &FinalCombatantState) {
        self.name = combatant.name.clone();
        self.hit_points = combatant.hit_points;
        self.active_modifiers = Vec::new();
        self.conditions = combatant.conditions.clone();
    }

    fn condition_labels(&self) -> Vec<String> {
        let mut labels = self.conditions.clone();
        for modifier in &self.active_modifiers {
            if !labels.contains(&modifier.label) {
                labels.push(modifier.label.clone());
            }
        }
        labels
    }

    fn action_resource_readout(&self) -> CombatantActionResourceReadout {
        CombatantActionResourceReadout {
            combatant_id: self.id.clone(),
            resources: self.action_resources.clone(),
        }
    }

    fn spend_action_resource(
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

    fn refresh_action_resource(
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
}

fn default_action_resources() -> Vec<ActionResourceState> {
    vec![ActionResourceState::standard_action_available()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::hexing_bolt_fixture_scenario;

    #[test]
    fn combat_state_initializes_standard_action_for_each_combatant() {
        let state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());

        let ledger = state.action_resource_ledger();

        assert_eq!(ledger.combatants.len(), 2);
        assert_eq!(ledger.combatants[0].combatant_id, "entity-adept");
        assert_eq!(
            ledger.combatants[0].resources,
            vec![ActionResourceState::standard_action_available()]
        );
        assert_eq!(ledger.combatants[1].combatant_id, "entity-raider");
        assert_eq!(
            ledger.combatants[1].resources[0].kind.code(),
            "standardAction"
        );
    }

    #[test]
    fn combat_state_spends_standard_action_once() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());

        let readout =
            state.spend_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let resources = state
            .action_resources_for("entity-adept")
            .expect("adept resources are initialized");

        assert!(readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceSpendDecisionKind::Spent
        );
        assert_eq!(
            readout.previous_resource,
            Some(ActionResourceState::standard_action_available())
        );
        assert_eq!(
            readout.next_resource,
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
        assert_eq!(
            resources.resources,
            vec![ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            )]
        );
    }

    #[test]
    fn combat_state_rejects_repeated_standard_action_spend_without_mutation() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        state.spend_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let before = state.action_resources_for("entity-adept");

        let readout =
            state.spend_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let after = state.action_resources_for("entity-adept");

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceSpendDecisionKind::RejectedByUnavailableResource
        );
        assert_eq!(before, after);
        assert_eq!(readout.previous_resource, readout.next_resource);
    }

    #[test]
    fn combat_state_rejects_missing_combatant_spend_without_mutation() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        let before = state.action_resource_ledger();

        let readout =
            state.spend_action_resource("entity-missing", ActionResourceKind::StandardAction);
        let after = state.action_resource_ledger();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceSpendDecisionKind::RejectedByMissingCombatant
        );
        assert_eq!(readout.previous_resource, None);
        assert_eq!(readout.next_resource, None);
        assert_eq!(before, after);
    }

    #[test]
    fn combat_state_refreshes_spent_standard_action() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        state.spend_action_resource("entity-adept", ActionResourceKind::StandardAction);

        let readout =
            state.refresh_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let resources = state
            .action_resources_for("entity-adept")
            .expect("adept resources are initialized");

        assert!(readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceRefreshDecisionKind::Refreshed
        );
        assert_eq!(readout.decision_kind.code(), "refreshed");
        assert_eq!(
            readout.previous_resource,
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
        assert_eq!(
            readout.next_resource,
            Some(ActionResourceState::standard_action_available())
        );
        assert_eq!(
            resources.resources,
            vec![ActionResourceState::standard_action_available()]
        );
    }

    #[test]
    fn combat_state_refreshes_full_standard_action_idempotently() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        let before = state.action_resource_ledger();

        let readout =
            state.refresh_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let after = state.action_resource_ledger();

        assert!(readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceRefreshDecisionKind::Refreshed
        );
        assert_eq!(
            readout.previous_resource,
            Some(ActionResourceState::standard_action_available())
        );
        assert_eq!(
            readout.next_resource,
            Some(ActionResourceState::standard_action_available())
        );
        assert_eq!(before, after);
    }

    #[test]
    fn combat_state_rejects_missing_combatant_refresh_without_mutation() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        let before = state.action_resource_ledger();

        let readout =
            state.refresh_action_resource("entity-missing", ActionResourceKind::StandardAction);
        let after = state.action_resource_ledger();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceRefreshDecisionKind::RejectedByMissingCombatant
        );
        assert_eq!(readout.decision_kind.code(), "rejectedByMissingCombatant");
        assert_eq!(readout.previous_resource, None);
        assert_eq!(readout.next_resource, None);
        assert_eq!(before, after);
    }

    #[test]
    fn combat_state_projection_update_preserves_action_resources() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        state.spend_action_resource("entity-adept", ActionResourceKind::StandardAction);
        let before = state.action_resources_for("entity-adept");
        let mut projection = state.project("Projected damage update.");
        projection.combatants[1].hit_points.current = 9;
        projection.combatants[1].conditions = vec!["rattled".to_string()];

        state.apply_projection(&projection);
        let after = state.action_resources_for("entity-adept");

        assert_eq!(before, after);
        assert_eq!(
            state.project("Current state.").combatants[1]
                .conditions
                .as_slice(),
            &["rattled".to_string()]
        );
    }
}
