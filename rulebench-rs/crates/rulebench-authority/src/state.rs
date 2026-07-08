mod combatant;

use combatant::CombatantState;

use crate::model::{
    ActionResourceKind, ActionResourceLedgerReadout, ActionResourceRefreshDecisionKind,
    ActionResourceRefreshReadout, ActionResourceSpendDecisionKind, ActionResourceSpendReadout,
    DamageOutcome, ModifierDurationExpirationReadout, ModifierOutcome, RulebenchScenario,
    ScenarioProjection,
};

#[cfg(test)]
use crate::model::{
    ActionResourceState, ActiveModifier, CombatantActionResourceReadout,
    ModifierDurationExpirationDecisionKind,
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

    #[cfg(test)]
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

    pub(crate) fn expire_temporary_modifiers_for(
        &mut self,
        combatant_id: &str,
    ) -> Vec<ModifierDurationExpirationReadout> {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return Vec::new();
        };

        combatant.expire_temporary_modifiers()
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
    fn combat_state_expires_temporary_modifier_at_turn_boundary() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "rattled",
                "rattled",
                "until end of next turn",
            ));
        let mut state = CombatState::from_scenario(&scenario);

        let readouts = state.expire_temporary_modifiers_for("entity-raider");

        assert_eq!(readouts.len(), 1);
        assert!(readouts[0].accepted);
        assert_eq!(
            readouts[0].decision_kind,
            ModifierDurationExpirationDecisionKind::Expired
        );
        assert_eq!(readouts[0].decision_kind.code(), "expired");
        assert_eq!(readouts[0].combatant_id, "entity-raider");
        assert_eq!(readouts[0].modifier_id, "rattled");
        assert_eq!(
            readouts[0].previous_modifier,
            ActiveModifier::temporary("rattled", "rattled", "until end of next turn")
        );
        assert_eq!(readouts[0].next_modifier, None);
        assert_eq!(
            readouts[0].reason,
            "Temporary modifier expired at turn boundary."
        );
        assert_eq!(state.active_modifiers_for("entity-raider"), Some(&[][..]));
        assert!(state
            .project("After duration expiration.")
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider remains present")
            .conditions
            .is_empty());
    }

    #[test]
    fn combat_state_preserves_permanent_modifier_at_turn_boundary() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1]
            .active_modifiers
            .push(ActiveModifier::permanent(
                "battle-drilled",
                "battle-drilled",
            ));
        let mut state = CombatState::from_scenario(&scenario);

        let readouts = state.expire_temporary_modifiers_for("entity-raider");

        assert!(readouts.is_empty());
        assert_eq!(
            state.active_modifiers_for("entity-raider"),
            Some(
                &[ActiveModifier::permanent(
                    "battle-drilled",
                    "battle-drilled"
                )][..]
            )
        );
        assert_eq!(
            state
                .project("Permanent modifier remains.")
                .combatants
                .iter()
                .find(|combatant| combatant.id == "entity-raider")
                .expect("raider remains present")
                .conditions,
            vec!["battle-drilled".to_string()]
        );
    }

    #[test]
    fn combat_state_expiration_noops_without_temporary_modifiers() {
        let mut state = CombatState::from_scenario(&hexing_bolt_fixture_scenario());
        let before = state.project("Before no-op expiration.");

        let readouts = state.expire_temporary_modifiers_for("entity-raider");
        let after = state.project("Before no-op expiration.");

        assert!(readouts.is_empty());
        assert_eq!(after, before);
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
