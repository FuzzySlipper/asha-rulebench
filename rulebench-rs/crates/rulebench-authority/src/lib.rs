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
mod projection;
mod resolver;
mod session;
mod state;

pub use catalog::{resolve_catalog_scenario, scenario_catalog_cases, scenario_catalog_summaries};
pub use fixtures::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    rejected_target_fixture_receipt,
};
pub use model::*;
pub use resolver::{resolve_use_action, validate_intent_shape};
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
