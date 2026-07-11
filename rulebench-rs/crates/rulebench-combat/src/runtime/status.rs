//! Combat session status, audit, and snapshot readbacks.

use super::*;
use std::collections::BTreeMap;

impl CombatSessionState {
    pub fn combat_log(&self) -> &[CombatLogEntry] {
        &self.combat_log
    }

    pub fn audit_log(&self) -> &[CommandAuditEntry] {
        &self.audit_log
    }

    pub fn action_usage_log(&self) -> &[ActionUsageEntry] {
        &self.action_usage_log
    }

    pub fn action_resource_transition_log(&self) -> &[ActionResourceTransitionEntry] {
        &self.action_resource_transition_log
    }

    pub fn equipment_transition_log(&self) -> &[EquipmentTransitionEntry] {
        &self.equipment_transition_log
    }

    pub fn equipment_ledger(&self) -> EquipmentLedgerReadout {
        self.state.equipment_ledger()
    }

    pub fn class_build_ledger(&self) -> ClassBuildLedgerReadout {
        self.state.class_build_ledger()
    }

    pub fn modifier_duration_expiration_log(&self) -> &[ModifierDurationExpirationEntry] {
        &self.modifier_duration_expiration_log
    }

    pub fn control_history(&self) -> &[CombatControlHistoryEntry] {
        &self.control_history
    }

    pub fn turn_transition_log(&self) -> &[TurnTransitionEntry] {
        &self.turn_transition_log
    }

    pub fn lifecycle_transition_log(&self) -> &[LifecycleTransitionEntry] {
        &self.lifecycle_transition_log
    }

    pub fn current_turn_action_usage(&self) -> ActionUsageSummary {
        current_turn_action_usage(&self.turn_order, &self.action_usage_log)
    }

    pub fn action_resource_ledger(&self) -> ActionResourceLedgerReadout {
        self.state.action_resource_ledger()
    }

    pub fn combatant_vitality(&self) -> CombatantVitalitySummary {
        let current_state = self.state.project("Current session state.");
        combatant_vitality_summary(&current_state)
    }

    pub fn combat_end_condition(&self) -> CombatEndConditionReadout {
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        combat_end_condition_readout(&current_scenario)
    }

    pub fn current_actor_options(&self) -> CurrentActorOptionSummary {
        let current_state = self.state.project("Current session state.");
        current_actor_option_summary(
            &self.lifecycle,
            &self.turn_order,
            &self.scenario,
            &current_state,
            &self.state.action_resource_ledger(),
            &self.state.equipment_ledger(),
            self.current_reaction_window().is_some(),
        )
    }

    pub fn current_actor_command_candidates(&self) -> CommandCandidateSummary {
        let current_state = self.state.project("Current session state.");
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        let current_actor_options = current_actor_option_summary(
            &self.lifecycle,
            &self.turn_order,
            &current_scenario,
            &current_state,
            &self.state.action_resource_ledger(),
            &self.state.equipment_ledger(),
            self.current_reaction_window().is_some(),
        );

        current_actor_command_candidates(
            &self.lifecycle,
            &current_scenario,
            &self.state.action_resource_ledger(),
            &self.state.equipment_ledger(),
            current_actor_options,
        )
    }

    pub fn preflight_command(&self, intent: UseActionIntent) -> CommandPreflightReadout {
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());
        command_preflight_readout(
            &self.lifecycle,
            self.turn_order.current_actor_id.clone(),
            &current_scenario,
            &self.state.action_resource_ledger(),
            &self.state.equipment_ledger(),
            self.current_reaction_window().is_some(),
            intent,
        )
    }

    pub fn snapshot(&self) -> CombatSessionSnapshot {
        let current_state = self.state.project("Current session state.");
        let current_state_fingerprint = fingerprint_projection(&current_state);
        let current_scenario = self.state.apply_to_scenario(self.scenario.clone());

        CombatSessionSnapshot {
            session_id: self.session_id.clone(),
            content_pack_set: self.scenario.content_pack_set.clone(),
            next_step_index: self.next_step_index,
            lifecycle: self.lifecycle.clone(),
            lifecycle_transition_log: self.lifecycle_transition_log.clone(),
            turn_order: self.turn_order.clone(),
            combat_log: self.combat_log.clone(),
            audit_log: self.audit_log.clone(),
            action_usage_log: self.action_usage_log.clone(),
            action_resource_transition_log: self.action_resource_transition_log.clone(),
            equipment_transition_log: self.equipment_transition_log.clone(),
            reaction_window_lifecycle_log: self.reaction_window_lifecycle_log.clone(),
            reaction_audit_log: self.reaction_audit_log.clone(),
            current_reaction_window: self.current_reaction_window().cloned(),
            modifier_duration_expiration_log: self.modifier_duration_expiration_log.clone(),
            turn_transition_log: self.turn_transition_log.clone(),
            action_resource_ledger: self.state.action_resource_ledger(),
            equipment_ledger: self.state.equipment_ledger(),
            class_build_ledger: self.state.class_build_ledger(),
            current_turn_action_usage: self.current_turn_action_usage(),
            combatant_vitality: combatant_vitality_summary(&current_state),
            combat_end_condition: combat_end_condition_readout(&current_scenario),
            finalization: self.finalization.clone(),
            current_actor_options: current_actor_option_summary(
                &self.lifecycle,
                &self.turn_order,
                &current_scenario,
                &current_state,
                &self.state.action_resource_ledger(),
                &self.state.equipment_ledger(),
                self.current_reaction_window().is_some(),
            ),
            current_state,
            current_state_fingerprint,
        }
    }
}

fn current_turn_action_usage(
    turn_order: &CombatTurnOrder,
    action_usage_log: &[ActionUsageEntry],
) -> ActionUsageSummary {
    let current_actor_id = turn_order.current_actor_id.clone();
    let current_actor_matches = |entry: &ActionUsageEntry| {
        current_actor_id
            .as_deref()
            .is_some_and(|actor_id| entry.actor_id == actor_id)
    };
    let current_turn_entries = action_usage_log
        .iter()
        .filter(|entry| entry.round_number == turn_order.round_number)
        .filter(|entry| entry.turn_index == turn_order.current_turn_index)
        .filter(|entry| current_actor_matches(entry));

    let mut used_action_ids = Vec::new();
    let mut used_ability_ids = Vec::new();
    for entry in current_turn_entries {
        used_action_ids.push(entry.action_id.clone());
        used_ability_ids.push(entry.ability_id.clone());
    }

    ActionUsageSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        current_actor_id,
        used_action_count: used_action_ids.len() as u32,
        used_action_ids,
        used_ability_ids,
    }
}

fn combatant_vitality_summary(projection: &ScenarioProjection) -> CombatantVitalitySummary {
    let mut combatants = Vec::new();
    let mut active_combatant_ids = Vec::new();
    let mut defeated_combatant_ids = Vec::new();

    for combatant in &projection.combatants {
        let defeated = combatant.hit_points.current <= 0;
        let entry = CombatantVitalityEntry {
            combatant_id: combatant.id.clone(),
            current_hit_points: combatant.hit_points.current,
            max_hit_points: combatant.hit_points.max,
            defeated,
        };

        if defeated {
            defeated_combatant_ids.push(combatant.id.clone());
        } else {
            active_combatant_ids.push(combatant.id.clone());
        }
        combatants.push(entry);
    }

    CombatantVitalitySummary {
        active_count: active_combatant_ids.len() as u32,
        defeated_count: defeated_combatant_ids.len() as u32,
        combatants,
        active_combatant_ids,
        defeated_combatant_ids,
    }
}

fn combat_end_condition_readout(scenario: &RulebenchScenario) -> CombatEndConditionReadout {
    let mut active_ally_count = 0;
    let mut active_enemy_count = 0;
    let mut defeated_ally_count = 0;
    let mut defeated_enemy_count = 0;
    let mut side_totals = BTreeMap::<String, u32>::new();
    let mut side_active = BTreeMap::<String, u32>::new();

    for combatant in &scenario.combatants {
        let defeated = combatant.hit_points.current <= 0;
        *side_totals.entry(combatant.side_id.clone()).or_default() += 1;
        if !defeated {
            *side_active.entry(combatant.side_id.clone()).or_default() += 1;
        }
        match (combatant.team, defeated) {
            (Team::Ally, false) => active_ally_count += 1,
            (Team::Ally, true) => defeated_ally_count += 1,
            (Team::Enemy, false) => active_enemy_count += 1,
            (Team::Enemy, true) => defeated_enemy_count += 1,
        }
    }

    let policy = scenario
        .selected_ruleset()
        .and_then(|ruleset| ruleset.validate_modules().ok())
        .and_then(|registry| registry.turn_control().cloned())
        .map(|configuration| configuration.combat_end_policy)
        .unwrap_or(rulebench_ruleset::CombatEndPolicy::LastSideStanding);
    let active_sides = side_totals
        .keys()
        .filter(|side_id| side_active.get(*side_id).copied().unwrap_or_default() > 0)
        .cloned()
        .collect::<Vec<_>>();
    let defeated_sides = side_totals
        .keys()
        .filter(|side_id| side_active.get(*side_id).copied().unwrap_or_default() == 0)
        .cloned()
        .collect::<Vec<_>>();

    let (condition_kind, outcome_kind, winning_sides) = match &policy {
        rulebench_ruleset::CombatEndPolicy::ExplicitOnly => (
            CombatEndConditionKind::ExplicitOnly,
            CombatOutcomeKind::Ongoing,
            Vec::new(),
        ),
        rulebench_ruleset::CombatEndPolicy::LastSideStanding => {
            let condition_kind = match active_sides.as_slice() {
                [] => CombatEndConditionKind::NoActiveCombatants,
                [side_id] if side_id == "ally" => CombatEndConditionKind::NoActiveEnemies,
                [side_id] if side_id == "enemy" => CombatEndConditionKind::NoActiveAllies,
                [_] => CombatEndConditionKind::LastSideStanding,
                _ => CombatEndConditionKind::Ongoing,
            };
            let outcome_kind = match active_sides.len() {
                0 => CombatOutcomeKind::Draw,
                1 => CombatOutcomeKind::Victory,
                _ => CombatOutcomeKind::Ongoing,
            };
            let winning_sides = if outcome_kind == CombatOutcomeKind::Victory {
                active_sides.clone()
            } else {
                Vec::new()
            };
            (condition_kind, outcome_kind, winning_sides)
        }
        rulebench_ruleset::CombatEndPolicy::ObjectiveSideVictory { side_id } => {
            let objective_active = side_active.get(side_id).copied().unwrap_or_default();
            let opposing_active = active_sides
                .iter()
                .filter(|active_side_id| *active_side_id != side_id)
                .count();
            let outcome_kind = if objective_active == 0 && opposing_active == 0 {
                CombatOutcomeKind::Draw
            } else if objective_active > 0 && opposing_active == 0 {
                CombatOutcomeKind::Victory
            } else if objective_active == 0 && opposing_active > 0 {
                CombatOutcomeKind::Defeat
            } else {
                CombatOutcomeKind::Ongoing
            };
            let condition_kind = match outcome_kind {
                CombatOutcomeKind::Victory => CombatEndConditionKind::ObjectiveSideVictory,
                CombatOutcomeKind::Defeat => CombatEndConditionKind::ObjectiveSideDefeated,
                CombatOutcomeKind::Draw => CombatEndConditionKind::NoActiveCombatants,
                CombatOutcomeKind::Ongoing | CombatOutcomeKind::ExplicitEnd => {
                    CombatEndConditionKind::Ongoing
                }
            };
            let winning_sides = match outcome_kind {
                CombatOutcomeKind::Victory => vec![side_id.clone()],
                CombatOutcomeKind::Defeat => active_sides.clone(),
                CombatOutcomeKind::Ongoing
                | CombatOutcomeKind::Draw
                | CombatOutcomeKind::ExplicitEnd => Vec::new(),
            };
            (condition_kind, outcome_kind, winning_sides)
        }
    };
    let combat_should_end = outcome_kind != CombatOutcomeKind::Ongoing;
    let reason = combat_end_condition_reason(&policy, condition_kind, outcome_kind);

    CombatEndConditionReadout {
        policy,
        combat_should_end,
        condition_kind,
        outcome_kind,
        active_sides,
        defeated_sides,
        winning_sides,
        active_ally_count,
        active_enemy_count,
        defeated_ally_count,
        defeated_enemy_count,
        reason,
    }
}

fn combat_end_condition_reason(
    policy: &rulebench_ruleset::CombatEndPolicy,
    kind: CombatEndConditionKind,
    outcome: CombatOutcomeKind,
) -> String {
    if policy == &rulebench_ruleset::CombatEndPolicy::ExplicitOnly {
        return "Combat continues until an explicit end command under the configured policy."
            .to_string();
    }
    let base = match kind {
        CombatEndConditionKind::Ongoing => {
            "Combat can continue because multiple configured sides have active combatants."
        }
        CombatEndConditionKind::NoActiveEnemies => {
            "Combat should end because no active enemies remain."
        }
        CombatEndConditionKind::NoActiveAllies => {
            "Combat should end because no active allies remain."
        }
        CombatEndConditionKind::NoActiveCombatants => {
            "Combat should end because no active combatants remain."
        }
        CombatEndConditionKind::ExplicitOnly => {
            "Combat continues until an explicit end command under the configured policy."
        }
        CombatEndConditionKind::ExplicitEnd => {
            "Combat should end because authority received an explicit end command."
        }
        CombatEndConditionKind::LastSideStanding => {
            "Combat should end because one configured side remains active."
        }
        CombatEndConditionKind::ObjectiveSideVictory => {
            "Combat should end because the configured objective side is the only active side."
        }
        CombatEndConditionKind::ObjectiveSideDefeated => {
            "Combat should end because the configured objective side has been defeated."
        }
    };
    if matches!(
        policy,
        rulebench_ruleset::CombatEndPolicy::ObjectiveSideVictory { .. }
    ) {
        format!("{base} Configured objective outcome: {}.", outcome.code())
    } else {
        base.to_string()
    }
}

fn current_actor_option_summary(
    lifecycle: &CombatLifecycle,
    turn_order: &CombatTurnOrder,
    scenario: &RulebenchScenario,
    projection: &ScenarioProjection,
    action_resources: &ActionResourceLedgerReadout,
    equipment: &EquipmentLedgerReadout,
    reaction_window_open: bool,
) -> CurrentActorOptionSummary {
    let current_actor_id = turn_order.current_actor_id.clone();
    let current_actor_defeated = current_actor_id
        .as_deref()
        .and_then(|actor_id| projected_combatant_by_id(projection, actor_id))
        .is_some_and(|actor| actor.hit_points.current <= 0);

    if lifecycle.phase == CombatLifecyclePhase::Ended {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::CombatEnded,
            Vec::new(),
        );
    }

    if reaction_window_open {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::ReactionWindowOpen,
            Vec::new(),
        );
    }

    let Some(actor_id) = current_actor_id.as_deref() else {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::NoCurrentActor,
            Vec::new(),
        );
    };

    if current_actor_defeated {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::CurrentActorDefeated,
            Vec::new(),
        );
    }

    let actions = scenario
        .actions
        .iter()
        .filter(|action| action.actor_id == actor_id)
        .map(|action| {
            current_actor_action_option(action, projection, action_resources, equipment, actor_id)
        })
        .collect::<Vec<_>>();

    if actions.is_empty() {
        return unavailable_current_actor_options(
            lifecycle,
            turn_order,
            current_actor_id,
            current_actor_defeated,
            CurrentActorOptionsUnavailableReason::NoMatchingActions,
            actions,
        );
    }

    let available = actions
        .iter()
        .any(|action| action.available && !action.target_options.is_empty());
    let unavailable_reason = if available {
        None
    } else if actions.iter().any(|action| action.available) {
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets)
    } else {
        Some(CurrentActorOptionsUnavailableReason::NoAvailableResources)
    };

    CurrentActorOptionSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        lifecycle_phase: lifecycle.phase,
        current_actor_id,
        current_actor_defeated,
        available,
        unavailable_reason,
        actions,
    }
}

fn current_actor_command_candidates(
    lifecycle: &CombatLifecycle,
    scenario: &RulebenchScenario,
    action_resources: &ActionResourceLedgerReadout,
    equipment: &EquipmentLedgerReadout,
    options: CurrentActorOptionSummary,
) -> CommandCandidateSummary {
    let candidates = if options.available {
        current_actor_id_command_candidates(
            lifecycle,
            scenario,
            action_resources,
            equipment,
            &options,
        )
    } else {
        Vec::new()
    };

    CommandCandidateSummary {
        round_number: options.round_number,
        turn_index: options.turn_index,
        lifecycle_phase: options.lifecycle_phase,
        current_actor_id: options.current_actor_id,
        current_actor_defeated: options.current_actor_defeated,
        available: !candidates.is_empty(),
        unavailable_reason: options.unavailable_reason,
        candidates,
    }
}

fn current_actor_id_command_candidates(
    lifecycle: &CombatLifecycle,
    scenario: &RulebenchScenario,
    action_resources: &ActionResourceLedgerReadout,
    equipment: &EquipmentLedgerReadout,
    options: &CurrentActorOptionSummary,
) -> Vec<CommandCandidateEntry> {
    let Some(actor_id) = options.current_actor_id.as_deref() else {
        return Vec::new();
    };

    options
        .actions
        .iter()
        .filter(|action| action.available)
        .flat_map(|action| {
            action.target_options.iter().map(move |target| {
                let intent = UseActionIntent::new(
                    actor_id,
                    action.action_id.clone(),
                    target.target_id.clone(),
                );
                let preflight = command_preflight_readout(
                    lifecycle,
                    options.current_actor_id.clone(),
                    scenario,
                    action_resources,
                    equipment,
                    false,
                    intent.clone(),
                );

                CommandCandidateEntry {
                    intent,
                    action_id: action.action_id.clone(),
                    ability_id: action.ability_id.clone(),
                    target_id: target.target_id.clone(),
                    target_name: target.target_name.clone(),
                    target_current_hit_points: target.current_hit_points,
                    target_max_hit_points: target.max_hit_points,
                    accepted: preflight.accepted,
                    decision_kind: preflight.decision_kind,
                    rejection: preflight.rejection,
                    target_legality: preflight.target_legality,
                    reason: preflight.reason,
                }
            })
        })
        .collect()
}

fn unavailable_current_actor_options(
    lifecycle: &CombatLifecycle,
    turn_order: &CombatTurnOrder,
    current_actor_id: Option<String>,
    current_actor_defeated: bool,
    reason: CurrentActorOptionsUnavailableReason,
    actions: Vec<CurrentActorActionOption>,
) -> CurrentActorOptionSummary {
    CurrentActorOptionSummary {
        round_number: turn_order.round_number,
        turn_index: turn_order.current_turn_index,
        lifecycle_phase: lifecycle.phase,
        current_actor_id,
        current_actor_defeated,
        available: false,
        unavailable_reason: Some(reason),
        actions,
    }
}

fn current_actor_action_option(
    action: &ActionDefinition,
    projection: &ScenarioProjection,
    action_resources: &ActionResourceLedgerReadout,
    equipment: &EquipmentLedgerReadout,
    actor_id: &str,
) -> CurrentActorActionOption {
    let ability_available = equipment
        .combatants
        .iter()
        .find(|combatant| combatant.combatant_id == actor_id)
        .is_some_and(|combatant| combatant.available_ability_ids.contains(&action.ability_id));
    let availability = if ability_available {
        action_resource_costs_available(action_resources, actor_id, &action.resource_costs)
    } else {
        Err((
            None,
            "Actor does not currently have the action ability.".to_string(),
        ))
    };
    let (available, unavailable_reason) = match availability {
        Ok(_) => (true, None),
        Err((_, reason)) => (false, Some(reason)),
    };
    let resource_states = action_resources
        .combatants
        .iter()
        .find(|combatant| combatant.combatant_id == actor_id)
        .map(|combatant| {
            action
                .resource_costs
                .iter()
                .filter_map(|cost| {
                    combatant
                        .resources
                        .iter()
                        .find(|resource| resource.resource_id == cost.resource_id)
                        .cloned()
                })
                .collect()
        })
        .unwrap_or_default();
    let target_mode = ActionTargetMode::from(action.targeting.target_kind);
    let target_options = action
        .targeting
        .visible_target_ids
        .iter()
        .filter_map(|target_id| projected_combatant_by_id(projection, target_id))
        .filter(|target| target.hit_points.current > 0)
        .map(|target| CurrentActorTargetOption {
            target_id: target.id.clone(),
            target_name: target.name.clone(),
            current_hit_points: target.hit_points.current,
            max_hit_points: target.hit_points.max,
            reason: "Target is legal for the current authoritative state.".to_string(),
        })
        .collect();
    let destination_options = if target_mode == ActionTargetMode::Cell {
        projection
            .board
            .cells
            .iter()
            .filter(|cell| !cell.blocks_movement && cell.occupant_ids.is_empty())
            .map(|cell| CurrentActorCellOption {
                position: cell.position,
                reason: "Cell is in bounds, unblocked, and unoccupied.".to_string(),
            })
            .collect()
    } else {
        Vec::new()
    };

    CurrentActorActionOption {
        action_id: action.id.clone(),
        ability_id: action.ability_id.clone(),
        action_name: action.name.clone(),
        available,
        unavailable_reason,
        resource_costs: action.resource_costs.clone(),
        resource_states,
        target_mode,
        target_options,
        destination_options,
    }
}

fn projected_combatant_by_id<'a>(
    projection: &'a ScenarioProjection,
    combatant_id: &str,
) -> Option<&'a FinalCombatantState> {
    projection
        .combatants
        .iter()
        .find(|combatant| combatant.id == combatant_id)
}
