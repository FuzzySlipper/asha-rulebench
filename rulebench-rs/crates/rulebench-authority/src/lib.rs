//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! rejections fail closed, accepted facts are represented as DomainEvent-shaped
//! records, and trace/readout values explain what happened. It does not claim to
//! be upstream ASHA or a complete combat resolver.

#![forbid(unsafe_code)]

mod catalog;
mod fixtures;
mod model;
mod resolver;
mod runtime;
mod session;
mod state;

pub use catalog::{resolve_catalog_scenario, scenario_catalog_cases, scenario_catalog_summaries};
pub use fixtures::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    rejected_target_fixture_receipt,
};
pub use model::*;
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{CombatSessionCommandSpec, CombatSessionState};
pub use session::{
    combat_session_summaries, combat_session_transcripts, resolve_combat_session_step,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_intent_shape_emits_one_domain_event() {
        let intent = UseActionIntent::new(
            "combatant.hexwright",
            "action.hexing_bolt",
            "combatant.marauder",
        );

        let receipt = validate_intent_shape(&intent);

        assert!(receipt.accepted);
        assert_eq!(receipt.authority_surface, AUTHORITY_SURFACE);
        assert_eq!(receipt.rejection, None);
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(receipt.trace.len(), 2);
        assert_eq!(receipt.trace[1].phase, TracePhase::Validation);
    }

    #[test]
    fn empty_actor_rejects_without_events() {
        let intent = UseActionIntent::new("", "action.hexing_bolt", "combatant.marauder");

        let receipt = validate_intent_shape(&intent);

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::EmptyActorId));
        assert!(receipt.events.is_empty());
        assert_eq!(RulebenchRejection::EmptyActorId.code(), "emptyActorId");
    }

    #[test]
    fn model_represents_current_accepted_hexing_bolt_fixture() {
        let scenario = hexing_bolt_fixture_scenario();
        let receipt = accepted_hexing_bolt_fixture_receipt();

        assert_eq!(scenario.metadata.id, "two-combatant-hexing-bolt");
        assert_eq!(scenario.grid.width, 6);
        assert_eq!(scenario.combatants.len(), 2);
        assert!(receipt.accepted);
        assert_eq!(receipt.events.len(), 4);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
        assert_eq!(
            receipt.damage.as_ref().map(|damage| damage.after.current),
            Some(9)
        );
        assert_eq!(
            receipt
                .modifier
                .as_ref()
                .map(|modifier| modifier.modifier_id.as_str()),
            Some("rattled")
        );
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].conditions.as_slice()),
            Some(&["rattled".to_string()][..])
        );
    }

    #[test]
    fn scenario_carries_hexing_bolt_action_catalog_entry() {
        let scenario = hexing_bolt_fixture_scenario();

        assert_eq!(scenario.actions.len(), 1);
        assert_eq!(scenario.actions[0].id, "hexing_bolt");
        assert_eq!(scenario.actions[0], scenario.selected_action);
        assert_eq!(
            scenario
                .action_by_id("hexing_bolt")
                .map(|action| action.name.as_str()),
            Some("Hexing Bolt")
        );
    }

    #[test]
    fn scenario_action_catalog_rejects_unknown_action_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.action_by_id("not_hexing_bolt").is_none());
    }

    #[test]
    fn scenario_carries_combatant_stat_blocks() {
        let scenario = hexing_bolt_fixture_scenario();
        let adept = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-adept")
            .expect("fixture has adept");
        let raider = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("fixture has raider");

        assert_eq!(adept.stat_by_id("mind").map(|stat| stat.value), Some(4));
        assert_eq!(
            adept.stat_by_id("initiative").map(|stat| stat.value),
            Some(3)
        );
        assert_eq!(raider.stat_by_id("body").map(|stat| stat.value), Some(3));
    }

    #[test]
    fn combatant_stat_lookup_rejects_unknown_stat() {
        let scenario = hexing_bolt_fixture_scenario();
        let adept = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-adept")
            .expect("fixture has adept");

        assert!(adept.stat_by_id("spell_slots").is_none());
    }

    #[test]
    fn scenario_carries_hexing_bolt_hit_operations() {
        let scenario = hexing_bolt_fixture_scenario();
        let action = scenario
            .action_by_id("hexing_bolt")
            .expect("fixture has hexing bolt");

        let damage = action.hit.damage_operation().expect("damage operation");
        let modifier = action.hit.modifier_operation().expect("modifier operation");

        assert_eq!(action.hit.operations.len(), 2);
        assert_eq!(damage.damage_bonus, 4);
        assert_eq!(damage.damage_type, "psychic");
        assert_eq!(modifier.modifier_id, "rattled");
        assert_eq!(modifier.modifier_label, "rattled");
        assert_eq!(modifier.modifier_duration, "until end of next turn");
    }

    #[test]
    fn hit_effect_operation_lookup_rejects_missing_operations() {
        let scenario = hexing_bolt_fixture_scenario();
        let action = scenario
            .action_by_id("hexing_bolt")
            .expect("fixture has hexing bolt");
        let mut hit = action.hit.clone();
        hit.operations.clear();

        assert!(hit.damage_operation().is_none());
        assert!(hit.modifier_operation().is_none());
    }

    #[test]
    fn resolver_accepts_hexing_bolt_hit_from_deterministic_roll_stream() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(receipt.rejection, None);
        assert_eq!(receipt.events.len(), 4);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(9)
        );
    }

    #[test]
    fn resolver_uses_actor_stat_for_attack_modifier() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].attack.modifier = 99;

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.modifier),
            Some(4)
        );
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
    }

    #[test]
    fn resolver_rejects_missing_attack_modifier_stat_source() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].attack.modifier_stat_id = "missing_mind".to_string();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
    }

    #[test]
    fn resolver_uses_hit_operations_for_damage_and_modifier() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].hit.damage_bonus = 99;
        scenario.actions[0].hit.damage_type = "wrong".to_string();
        scenario.actions[0].hit.modifier_id = "wrong".to_string();
        scenario.actions[0].hit.modifier_label = "wrong".to_string();
        scenario.actions[0].hit.modifier_duration = "wrong".to_string();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
        assert_eq!(
            receipt
                .damage
                .as_ref()
                .map(|damage| damage.damage_type.as_str()),
            Some("psychic")
        );
        assert_eq!(
            receipt
                .modifier
                .as_ref()
                .map(|modifier| modifier.label.as_str()),
            Some("rattled")
        );
        assert_eq!(
            receipt
                .modifier
                .as_ref()
                .map(|modifier| modifier.duration.as_str()),
            Some("until end of next turn")
        );
    }

    #[test]
    fn resolver_rejects_missing_hit_operations() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].hit.operations.clear();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
        assert!(receipt.damage.is_none());
        assert!(receipt.modifier.is_none());
    }

    #[test]
    fn resolver_uses_action_catalog_for_action_lookup() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_action.id = "display_only_hexing_bolt".to_string();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(receipt.rejection, None);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(receipt.events.len(), 4);
    }

    #[test]
    fn resolver_rejects_non_hostile_target_without_events_or_damage() {
        let receipt = rejected_target_fixture_receipt();

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
        assert!(receipt.damage.is_none());
        assert_eq!(
            receipt
                .target_legality
                .as_ref()
                .map(|target| target.accepted),
            Some(false)
        );
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(18)
        );
    }

    #[test]
    fn resolver_rejects_missing_attack_roll_without_events() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[],
        );

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::MissingAttackRoll)
        );
        assert!(receipt.events.is_empty());
        assert!(receipt.damage.is_none());
    }

    #[test]
    fn resolver_rejects_invalid_action_without_events() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "not_hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
    }

    #[test]
    fn resolver_rejects_action_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions.clear();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
    }

    #[test]
    fn catalog_enumerates_stable_scenario_summaries() {
        let summaries = scenario_catalog_summaries();

        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "hexing-bolt-hit",
                "hexing-bolt-miss",
                "hexing-bolt-self-target-rejected"
            ]
        );
        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.outcome_class.code())
                .collect::<Vec<_>>(),
            vec!["acceptedHit", "acceptedMiss", "rejectedTargetLegality"]
        );
    }

    #[test]
    fn catalog_resolves_accepted_hit_case() {
        let resolution = resolve_catalog_scenario("hexing-bolt-hit").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::AcceptedHit
        );
        assert_eq!(resolution.scenario.metadata.id, "hexing-bolt-hit");
        assert!(resolution.receipt.accepted);
        assert_eq!(
            resolution
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(resolution.receipt.events.len(), 4);
    }

    #[test]
    fn catalog_resolves_accepted_miss_case() {
        let resolution = resolve_catalog_scenario("hexing-bolt-miss").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::AcceptedMiss
        );
        assert!(resolution.receipt.accepted);
        assert_eq!(
            resolution
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert!(resolution.receipt.damage.is_none());
        assert!(resolution.receipt.modifier.is_none());
        assert_eq!(resolution.receipt.events.len(), 2);
        assert_eq!(
            resolution
                .receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(18)
        );
    }

    #[test]
    fn catalog_resolves_rejected_target_legality_case() {
        let resolution =
            resolve_catalog_scenario("hexing-bolt-self-target-rejected").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::RejectedTargetLegality
        );
        assert!(!resolution.receipt.accepted);
        assert_eq!(
            resolution.receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(resolution.receipt.events.is_empty());
        assert_eq!(
            resolution
                .receipt
                .target_legality
                .as_ref()
                .map(|target| target.reason.as_str()),
            Some("Target is not hostile.")
        );
    }

    #[test]
    fn catalog_rejects_unknown_scenario_id() {
        let error = resolve_catalog_scenario("not-a-scenario").expect_err("unknown id fails");

        assert_eq!(error, ScenarioCatalogError::UnknownScenarioId);
    }

    #[test]
    fn combat_session_enumerates_stable_summary_and_steps() {
        let summaries = combat_session_summaries();

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, "hexing-bolt-opening-exchange");
        assert_eq!(
            summaries[0]
                .steps
                .iter()
                .map(|step| step.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "adept-hexing-bolt-hit",
                "adept-hexing-bolt-miss",
                "adept-hexing-bolt-self-target-rejected"
            ]
        );
        assert_eq!(
            summaries[0]
                .steps
                .iter()
                .map(|step| step.log_index)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn combat_session_first_step_records_accepted_hit() {
        let readout =
            resolve_combat_session_step("hexing-bolt-opening-exchange", "adept-hexing-bolt-hit")
                .expect("step exists");

        assert_eq!(readout.step.index, 0);
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(readout.command.roll_stream, vec![17, 5]);
        assert!(readout.receipt.accepted);
        assert_eq!(readout.receipt.events.len(), 4);
        assert_eq!(readout.combat_log.len(), 1);
        assert_eq!(readout.combat_log[0].log_index, 1);
        assert_eq!(
            readout.combat_log[0].event_types,
            vec![
                "ActionUsed".to_string(),
                "AttackRolled".to_string(),
                "DamageApplied".to_string(),
                "ModifierApplied".to_string()
            ]
        );
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 18);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_session_later_miss_preserves_prior_authority_state() {
        let readout =
            resolve_combat_session_step("hexing-bolt-opening-exchange", "adept-hexing-bolt-miss")
                .expect("step exists");

        assert_eq!(readout.step.index, 1);
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert!(readout.receipt.accepted);
        assert_eq!(
            readout
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert_eq!(readout.receipt.events.len(), 2);
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_session_rejected_step_preserves_prior_authority_state_without_events() {
        let readout = resolve_combat_session_step(
            "hexing-bolt-opening-exchange",
            "adept-hexing-bolt-self-target-rejected",
        )
        .expect("step exists");

        assert_eq!(readout.step.index, 2);
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::RejectedTargetLegality
        );
        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(readout.receipt.events.is_empty());
        assert!(readout.combat_log[0].event_types.is_empty());
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_state_projects_initial_scenario_facts() {
        let scenario = hexing_bolt_fixture_scenario();
        let state = crate::state::CombatState::from_scenario(&scenario);

        let projection = state.project("Initial combat state.");

        assert_eq!(projection.summary, "Initial combat state.");
        assert_eq!(projection.combatants.len(), 2);
        assert_eq!(projection.combatants[0].id, "entity-adept");
        assert_eq!(projection.combatants[0].hit_points.current, 24);
        assert_eq!(projection.combatants[1].id, "entity-raider");
        assert_eq!(projection.combatants[1].hit_points.current, 18);
        assert!(projection.combatants[1].conditions.is_empty());
    }

    #[test]
    fn combat_state_applies_hit_damage_and_condition() {
        let scenario = hexing_bolt_fixture_scenario();
        let receipt = accepted_hexing_bolt_fixture_receipt();
        let damage = receipt.damage.as_ref().expect("fixture hit has damage");
        let modifier = receipt.modifier.as_ref().expect("fixture hit has modifier");
        let mut state = crate::state::CombatState::from_scenario(&scenario);

        assert_eq!(state.active_modifiers_for("entity-raider"), Some(&[][..]));
        state.apply_hit(damage, modifier);
        state.apply_hit(damage, modifier);
        let projection = state.project("After accepted hit.");
        let active_modifiers = state
            .active_modifiers_for("entity-raider")
            .expect("raider state exists");

        assert_eq!(active_modifiers.len(), 1);
        assert_eq!(active_modifiers[0].modifier_id, "rattled");
        assert_eq!(active_modifiers[0].label, "rattled");
        assert_eq!(active_modifiers[0].duration, "until end of next turn");
        assert_eq!(active_modifiers[0].tenure, ModifierTenure::Temporary);
        assert_eq!(projection.combatants[1].hit_points.current, 9);
        assert_eq!(
            projection.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_state_preserves_prior_state_for_miss_noop_projection() {
        let first_step =
            resolve_combat_session_step("hexing-bolt-opening-exchange", "adept-hexing-bolt-hit")
                .expect("hit step exists");
        let state = crate::state::CombatState::from_projection(&first_step.state_after);

        let projection = state.project("Attack missed; no authority state changed.");

        assert_eq!(projection.combatants[1].hit_points.current, 9);
        assert_eq!(
            projection.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_state_preserves_prior_state_for_rejection_projection() {
        let miss_step =
            resolve_combat_session_step("hexing-bolt-opening-exchange", "adept-hexing-bolt-miss")
                .expect("miss step exists");
        let state = crate::state::CombatState::from_projection(&miss_step.state_after);

        let projection = state.project("No authority state changed; intent rejected.");

        assert_eq!(projection.combatants[1].hit_points.current, 9);
        assert_eq!(
            projection.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_state_applies_projected_state_back_to_scenario() {
        let scenario = hexing_bolt_fixture_scenario();
        let receipt = accepted_hexing_bolt_fixture_receipt();
        let projection = receipt.projection.as_ref().expect("fixture has projection");

        let next_scenario =
            crate::state::CombatState::from_projection(projection).apply_to_scenario(scenario);

        assert_eq!(next_scenario.combatants[1].hit_points.current, 9);
        assert_eq!(
            next_scenario.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn combat_state_applies_active_modifiers_back_to_scenario() {
        let scenario = hexing_bolt_fixture_scenario();
        let receipt = accepted_hexing_bolt_fixture_receipt();
        let damage = receipt.damage.as_ref().expect("fixture hit has damage");
        let modifier = receipt.modifier.as_ref().expect("fixture hit has modifier");
        let mut state = crate::state::CombatState::from_scenario(&scenario);

        state.apply_hit(damage, modifier);
        let next_scenario = state.apply_to_scenario(scenario);
        let raider = next_scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == "entity-raider")
            .expect("raider exists");

        assert_eq!(raider.active_modifiers.len(), 1);
        assert_eq!(raider.active_modifiers[0].modifier_id, "rattled");
        assert_eq!(raider.active_modifiers[0].label, "rattled");
        assert_eq!(
            raider.active_modifiers[0].duration,
            "until end of next turn"
        );
        assert_eq!(raider.active_modifiers[0].tenure, ModifierTenure::Temporary);
        assert_eq!(raider.conditions, vec!["rattled".to_string()]);
    }

    #[test]
    fn session_runtime_accepts_hit_command_and_advances_state() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ready);
        assert_eq!(session.lifecycle().started_at_step, None);
        assert_eq!(session.lifecycle().ended_at_step, None);

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(readout.session_id, "runtime-hexing-bolt");
        assert_eq!(readout.step.index, 0);
        assert_eq!(readout.step.log_index, 1);
        assert_eq!(readout.command.step_index, 0);
        assert!(readout.receipt.accepted);
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 18);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
        assert_eq!(readout.combat_log.len(), 1);
        assert_eq!(session.next_step_index(), 1);
        assert_eq!(session.combat_log().len(), 1);
        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::InProgress);
        assert_eq!(session.lifecycle().started_at_step, Some(0));
        assert_eq!(session.lifecycle().ended_at_step, None);
    }

    #[test]
    fn session_runtime_miss_preserves_prior_state() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-miss",
            "Runtime miss",
            "Adept misses Raider through the command runtime.",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ));

        assert_eq!(readout.step.index, 1);
        assert!(readout.receipt.accepted);
        assert_eq!(
            readout
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn session_runtime_rejection_preserves_prior_state() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-miss",
            "Runtime miss",
            "Adept misses Raider through the command runtime.",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ));

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-rejected",
            "Runtime rejected",
            "Adept targets themself through the command runtime.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));

        assert_eq!(readout.step.index, 2);
        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(readout.receipt.events.is_empty());
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.state_after.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn session_runtime_accumulates_log_entries_and_step_index() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        for (id, outcome_class, intent, rolls) in [
            (
                "runtime-hit",
                CommandOutcomeClass::AcceptedHit,
                UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                vec![17, 5],
            ),
            (
                "runtime-miss",
                CommandOutcomeClass::AcceptedMiss,
                UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                vec![2, 5],
            ),
            (
                "runtime-rejected",
                CommandOutcomeClass::RejectedTargetLegality,
                UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
                vec![17, 5],
            ),
        ] {
            session.submit_command(CombatSessionCommandSpec::new(
                id,
                id,
                id,
                outcome_class,
                intent,
                rolls,
            ));
        }

        assert_eq!(session.next_step_index(), 3);
        assert_eq!(session.lifecycle().started_at_step, Some(0));
        assert_eq!(
            session
                .combat_log()
                .iter()
                .map(|entry| entry.log_index)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            session
                .combat_log()
                .iter()
                .map(|entry| entry.event_types.len())
                .collect::<Vec<_>>(),
            vec![4, 2, 0]
        );
    }

    #[test]
    fn session_runtime_can_end_combat_lifecycle() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-miss",
            "Runtime miss",
            "Adept misses Raider through the command runtime.",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ));

        session.end_combat();

        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
        assert_eq!(session.lifecycle().started_at_step, Some(0));
        assert_eq!(session.lifecycle().ended_at_step, Some(2));
        assert_eq!(session.next_step_index(), 2);
    }

    #[test]
    fn combat_session_rejects_unknown_session_id() {
        let error = resolve_combat_session_step("not-a-session", "adept-hexing-bolt-hit")
            .expect_err("unknown session fails");

        assert_eq!(error, CombatSessionError::UnknownSessionId);
    }

    #[test]
    fn combat_session_rejects_unknown_step_id() {
        let error = resolve_combat_session_step("hexing-bolt-opening-exchange", "not-a-step")
            .expect_err("unknown step fails");

        assert_eq!(error, CombatSessionError::UnknownStepId);
    }
}
