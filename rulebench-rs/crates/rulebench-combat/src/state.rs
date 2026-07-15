//! Mutable authoritative combat state.

mod combatant;

use combatant::CombatantState;

use crate::model::{
    ActionResourceKind, ActionResourceLedgerReadout, ActionResourceRefreshDecisionKind,
    ActionResourceRefreshReadout, ActionResourceSpendDecisionKind, ActionResourceSpendReadout,
    ActiveModifier, ClassBuildLedgerReadout, CombatantActionResourceReadout,
    CombatantClassBuildReadout, CombatantEquipmentReadout, DamageOutcome, EquipmentLedgerReadout,
    GridPosition, HealingOutcome, ItemDefinition, ModifierDefinition,
    ModifierDurationExpirationReadout, ModifierOutcome, ResourceChangeOutcome, RulebenchRejection,
    RulebenchScenario, ScenarioProjection, SpatialBoardState, SpatialCellState,
    TemporaryVitalityOutcome,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatState {
    board: SpatialBoardState,
    combatants: Vec<CombatantState>,
}

impl CombatState {
    pub fn from_scenario(scenario: &RulebenchScenario) -> Self {
        let mut combatants = scenario
            .combatants
            .iter()
            .map(|combatant| {
                CombatantState::from_combatant(
                    combatant,
                    &scenario.items,
                    &scenario.classes,
                    &scenario.modifiers,
                )
            })
            .collect::<Vec<_>>();
        for combatant in &mut combatants {
            let allowance = scenario
                .actions
                .iter()
                .filter(|action| action.actor_id == combatant.id)
                .filter_map(|action| action.movement.as_ref())
                .map(|movement| movement.allowance)
                .max()
                .unwrap_or_default();
            combatant.movement_maximum = allowance;
            combatant.movement_remaining = allowance;
        }
        Self {
            board: board_from_scenario(scenario),
            combatants,
        }
    }

    pub fn from_projection(projection: &ScenarioProjection) -> Self {
        Self {
            board: projection.board.clone(),
            combatants: projection
                .combatants
                .iter()
                .map(CombatantState::from_final_state)
                .collect(),
        }
    }

    pub fn project(&self, summary: &str) -> ScenarioProjection {
        let combatants = self
            .combatants
            .iter()
            .map(CombatantState::to_final_state)
            .collect::<Vec<_>>();
        let mut board = self.board.clone();
        for cell in &mut board.cells {
            cell.occupant_ids = combatants
                .iter()
                .filter(|combatant| combatant.position == cell.position)
                .map(|combatant| combatant.id.clone())
                .collect();
            cell.occupant_ids.sort();
        }
        ScenarioProjection {
            summary: summary.to_string(),
            board,
            combatants,
        }
    }

    pub fn active_combatant_ids(&self) -> Vec<String> {
        self.combatants
            .iter()
            .filter(|combatant| combatant.hit_points.current > 0)
            .map(|combatant| combatant.id.clone())
            .collect()
    }

    pub fn refresh_movement_for(&mut self, combatant_id: &str) {
        if let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|item| item.id == combatant_id)
        {
            combatant.movement_remaining = combatant.movement_maximum;
        }
    }

    pub fn clear_all_movement(&mut self) {
        for combatant in &mut self.combatants {
            combatant.movement_remaining = 0;
        }
    }

    pub fn apply_movement(
        &mut self,
        combatant_id: &str,
        destination: GridPosition,
        cost: u32,
    ) -> bool {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|item| item.id == combatant_id)
        else {
            return false;
        };
        if cost > combatant.movement_remaining {
            return false;
        }
        combatant.position = destination;
        combatant.movement_remaining -= cost;
        true
    }

    pub fn apply_effect_movement(&mut self, combatant_id: &str, destination: GridPosition) -> bool {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|item| item.id == combatant_id)
        else {
            return false;
        };
        combatant.position = destination;
        true
    }

    pub fn preview_resource_change(
        &self,
        combatant_id: &str,
        resource_id: &str,
        delta: i32,
    ) -> Result<ResourceChangeOutcome, RulebenchRejection> {
        let resource = self
            .combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .and_then(|combatant| {
                combatant
                    .action_resources
                    .iter()
                    .find(|resource| resource.resource_id == resource_id)
            })
            .ok_or(RulebenchRejection::EffectResourceMissing)?;
        let Some(after) = resource.current.checked_add(delta) else {
            return Err(RulebenchRejection::EffectResourceOutOfBounds);
        };
        if after < 0 || after > resource.max {
            return Err(RulebenchRejection::EffectResourceOutOfBounds);
        }
        Ok(ResourceChangeOutcome {
            target_id: combatant_id.to_string(),
            resource_id: resource_id.to_string(),
            requested_delta: delta,
            before: resource.current,
            after,
            maximum: resource.max,
        })
    }

    pub fn apply_resource_change(&mut self, outcome: &ResourceChangeOutcome) -> bool {
        let Some(resource) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == outcome.target_id)
            .and_then(|combatant| {
                combatant
                    .action_resources
                    .iter_mut()
                    .find(|resource| resource.resource_id == outcome.resource_id)
            })
        else {
            return false;
        };
        resource.current = outcome.after;
        resource.available = outcome.after > 0;
        true
    }

    pub fn apply_hit(&mut self, damage: &DamageOutcome, modifier: Option<&ModifierOutcome>) {
        for combatant in &mut self.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
                combatant.temporary_vitality = damage.temporary_vitality_after;
            }
            if modifier.is_some_and(|modifier| combatant.id == modifier.target_id) {
                combatant.apply_modifier(modifier.expect("checked modifier presence"));
            }
        }
    }

    pub fn apply_modifier(&mut self, modifier: &ModifierOutcome) {
        if let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == modifier.target_id)
        {
            combatant.apply_modifier(modifier);
        }
    }

    pub fn apply_healing(&mut self, healing: &HealingOutcome) {
        if let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == healing.target_id)
        {
            combatant.hit_points = healing.after;
        }
    }

    pub fn apply_temporary_vitality(&mut self, vitality: &TemporaryVitalityOutcome) {
        if let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == vitality.target_id)
        {
            combatant.temporary_vitality = vitality.after;
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

    pub fn equipment_ledger(&self) -> EquipmentLedgerReadout {
        EquipmentLedgerReadout {
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::equipment_readout)
                .collect(),
        }
    }

    pub fn class_build_ledger(&self) -> ClassBuildLedgerReadout {
        ClassBuildLedgerReadout {
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::class_build_readout)
                .collect(),
        }
    }

    pub fn class_build_for(&self, combatant_id: &str) -> Option<CombatantClassBuildReadout> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(CombatantState::class_build_readout)
    }

    pub fn equipment_for(&self, combatant_id: &str) -> Option<CombatantEquipmentReadout> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(CombatantState::equipment_readout)
    }

    pub fn equip_item(
        &mut self,
        combatant_id: &str,
        item: &ItemDefinition,
        modifiers: &[ModifierDefinition],
    ) -> bool {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return false;
        };
        combatant.apply_item_grants(item, modifiers);
        combatant.equipped_item_ids.push(item.id.clone());
        true
    }

    pub fn unequip_item(
        &mut self,
        combatant_id: &str,
        item: &ItemDefinition,
        items: &[ItemDefinition],
    ) -> bool {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return false;
        };
        combatant
            .equipped_item_ids
            .retain(|equipped_item_id| equipped_item_id != &item.id);
        let remaining_items = combatant
            .equipped_item_ids
            .iter()
            .filter_map(|item_id| items.iter().find(|candidate| candidate.id == *item_id))
            .collect::<Vec<_>>();
        combatant.remove_item_grants(item, &remaining_items);
        true
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
        resource_id: &str,
        amount: u32,
    ) -> ActionResourceSpendReadout {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return ActionResourceSpendReadout {
                combatant_id: combatant_id.to_string(),
                resource_id: resource_id.to_string(),
                resource_kind: ActionResourceKind::StandardAction,
                amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByMissingCombatant,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant is not present in the action resource ledger.".to_string(),
            };
        };

        combatant.spend_action_resource(resource_id, amount)
    }

    pub fn refresh_action_resource(
        &mut self,
        combatant_id: &str,
        resource_id: &str,
    ) -> ActionResourceRefreshReadout {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return ActionResourceRefreshReadout {
                combatant_id: combatant_id.to_string(),
                resource_id: resource_id.to_string(),
                resource_kind: ActionResourceKind::StandardAction,
                accepted: false,
                decision_kind: ActionResourceRefreshDecisionKind::RejectedByMissingCombatant,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant is not present in the action resource ledger.".to_string(),
            };
        };

        combatant.refresh_action_resource(resource_id)
    }

    pub fn advance_action_resources_for_turn_start(
        &mut self,
        combatant_id: &str,
    ) -> Vec<ActionResourceRefreshReadout> {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return Vec::new();
        };

        combatant.advance_action_resources_for_turn_start()
    }

    pub fn refresh_action_resources_for_combat_start(
        &mut self,
    ) -> Vec<ActionResourceRefreshReadout> {
        self.combatants
            .iter_mut()
            .flat_map(CombatantState::refresh_action_resources_for_combat_start)
            .collect()
    }

    pub fn advance_turn_counted_modifiers_for(
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

        combatant.advance_turn_counted_modifiers()
    }

    pub fn advance_all_round_counted_modifiers(
        &mut self,
    ) -> Vec<ModifierDurationExpirationReadout> {
        self.combatants
            .iter_mut()
            .flat_map(CombatantState::advance_round_counted_modifiers)
            .collect()
    }

    pub fn expire_modifiers_for_event(
        &mut self,
        combatant_id: &str,
        event: &str,
    ) -> Vec<ModifierDurationExpirationReadout> {
        let Some(combatant) = self
            .combatants
            .iter_mut()
            .find(|combatant| combatant.id == combatant_id)
        else {
            return Vec::new();
        };

        combatant.expire_modifiers_for_event(event)
    }

    pub fn expire_all_modifiers_for_event(
        &mut self,
        event: &str,
    ) -> Vec<ModifierDurationExpirationReadout> {
        self.combatants
            .iter_mut()
            .flat_map(|combatant| combatant.expire_modifiers_for_event(event))
            .collect()
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
                combatant.temporary_vitality = state.temporary_vitality;
                combatant.active_modifiers = state.active_modifiers.clone();
                combatant.conditions = state.condition_labels();
                combatant.position = state.position;
                combatant.equipped_item_ids = state.equipped_item_ids.clone();
            }
        }
        scenario
    }
}

fn board_from_scenario(scenario: &RulebenchScenario) -> SpatialBoardState {
    let mut cells = Vec::new();
    for y in 0..scenario.grid.height {
        for x in 0..scenario.grid.width {
            let position = GridPosition { x, y };
            let terrain_tags = scenario
                .grid
                .cells
                .iter()
                .find(|cell| cell.position == position)
                .map(|cell| cell.terrain_tags.clone())
                .unwrap_or_default();
            let blocks_movement = terrain_tags
                .iter()
                .any(|tag| tag == "blocked" || tag == "wall");
            cells.push(SpatialCellState {
                position,
                terrain_tags,
                blocks_movement,
                occupant_ids: Vec::new(),
            });
        }
    }
    SpatialBoardState {
        id: scenario.metadata.id.clone(),
        width: scenario.grid.width,
        height: scenario.grid.height,
        cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use rulebench_ruleset::ActionResourceCost;

    fn test_scenario() -> RulebenchScenario {
        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "state-test".to_string(),
                title: "State Test".to_string(),
                summary: "Minimal state test scenario.".to_string(),
                seed_label: "state-test".to_string(),
            },
            content_pack_set: None,
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
            side_id: match team {
                Team::Ally => "ally",
                Team::Enemy => "enemy",
            }
            .to_string(),
            initiative: 0,
            position: GridPosition { x: 0, y: 0 },
            hit_points: BoundedValue {
                current: 12,
                max: 12,
            },
            temporary_vitality: 0,
            class_inputs: Vec::new(),
            stats: StatBlock {
                base_stats: Vec::new(),
                derived_stats: Vec::new(),
            },
            defenses: Vec::new(),
            resource_pools: vec![ActionResourcePool::standard_action()],
            inventory_item_ids: Vec::new(),
            equipped_item_ids: Vec::new(),
            base_ability_ids: Vec::new(),
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
                operation_pipeline: None,
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
            resource_costs: vec![ActionResourceCost::standard_action()],
            movement: None,
            action_text: "Test action.".to_string(),
            effect_text: "Test effect.".to_string(),
        }
    }

    fn test_modifier(
        source_id: &str,
        stacking_policy: ModifierStackingPolicy,
        turns: u32,
    ) -> ModifierOutcome {
        ModifierOutcome {
            target_id: "entity-raider".to_string(),
            modifier_id: "rattled".to_string(),
            source_id: source_id.to_string(),
            label: "rattled".to_string(),
            duration: format!("{turns} turns"),
            stacking_group: "rattled".to_string(),
            stacking_policy,
            duration_policy: ModifierDurationPolicy::Turns(turns),
            remaining_turns: Some(turns),
            remaining_rounds: None,
        }
    }

    fn test_round_modifier(rounds: u32) -> ModifierOutcome {
        ModifierOutcome {
            target_id: "entity-raider".to_string(),
            modifier_id: "rattled".to_string(),
            source_id: "round-source".to_string(),
            label: "rattled".to_string(),
            duration: format!("{rounds} rounds"),
            stacking_group: "round-rattled".to_string(),
            stacking_policy: ModifierStackingPolicy::Refresh,
            duration_policy: ModifierDurationPolicy::Rounds(rounds),
            remaining_turns: None,
            remaining_rounds: Some(rounds),
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

        let readout = state.spend_action_resource("entity-adept", "standard-action", 1);
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
        state.spend_action_resource("entity-adept", "standard-action", 1);
        let before = state.action_resources_for("entity-adept");

        let readout = state.spend_action_resource("entity-adept", "standard-action", 1);
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
    fn combat_state_rejects_cost_above_available_resource_without_mutation() {
        let mut state = CombatState::from_scenario(&test_scenario());
        let before = state.action_resource_ledger();

        let readout = state.spend_action_resource("entity-adept", "standard-action", 2);
        let after = state.action_resource_ledger();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            ActionResourceSpendDecisionKind::RejectedByInsufficientResource
        );
        assert_eq!(readout.amount, 2);
        assert_eq!(readout.previous_resource, readout.next_resource);
        assert_eq!(before, after);
    }

    #[test]
    fn combat_state_rejects_missing_combatant_spend_without_mutation() {
        let mut state = CombatState::from_scenario(&test_scenario());
        let before = state.action_resource_ledger();

        let readout = state.spend_action_resource("entity-missing", "standard-action", 1);
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
        state.spend_action_resource("entity-adept", "standard-action", 1);

        let readout = state.refresh_action_resource("entity-adept", "standard-action");
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

        let readout = state.refresh_action_resource("entity-adept", "standard-action");
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

        let readout = state.refresh_action_resource("entity-missing", "standard-action");
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

        let readouts = state.advance_turn_counted_modifiers_for("entity-raider");

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
            "Turn-counted modifier expired at turn boundary."
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
    fn combat_state_stacks_replaces_and_refreshes_modifiers_by_group() {
        let mut state = CombatState::from_scenario(&test_scenario());
        state.combatants[1].apply_modifier(&test_modifier(
            "source-one",
            ModifierStackingPolicy::Stack,
            2,
        ));
        state.combatants[1].apply_modifier(&test_modifier(
            "source-two",
            ModifierStackingPolicy::Stack,
            2,
        ));
        assert_eq!(
            state
                .active_modifiers_for("entity-raider")
                .expect("raider")
                .len(),
            2
        );

        state.combatants[1].apply_modifier(&test_modifier(
            "source-three",
            ModifierStackingPolicy::Replace,
            2,
        ));
        assert_eq!(
            state
                .active_modifiers_for("entity-raider")
                .expect("raider")
                .len(),
            1
        );
        assert_eq!(
            state.active_modifiers_for("entity-raider").expect("raider")[0].source_id,
            "source-three"
        );

        state.combatants[1].apply_modifier(&test_modifier(
            "source-four",
            ModifierStackingPolicy::Refresh,
            3,
        ));
        assert_eq!(
            state
                .active_modifiers_for("entity-raider")
                .expect("raider")
                .len(),
            1
        );
        assert_eq!(
            state.active_modifiers_for("entity-raider").expect("raider")[0].source_id,
            "source-four"
        );
        assert_eq!(
            state.active_modifiers_for("entity-raider").expect("raider")[0].remaining_turns,
            Some(3)
        );
    }

    #[test]
    fn combat_state_advances_turn_counted_modifier_before_expiration() {
        let mut state = CombatState::from_scenario(&test_scenario());
        state.combatants[1].apply_modifier(&test_modifier(
            "source",
            ModifierStackingPolicy::Refresh,
            2,
        ));

        let first = state.advance_turn_counted_modifiers_for("entity-raider");
        assert_eq!(
            first[0].decision_kind,
            ModifierDurationExpirationDecisionKind::Advanced
        );
        assert_eq!(
            first[0]
                .next_modifier
                .as_ref()
                .and_then(|modifier| modifier.remaining_turns),
            Some(1)
        );

        let second = state.advance_turn_counted_modifiers_for("entity-raider");
        assert_eq!(
            second[0].decision_kind,
            ModifierDurationExpirationDecisionKind::Expired
        );
        assert!(state
            .active_modifiers_for("entity-raider")
            .expect("raider")
            .is_empty());
    }

    #[test]
    fn combat_state_advances_round_counted_modifier_at_round_boundary() {
        let mut state = CombatState::from_scenario(&test_scenario());
        state.combatants[1].apply_modifier(&test_round_modifier(2));

        let first = state.advance_all_round_counted_modifiers();
        assert_eq!(
            first[0].decision_kind,
            ModifierDurationExpirationDecisionKind::Advanced
        );
        assert_eq!(
            first[0]
                .next_modifier
                .as_ref()
                .and_then(|modifier| modifier.remaining_rounds),
            Some(1)
        );
        assert_eq!(
            first[0].reason,
            "Round-counted modifier duration advanced at round boundary."
        );

        let second = state.advance_all_round_counted_modifiers();
        assert_eq!(
            second[0].decision_kind,
            ModifierDurationExpirationDecisionKind::Expired
        );
        assert!(state
            .active_modifiers_for("entity-raider")
            .expect("raider")
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

        let readouts = state.advance_turn_counted_modifiers_for("entity-raider");

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

        let readouts = state.advance_turn_counted_modifiers_for("entity-raider");
        let after = state.project("Before no-op expiration.");

        assert!(readouts.is_empty());
        assert_eq!(after, before);
    }

    #[test]
    fn combat_state_projection_update_preserves_action_resources() {
        let mut state = CombatState::from_scenario(&test_scenario());
        state.spend_action_resource("entity-adept", "standard-action", 1);
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

    #[test]
    fn combat_state_owns_board_positions_occupancy_and_movement_state() {
        let mut scenario = test_scenario();
        scenario.combatants[1].position = GridPosition { x: 1, y: 0 };
        scenario.grid.cells.push(GridCell {
            position: GridPosition { x: 1, y: 0 },
            terrain_tags: vec!["wall".to_string()],
        });

        let state = CombatState::from_scenario(&scenario);
        let projection = state.project("Spatial state.");

        assert_eq!(projection.board.id, "state-test");
        assert_eq!(projection.board.cells.len(), 2);
        assert_eq!(projection.board.cells[0].occupant_ids, vec!["entity-adept"]);
        assert_eq!(
            projection.board.cells[1].occupant_ids,
            vec!["entity-raider"]
        );
        assert!(projection.board.cells[1].blocks_movement);
        assert_eq!(
            projection.combatants[1].position,
            GridPosition { x: 1, y: 0 }
        );
        assert_eq!(projection.combatants[1].movement_remaining, 0);
    }

    #[test]
    fn spatial_position_changes_authoritative_state_fingerprint() {
        let state = CombatState::from_scenario(&test_scenario());
        let before = state.project("Spatial state.");
        let mut after = before.clone();
        after.combatants[0].position = GridPosition { x: 1, y: 0 };
        after.board.cells[0].occupant_ids.clear();
        after.board.cells[1].occupant_ids = vec!["entity-adept".to_string()];

        assert_ne!(
            crate::fingerprint_projected_state(&before),
            crate::fingerprint_projected_state(&after)
        );
    }
}
