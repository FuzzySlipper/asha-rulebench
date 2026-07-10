//! Mutable authoritative combat state.

mod combatant;

use combatant::CombatantState;

use crate::model::{
    ActionResourceKind, ActionResourceLedgerReadout, ActionResourceRefreshDecisionKind,
    ActionResourceRefreshReadout, ActionResourceSpendDecisionKind, ActionResourceSpendReadout,
    ActiveModifier, CombatantActionResourceReadout, DamageOutcome,
    ModifierDurationExpirationReadout, ModifierOutcome, RulebenchScenario, ScenarioProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatState {
    combatants: Vec<CombatantState>,
}

impl CombatState {
    pub fn from_scenario(scenario: &RulebenchScenario) -> Self {
        Self {
            combatants: scenario
                .combatants
                .iter()
                .map(CombatantState::from_combatant)
                .collect(),
        }
    }

    pub fn from_projection(projection: &ScenarioProjection) -> Self {
        Self {
            combatants: projection
                .combatants
                .iter()
                .map(CombatantState::from_final_state)
                .collect(),
        }
    }

    pub fn project(&self, summary: &str) -> ScenarioProjection {
        ScenarioProjection {
            summary: summary.to_string(),
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::to_final_state)
                .collect(),
        }
    }

    pub fn apply_hit(&mut self, damage: &DamageOutcome, modifier: &ModifierOutcome) {
        for combatant in &mut self.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
            }
            if combatant.id == modifier.target_id {
                combatant.apply_modifier(modifier);
            }
        }
    }

    pub fn apply_projection(&mut self, projection: &ScenarioProjection) {
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

    pub fn action_resource_ledger(&self) -> ActionResourceLedgerReadout {
        ActionResourceLedgerReadout {
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::action_resource_readout)
                .collect(),
        }
    }

    pub fn action_resources_for(
        &self,
        combatant_id: &str,
    ) -> Option<CombatantActionResourceReadout> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(CombatantState::action_resource_readout)
    }

    pub fn spend_action_resource(
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

    pub fn refresh_action_resource(
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

    pub fn expire_temporary_modifiers_for(
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

    pub fn active_modifiers_for(&self, combatant_id: &str) -> Option<&[ActiveModifier]> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(|combatant| combatant.active_modifiers.as_slice())
    }

    pub fn apply_to_scenario(&self, mut scenario: RulebenchScenario) -> RulebenchScenario {
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
    use crate::model::*;

    fn test_scenario() -> RulebenchScenario {
        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "state-test".to_string(),
                title: "State Test".to_string(),
                summary: "Minimal state test scenario.".to_string(),
                seed_label: "state-test".to_string(),
            },
            rulesets: Vec::new(),
            selected_ruleset_id: "test-rules".to_string(),
            grid: Grid {
                width: 2,
                height: 1,
                cells: Vec::new(),
            },
            combatants: vec![
                test_combatant("entity-adept", Team::Ally),
                test_combatant("entity-raider", Team::Enemy),
            ],
            entities: Vec::new(),
            abilities: Vec::new(),
            selected_ability_id: None,
            classes: Vec::new(),
            selected_class_id: None,
            stat_definitions: Vec::new(),
            modifiers: Vec::new(),
            items: Vec::new(),
            selected_item_id: None,
            actions: Vec::new(),
            selected_action: test_action(),
        }
    }

    fn test_combatant(id: &str, team: Team) -> Combatant {
        Combatant {
            id: id.to_string(),
            entity_id: id.to_string(),
            name: id.to_string(),
            team,
            position: GridPosition { x: 0, y: 0 },
            hit_points: BoundedValue {
                current: 12,
                max: 12,
            },
            class_ids: Vec::new(),
            stats: StatBlock {
                base_stats: Vec::new(),
                derived_stats: Vec::new(),
            },
            defenses: Vec::new(),
            equipped_item_ids: Vec::new(),
            active_modifiers: Vec::new(),
            conditions: Vec::new(),
            is_actor: true,
        }
    }

    fn test_action() -> ActionDefinition {
        ActionDefinition {
            id: "test-action".to_string(),
            ruleset_id: "test-ruleset".to_string(),
            ability_id: "test-ability".to_string(),
            name: "Test Action".to_string(),
            actor_id: "entity-adept".to_string(),
            targeting: TargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 1,
                visibility_requirement: VisibilityRequirement::Ignored,
                target_ids: vec!["entity-raider".to_string()],
                visible_target_ids: vec!["entity-raider".to_string()],
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 0,
                modifier_stat_id: "attack".to_string(),
                defense: DefenseReference {
                    id: "defense".to_string(),
                    label: "Defense".to_string(),
                },
            }),
            hit: HitEffect {
                damage_bonus: 0,
                damage_type: "test".to_string(),
                modifier_id: "test-modifier".to_string(),
                modifier_label: "Test Modifier".to_string(),
                modifier_duration: "test".to_string(),
                operations: Vec::new(),
            },
            action_text: "Test action.".to_string(),
            effect_text: "Test effect.".to_string(),
        }
    }

    #[test]
    fn combat_state_initializes_standard_action_for_each_combatant() {
        let state = CombatState::from_scenario(&test_scenario());

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
        let mut state = CombatState::from_scenario(&test_scenario());

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
        let mut state = CombatState::from_scenario(&test_scenario());
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
        let mut state = CombatState::from_scenario(&test_scenario());
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
        let mut state = CombatState::from_scenario(&test_scenario());
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
        let mut state = CombatState::from_scenario(&test_scenario());
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
        let mut state = CombatState::from_scenario(&test_scenario());
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
        let mut scenario = test_scenario();
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
        let mut scenario = test_scenario();
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
        let mut state = CombatState::from_scenario(&test_scenario());
        let before = state.project("Before no-op expiration.");

        let readouts = state.expire_temporary_modifiers_for("entity-raider");
        let after = state.project("Before no-op expiration.");

        assert!(readouts.is_empty());
        assert_eq!(after, before);
    }

    #[test]
    fn combat_state_projection_update_preserves_action_resources() {
        let mut state = CombatState::from_scenario(&test_scenario());
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
