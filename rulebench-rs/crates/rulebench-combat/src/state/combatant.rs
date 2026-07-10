//! Internal mutable combatant state.

use crate::model::{
    ActionResourceKind, ActionResourceRefreshDecisionKind, ActionResourceRefreshReadout,
    ActionResourceSpendDecisionKind, ActionResourceSpendReadout, ActionResourceState,
    ActiveModifier, BoundedValue, ClassBuildInputReadout, ClassDefinition, ClassLevelInput,
    Combatant, CombatantActionResourceReadout, FinalCombatantState, ItemDefinition,
    ModifierDefinition, ModifierDurationExpirationDecisionKind, ModifierDurationExpirationReadout,
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
    pub(super) inventory_item_ids: Vec<String>,
    pub(super) equipped_item_ids: Vec<String>,
    pub(super) base_ability_ids: Vec<String>,
    pub(super) available_ability_ids: Vec<String>,
    pub(super) class_inputs: Vec<ClassBuildInputReadout>,
}

impl CombatantState {
    pub(super) fn from_combatant(
        combatant: &Combatant,
        items: &[ItemDefinition],
        classes: &[ClassDefinition],
        modifiers: &[ModifierDefinition],
    ) -> Self {
        let mut state = Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            temporary_vitality: combatant.temporary_vitality,
            active_modifiers: combatant.active_modifiers.clone(),
            conditions: combatant.conditions.clone(),
            action_resources: action_resources_from_combatant(combatant),
            inventory_item_ids: combatant.inventory_item_ids.clone(),
            equipped_item_ids: Vec::new(),
            base_ability_ids: combatant.base_ability_ids.clone(),
            available_ability_ids: combatant.base_ability_ids.clone(),
            class_inputs: Vec::new(),
        };
        for input in &combatant.class_inputs {
            if let Some(class) = classes.iter().find(|class| class.id == input.class_id) {
                state.apply_class_grants(class, input, modifiers);
            }
        }
        state.base_ability_ids = state.available_ability_ids.clone();
        for item_id in &combatant.equipped_item_ids {
            if let Some(item) = items.iter().find(|item| item.id == *item_id) {
                state.apply_item_grants(item, modifiers);
                state.equipped_item_ids.push(item.id.clone());
            }
        }
        state
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
            action_resources: vec![ActionResourceState::standard_action_available()],
            inventory_item_ids: Vec::new(),
            equipped_item_ids: Vec::new(),
            base_ability_ids: Vec::new(),
            available_ability_ids: Vec::new(),
            class_inputs: Vec::new(),
        }
    }

    pub(super) fn apply_class_grants(
        &mut self,
        class: &ClassDefinition,
        input: &ClassLevelInput,
        modifiers: &[ModifierDefinition],
    ) {
        let grants = class
            .level_grants
            .iter()
            .filter(|grant| grant.level <= input.level)
            .collect::<Vec<_>>();
        let source_ids = grants
            .iter()
            .map(|grant| class_grant_source_id(class, grant.level))
            .collect::<Vec<_>>();
        for (grant, source_id) in grants.iter().zip(&source_ids) {
            for modifier_id in &grant.granted_modifier_ids {
                if let Some(definition) = modifiers
                    .iter()
                    .find(|modifier| modifier.id == *modifier_id)
                {
                    self.apply_active_modifier(ActiveModifier {
                        modifier_id: definition.id.clone(),
                        source_id: source_id.clone(),
                        label: definition.label.clone(),
                        duration: "class grant".to_string(),
                        tenure: ModifierTenure::Permanent,
                        stacking_group: definition.stacking_group.clone(),
                        stacking_policy: definition.stacking_policy,
                        duration_policy: ModifierDurationPolicy::Permanent,
                        remaining_turns: None,
                        remaining_rounds: None,
                    });
                }
            }
            for ability_id in &grant.granted_ability_ids {
                if !self.available_ability_ids.contains(ability_id) {
                    self.available_ability_ids.push(ability_id.clone());
                }
            }
            self.action_resources.extend(
                grant.granted_resource_pools.iter().map(|pool| {
                    ActionResourceState::from_pool_with_source(pool, source_id.clone())
                }),
            );
        }
        self.class_inputs.push(ClassBuildInputReadout {
            class_id: input.class_id.clone(),
            version: input.version.clone(),
            level: input.level,
            applied_grant_levels: grants.iter().map(|grant| grant.level).collect(),
            source_ids,
        });
    }

    pub(super) fn apply_item_grants(
        &mut self,
        item: &ItemDefinition,
        modifiers: &[ModifierDefinition],
    ) {
        for modifier_id in &item.granted_modifier_ids {
            if let Some(definition) = modifiers
                .iter()
                .find(|modifier| modifier.id == *modifier_id)
            {
                self.apply_active_modifier(ActiveModifier {
                    modifier_id: definition.id.clone(),
                    source_id: item.id.clone(),
                    label: definition.label.clone(),
                    duration: "while equipped".to_string(),
                    tenure: ModifierTenure::Permanent,
                    stacking_group: definition.stacking_group.clone(),
                    stacking_policy: definition.stacking_policy,
                    duration_policy: ModifierDurationPolicy::Permanent,
                    remaining_turns: None,
                    remaining_rounds: None,
                });
            }
        }
        for ability_id in &item.granted_ability_ids {
            if !self.available_ability_ids.contains(ability_id) {
                self.available_ability_ids.push(ability_id.clone());
            }
        }
        self.action_resources.extend(
            item.granted_resource_pools
                .iter()
                .map(|pool| ActionResourceState::from_pool_with_source(pool, item.id.clone())),
        );
    }

    pub(super) fn remove_item_grants(
        &mut self,
        item: &ItemDefinition,
        remaining_items: &[&ItemDefinition],
    ) {
        self.active_modifiers
            .retain(|modifier| modifier.source_id != item.id);
        self.action_resources
            .retain(|resource| resource.source_id != item.id);
        self.available_ability_ids = self.base_ability_ids.clone();
        for remaining_item in remaining_items {
            for ability_id in &remaining_item.granted_ability_ids {
                if !self.available_ability_ids.contains(ability_id) {
                    self.available_ability_ids.push(ability_id.clone());
                }
            }
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
        self.apply_active_modifier(active_modifier);
    }

    fn apply_active_modifier(&mut self, active_modifier: ActiveModifier) {
        match active_modifier.stacking_policy {
            ModifierStackingPolicy::Stack => self.active_modifiers.push(active_modifier),
            ModifierStackingPolicy::Replace => {
                self.active_modifiers
                    .retain(|active| active.stacking_group != active_modifier.stacking_group);
                self.active_modifiers.push(active_modifier);
            }
            ModifierStackingPolicy::Refresh => {
                if let Some(existing) = self
                    .active_modifiers
                    .iter_mut()
                    .find(|active| active.stacking_group == active_modifier.stacking_group)
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

    pub(super) fn equipment_readout(&self) -> crate::model::CombatantEquipmentReadout {
        crate::model::CombatantEquipmentReadout {
            combatant_id: self.id.clone(),
            inventory_item_ids: self.inventory_item_ids.clone(),
            equipped_item_ids: self.equipped_item_ids.clone(),
            available_ability_ids: self.available_ability_ids.clone(),
        }
    }

    pub(super) fn class_build_readout(&self) -> crate::model::CombatantClassBuildReadout {
        crate::model::CombatantClassBuildReadout {
            combatant_id: self.id.clone(),
            class_inputs: self.class_inputs.clone(),
        }
    }

    pub(super) fn action_resource_readout(&self) -> CombatantActionResourceReadout {
        CombatantActionResourceReadout {
            combatant_id: self.id.clone(),
            resources: self.action_resources.clone(),
        }
    }

    pub(super) fn spend_action_resource(
        &mut self,
        resource_id: &str,
        amount: u32,
    ) -> ActionResourceSpendReadout {
        let Some(resource) = self
            .action_resources
            .iter_mut()
            .find(|resource| resource.resource_id == resource_id)
        else {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: ActionResourceKind::StandardAction,
                amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByMissingResource,
                previous_resource: None,
                next_resource: None,
                reason: "Combatant does not have the requested action resource.".to_string(),
            };
        };

        let previous_resource = resource.clone();
        let requested_amount = amount;
        let Ok(amount) = i32::try_from(requested_amount) else {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: resource.kind,
                amount: requested_amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByInvalidAmount,
                previous_resource: Some(previous_resource.clone()),
                next_resource: Some(previous_resource),
                reason: "Action resource cost exceeds supported resource range.".to_string(),
            };
        };
        if amount <= 0 {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: resource.kind,
                amount: requested_amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByInvalidAmount,
                previous_resource: Some(previous_resource.clone()),
                next_resource: Some(previous_resource),
                reason: "Action resource cost must be greater than zero.".to_string(),
            };
        }
        if !resource.available {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: resource.kind,
                amount: requested_amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByUnavailableResource,
                previous_resource: Some(previous_resource.clone()),
                next_resource: Some(previous_resource),
                reason: "Action resource is not available.".to_string(),
            };
        }
        if resource.current < amount {
            return ActionResourceSpendReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: resource.kind,
                amount: requested_amount,
                accepted: false,
                decision_kind: ActionResourceSpendDecisionKind::RejectedByInsufficientResource,
                previous_resource: Some(previous_resource.clone()),
                next_resource: Some(previous_resource),
                reason: "Action resource cannot cover the declared cost.".to_string(),
            };
        }

        resource.current -= amount;
        resource.available = resource.current > 0;
        resource.remaining_refresh_turns = match &resource.refresh_policy {
            crate::model::ActionResourceRefreshPolicy::Turns(turns) => Some(*turns),
            _ => None,
        };
        let next_resource = resource.clone();

        ActionResourceSpendReadout {
            combatant_id: self.id.clone(),
            resource_id: resource_id.to_string(),
            resource_kind: resource.kind,
            amount: requested_amount,
            accepted: true,
            decision_kind: ActionResourceSpendDecisionKind::Spent,
            previous_resource: Some(previous_resource),
            next_resource: Some(next_resource),
            reason: "Action resource spent.".to_string(),
        }
    }

    pub(super) fn refresh_action_resource(
        &mut self,
        resource_id: &str,
    ) -> ActionResourceRefreshReadout {
        let Some(resource) = self
            .action_resources
            .iter_mut()
            .find(|resource| resource.resource_id == resource_id)
        else {
            return ActionResourceRefreshReadout {
                combatant_id: self.id.clone(),
                resource_id: resource_id.to_string(),
                resource_kind: ActionResourceKind::StandardAction,
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
        resource.remaining_refresh_turns = None;
        let next_resource = resource.clone();

        ActionResourceRefreshReadout {
            combatant_id: self.id.clone(),
            resource_id: resource_id.to_string(),
            resource_kind: resource.kind,
            accepted: true,
            decision_kind: ActionResourceRefreshDecisionKind::Refreshed,
            previous_resource: Some(previous_resource),
            next_resource: Some(next_resource),
            reason: "Action resource refreshed.".to_string(),
        }
    }

    pub(super) fn advance_action_resources_for_turn_start(
        &mut self,
    ) -> Vec<ActionResourceRefreshReadout> {
        let resource_ids = self
            .action_resources
            .iter()
            .map(|resource| resource.resource_id.clone())
            .collect::<Vec<_>>();

        resource_ids
            .into_iter()
            .filter_map(|resource_id| self.advance_action_resource_for_turn_start(&resource_id))
            .collect()
    }

    pub(super) fn refresh_action_resources_for_combat_start(
        &mut self,
    ) -> Vec<ActionResourceRefreshReadout> {
        let resource_ids = self
            .action_resources
            .iter()
            .filter(|resource| {
                resource.refresh_policy == crate::model::ActionResourceRefreshPolicy::CombatStart
            })
            .map(|resource| resource.resource_id.clone())
            .collect::<Vec<_>>();

        resource_ids
            .into_iter()
            .map(|resource_id| self.refresh_action_resource(&resource_id))
            .collect()
    }

    fn advance_action_resource_for_turn_start(
        &mut self,
        resource_id: &str,
    ) -> Option<ActionResourceRefreshReadout> {
        let resource = self
            .action_resources
            .iter_mut()
            .find(|resource| resource.resource_id == resource_id)?;
        let previous_resource = resource.clone();

        match &resource.refresh_policy {
            crate::model::ActionResourceRefreshPolicy::TurnStart => {
                resource.current = resource.max;
                resource.available = resource.current > 0;
                resource.remaining_refresh_turns = None;
                Some(ActionResourceRefreshReadout {
                    combatant_id: self.id.clone(),
                    resource_id: resource.resource_id.clone(),
                    resource_kind: resource.kind,
                    accepted: true,
                    decision_kind: ActionResourceRefreshDecisionKind::Refreshed,
                    previous_resource: Some(previous_resource),
                    next_resource: Some(resource.clone()),
                    reason: "Action resource refreshed at turn start.".to_string(),
                })
            }
            crate::model::ActionResourceRefreshPolicy::Turns(turns)
                if resource.current < resource.max =>
            {
                let remaining = resource.remaining_refresh_turns.unwrap_or(*turns);
                if remaining <= 1 {
                    resource.current = resource.max;
                    resource.available = resource.current > 0;
                    resource.remaining_refresh_turns = None;
                    Some(ActionResourceRefreshReadout {
                        combatant_id: self.id.clone(),
                        resource_id: resource.resource_id.clone(),
                        resource_kind: resource.kind,
                        accepted: true,
                        decision_kind: ActionResourceRefreshDecisionKind::Refreshed,
                        previous_resource: Some(previous_resource),
                        next_resource: Some(resource.clone()),
                        reason: "Cooldown resource refreshed at turn start.".to_string(),
                    })
                } else {
                    resource.remaining_refresh_turns = Some(remaining - 1);
                    Some(ActionResourceRefreshReadout {
                        combatant_id: self.id.clone(),
                        resource_id: resource.resource_id.clone(),
                        resource_kind: resource.kind,
                        accepted: true,
                        decision_kind: ActionResourceRefreshDecisionKind::CooldownAdvanced,
                        previous_resource: Some(previous_resource),
                        next_resource: Some(resource.clone()),
                        reason: "Cooldown resource advanced at turn start.".to_string(),
                    })
                }
            }
            crate::model::ActionResourceRefreshPolicy::Never
            | crate::model::ActionResourceRefreshPolicy::CombatStart
            | crate::model::ActionResourceRefreshPolicy::Turns(_) => None,
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

fn action_resources_from_combatant(combatant: &Combatant) -> Vec<ActionResourceState> {
    if combatant.resource_pools.is_empty() {
        return vec![ActionResourceState::standard_action_available()];
    }

    combatant
        .resource_pools
        .iter()
        .map(ActionResourceState::from_pool)
        .collect()
}

fn class_grant_source_id(class: &ClassDefinition, level: u32) -> String {
    format!("class:{}@{}:{}", class.id, class.version, level)
}
