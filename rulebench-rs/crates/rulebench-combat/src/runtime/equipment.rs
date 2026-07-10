//! Authoritative inventory and equipment command handling.

use super::*;
use crate::modifiers::effective_stats_for_combatant;

impl CombatSessionState {
    pub fn submit_equipment_command(
        &mut self,
        command: EquipmentCommandSpec,
    ) -> EquipmentCommandReadout {
        let previous_equipment = self.state.equipment_for(&command.combatant_id);
        let rejection = self.validate_equipment_command(&command, previous_equipment.as_ref());
        if let Some((decision_kind, reason)) = rejection {
            return EquipmentCommandReadout {
                command,
                accepted: false,
                decision_kind,
                previous_equipment: previous_equipment.clone(),
                next_equipment: previous_equipment,
                reason,
            };
        }

        let item = self
            .scenario
            .item_by_id(&command.item_id)
            .expect("validated equipment command item")
            .clone();
        let mutated = match command.kind {
            EquipmentCommandKind::Equip => {
                self.state
                    .equip_item(&command.combatant_id, &item, &self.scenario.modifiers)
            }
            EquipmentCommandKind::Unequip => {
                self.state
                    .unequip_item(&command.combatant_id, &item, &self.scenario.items)
            }
        };
        debug_assert!(
            mutated,
            "validated equipment command must mutate a combatant"
        );
        let next_equipment = self
            .state
            .equipment_for(&command.combatant_id)
            .expect("mutated combatant equipment readout");
        let previous_equipment = previous_equipment.expect("validated combatant equipment");
        let reason = match command.kind {
            EquipmentCommandKind::Equip => "Item equipped and grants applied.",
            EquipmentCommandKind::Unequip => "Item unequipped and grants removed.",
        }
        .to_string();
        self.equipment_transition_log
            .push(EquipmentTransitionEntry {
                sequence: self.equipment_transition_log.len() as u32,
                transition_kind: command.kind,
                combatant_id: command.combatant_id.clone(),
                item_id: item.id.clone(),
                equipment_slot: item.equipment_slot.clone(),
                granted_modifier_ids: item.granted_modifier_ids.clone(),
                granted_ability_ids: item.granted_ability_ids.clone(),
                granted_resource_ids: item
                    .granted_resource_pools
                    .iter()
                    .map(|pool| pool.id.clone())
                    .collect(),
                previous_equipment: previous_equipment.clone(),
                next_equipment: next_equipment.clone(),
                reason: reason.clone(),
            });

        EquipmentCommandReadout {
            command,
            accepted: true,
            decision_kind: EquipmentDecisionKind::Accepted,
            previous_equipment: Some(previous_equipment),
            next_equipment: Some(next_equipment),
            reason,
        }
    }

    fn validate_equipment_command(
        &self,
        command: &EquipmentCommandSpec,
        equipment: Option<&CombatantEquipmentReadout>,
    ) -> Option<(EquipmentDecisionKind, String)> {
        if self.lifecycle.phase == CombatLifecyclePhase::Ended {
            return Some((
                EquipmentDecisionKind::RejectedByLifecycle,
                "Equipment cannot change after combat ends.".to_string(),
            ));
        }
        let Some(equipment) = equipment else {
            return Some((
                EquipmentDecisionKind::RejectedByCombatant,
                "Equipment command combatant is not present.".to_string(),
            ));
        };
        let Some(item) = self.scenario.item_by_id(&command.item_id) else {
            return Some((
                EquipmentDecisionKind::RejectedByItem,
                "Equipment command item is not present in the catalog.".to_string(),
            ));
        };
        if !equipment.inventory_item_ids.contains(&item.id) {
            return Some((
                EquipmentDecisionKind::RejectedByOwnership,
                "Combatant does not own the requested item.".to_string(),
            ));
        }

        match command.kind {
            EquipmentCommandKind::Unequip => {
                if !equipment.equipped_item_ids.contains(&item.id) {
                    return Some((
                        EquipmentDecisionKind::RejectedByEquippedState,
                        "Item is not currently equipped.".to_string(),
                    ));
                }
            }
            EquipmentCommandKind::Equip => {
                if equipment.equipped_item_ids.contains(&item.id) {
                    return Some((
                        EquipmentDecisionKind::RejectedByEquippedState,
                        "Item is already equipped.".to_string(),
                    ));
                }
                if equipment.equipped_item_ids.iter().any(|equipped_id| {
                    self.scenario
                        .item_by_id(equipped_id)
                        .is_some_and(|equipped| equipped.equipment_slot == item.equipment_slot)
                }) {
                    return Some((
                        EquipmentDecisionKind::RejectedBySlotConflict,
                        format!(
                            "Equipment slot {} is already occupied.",
                            item.equipment_slot
                        ),
                    ));
                }
                let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
                let effective_stats =
                    effective_stats_for_combatant(&current_scenario, &command.combatant_id);
                for requirement in &item.requirements {
                    let met = effective_stats.as_ref().is_some_and(|readout| {
                        readout.stats.iter().any(|stat| {
                            stat.stat_id == requirement.stat_id
                                && stat.effective_value >= requirement.minimum
                        })
                    });
                    if !met {
                        return Some((
                            EquipmentDecisionKind::RejectedByRequirement,
                            format!(
                                "Combatant does not meet {} >= {} for item {}.",
                                requirement.stat_id, requirement.minimum, item.id
                            ),
                        ));
                    }
                }
                let resources = self.state.action_resources_for(&command.combatant_id);
                if item.granted_resource_pools.iter().any(|pool| {
                    resources.as_ref().is_some_and(|readout| {
                        readout
                            .resources
                            .iter()
                            .any(|resource| resource.resource_id == pool.id)
                    })
                }) {
                    return Some((
                        EquipmentDecisionKind::RejectedByResourceConflict,
                        "Item grants a resource pool id already present on the combatant."
                            .to_string(),
                    ));
                }
            }
        }
        None
    }
}
