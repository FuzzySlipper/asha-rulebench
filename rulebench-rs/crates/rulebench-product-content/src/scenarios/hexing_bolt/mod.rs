mod catalog;
mod fixture;
mod session;

pub use catalog::{content_validation_readouts, ruleset_catalog_readout, scenario_catalog_cases};
pub use fixture::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    hexing_bolt_scenario_package, rejected_target_fixture_receipt, turn_control_fixture_scenario,
};
pub use session::{
    combat_session_automatic_run_readouts, combat_session_automatic_run_replay_readouts,
    combat_session_control_history_readouts, combat_session_script_readouts,
    combat_session_transcripts,
};

pub fn registration() -> crate::ScenarioPackageRegistration {
    crate::ScenarioPackageRegistration::new(
        hexing_bolt_scenario_package(),
        crate::ScenarioPackageReadbackFactories {
            catalog_cases: scenario_catalog_cases,
            ruleset_catalog_readout,
            content_validation_readouts,
            session_transcripts: combat_session_transcripts,
            control_history_readouts: combat_session_control_history_readouts,
            script_readouts: combat_session_script_readouts,
            automatic_run_readouts: combat_session_automatic_run_readouts,
            automatic_run_replay_readouts: combat_session_automatic_run_replay_readouts,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_combat::{
        resolve_use_action, ActionResourceKind, ActionResourcePool, ActionResourceRefreshPolicy,
        CombatSessionIntentCommandSpec, CombatSessionState, DomainEvent, NamedNumber,
        ReactionCommandSpec, UseActionIntent, ASHA_RPG_AUTHORITY_SURFACE,
    };
    use rulebench_content::{
        fingerprint_content_pack_set, AuthoredActionAbilityGrantReceipt,
        AuthoredActionBindingReceipt, ContentFingerprint, ContentPackReference,
        ContentPackSetReference, AUTHORED_ACTION_BINDING_VERSION,
        AUTHORED_ACTION_CHECK_VOCABULARY_VERSION, AUTHORED_ACTION_DEFINITION_FINGERPRINT_ALGORITHM,
        CONTENT_PACK_FINGERPRINT_ALGORITHM, CONTENT_PACK_SET_FINGERPRINT_ALGORITHM,
    };

    #[test]
    fn hexing_bolt_package_owns_valid_product_data() {
        let package = hexing_bolt_scenario_package();

        assert!(package.validate().is_ok());
        assert_eq!(package.identity.id, "asha-rulebench.hexing-bolt");
        assert_eq!(package.scripts.len(), 1);
    }

    #[test]
    fn hexing_bolt_package_owns_deterministic_receipts_and_transcript_evidence() {
        let accepted_receipt = accepted_hexing_bolt_fixture_receipt();
        let rejected_receipt = rejected_target_fixture_receipt();

        assert!(accepted_receipt.accepted);
        assert!(!rejected_receipt.accepted);
        assert_eq!(scenario_catalog_cases().len(), 4);
        assert_eq!(combat_session_transcripts().len(), 1);
    }

    #[test]
    fn typescript_authored_representative_corpus_executes_only_through_rpg_authority() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].hit_points.current = 18;
        scenario.combatants[1].position.x = 3;
        scenario.combatants[0]
            .resource_pools
            .push(ActionResourcePool {
                id: "focus".to_string(),
                kind: ActionResourceKind::Charge,
                initial: 0,
                maximum: 2,
                refresh_policy: ActionResourceRefreshPolicy::Never,
            });
        for combatant in &mut scenario.combatants {
            combatant.defenses.push(NamedNumber {
                id: "resolve".to_string(),
                label: "Resolve".to_string(),
                value: 12,
            });
            combatant.stats.base_stats.push(NamedNumber {
                id: "power".to_string(),
                label: "Power".to_string(),
                value: 3,
            });
            combatant.stats.base_stats.push(NamedNumber {
                id: "focus".to_string(),
                label: "Focus".to_string(),
                value: 3,
            });
        }
        let mut second_raider = scenario.combatants[1].clone();
        second_raider.id = "entity-raider-two".to_string();
        second_raider.position.x = 5;
        scenario.combatants.push(second_raider);

        let cases = vec![
            (
                UseActionIntent::new("entity-adept", "action.anchor-lash", "entity-raider"),
                vec![17, 5],
                "damage",
            ),
            (
                UseActionIntent::new("entity-adept", "action.binding-spark", "entity-raider"),
                vec![2, 4],
                "modifier",
            ),
            (
                UseActionIntent::new("entity-adept", "action.rallying-mend", "entity-adept"),
                vec![3],
                "healing",
            ),
            (
                UseActionIntent::for_targets(
                    "entity-adept",
                    "action.shatterline-burst",
                    vec!["entity-raider".to_string(), "entity-raider-two".to_string()],
                ),
                vec![2, 3, 4, 3, 2, 2],
                "multi-target",
            ),
            (
                UseActionIntent::new("entity-adept", "action.tactical-shift", "entity-adept"),
                Vec::new(),
                "movement",
            ),
            (
                UseActionIntent::new(
                    "entity-adept",
                    "action.intercepting-strike",
                    "entity-raider",
                ),
                vec![17, 6],
                "branching attack",
            ),
        ];

        for (intent, rolls, label) in cases {
            let receipt = resolve_use_action(&scenario, intent, &rolls);
            assert!(receipt.accepted, "{label}: {:?}", receipt.rejection);
            assert_eq!(receipt.authority_surface, ASHA_RPG_AUTHORITY_SURFACE);
            assert!(receipt
                .trace
                .iter()
                .any(|entry| entry.message.starts_with("RPG_")));
        }

        let burst = resolve_use_action(
            &scenario,
            UseActionIntent::for_targets(
                "entity-adept",
                "action.shatterline-burst",
                vec!["entity-raider".to_string(), "entity-raider-two".to_string()],
            ),
            &[2, 3, 4, 3, 2, 2],
        );
        assert_eq!(burst.target_results.len(), 2);
        assert_eq!(
            burst
                .events
                .iter()
                .filter(|event| matches!(event, DomainEvent::DamageApplied { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn typescript_authored_reaction_metadata_pauses_and_resumes_kernel_resolution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        let root = ContentPackReference {
            id: "pack.typescript-authored-reaction".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: ContentFingerprint {
                algorithm: CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
                value: "typescript-authored-reaction".to_string(),
            },
        };
        let packs = vec![root.clone()];
        let content_pack_set = ContentPackSetReference {
            fingerprint: fingerprint_content_pack_set(&root, &packs),
            root,
            packs,
        };
        assert_eq!(
            content_pack_set.fingerprint.algorithm,
            CONTENT_PACK_SET_FINGERPRINT_ALGORITHM
        );
        scenario.content_pack_set = Some(content_pack_set.clone());
        scenario.authored_action_binding = Some(AuthoredActionBindingReceipt {
            binding_version: AUTHORED_ACTION_BINDING_VERSION.to_string(),
            content_pack_set,
            action_id: "hexing_bolt".to_string(),
            action_definition_fingerprint: ContentFingerprint {
                algorithm: AUTHORED_ACTION_DEFINITION_FINGERPRINT_ALGORITHM.to_string(),
                value: "typescript-rpg-source".to_string(),
            },
            ability_id: scenario.selected_action.ability_id.clone(),
            scenario_id: scenario.metadata.id.clone(),
            actor_id: "entity-adept".to_string(),
            grant: AuthoredActionAbilityGrantReceipt {
                actor_id: "entity-adept".to_string(),
                ability_id: scenario.selected_action.ability_id.clone(),
            },
            targeting_operation_vocabulary_version: rpg_ir::OperationPipelineV2::VOCABULARY_VERSION
                .to_string(),
            check_vocabulary_version: AUTHORED_ACTION_CHECK_VOCABULARY_VERSION.to_string(),
            effect_operation_vocabulary_version: rpg_ir::EffectOperationId::VOCABULARY_VERSION
                .to_string(),
        });

        let initial_vitality = scenario.combatants[1].hit_points.current;
        let mut session = CombatSessionState::new("typescript-reaction", scenario);
        let step = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "typescript-reaction-trigger",
            "TypeScript-authored reaction",
            "The SDK metadata opens product orchestration around a compiled RPG result.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        assert!(step.receipt.accepted);
        assert_eq!(step.receipt.authority_surface, ASHA_RPG_AUTHORITY_SURFACE);
        assert_eq!(
            step.state_after.combatants[1].hit_points.current, initial_vitality,
            "before-effect reaction must pause the compiled result"
        );

        let window = session
            .current_reaction_window()
            .cloned()
            .expect("TypeScript metadata opens the declared reaction window");
        assert_eq!(window.options.len(), 1);
        assert_eq!(window.options[0].reactor_id, "entity-raider");
        assert!(window.options[0].option_id.starts_with("reaction.brace."));

        let resumed =
            session.submit_reaction_command(ReactionCommandSpec::pass(window.id, "entity-raider"));
        assert!(resumed.resumed_pending_resolution);
        assert!(session.current_reaction_window().is_none());
        assert!(
            session.snapshot().current_state.combatants[1]
                .hit_points
                .current
                < initial_vitality
        );
    }
}
