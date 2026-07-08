//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! rejections fail closed, accepted facts are represented as DomainEvent-shaped
//! records, and trace/readout values explain what happened. It does not claim to
//! be upstream ASHA or a complete combat resolver.

#![forbid(unsafe_code)]

mod audit;
mod catalog;
mod content;
mod fixtures;
mod model;
mod modifiers;
mod resolver;
mod runtime;
mod session;
mod state;

pub use audit::{
    fingerprint_projected_state, fingerprint_projection, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use catalog::{
    content_validation_readouts, resolve_catalog_scenario, ruleset_catalog_readout,
    scenario_catalog_cases, scenario_catalog_summaries,
};
pub use content::{validate_scenario_content, validate_scenario_content_report};
pub use fixtures::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    rejected_target_fixture_receipt,
};
pub use model::*;
pub use modifiers::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
};
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
    CombatSessionCommandSpec, CombatSessionIntentCommandSpec, CombatSessionScriptCommandKind,
    CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
    CombatSessionState,
};
pub use session::{
    combat_session_control_history_readouts, combat_session_script_readouts,
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
        assert_eq!(scenario.rulesets.len(), 1);
        assert_eq!(
            scenario.selected_ruleset_id,
            "asha-rulebench.hexing-bolt.v0"
        );
        assert_eq!(
            scenario
                .selected_ruleset()
                .map(|ruleset| ruleset.id.as_str()),
            Some("asha-rulebench.hexing-bolt.v0")
        );
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

        assert!(validate_scenario_content(&scenario).is_empty());
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
    fn scenario_carries_ability_spell_catalog_and_action_reference() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(
            scenario
                .abilities
                .iter()
                .map(|ability| ability.id.as_str())
                .collect::<Vec<_>>(),
            vec!["ability.hexing-bolt"]
        );
        assert_eq!(
            scenario
                .ability_by_id("ability.hexing-bolt")
                .map(|ability| ability.kind),
            Some(AbilityDefinitionKind::Spell)
        );
        assert_eq!(
            scenario
                .ability_by_id("ability.hexing-bolt")
                .map(|ability| ability.kind.code()),
            Some("spell")
        );
        assert_eq!(
            scenario.selected_ability_id.as_deref(),
            Some("ability.hexing-bolt")
        );
        assert_eq!(
            scenario
                .action_by_id("hexing_bolt")
                .map(|action| action.ability_id.as_str()),
            Some("ability.hexing-bolt")
        );
    }

    #[test]
    fn scenario_ability_catalog_rejects_unknown_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.ability_by_id("ability.missing").is_none());
    }

    #[test]
    fn scenario_carries_entity_catalog_and_combatant_references() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(
            scenario
                .entities
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec!["entity.adept", "entity.raider"]
        );
        assert_eq!(
            scenario
                .entity_by_id("entity.adept")
                .map(|entity| entity.name.as_str()),
            Some("Adept")
        );
        assert_eq!(scenario.combatants[0].entity_id, "entity.adept");
        assert_eq!(scenario.combatants[1].entity_id, "entity.raider");
    }

    #[test]
    fn scenario_entity_catalog_rejects_unknown_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.entity_by_id("entity.missing").is_none());
    }

    #[test]
    fn scenario_carries_item_catalog_and_equipped_item_references() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(
            scenario
                .items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["item.hex-focus", "item.raider-mail"]
        );
        assert_eq!(
            scenario
                .item_by_id("item.hex-focus")
                .map(|item| item.name.as_str()),
            Some("Hex Focus")
        );
        assert_eq!(scenario.selected_item_id.as_deref(), Some("item.hex-focus"));
        assert_eq!(
            scenario.combatants[0].equipped_item_ids,
            vec!["item.hex-focus".to_string()]
        );
        assert_eq!(
            scenario.combatants[1].equipped_item_ids,
            vec!["item.raider-mail".to_string()]
        );
    }

    #[test]
    fn scenario_item_catalog_rejects_unknown_item_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.item_by_id("item.missing").is_none());
    }

    #[test]
    fn scenario_carries_class_catalog_and_stat_definitions() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(
            scenario
                .classes
                .iter()
                .map(|class| class.id.as_str())
                .collect::<Vec<_>>(),
            vec!["class.hex-adept", "class.raider"]
        );
        assert_eq!(
            scenario
                .class_by_id("class.hex-adept")
                .map(|class| class.name.as_str()),
            Some("Hex Adept")
        );
        assert_eq!(
            scenario.selected_class_id.as_deref(),
            Some("class.hex-adept")
        );
        assert_eq!(
            scenario.combatants[0].class_ids,
            vec!["class.hex-adept".to_string()]
        );
        assert_eq!(
            scenario.stat_definition_by_id("mind").map(|stat| stat.kind),
            Some(StatDefinitionKind::Base)
        );
        assert_eq!(
            scenario
                .stat_definition_by_id("initiative")
                .map(|stat| stat.kind.code()),
            Some("derived")
        );
    }

    #[test]
    fn scenario_class_and_stat_catalog_reject_unknown_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.class_by_id("class.missing").is_none());
        assert!(scenario.stat_definition_by_id("luck").is_none());
    }

    #[test]
    fn scenario_carries_modifier_catalog() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(validate_scenario_content(&scenario).is_empty());
        assert_eq!(
            scenario
                .modifiers
                .iter()
                .map(|modifier| modifier.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rattled", "battle-drilled"]
        );
        assert_eq!(
            scenario
                .modifier_by_id("rattled")
                .map(|modifier| modifier.label.as_str()),
            Some("rattled")
        );
        assert_eq!(
            scenario
                .modifier_by_id("rattled")
                .map(|modifier| modifier.default_tenure.code()),
            Some("temporary")
        );
        assert_eq!(
            scenario
                .modifier_by_id("rattled")
                .map(|modifier| modifier.stat_adjustments.as_slice()),
            Some(
                &[ModifierStatAdjustment {
                    stat_id: "mind".to_string(),
                    stat_label: "Mind".to_string(),
                    delta: -1,
                }][..]
            )
        );
        assert_eq!(
            scenario
                .modifier_by_id("battle-drilled")
                .map(|modifier| modifier.default_tenure.code()),
            Some("permanent")
        );
        assert_eq!(
            scenario
                .modifier_by_id("battle-drilled")
                .map(|modifier| modifier.stat_adjustments.as_slice()),
            Some(
                &[ModifierStatAdjustment {
                    stat_id: "initiative".to_string(),
                    stat_label: "Initiative".to_string(),
                    delta: 1,
                }][..]
            )
        );
    }

    #[test]
    fn scenario_modifier_catalog_rejects_unknown_lookup() {
        let scenario = hexing_bolt_fixture_scenario();

        assert!(scenario.modifier_by_id("stunned").is_none());
    }

    #[test]
    fn active_modifier_stat_adjustment_readout_is_empty_without_active_modifiers() {
        let scenario = hexing_bolt_fixture_scenario();

        let readout = active_modifier_stat_adjustments_for_combatant(&scenario, "entity-raider")
            .expect("fixture has raider");

        assert_eq!(readout.combatant_id, "entity-raider");
        assert!(readout.contributions.is_empty());
    }

    #[test]
    fn active_modifier_stat_adjustment_readout_resolves_rattled_contribution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "rattled",
                "rattled",
                "until end of next turn",
            ));

        let readout = active_modifier_stat_adjustments_for_combatant(&scenario, "entity-raider")
            .expect("fixture has raider");

        assert_eq!(readout.combatant_id, "entity-raider");
        assert_eq!(
            readout.contributions,
            vec![ModifierStatAdjustmentContribution {
                modifier_id: "rattled".to_string(),
                modifier_label: "rattled".to_string(),
                tenure: ModifierTenure::Temporary,
                stat_id: "mind".to_string(),
                stat_label: "Mind".to_string(),
                delta: -1,
            }]
        );
    }

    #[test]
    fn active_modifier_stat_adjustment_readout_rejects_missing_combatant() {
        let scenario = hexing_bolt_fixture_scenario();

        let readout = active_modifier_stat_adjustments_for_combatant(&scenario, "entity-missing");

        assert!(readout.is_none());
    }

    #[test]
    fn active_modifier_stat_adjustment_readout_preserves_permanent_tenure() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::permanent(
                "battle-drilled",
                "battle drilled",
            ));

        let readout = active_modifier_stat_adjustments_for_combatant(&scenario, "entity-adept")
            .expect("fixture has adept");

        assert_eq!(
            readout.contributions,
            vec![ModifierStatAdjustmentContribution {
                modifier_id: "battle-drilled".to_string(),
                modifier_label: "battle drilled".to_string(),
                tenure: ModifierTenure::Permanent,
                stat_id: "initiative".to_string(),
                stat_label: "Initiative".to_string(),
                delta: 1,
            }]
        );
        assert_eq!(
            readout.contributions[0].tenure.code(),
            ModifierTenure::Permanent.code()
        );
    }

    #[test]
    fn effective_stat_readout_lists_base_values_without_modifiers() {
        let scenario = hexing_bolt_fixture_scenario();

        let readout =
            effective_stats_for_combatant(&scenario, "entity-raider").expect("fixture has raider");

        assert_eq!(readout.combatant_id, "entity-raider");
        assert_eq!(readout.stats.len(), 3);

        let mind = readout
            .stats
            .iter()
            .find(|stat| stat.stat_id == "mind")
            .expect("raider has mind");
        assert_eq!(mind.stat_label, "Mind");
        assert_eq!(mind.base_value, 1);
        assert_eq!(mind.total_modifier_delta, 0);
        assert_eq!(mind.effective_value, 1);
        assert!(mind.contributions.is_empty());

        let initiative = readout
            .stats
            .iter()
            .find(|stat| stat.stat_id == "initiative")
            .expect("raider has initiative");
        assert_eq!(initiative.base_value, 1);
        assert_eq!(initiative.effective_value, 1);
    }

    #[test]
    fn effective_stat_readout_applies_temporary_modifier_contribution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "rattled",
                "rattled",
                "until end of next turn",
            ));

        let readout =
            effective_stats_for_combatant(&scenario, "entity-raider").expect("fixture has raider");
        let mind = readout
            .stats
            .iter()
            .find(|stat| stat.stat_id == "mind")
            .expect("raider has mind");

        assert_eq!(mind.base_value, 1);
        assert_eq!(mind.total_modifier_delta, -1);
        assert_eq!(mind.effective_value, 0);
        assert_eq!(
            mind.contributions,
            vec![ModifierStatAdjustmentContribution {
                modifier_id: "rattled".to_string(),
                modifier_label: "rattled".to_string(),
                tenure: ModifierTenure::Temporary,
                stat_id: "mind".to_string(),
                stat_label: "Mind".to_string(),
                delta: -1,
            }]
        );
    }

    #[test]
    fn effective_stat_readout_applies_permanent_modifier_contribution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::permanent(
                "battle-drilled",
                "battle drilled",
            ));

        let readout =
            effective_stats_for_combatant(&scenario, "entity-adept").expect("fixture has adept");
        let initiative = readout
            .stats
            .iter()
            .find(|stat| stat.stat_id == "initiative")
            .expect("adept has initiative");

        assert_eq!(initiative.base_value, 3);
        assert_eq!(initiative.total_modifier_delta, 1);
        assert_eq!(initiative.effective_value, 4);
        assert_eq!(
            initiative.contributions,
            vec![ModifierStatAdjustmentContribution {
                modifier_id: "battle-drilled".to_string(),
                modifier_label: "battle drilled".to_string(),
                tenure: ModifierTenure::Permanent,
                stat_id: "initiative".to_string(),
                stat_label: "Initiative".to_string(),
                delta: 1,
            }]
        );
    }

    #[test]
    fn effective_stat_readout_rejects_missing_combatant() {
        let scenario = hexing_bolt_fixture_scenario();

        let readout = effective_stats_for_combatant(&scenario, "entity-missing");

        assert!(readout.is_none());
    }

    #[test]
    fn effective_stat_readout_does_not_change_hexing_bolt_resolution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::permanent(
                "battle-drilled",
                "battle drilled",
            ));

        let readout =
            effective_stats_for_combatant(&scenario, "entity-adept").expect("fixture has adept");
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert_eq!(
            readout
                .stats
                .iter()
                .find(|stat| stat.stat_id == "initiative")
                .map(|stat| stat.effective_value),
            Some(4)
        );
        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.modifier),
            Some(4)
        );
        assert_eq!(
            receipt.damage.as_ref().map(|damage| damage.after.current),
            Some(9)
        );
    }

    #[test]
    fn active_modifier_stat_adjustment_readout_feeds_attack_modifier_resolution() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "rattled",
                "rattled",
                "until end of next turn",
            ));

        let readout = active_modifier_stat_adjustments_for_combatant(&scenario, "entity-adept")
            .expect("fixture has adept");
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert_eq!(
            readout
                .contributions
                .iter()
                .map(|contribution| contribution.delta)
                .collect::<Vec<_>>(),
            vec![-1]
        );
        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.modifier),
            Some(3)
        );
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(20)
        );
    }

    #[test]
    fn content_diagnostics_report_empty_ruleset_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.rulesets[0].id.clear();
        scenario.selected_ruleset_id.clear();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, ContentDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyRulesetId);
        assert_eq!(
            ContentDiagnosticCode::EmptyRulesetId.code(),
            "emptyRulesetId"
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_ruleset_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.rulesets.push(scenario.rulesets[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateRulesetId
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("asha-rulebench.hexing-bolt.v0".to_string())
        );
        assert_eq!(
            ContentDiagnosticCode::DuplicateRulesetId.code(),
            "duplicateRulesetId"
        );
    }

    #[test]
    fn content_diagnostics_report_selected_ruleset_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_ruleset_id = "asha-rulebench.missing.v0".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("asha-rulebench.missing.v0".to_string())
        );
        assert_eq!(
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog.code(),
            "selectedRulesetMissingFromCatalog"
        );
    }

    #[test]
    fn content_diagnostics_report_empty_action_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].id.clear();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ContentDiagnosticCode::EmptyActionId,
                ContentDiagnosticCode::SelectedActionMissingFromCatalog,
            ]
        );
    }

    #[test]
    fn content_diagnostics_report_empty_ability_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.abilities.push(AbilityDefinition {
            id: String::new(),
            name: "Nameless".to_string(),
            kind: AbilityDefinitionKind::Ability,
            summary: "Invalid ability fixture.".to_string(),
            tags: Vec::new(),
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, ContentDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyAbilityId);
        assert_eq!(
            ContentDiagnosticCode::EmptyAbilityId.code(),
            "emptyAbilityId"
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_ability_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.abilities.push(scenario.abilities[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateAbilityId
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("ability.hexing-bolt".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_selected_ability_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_ability_id = Some("ability.missing".to_string());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::SelectedAbilityMissingFromCatalog
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("ability.missing".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_empty_entity_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.entities.push(EntityDefinition {
            id: String::new(),
            name: "Nameless".to_string(),
            summary: "Invalid entity fixture.".to_string(),
            tags: Vec::new(),
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, ContentDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyEntityId);
        assert_eq!(ContentDiagnosticCode::EmptyEntityId.code(), "emptyEntityId");
    }

    #[test]
    fn content_diagnostics_report_duplicate_entity_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.entities.push(scenario.entities[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateEntityId
        );
        assert_eq!(diagnostics[0].content_id, Some("entity.adept".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_combatant_entity() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].entity_id = "entity.missing".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingCombatantEntity
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("entity.missing".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_missing_action_ability() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].ability_id = "ability.missing".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingActionAbility
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("ability.missing".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_empty_item_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.items[1].id.clear();
        scenario.combatants[1].equipped_item_ids.clear();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, ContentDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyItemId);
        assert_eq!(ContentDiagnosticCode::EmptyItemId.code(), "emptyItemId");
    }

    #[test]
    fn content_diagnostics_report_empty_class_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.classes.push(ClassDefinition {
            id: String::new(),
            name: "Nameless".to_string(),
            summary: "Invalid class fixture.".to_string(),
            tags: Vec::new(),
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, ContentDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyClassId);
        assert_eq!(ContentDiagnosticCode::EmptyClassId.code(), "emptyClassId");
    }

    #[test]
    fn content_diagnostics_report_duplicate_class_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.classes.push(scenario.classes[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::DuplicateClassId);
        assert_eq!(
            diagnostics[0].content_id,
            Some("class.hex-adept".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_selected_class_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_class_id = Some("class.missing".to_string());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::SelectedClassMissingFromCatalog
        );
        assert_eq!(diagnostics[0].content_id, Some("class.missing".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_combatant_class() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].class_ids = vec!["class.missing".to_string()];

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingCombatantClass
        );
        assert_eq!(diagnostics[0].content_id, Some("class.missing".to_string()));
    }

    #[test]
    fn content_diagnostics_report_empty_stat_definition_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.stat_definitions.push(StatDefinition {
            id: String::new(),
            label: "Empty".to_string(),
            kind: StatDefinitionKind::Base,
            summary: "Invalid stat definition fixture.".to_string(),
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::EmptyStatDefinitionId
        );
        assert_eq!(
            ContentDiagnosticCode::EmptyStatDefinitionId.code(),
            "emptyStatDefinitionId"
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_stat_definition_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario
            .stat_definitions
            .push(scenario.stat_definitions[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateStatDefinitionId
        );
        assert_eq!(diagnostics[0].content_id, Some("mind".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_combatant_stat_definition() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].stats.base_stats.push(NamedNumber {
            id: "luck".to_string(),
            label: "Luck".to_string(),
            value: 2,
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingCombatantStatDefinition
        );
        assert_eq!(diagnostics[0].content_id, Some("luck".to_string()));
    }

    #[test]
    fn content_diagnostics_report_empty_modifier_id() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.modifiers.push(ModifierDefinition {
            id: String::new(),
            label: "empty".to_string(),
            summary: "Invalid modifier fixture.".to_string(),
            default_tenure: ModifierTenure::Temporary,
            stat_adjustments: Vec::new(),
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::EmptyModifierId);
        assert_eq!(
            ContentDiagnosticCode::EmptyModifierId.code(),
            "emptyModifierId"
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_modifier_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.modifiers.push(scenario.modifiers[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateModifierId
        );
        assert_eq!(diagnostics[0].content_id, Some("rattled".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_modifier_stat_adjustment_target() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.modifiers[0].stat_adjustments[0].stat_id = "missing-mind".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingModifierStatAdjustmentTarget
        );
        assert_eq!(diagnostics[0].content_id, Some("missing-mind".to_string()));
        assert_eq!(
            ContentDiagnosticCode::MissingModifierStatAdjustmentTarget.code(),
            "missingModifierStatAdjustmentTarget"
        );
    }

    #[test]
    fn content_validation_report_counts_modifier_stat_adjustment_target_errors() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.modifiers[0].stat_adjustments[0].stat_id = "missing-mind".to_string();

        let report = validate_scenario_content_report(&scenario);

        assert!(!report.accepted);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 0);
        assert_eq!(
            report.diagnostics[0].code,
            ContentDiagnosticCode::MissingModifierStatAdjustmentTarget
        );
    }

    #[test]
    fn content_diagnostics_report_missing_hit_modifier_definition() {
        let mut scenario = hexing_bolt_fixture_scenario();
        if let HitEffectOperation::ApplyModifier(modifier) =
            &mut scenario.actions[0].hit.operations[1]
        {
            modifier.modifier_id = "missing-rattle".to_string();
        }

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingHitModifierDefinition
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("missing-rattle".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_missing_active_modifier_definition() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "missing-active",
                "missing active",
                "until reviewed",
            ));

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingActiveModifierDefinition
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("missing-active".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_item_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.items.push(scenario.items[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, ContentDiagnosticCode::DuplicateItemId);
        assert_eq!(
            diagnostics[0].content_id,
            Some("item.hex-focus".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_selected_item_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_item_id = Some("item.missing-focus".to_string());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::SelectedItemMissingFromCatalog
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("item.missing-focus".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_missing_equipped_item() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].equipped_item_ids = vec!["item.missing-focus".to_string()];

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingEquippedItem
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("item.missing-focus".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_duplicate_action_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions.push(scenario.actions[0].clone());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::DuplicateActionId
        );
        assert_eq!(diagnostics[0].content_id, Some("hexing_bolt".to_string()));
    }

    #[test]
    fn content_diagnostics_report_selected_action_missing_from_catalog() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_action.id = "unlisted_action".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::SelectedActionMissingFromCatalog
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("unlisted_action".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_missing_action_actor() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].actor_id = "entity-missing-actor".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingActionActor
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("entity-missing-actor".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_missing_action_target() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].target_ids = vec!["entity-missing-target".to_string()];
        scenario.actions[0].visible_target_ids = vec!["entity-missing-target".to_string()];

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingActionTarget
        );
        assert_eq!(
            diagnostics[0].content_id,
            Some("entity-missing-target".to_string())
        );
    }

    #[test]
    fn content_diagnostics_report_visible_target_outside_target_ids() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0]
            .visible_target_ids
            .push("entity-adept".to_string());

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::VisibleTargetOutsideTargetIds
        );
        assert_eq!(diagnostics[0].content_id, Some("entity-adept".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_attack_modifier_stat() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].attack.modifier_stat_id = "missing-mind".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingAttackModifierStat
        );
        assert_eq!(diagnostics[0].content_id, Some("missing-mind".to_string()));
    }

    #[test]
    fn content_diagnostics_report_missing_target_defense() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].attack.defense_id = "missing-nerve".to_string();

        let diagnostics = validate_scenario_content(&scenario);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            ContentDiagnosticCode::MissingTargetDefense
        );
        assert_eq!(diagnostics[0].content_id, Some("missing-nerve".to_string()));
    }

    #[test]
    fn content_validation_report_accepts_valid_fixture() {
        let scenario = hexing_bolt_fixture_scenario();

        let report = validate_scenario_content_report(&scenario);

        assert!(report.accepted);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 0);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn content_validation_report_counts_error_diagnostics() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.rulesets[0].id.clear();
        scenario.selected_ruleset_id.clear();
        scenario.entities.push(scenario.entities[0].clone());

        let report = validate_scenario_content_report(&scenario);

        assert!(!report.accepted);
        assert_eq!(report.error_count, 2);
        assert_eq!(report.warning_count, 0);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ContentDiagnosticCode::EmptyRulesetId,
                ContentDiagnosticCode::DuplicateEntityId,
            ]
        );
    }

    #[test]
    fn content_validation_report_preserves_diagnostic_details() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.selected_item_id = Some("item.missing-focus".to_string());

        let report = validate_scenario_content_report(&scenario);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            ContentDiagnosticCode::SelectedItemMissingFromCatalog
        );
        assert_eq!(
            report.diagnostics[0].content_id,
            Some("item.missing-focus".to_string())
        );
        assert!(report.diagnostics[0]
            .message
            .contains("not present in the scenario item catalog"));
    }

    #[test]
    fn content_validation_report_accepts_warning_only_diagnostics() {
        let report = ContentValidationReport::from_diagnostics(vec![ContentDiagnostic {
            severity: ContentDiagnosticSeverity::Warning,
            code: ContentDiagnosticCode::EmptyRulesetId,
            content_id: None,
            message: "Warning-only fixtures remain accepted until errors exist.".to_string(),
        }]);

        assert!(report.accepted);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(ContentDiagnosticSeverity::Warning.code(), "warning");
    }

    #[test]
    fn generated_content_validation_readouts_include_clean_and_invalid_reports() {
        let readouts = content_validation_readouts();

        let clean_readout = readouts
            .iter()
            .find(|readout| readout.scenario_id == "hexing-bolt-hit")
            .expect("clean catalog validation readout exists");
        assert!(clean_readout.report.accepted);
        assert!(clean_readout.report.diagnostics.is_empty());

        let invalid_ruleset = readouts
            .iter()
            .find(|readout| readout.scenario_id == "hexing-bolt-invalid-selected-ruleset")
            .expect("invalid selected ruleset validation readout exists");
        assert!(!invalid_ruleset.report.accepted);
        assert_eq!(invalid_ruleset.report.error_count, 1);
        assert_eq!(
            invalid_ruleset.report.diagnostics[0].code,
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog
        );
        assert_eq!(
            invalid_ruleset.report.diagnostics[0].content_id,
            Some("asha-rulebench.missing.v0".to_string())
        );

        let invalid_ability = readouts
            .iter()
            .find(|readout| readout.scenario_id == "hexing-bolt-invalid-selected-ability")
            .expect("invalid selected ability validation readout exists");
        assert!(!invalid_ability.report.accepted);
        assert_eq!(
            invalid_ability.report.diagnostics[0].code,
            ContentDiagnosticCode::SelectedAbilityMissingFromCatalog
        );
        assert_eq!(
            invalid_ability.report.diagnostics[0].content_id,
            Some("ability.missing".to_string())
        );

        let invalid_equipped_item = readouts
            .iter()
            .find(|readout| readout.scenario_id == "hexing-bolt-invalid-equipped-item")
            .expect("invalid equipped item validation readout exists");
        assert!(!invalid_equipped_item.report.accepted);
        assert_eq!(
            invalid_equipped_item.report.diagnostics[0].code,
            ContentDiagnosticCode::MissingEquippedItem
        );
        assert_eq!(
            invalid_equipped_item.report.diagnostics[0].content_id,
            Some("item.missing-focus".to_string())
        );
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
    fn item_equipment_content_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].conditions.as_slice()),
            Some(&["rattled".to_string()][..])
        );
    }

    #[test]
    fn class_stat_content_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

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
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
    }

    #[test]
    fn modifier_content_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
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
        assert_eq!(
            scenario.modifiers[0].stat_adjustments[0],
            ModifierStatAdjustment {
                stat_id: "mind".to_string(),
                stat_label: "Mind".to_string(),
                delta: -1,
            }
        );
    }

    #[test]
    fn ability_spell_content_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(receipt.rejection, None);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
    }

    #[test]
    fn entity_content_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
    }

    #[test]
    fn content_validation_report_does_not_change_hexing_bolt_resolution() {
        let scenario = hexing_bolt_fixture_scenario();

        let report = validate_scenario_content_report(&scenario);
        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(report.accepted);
        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].conditions.as_slice()),
            Some(&["rattled".to_string()][..])
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
    fn resolver_uses_effective_actor_stat_for_attack_modifier() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0]
            .active_modifiers
            .push(ActiveModifier::temporary(
                "rattled",
                "rattled",
                "until end of next turn",
            ));

        let receipt = resolve_use_action(
            &scenario,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[9, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.modifier),
            Some(3)
        );
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(12)
        );
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert!(receipt.damage.is_none());
        assert!(receipt.modifier.is_none());
        assert_eq!(receipt.events.len(), 2);
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
    fn session_runtime_intent_command_derives_accepted_hit_outcome() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::standard_action_available())
        );

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-hit",
            "Runtime derived hit",
            "Rust derives accepted hit outcome from receipt evidence.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert!(readout.receipt.accepted);
        assert_eq!(readout.step.outcome_class, CommandOutcomeClass::AcceptedHit);
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(
            readout.combat_log[0].outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_eq!(
            readout
                .audit_entry
                .preflight_decision_kind
                .map(CommandPreflightDecisionKind::code),
            Some("accepted")
        );
        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
    }

    #[test]
    fn session_runtime_intent_command_rejects_spent_action_resource() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-first-action",
            "Runtime first action",
            "Adept spends the standard action resource.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let before = session.snapshot();

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-spent-resource-rejected",
            "Runtime spent resource rejection",
            "Rust rejects repeated use-action after the standard action is spent.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let after = session.snapshot();

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByPreflight
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByActionResource)
        );
        assert_eq!(
            readout
                .audit_entry
                .preflight_decision_kind
                .map(CommandPreflightDecisionKind::code),
            Some("rejectedByActionResource")
        );
        assert!(readout.receipt.events.is_empty());
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
        assert_eq!(
            after.current_state_fingerprint,
            before.current_state_fingerprint
        );
        assert_eq!(after.action_usage_log.len(), before.action_usage_log.len());
        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
    }

    #[test]
    fn session_runtime_intent_command_resolver_rejection_does_not_spend_action_resource() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before = session.action_resource_ledger();

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-roll-missing-rejected",
            "Runtime missing roll rejection",
            "Rust rejects missing attack rolls without spending action resources.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![],
        ));
        let after = session.action_resource_ledger();

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::MissingAttackRoll)
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByResolver
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_eq!(before, after);
        assert!(session.action_usage_log().is_empty());
    }

    #[test]
    fn session_runtime_intent_command_derives_accepted_miss_outcome() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-miss",
            "Runtime derived miss",
            "Rust derives accepted miss outcome from receipt evidence.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ));

        assert!(readout.receipt.accepted);
        assert_eq!(
            readout
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_eq!(readout.audit_entry.event_count, 2);
    }

    #[test]
    fn session_runtime_intent_command_derives_target_legality_rejection_outcome() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-target-rejected",
            "Runtime derived target rejection",
            "Rust derives target legality rejection outcome from receipt evidence.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::RejectedTargetLegality
        );
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::RejectedTargetLegality
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByPreflight
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByTargetLegality)
        );
    }

    #[test]
    fn session_runtime_intent_command_records_shape_preflight_rejection() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_fingerprint = session.snapshot().current_state_fingerprint;

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-shape-rejected",
            "Runtime derived shape rejection",
            "Rust rejects malformed commands before roll resolution.",
            UseActionIntent::new("", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let after_snapshot = session.snapshot();

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::EmptyActorId)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::RejectedInvalidCommand
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByPreflight
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByShape)
        );
        assert_eq!(readout.audit_entry.event_count, 0);
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
        assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
        assert!(after_snapshot.action_usage_log.is_empty());
        assert_eq!(after_snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
    }

    #[test]
    fn session_runtime_intent_command_records_action_ownership_preflight_rejection() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        let before_fingerprint = session.snapshot().current_state_fingerprint;

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-action-owner-rejected",
            "Runtime derived action ownership rejection",
            "Rust rejects action ownership before roll resolution.",
            UseActionIntent::new("entity-raider", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));
        let after_snapshot = session.snapshot();

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByPreflight
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByActionOwnership)
        );
        assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
        assert!(after_snapshot.action_usage_log.is_empty());
    }

    #[test]
    fn session_runtime_intent_command_records_target_lookup_preflight_rejection() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_fingerprint = session.snapshot().current_state_fingerprint;

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-target-lookup-rejected",
            "Runtime derived target lookup rejection",
            "Rust rejects missing targets before roll resolution.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-missing"),
            vec![17, 5],
        ));
        let after_snapshot = session.snapshot();

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidTarget)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::RejectedTargetLegality
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByPreflight
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByTargetLookup)
        );
        assert_eq!(after_snapshot.current_state_fingerprint, before_fingerprint);
        assert!(after_snapshot.action_usage_log.is_empty());
    }

    #[test]
    fn session_runtime_intent_command_derives_lifecycle_invalid_outcome() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-ended-rejected",
            "Runtime derived ended rejection",
            "Rust derives invalid command outcome after combat end.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::RejectedInvalidCommand
        );
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::RejectedInvalidCommand
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByLifecycle
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByLifecycle)
        );
    }

    #[test]
    fn session_runtime_intent_command_derives_turn_order_invalid_outcome() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();

        let readout = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-derived-turn-rejected",
            "Runtime derived turn rejection",
            "Rust derives invalid command outcome for the wrong turn actor.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(
            readout.step.outcome_class,
            CommandOutcomeClass::RejectedInvalidCommand
        );
        assert_eq!(
            readout.command.outcome_class,
            CommandOutcomeClass::RejectedInvalidCommand
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByTurnOrder
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByTurnOrder)
        );
    }

    #[test]
    fn session_runtime_runs_mixed_combat_script_with_reviewable_step_readback() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.run_script(CombatSessionScriptSpec::new(
            "opening-control-script",
            "Opening control script",
            "Explicit control and intent commands run through Rust authority.",
            vec![
                CombatSessionScriptStepSpec::control(
                    "script-start",
                    "Start combat",
                    "Explicitly start combat before action resolution.",
                    CombatControlCommandSpec::explicit_start(),
                ),
                CombatSessionScriptStepSpec::control(
                    "script-repeat-start",
                    "Repeat start",
                    "Repeated start records rejected no-op control evidence.",
                    CombatControlCommandSpec::explicit_start(),
                ),
                CombatSessionScriptStepSpec::intent(
                    "script-hit-step",
                    "Adept hit",
                    "Adept uses Hexing Bolt against Raider.",
                    CombatSessionIntentCommandSpec::new(
                        "script-runtime-hit",
                        "Script runtime hit",
                        "Scripted accepted hit command.",
                        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                        vec![17, 5],
                    ),
                ),
                CombatSessionScriptStepSpec::control(
                    "script-advance-turn",
                    "Advance turn",
                    "Advance from Adept to Raider.",
                    CombatControlCommandSpec::advance_turn(),
                ),
                CombatSessionScriptStepSpec::intent(
                    "script-wrong-actor-step",
                    "Wrong actor attempt",
                    "Adept attempts another action on Raider turn.",
                    CombatSessionIntentCommandSpec::new(
                        "script-runtime-wrong-actor",
                        "Script runtime wrong actor",
                        "Scripted turn-order rejection.",
                        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                        vec![17, 5],
                    ),
                ),
                CombatSessionScriptStepSpec::control(
                    "script-end",
                    "End combat",
                    "Explicitly end combat after scripted commands.",
                    CombatControlCommandSpec::explicit_end(),
                ),
            ],
        ));

        assert_eq!(readout.session_id, "runtime-hexing-bolt");
        assert_eq!(readout.script_id, "opening-control-script");
        assert_eq!(readout.steps.len(), 6);
        assert_eq!(
            readout
                .steps
                .iter()
                .map(|step| step.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5]
        );

        let start = &readout.steps[0];
        assert_eq!(start.command_kind, CombatSessionScriptCommandKind::Control);
        assert_eq!(start.command_kind.code(), "control");
        assert!(start.accepted);
        assert_eq!(
            start.decision_kind,
            CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
        );
        assert_eq!(start.decision_kind.code(), "accepted");
        assert_eq!(start.control_history_sequence, Some(0));
        assert_eq!(start.command_audit_sequence, None);
        assert_eq!(start.runtime_step_id, None);
        assert_eq!(start.reason, "Combat explicitly started.");

        let repeated_start = &readout.steps[1];
        assert!(!repeated_start.accepted);
        assert_eq!(
            repeated_start.decision_kind,
            CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::RejectedNoop)
        );
        assert_eq!(repeated_start.decision_kind.code(), "rejectedNoop");
        assert_eq!(repeated_start.control_history_sequence, Some(1));
        assert_eq!(
            repeated_start.state_before_fingerprint,
            repeated_start.state_after_fingerprint
        );
        assert_eq!(repeated_start.reason, "Combat is already in progress.");

        let hit = &readout.steps[2];
        assert_eq!(hit.command_kind, CombatSessionScriptCommandKind::Intent);
        assert_eq!(hit.command_kind.code(), "intent");
        assert!(hit.accepted);
        assert_eq!(
            hit.decision_kind,
            CombatSessionScriptDecisionKind::Intent(CommandDecisionKind::AcceptedByResolver)
        );
        assert_eq!(hit.decision_kind.code(), "acceptedByResolver");
        assert_eq!(hit.runtime_step_id, Some("script-runtime-hit".to_string()));
        assert_eq!(hit.command_audit_sequence, Some(0));
        assert_eq!(hit.control_history_sequence, None);
        assert_ne!(hit.state_before_fingerprint, hit.state_after_fingerprint);
        assert_eq!(hit.reason, "Intent command accepted by resolver.");

        let advance_turn = &readout.steps[3];
        assert!(advance_turn.accepted);
        assert_eq!(
            advance_turn.decision_kind,
            CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
        );
        assert_eq!(advance_turn.control_history_sequence, Some(2));
        assert_eq!(
            advance_turn.state_before_fingerprint,
            advance_turn.state_after_fingerprint
        );

        let wrong_actor = &readout.steps[4];
        assert!(!wrong_actor.accepted);
        assert_eq!(
            wrong_actor.decision_kind,
            CombatSessionScriptDecisionKind::Intent(CommandDecisionKind::RejectedByTurnOrder)
        );
        assert_eq!(wrong_actor.decision_kind.code(), "rejectedByTurnOrder");
        assert_eq!(
            wrong_actor.runtime_step_id,
            Some("script-runtime-wrong-actor".to_string())
        );
        assert_eq!(wrong_actor.command_audit_sequence, Some(1));
        assert_eq!(
            wrong_actor.state_before_fingerprint,
            wrong_actor.state_after_fingerprint
        );
        assert_eq!(wrong_actor.reason, "Intent command rejected by turn order.");

        let end = &readout.steps[5];
        assert!(end.accepted);
        assert_eq!(
            end.decision_kind,
            CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
        );
        assert_eq!(end.control_history_sequence, Some(3));
        assert_eq!(end.reason, "Combat explicitly ended.");

        assert_eq!(session.control_history().len(), 4);
        assert_eq!(session.audit_log().len(), 2);
        assert_eq!(session.combat_log().len(), 2);
        assert_eq!(session.next_step_index(), 2);
        assert_eq!(
            session.audit_log()[1].preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByTurnOrder)
        );
        assert_eq!(
            readout.final_snapshot.lifecycle.phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(readout.final_snapshot.lifecycle.ended_at_step, Some(2));
        assert_eq!(readout.final_snapshot.audit_log.len(), 2);
    }

    #[test]
    fn session_runtime_empty_combat_script_is_read_only() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_script = session.snapshot();

        let readout = session.run_script(CombatSessionScriptSpec::new(
            "empty-script",
            "Empty script",
            "No commands are submitted.",
            Vec::new(),
        ));

        assert_eq!(readout.session_id, "runtime-hexing-bolt");
        assert_eq!(readout.script_id, "empty-script");
        assert!(readout.steps.is_empty());
        assert_eq!(readout.final_snapshot, before_script);
        assert!(session.combat_log().is_empty());
        assert!(session.audit_log().is_empty());
        assert!(session.control_history().is_empty());
    }

    #[test]
    fn session_runtime_script_selected_candidate_accepts_hit() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.run_script(CombatSessionScriptSpec::new(
            "selected-candidate-script",
            "Selected candidate script",
            "Script selects a Rust-visible command candidate for submission.",
            vec![CombatSessionScriptStepSpec::selected_candidate(
                "script-selected-hit-step",
                "Selected Hexing Bolt hit",
                "The current actor selected Hexing Bolt against Raider.",
                CombatSessionCandidateSelectionSpec::new(
                    "script-selected-runtime-hit",
                    "Script selected runtime hit",
                    "Selected-candidate script command resolves as a hit.",
                    "hexing_bolt",
                    "entity-raider",
                    vec![17, 5],
                ),
            )],
        ));

        assert_eq!(readout.steps.len(), 1);
        let step = &readout.steps[0];
        assert_eq!(
            step.command_kind,
            CombatSessionScriptCommandKind::SelectedCandidate
        );
        assert_eq!(step.command_kind.code(), "selectedCandidate");
        assert!(step.accepted);
        assert_eq!(
            step.decision_kind,
            CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(
                CommandDecisionKind::AcceptedByResolver
            )
        );
        assert_eq!(step.decision_kind.code(), "acceptedByResolver");
        assert_eq!(
            step.runtime_step_id,
            Some("script-selected-runtime-hit".to_string())
        );
        assert_eq!(step.command_audit_sequence, Some(0));
        assert_eq!(step.control_history_sequence, None);
        assert_ne!(step.state_before_fingerprint, step.state_after_fingerprint);
        assert_eq!(
            step.reason,
            "Selected candidate command accepted by resolver."
        );

        let snapshot = session.snapshot();
        assert_eq!(snapshot.next_step_index, 1);
        assert_eq!(snapshot.combat_log.len(), 1);
        assert_eq!(snapshot.audit_log.len(), 1);
        assert_eq!(snapshot.action_usage_log.len(), 1);
        assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 9);
        assert_eq!(
            snapshot.current_state.combatants[1].conditions,
            vec!["rattled"]
        );
        assert_eq!(readout.final_snapshot, snapshot);
    }

    #[test]
    fn session_runtime_script_selected_candidate_rejection_is_read_only() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        let before_script = session.snapshot();

        let readout = session.run_script(CombatSessionScriptSpec::new(
            "selected-candidate-rejected-script",
            "Selected candidate rejected script",
            "Script selects a candidate when Raider has no available action.",
            vec![CombatSessionScriptStepSpec::selected_candidate(
                "script-selected-unavailable-step",
                "Selected unavailable candidate",
                "The current actor has no matching command candidate.",
                CombatSessionCandidateSelectionSpec::new(
                    "script-selected-unavailable",
                    "Script selected unavailable",
                    "Raider has no command candidates in this fixture.",
                    "hexing_bolt",
                    "entity-raider",
                    vec![17, 5],
                ),
            )],
        ));

        assert_eq!(readout.steps.len(), 1);
        let step = &readout.steps[0];
        assert_eq!(
            step.command_kind,
            CombatSessionScriptCommandKind::SelectedCandidate
        );
        assert!(!step.accepted);
        assert_eq!(
            step.decision_kind,
            CombatSessionScriptDecisionKind::SelectedCandidateSelection(
                CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
            )
        );
        assert_eq!(step.decision_kind.code(), "rejectedByUnavailableCandidates");
        assert_eq!(step.runtime_step_id, None);
        assert_eq!(step.command_audit_sequence, None);
        assert_eq!(step.control_history_sequence, None);
        assert_eq!(step.state_before_fingerprint, step.state_after_fingerprint);
        assert_eq!(
            step.reason,
            "No command candidates are available because the current actor has no matching actions."
        );

        let after_script = session.snapshot();
        assert_eq!(after_script, before_script);
        assert_eq!(readout.final_snapshot, before_script);
        assert!(session.combat_log().is_empty());
        assert!(session.audit_log().is_empty());
        assert!(session.action_usage_log().is_empty());
        assert_eq!(session.turn_transition_log().len(), 1);
    }

    #[test]
    fn session_runtime_existing_command_spec_preserves_supplied_outcome_class() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-compat-ended-rejected",
            "Runtime compatibility ended rejection",
            "Existing transcript spec preserves caller-supplied outcome class.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert!(!readout.receipt.accepted);
        assert_eq!(readout.step.outcome_class, CommandOutcomeClass::AcceptedHit);
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByLifecycle
        );
        assert_eq!(readout.audit_entry.preflight_decision_kind, None);
    }

    #[test]
    fn session_runtime_records_accepted_hit_audit_entry() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(readout.audit_entry.id, "audit-runtime-hit");
        assert_eq!(readout.audit_entry.step_id, "runtime-hit");
        assert_eq!(readout.audit_entry.sequence, 0);
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::AcceptedHit
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert_eq!(
            readout.audit_entry.decision_kind.code(),
            "acceptedByResolver"
        );
        assert!(readout.audit_entry.accepted);
        assert_eq!(readout.audit_entry.rejection, None);
        assert_eq!(readout.audit_entry.event_count, 4);
        assert_eq!(
            readout.audit_entry.trace_count,
            readout.receipt.trace.len() as u32
        );
        assert_eq!(
            readout.audit_entry.state_before_fingerprint.algorithm,
            STATE_FINGERPRINT_ALGORITHM
        );
        assert_ne!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
        assert_eq!(session.audit_log(), &[readout.audit_entry]);
    }

    #[test]
    fn session_runtime_records_accepted_hit_action_usage() {
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

        assert_eq!(session.action_usage_log().len(), 1);
        let usage = &session.action_usage_log()[0];
        assert_eq!(usage.id, "action-usage-runtime-hit");
        assert_eq!(usage.step_id, "runtime-hit");
        assert_eq!(usage.step_index, 0);
        assert_eq!(usage.round_number, 1);
        assert_eq!(usage.turn_index, 0);
        assert_eq!(usage.actor_id, "entity-adept");
        assert_eq!(usage.action_id, "hexing_bolt");
        assert_eq!(usage.ability_id, "ability.hexing-bolt");
        assert_eq!(usage.target_id, "entity-raider");
        assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedHit);
    }

    #[test]
    fn session_runtime_current_turn_action_usage_is_empty_initially() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let summary = session.current_turn_action_usage();

        assert_eq!(summary.round_number, 1);
        assert_eq!(summary.turn_index, 0);
        assert_eq!(summary.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(summary.used_action_count, 0);
        assert!(summary.used_action_ids.is_empty());
        assert!(summary.used_ability_ids.is_empty());
    }

    #[test]
    fn session_runtime_current_actor_options_read_initial_action_and_target() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let options = session.current_actor_options();

        assert_eq!(options.round_number, 1);
        assert_eq!(options.turn_index, 0);
        assert_eq!(options.lifecycle_phase, CombatLifecyclePhase::Ready);
        assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
        assert!(!options.current_actor_defeated);
        assert!(options.available);
        assert_eq!(options.unavailable_reason, None);
        assert_eq!(options.actions.len(), 1);
        assert_eq!(options.actions[0].action_id, "hexing_bolt");
        assert_eq!(options.actions[0].ability_id, "ability.hexing-bolt");
        assert_eq!(options.actions[0].action_name, "Hexing Bolt");
        assert_eq!(options.actions[0].target_options.len(), 1);
        assert_eq!(
            options.actions[0].target_options[0].target_id,
            "entity-raider"
        );
        assert_eq!(options.actions[0].target_options[0].target_name, "Raider");
        assert_eq!(options.actions[0].target_options[0].current_hit_points, 18);
        assert_eq!(options.actions[0].target_options[0].max_hit_points, 18);
    }

    #[test]
    fn session_runtime_current_actor_options_snapshot_readback_uses_current_state() {
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

        let snapshot = session.snapshot();

        assert!(snapshot.current_actor_options.available);
        assert_eq!(
            snapshot.current_actor_options.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert_eq!(snapshot.current_actor_options.actions.len(), 1);
        assert_eq!(
            snapshot.current_actor_options.actions[0].target_options[0].target_id,
            "entity-raider"
        );
        assert_eq!(
            snapshot.current_actor_options.actions[0].target_options[0].current_hit_points,
            9
        );
    }

    #[test]
    fn session_runtime_current_actor_options_report_no_actions_after_turn_advance() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();

        let options = session.current_actor_options();

        assert!(!options.available);
        assert_eq!(options.current_actor_id, Some("entity-raider".to_string()));
        assert_eq!(
            options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
        );
        assert_eq!(
            options.unavailable_reason.map(|reason| reason.code()),
            Some("noMatchingActions")
        );
        assert!(options.actions.is_empty());
    }

    #[test]
    fn session_runtime_current_actor_options_report_ended_combat_unavailable() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();

        let options = session.current_actor_options();

        assert_eq!(options.lifecycle_phase, CombatLifecyclePhase::Ended);
        assert!(!options.available);
        assert_eq!(
            options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::CombatEnded)
        );
        assert_eq!(
            options.unavailable_reason.map(|reason| reason.code()),
            Some("combatEnded")
        );
        assert!(options.actions.is_empty());
    }

    #[test]
    fn session_runtime_current_actor_options_report_defeated_current_actor_unavailable() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].hit_points.current = 0;
        let session = CombatSessionState::new("runtime-defeated-actor", scenario);

        let options = session.current_actor_options();

        assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
        assert!(options.current_actor_defeated);
        assert!(!options.available);
        assert_eq!(
            options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::CurrentActorDefeated)
        );
        assert_eq!(
            options.unavailable_reason.map(|reason| reason.code()),
            Some("currentActorDefeated")
        );
        assert!(options.actions.is_empty());
    }

    #[test]
    fn session_runtime_current_actor_options_filter_defeated_visible_targets() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let session = CombatSessionState::new("runtime-defeated-target", scenario);

        let options = session.current_actor_options();

        assert_eq!(options.current_actor_id, Some("entity-adept".to_string()));
        assert!(!options.current_actor_defeated);
        assert!(!options.available);
        assert_eq!(
            options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets)
        );
        assert_eq!(
            options.unavailable_reason.map(|reason| reason.code()),
            Some("noVisibleActiveTargets")
        );
        assert_eq!(options.actions.len(), 1);
        assert_eq!(options.actions[0].action_id, "hexing_bolt");
        assert!(options.actions[0].target_options.is_empty());
    }

    #[test]
    fn session_runtime_command_candidates_read_initial_current_actor_intents() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let candidates = session.current_actor_command_candidates();

        assert!(candidates.available);
        assert_eq!(candidates.round_number, 1);
        assert_eq!(candidates.turn_index, 0);
        assert_eq!(candidates.lifecycle_phase, CombatLifecyclePhase::Ready);
        assert_eq!(
            candidates.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert!(!candidates.current_actor_defeated);
        assert_eq!(candidates.unavailable_reason, None);
        assert_eq!(candidates.candidates.len(), 1);

        let candidate = &candidates.candidates[0];
        assert_eq!(
            candidate.intent,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
        );
        assert_eq!(candidate.action_id, "hexing_bolt");
        assert_eq!(candidate.ability_id, "ability.hexing-bolt");
        assert_eq!(candidate.target_id, "entity-raider");
        assert_eq!(candidate.target_name, "Raider");
        assert_eq!(candidate.target_current_hit_points, 18);
        assert_eq!(candidate.target_max_hit_points, 18);
        assert!(candidate.accepted);
        assert_eq!(
            candidate.decision_kind,
            CommandPreflightDecisionKind::Accepted
        );
        assert_eq!(candidate.decision_kind.code(), "accepted");
        assert_eq!(candidate.rejection, None);
        assert_eq!(
            candidate
                .target_legality
                .as_ref()
                .map(|legality| legality.accepted),
            Some(true)
        );
        assert_eq!(
            candidate.reason,
            "Command is admissible before roll resolution."
        );
    }

    #[test]
    fn session_runtime_command_candidates_read_current_state_after_hit() {
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

        let candidates = session.current_actor_command_candidates();

        assert!(candidates.available);
        assert_eq!(candidates.candidates.len(), 1);
        assert_eq!(
            candidates.candidates[0].intent,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
        );
        assert_eq!(candidates.candidates[0].target_current_hit_points, 9);
        assert_eq!(candidates.candidates[0].target_max_hit_points, 18);
        assert!(!candidates.candidates[0].accepted);
        assert_eq!(
            candidates.candidates[0].decision_kind,
            CommandPreflightDecisionKind::RejectedByActionResource
        );
        assert_eq!(
            candidates.candidates[0].reason,
            "Actor has no available standard action resource."
        );
    }

    #[test]
    fn session_runtime_command_candidates_report_no_candidates_when_unavailable() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();

        let candidates = session.current_actor_command_candidates();

        assert!(!candidates.available);
        assert_eq!(
            candidates.current_actor_id,
            Some("entity-raider".to_string())
        );
        assert_eq!(
            candidates.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
        );
        assert!(candidates.candidates.is_empty());
    }

    #[test]
    fn session_runtime_command_candidates_report_ended_combat_unavailable() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();

        let candidates = session.current_actor_command_candidates();

        assert!(!candidates.available);
        assert_eq!(candidates.lifecycle_phase, CombatLifecyclePhase::Ended);
        assert_eq!(
            candidates.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::CombatEnded)
        );
        assert!(candidates.candidates.is_empty());
    }

    #[test]
    fn session_runtime_command_candidates_are_read_only() {
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
        let before_candidates = session.snapshot();

        let candidates = session.current_actor_command_candidates();
        let after_candidates = session.snapshot();

        assert!(candidates.available);
        assert_eq!(candidates.candidates.len(), 1);
        assert_eq!(after_candidates, before_candidates);
        assert_eq!(session.next_step_index(), 1);
        assert_eq!(session.combat_log().len(), 1);
        assert_eq!(session.audit_log().len(), 1);
        assert_eq!(session.action_usage_log().len(), 1);
        assert_eq!(session.turn_transition_log().len(), 0);
        assert_eq!(session.lifecycle_transition_log().len(), 1);
    }

    #[test]
    fn session_runtime_candidate_selection_plans_current_actor_command() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_plan = session.snapshot();

        let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "planned-hit",
            "Planned hit",
            "Caller selected the Hexing Bolt candidate.",
            "hexing_bolt",
            "entity-raider",
            vec![17, 5],
        ));
        let after_plan = session.snapshot();

        assert!(plan.accepted);
        assert_eq!(
            plan.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::Accepted
        );
        assert_eq!(plan.decision_kind.code(), "accepted");
        assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(plan.unavailable_reason, None);
        assert_eq!(
            plan.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_eq!(plan.rejection, None);
        assert_eq!(
            plan.reason,
            "Selected command candidate planned for deterministic submission."
        );

        let command = plan.command.as_ref().expect("accepted plan has command");
        assert_eq!(command.id, "planned-hit");
        assert_eq!(command.title, "Planned hit");
        assert_eq!(
            command.summary,
            "Caller selected the Hexing Bolt candidate."
        );
        assert_eq!(
            command.intent,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider")
        );
        assert_eq!(command.roll_stream, vec![17, 5]);
        assert_eq!(after_plan, before_plan);
    }

    #[test]
    fn session_runtime_candidate_selection_rejects_unavailable_candidates() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        let before_plan = session.snapshot();

        let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "planned-hit",
            "Planned hit",
            "Raider has no command candidates in this fixture.",
            "hexing_bolt",
            "entity-raider",
            vec![17, 5],
        ));
        let after_plan = session.snapshot();

        assert!(!plan.accepted);
        assert_eq!(
            plan.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
        );
        assert_eq!(plan.decision_kind.code(), "rejectedByUnavailableCandidates");
        assert_eq!(plan.current_actor_id, Some("entity-raider".to_string()));
        assert_eq!(
            plan.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::NoMatchingActions)
        );
        assert_eq!(
            plan.reason,
            "No command candidates are available because the current actor has no matching actions."
        );
        assert_eq!(plan.command, None);
        assert_eq!(after_plan, before_plan);
    }

    #[test]
    fn session_runtime_candidate_selection_rejects_missing_candidate() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_plan = session.snapshot();

        let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "planned-missing",
            "Planned missing",
            "Caller selected a target that is not in current candidates.",
            "hexing_bolt",
            "missing-target",
            vec![17, 5],
        ));
        let after_plan = session.snapshot();

        assert!(!plan.accepted);
        assert_eq!(
            plan.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate
        );
        assert_eq!(plan.decision_kind.code(), "rejectedByMissingCandidate");
        assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(plan.unavailable_reason, None);
        assert_eq!(plan.preflight_decision_kind, None);
        assert_eq!(plan.rejection, None);
        assert_eq!(
            plan.reason,
            "Selected command candidate is not available for the current actor."
        );
        assert_eq!(plan.command, None);
        assert_eq!(after_plan, before_plan);
    }

    #[test]
    fn session_runtime_candidate_selection_rejects_candidate_failed_by_preflight() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].target_ids = vec!["entity-adept".to_string()];
        scenario.actions[0].visible_target_ids = vec!["entity-adept".to_string()];
        scenario.selected_action = scenario.actions[0].clone();
        let session = CombatSessionState::new("runtime-self-target-candidate", scenario);
        let candidates = session.current_actor_command_candidates();

        assert!(candidates.available);
        assert_eq!(candidates.candidates.len(), 1);
        assert!(!candidates.candidates[0].accepted);
        assert_eq!(
            candidates.candidates[0].decision_kind,
            CommandPreflightDecisionKind::RejectedByTargetLegality
        );
        let before_plan = session.snapshot();

        let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "planned-self-target",
            "Planned self target",
            "Caller selected a visible but illegal self target.",
            "hexing_bolt",
            "entity-adept",
            vec![17, 5],
        ));
        let after_plan = session.snapshot();

        assert!(!plan.accepted);
        assert_eq!(
            plan.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight
        );
        assert_eq!(plan.decision_kind.code(), "rejectedByPreflight");
        assert_eq!(plan.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(
            plan.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::RejectedByTargetLegality)
        );
        assert_eq!(
            plan.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert_eq!(plan.command, None);
        assert_eq!(after_plan, before_plan);
    }

    #[test]
    fn session_runtime_candidate_selection_plan_can_be_submitted() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let plan = session.plan_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "planned-hit",
            "Planned hit",
            "Caller selected the Hexing Bolt candidate.",
            "hexing_bolt",
            "entity-raider",
            vec![17, 5],
        ));
        let command = plan.command.expect("accepted plan has command");

        let readout = session.submit_intent_command(command);

        assert!(readout.receipt.accepted);
        assert_eq!(readout.step.id, "planned-hit");
        assert_eq!(
            readout.command,
            CommandAttempt {
                step_id: "planned-hit".to_string(),
                step_index: 0,
                actor_id: "entity-adept".to_string(),
                action_id: "hexing_bolt".to_string(),
                target_id: "entity-raider".to_string(),
                roll_stream: vec![17, 5],
                outcome_class: CommandOutcomeClass::AcceptedHit,
            }
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert_eq!(
            readout.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_eq!(session.audit_log().len(), 1);
        assert_eq!(session.action_usage_log().len(), 1);
        assert_eq!(
            session.snapshot().current_state.combatants[1]
                .hit_points
                .current,
            9
        );
    }

    #[test]
    fn session_runtime_selected_candidate_submission_accepts_hit() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "selected-hit",
            "Selected hit",
            "Caller selected Hexing Bolt through the selected-candidate submission path.",
            "hexing_bolt",
            "entity-raider",
            vec![17, 5],
        ));

        assert!(execution.selection.accepted);
        assert_eq!(
            execution.selection.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::Accepted
        );
        assert_eq!(
            execution
                .selection
                .command
                .as_ref()
                .map(|command| command.intent.clone()),
            Some(UseActionIntent::new(
                "entity-adept",
                "hexing_bolt",
                "entity-raider"
            ))
        );
        let submitted_step = execution
            .submitted_step
            .as_ref()
            .expect("accepted selection submits command");
        assert!(submitted_step.receipt.accepted);
        assert_eq!(submitted_step.step.id, "selected-hit");
        assert_eq!(
            submitted_step.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert_eq!(
            submitted_step.audit_entry.preflight_decision_kind,
            Some(CommandPreflightDecisionKind::Accepted)
        );
        assert_ne!(
            submitted_step.audit_entry.state_before_fingerprint,
            submitted_step.audit_entry.state_after_fingerprint
        );

        let snapshot = session.snapshot();
        assert_eq!(snapshot.next_step_index, 1);
        assert_eq!(snapshot.combat_log.len(), 1);
        assert_eq!(snapshot.audit_log.len(), 1);
        assert_eq!(snapshot.action_usage_log.len(), 1);
        assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 9);
        assert_eq!(
            snapshot.current_state.combatants[1].conditions,
            vec!["rattled"]
        );
    }

    #[test]
    fn session_runtime_selected_candidate_submission_accepts_miss_noop() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "selected-miss",
            "Selected miss",
            "Caller selected Hexing Bolt with deterministic miss rolls.",
            "hexing_bolt",
            "entity-raider",
            vec![2, 5],
        ));

        assert!(execution.selection.accepted);
        let submitted_step = execution
            .submitted_step
            .as_ref()
            .expect("accepted selection submits command");
        assert!(submitted_step.receipt.accepted);
        assert_eq!(
            submitted_step.step.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert_eq!(
            submitted_step.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert_eq!(
            submitted_step.audit_entry.state_before_fingerprint,
            submitted_step.audit_entry.state_after_fingerprint
        );

        let snapshot = session.snapshot();
        assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 18);
        assert!(snapshot.current_state.combatants[1].conditions.is_empty());
        assert_eq!(snapshot.action_usage_log.len(), 1);
        assert_eq!(
            snapshot.action_usage_log[0].outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
    }

    #[test]
    fn session_runtime_selected_candidate_submission_rejected_plan_is_read_only() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        let before_execution = session.snapshot();

        let execution = session.submit_candidate_command(CombatSessionCandidateSelectionSpec::new(
            "selected-unavailable",
            "Selected unavailable",
            "Raider has no command candidates in this fixture.",
            "hexing_bolt",
            "entity-raider",
            vec![17, 5],
        ));
        let after_execution = session.snapshot();

        assert!(!execution.selection.accepted);
        assert_eq!(
            execution.selection.decision_kind,
            CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates
        );
        assert_eq!(execution.submitted_step, None);
        assert_eq!(after_execution, before_execution);
        assert!(session.combat_log().is_empty());
        assert!(session.audit_log().is_empty());
        assert!(session.action_usage_log().is_empty());
        assert_eq!(session.turn_transition_log().len(), 1);
    }

    #[test]
    fn session_runtime_command_preflight_accepts_current_actor_action_target_without_rolls() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));

        assert!(preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::Accepted
        );
        assert_eq!(preflight.decision_kind.code(), "accepted");
        assert_eq!(preflight.rejection, None);
        assert_eq!(preflight.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(
            preflight
                .target_legality
                .as_ref()
                .map(|legality| legality.accepted),
            Some(true)
        );
        assert_eq!(
            preflight
                .target_legality
                .as_ref()
                .map(|legality| legality.reason.as_str()),
            Some("Target is hostile, within range, and line of sight is clear.")
        );
        assert_eq!(
            preflight.reason,
            "Command is admissible before roll resolution."
        );
    }

    #[test]
    fn session_runtime_command_preflight_rejects_empty_shape() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let preflight =
            session.preflight_command(UseActionIntent::new("", "hexing_bolt", "entity-raider"));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByShape
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByShape");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::EmptyActorId));
        assert_eq!(preflight.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(preflight.target_legality, None);
    }

    #[test]
    fn session_runtime_command_preflight_rejects_ended_combat() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByLifecycle
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByLifecycle");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
        assert_eq!(preflight.reason, "Combat is already ended.");
    }

    #[test]
    fn session_runtime_command_preflight_rejects_wrong_turn_actor() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByTurnOrder
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByTurnOrder");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
        assert_eq!(
            preflight.current_actor_id,
            Some("entity-raider".to_string())
        );
    }

    #[test]
    fn session_runtime_command_preflight_rejects_invalid_actor_without_current_actor() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants.clear();
        let session = CombatSessionState::new("runtime-empty", scenario);

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByActorLookup
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByActorLookup");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidActor));
        assert_eq!(preflight.current_actor_id, None);
    }

    #[test]
    fn session_runtime_command_preflight_rejects_invalid_action() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "not_hexing_bolt",
            "entity-raider",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByActionLookup
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByActionLookup");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
    }

    #[test]
    fn session_runtime_command_preflight_rejects_action_actor_mismatch() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.actions[0].actor_id = "entity-raider".to_string();
        let session = CombatSessionState::new("runtime-action-mismatch", scenario);

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByActionOwnership
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByActionOwnership");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidAction));
    }

    #[test]
    fn session_runtime_command_preflight_rejects_invalid_target() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-missing",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByTargetLookup
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByTargetLookup");
        assert_eq!(preflight.rejection, Some(RulebenchRejection::InvalidTarget));
    }

    #[test]
    fn session_runtime_command_preflight_rejects_target_legality_failure() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-adept",
        ));

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByTargetLegality
        );
        assert_eq!(preflight.decision_kind.code(), "rejectedByTargetLegality");
        assert_eq!(
            preflight.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert_eq!(
            preflight
                .target_legality
                .as_ref()
                .map(|legality| legality.reason.as_str()),
            Some("Target is not hostile.")
        );
    }

    #[test]
    fn session_runtime_command_preflight_is_read_only() {
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
        let before_preflight = session.snapshot();

        let preflight = session.preflight_command(UseActionIntent::new(
            "entity-adept",
            "hexing_bolt",
            "entity-raider",
        ));
        let after_preflight = session.snapshot();

        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByActionResource
        );
        assert_eq!(
            preflight.action_resource,
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
        assert_eq!(after_preflight, before_preflight);
        assert_eq!(session.next_step_index(), 1);
        assert_eq!(session.combat_log().len(), 1);
        assert_eq!(session.audit_log().len(), 1);
        assert_eq!(session.action_usage_log().len(), 1);
        assert_eq!(session.turn_transition_log().len(), 0);
        assert_eq!(session.lifecycle_transition_log().len(), 1);
    }

    #[test]
    fn session_runtime_current_turn_action_usage_summarizes_accepted_hit() {
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
        let summary = session.current_turn_action_usage();

        assert_eq!(summary.round_number, 1);
        assert_eq!(summary.turn_index, 0);
        assert_eq!(summary.current_actor_id, Some("entity-adept".to_string()));
        assert_eq!(summary.used_action_count, 1);
        assert_eq!(summary.used_action_ids, vec!["hexing_bolt".to_string()]);
        assert_eq!(
            summary.used_ability_ids,
            vec!["ability.hexing-bolt".to_string()]
        );
    }

    #[test]
    fn session_runtime_vitality_summary_reads_initial_active_combatants() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let summary = session.combatant_vitality();

        assert_eq!(summary.combatants.len(), 2);
        assert_eq!(summary.active_count, 2);
        assert_eq!(summary.defeated_count, 0);
        assert_eq!(
            summary.active_combatant_ids,
            vec!["entity-adept".to_string(), "entity-raider".to_string()]
        );
        assert!(summary.defeated_combatant_ids.is_empty());
        assert_eq!(summary.combatants[0].combatant_id, "entity-adept");
        assert_eq!(summary.combatants[0].current_hit_points, 24);
        assert_eq!(summary.combatants[0].max_hit_points, 24);
        assert!(!summary.combatants[0].defeated);
        assert_eq!(summary.combatants[1].combatant_id, "entity-raider");
        assert_eq!(summary.combatants[1].current_hit_points, 18);
        assert_eq!(summary.combatants[1].max_hit_points, 18);
        assert!(!summary.combatants[1].defeated);
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
    fn session_runtime_vitality_summary_keeps_damaged_combatant_active_above_zero() {
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
        let summary = session.combatant_vitality();

        assert_eq!(summary.active_count, 2);
        assert_eq!(summary.defeated_count, 0);
        assert_eq!(summary.combatants[1].combatant_id, "entity-raider");
        assert_eq!(summary.combatants[1].current_hit_points, 9);
        assert_eq!(summary.combatants[1].max_hit_points, 18);
        assert!(!summary.combatants[1].defeated);
    }

    #[test]
    fn session_runtime_combat_end_condition_reads_ongoing_combat() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let readout = session.combat_end_condition();

        assert!(!readout.combat_should_end);
        assert_eq!(readout.condition_kind, CombatEndConditionKind::Ongoing);
        assert_eq!(readout.condition_kind.code(), "ongoing");
        assert_eq!(readout.active_ally_count, 1);
        assert_eq!(readout.active_enemy_count, 1);
        assert_eq!(readout.defeated_ally_count, 0);
        assert_eq!(readout.defeated_enemy_count, 0);
        assert_eq!(
            readout.reason,
            "Combat can continue because both sides have active combatants."
        );
        assert_eq!(session.snapshot().combat_end_condition, readout);
    }

    #[test]
    fn session_runtime_combat_end_condition_reports_no_active_enemies() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let session = CombatSessionState::new("runtime-no-active-enemies", scenario);

        let readout = session.combat_end_condition();

        assert!(readout.combat_should_end);
        assert_eq!(
            readout.condition_kind,
            CombatEndConditionKind::NoActiveEnemies
        );
        assert_eq!(readout.condition_kind.code(), "noActiveEnemies");
        assert_eq!(readout.active_ally_count, 1);
        assert_eq!(readout.active_enemy_count, 0);
        assert_eq!(readout.defeated_ally_count, 0);
        assert_eq!(readout.defeated_enemy_count, 1);
        assert_eq!(
            readout.reason,
            "Combat should end because no active enemies remain."
        );
    }

    #[test]
    fn session_runtime_combat_end_condition_reports_no_active_allies() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].hit_points.current = -1;
        let session = CombatSessionState::new("runtime-no-active-allies", scenario);

        let readout = session.combat_end_condition();

        assert!(readout.combat_should_end);
        assert_eq!(
            readout.condition_kind,
            CombatEndConditionKind::NoActiveAllies
        );
        assert_eq!(readout.condition_kind.code(), "noActiveAllies");
        assert_eq!(readout.active_ally_count, 0);
        assert_eq!(readout.active_enemy_count, 1);
        assert_eq!(readout.defeated_ally_count, 1);
        assert_eq!(readout.defeated_enemy_count, 0);
        assert_eq!(
            readout.reason,
            "Combat should end because no active allies remain."
        );
    }

    #[test]
    fn session_runtime_combat_end_condition_reports_no_active_combatants() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants.clear();
        let session = CombatSessionState::new("runtime-no-active-combatants", scenario);

        let readout = session.combat_end_condition();

        assert!(readout.combat_should_end);
        assert_eq!(
            readout.condition_kind,
            CombatEndConditionKind::NoActiveCombatants
        );
        assert_eq!(readout.condition_kind.code(), "noActiveCombatants");
        assert_eq!(readout.active_ally_count, 0);
        assert_eq!(readout.active_enemy_count, 0);
        assert_eq!(readout.defeated_ally_count, 0);
        assert_eq!(readout.defeated_enemy_count, 0);
        assert_eq!(
            readout.reason,
            "Combat should end because no active combatants remain."
        );
        assert_eq!(session.snapshot().combat_end_condition, readout);
    }

    #[test]
    fn session_runtime_records_miss_noop_audit_entry() {
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

        assert_eq!(readout.audit_entry.id, "audit-runtime-miss");
        assert_eq!(readout.audit_entry.sequence, 1);
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::AcceptedMiss
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert!(readout.audit_entry.accepted);
        assert_eq!(readout.audit_entry.rejection, None);
        assert_eq!(readout.audit_entry.event_count, 2);
        assert_eq!(
            readout.audit_entry.trace_count,
            readout.receipt.trace.len() as u32
        );
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
    }

    #[test]
    fn session_runtime_records_accepted_miss_action_usage() {
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

        assert_eq!(session.action_usage_log().len(), 2);
        let usage = &session.action_usage_log()[1];
        assert_eq!(usage.id, "action-usage-runtime-miss");
        assert_eq!(usage.step_id, "runtime-miss");
        assert_eq!(usage.step_index, 1);
        assert_eq!(usage.round_number, 1);
        assert_eq!(usage.turn_index, 0);
        assert_eq!(usage.actor_id, "entity-adept");
        assert_eq!(usage.action_id, "hexing_bolt");
        assert_eq!(usage.ability_id, "ability.hexing-bolt");
        assert_eq!(usage.target_id, "entity-raider");
        assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedMiss);
    }

    #[test]
    fn session_runtime_current_turn_action_usage_includes_accepted_miss() {
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

        let summary = session.current_turn_action_usage();

        assert_eq!(summary.used_action_count, 2);
        assert_eq!(
            summary.used_action_ids,
            vec!["hexing_bolt".to_string(), "hexing_bolt".to_string()]
        );
        assert_eq!(
            summary.used_ability_ids,
            vec![
                "ability.hexing-bolt".to_string(),
                "ability.hexing-bolt".to_string()
            ]
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
    fn session_runtime_records_rejected_command_audit_entry() {
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
            "runtime-rejected",
            "Runtime rejected",
            "Adept targets themself through the command runtime.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));

        assert_eq!(readout.audit_entry.id, "audit-runtime-rejected");
        assert_eq!(readout.audit_entry.sequence, 1);
        assert_eq!(
            readout.audit_entry.outcome_class,
            CommandOutcomeClass::RejectedTargetLegality
        );
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByResolver
        );
        assert!(!readout.audit_entry.accepted);
        assert_eq!(
            readout.audit_entry.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert_eq!(readout.audit_entry.event_count, 0);
        assert_eq!(
            readout.audit_entry.trace_count,
            readout.receipt.trace.len() as u32
        );
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
    }

    #[test]
    fn session_runtime_rejected_command_does_not_record_action_usage() {
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
        let before_rejection = session.action_usage_log().to_vec();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-rejected",
            "Runtime rejected",
            "Adept targets themself through the command runtime.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));

        assert_eq!(session.action_usage_log(), before_rejection.as_slice());
    }

    #[test]
    fn session_runtime_current_turn_action_usage_ignores_rejected_commands() {
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
        let before_rejection = session.current_turn_action_usage();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-rejected",
            "Runtime rejected",
            "Adept targets themself through the command runtime.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));

        assert_eq!(session.current_turn_action_usage(), before_rejection);
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
    fn session_runtime_accumulates_audit_entries_separately_from_combat_log() {
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

        assert_eq!(session.combat_log().len(), 3);
        assert_eq!(session.audit_log().len(), 3);
        assert_eq!(
            session
                .audit_log()
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "audit-runtime-hit",
                "audit-runtime-miss",
                "audit-runtime-rejected"
            ]
        );
        assert_eq!(
            session
                .audit_log()
                .iter()
                .map(|entry| entry.accepted)
                .collect::<Vec<_>>(),
            vec![true, true, false]
        );
        assert_eq!(
            session
                .audit_log()
                .iter()
                .map(|entry| entry.decision_kind)
                .collect::<Vec<_>>(),
            vec![
                CommandDecisionKind::AcceptedByResolver,
                CommandDecisionKind::AcceptedByResolver,
                CommandDecisionKind::RejectedByResolver
            ]
        );
        assert_eq!(
            session
                .audit_log()
                .iter()
                .map(|entry| entry.rejection)
                .collect::<Vec<_>>(),
            vec![None, None, Some(RulebenchRejection::TargetLegalityFailed)]
        );
        assert_eq!(
            session
                .audit_log()
                .iter()
                .map(|entry| entry.event_count)
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
        assert_eq!(session.lifecycle_transition_log().len(), 2);
        assert_eq!(
            session.lifecycle_transition_log()[0].trigger,
            LifecycleTransitionTrigger::CommandStart
        );
        assert_eq!(
            session.lifecycle_transition_log()[1].trigger,
            LifecycleTransitionTrigger::ExplicitEnd
        );
        assert_eq!(session.lifecycle_transition_log()[1].sequence, 1);
        assert_eq!(session.lifecycle_transition_log()[1].step_index, 2);
        assert_eq!(
            session.lifecycle_transition_log()[1].previous_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(
            session.lifecycle_transition_log()[1].next_phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(
            session.lifecycle_transition_log()[1].started_at_step,
            Some(0)
        );
        assert_eq!(session.lifecycle_transition_log()[1].ended_at_step, Some(2));
    }

    #[test]
    fn session_runtime_lifecycle_transition_history_is_empty_initially() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        assert!(session.lifecycle_transition_log().is_empty());
        assert!(session.snapshot().lifecycle_transition_log.is_empty());
    }

    #[test]
    fn session_runtime_can_start_combat_explicitly() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_start = session.snapshot();

        session.start_combat();
        let after_start = session.snapshot();

        assert_eq!(before_start.lifecycle.phase, CombatLifecyclePhase::Ready);
        assert_eq!(
            after_start.lifecycle.phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(after_start.lifecycle.started_at_step, Some(0));
        assert_eq!(after_start.lifecycle.ended_at_step, None);
        assert_eq!(after_start.next_step_index, before_start.next_step_index);
        assert_eq!(after_start.lifecycle_transition_log.len(), 1);
        assert_eq!(after_start.lifecycle_transition_log[0].sequence, 0);
        assert_eq!(
            after_start.lifecycle_transition_log[0].trigger,
            LifecycleTransitionTrigger::ExplicitStart
        );
        assert_eq!(
            after_start.lifecycle_transition_log[0].trigger.code(),
            "explicitStart"
        );
        assert_eq!(after_start.lifecycle_transition_log[0].step_index, 0);
        assert_eq!(
            after_start.lifecycle_transition_log[0].previous_phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(
            after_start.lifecycle_transition_log[0].next_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(
            after_start.lifecycle_transition_log[0].started_at_step,
            Some(0)
        );
        assert_eq!(after_start.lifecycle_transition_log[0].ended_at_step, None);
        assert_eq!(after_start.turn_order, before_start.turn_order);
        assert_eq!(after_start.combat_log, before_start.combat_log);
        assert_eq!(after_start.audit_log, before_start.audit_log);
        assert_eq!(
            after_start.current_state_fingerprint,
            before_start.current_state_fingerprint
        );
    }

    #[test]
    fn session_runtime_control_command_starts_combat_with_readout() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_start = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_start.current_state);

        let readout = session.submit_control_command(CombatControlCommandSpec::explicit_start());
        let after_start = session.snapshot();

        assert!(readout.accepted);
        assert_eq!(
            readout.command_kind,
            CombatControlCommandKind::ExplicitStart
        );
        assert_eq!(readout.command_kind.code(), "explicitStart");
        assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(readout.decision_kind.code(), "accepted");
        assert_eq!(readout.previous_lifecycle, before_start.lifecycle);
        assert_eq!(readout.next_lifecycle, after_start.lifecycle);
        assert_eq!(readout.previous_turn_order, before_start.turn_order);
        assert_eq!(readout.next_turn_order, before_start.turn_order);
        assert_eq!(
            readout.lifecycle_transition,
            Some(after_start.lifecycle_transition_log[0].clone())
        );
        assert_eq!(readout.turn_advance, None);
        assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(readout.reason, "Combat explicitly started.");
        assert_eq!(
            after_start.lifecycle.phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(after_start.lifecycle_transition_log.len(), 1);
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(history.sequence, 0);
        assert_eq!(
            history.command_kind,
            CombatControlCommandKind::ExplicitStart
        );
        assert!(history.accepted);
        assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(
            history.previous_lifecycle_phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(
            history.next_lifecycle_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(
            history.previous_round_number,
            before_start.turn_order.round_number
        );
        assert_eq!(
            history.previous_turn_index,
            before_start.turn_order.current_turn_index
        );
        assert_eq!(
            history.previous_actor_id,
            before_start.turn_order.current_actor_id
        );
        assert_eq!(
            history.next_round_number,
            before_start.turn_order.round_number
        );
        assert_eq!(
            history.next_turn_index,
            before_start.turn_order.current_turn_index
        );
        assert_eq!(
            history.next_actor_id,
            before_start.turn_order.current_actor_id
        );
        assert_eq!(history.lifecycle_transition_sequence, Some(0));
        assert_eq!(history.turn_transition_sequence, None);
        assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(history.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(history.reason, "Combat explicitly started.");
    }

    #[test]
    fn session_runtime_control_command_rejects_repeated_start_without_transition() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.start_combat();
        assert!(session.control_history().is_empty());
        let before_repeat = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_repeat.current_state);

        let readout = session.submit_control_command(CombatControlCommandSpec::explicit_start());
        let after_repeat = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(
            readout.command_kind,
            CombatControlCommandKind::ExplicitStart
        );
        assert_eq!(
            readout.decision_kind,
            CombatControlDecisionKind::RejectedNoop
        );
        assert_eq!(readout.decision_kind.code(), "rejectedNoop");
        assert_eq!(readout.previous_lifecycle, before_repeat.lifecycle);
        assert_eq!(readout.next_lifecycle, before_repeat.lifecycle);
        assert_eq!(readout.previous_turn_order, before_repeat.turn_order);
        assert_eq!(readout.next_turn_order, before_repeat.turn_order);
        assert_eq!(readout.lifecycle_transition, None);
        assert_eq!(readout.turn_advance, None);
        assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(readout.reason, "Combat is already in progress.");
        assert_eq!(
            after_repeat.lifecycle_transition_log,
            before_repeat.lifecycle_transition_log
        );
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(history.sequence, 0);
        assert_eq!(
            history.command_kind,
            CombatControlCommandKind::ExplicitStart
        );
        assert!(!history.accepted);
        assert_eq!(
            history.decision_kind,
            CombatControlDecisionKind::RejectedNoop
        );
        assert_eq!(
            history.previous_lifecycle_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(
            history.next_lifecycle_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(history.lifecycle_transition_sequence, None);
        assert_eq!(history.turn_transition_sequence, None);
        assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(history.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(history.reason, "Combat is already in progress.");
    }

    #[test]
    fn session_runtime_command_start_records_lifecycle_transition() {
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

        assert_eq!(session.lifecycle_transition_log().len(), 1);
        let transition = &session.lifecycle_transition_log()[0];
        assert_eq!(transition.sequence, 0);
        assert_eq!(transition.trigger, LifecycleTransitionTrigger::CommandStart);
        assert_eq!(transition.trigger.code(), "commandStart");
        assert_eq!(transition.step_index, 0);
        assert_eq!(transition.previous_phase, CombatLifecyclePhase::Ready);
        assert_eq!(transition.next_phase, CombatLifecyclePhase::InProgress);
        assert_eq!(transition.started_at_step, Some(0));
        assert_eq!(transition.ended_at_step, None);
    }

    #[test]
    fn session_runtime_explicit_start_is_idempotent_while_in_progress() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.start_combat();
        let before_repeat = session.snapshot();

        session.start_combat();
        let after_repeat = session.snapshot();

        assert_eq!(after_repeat.lifecycle, before_repeat.lifecycle);
        assert_eq!(after_repeat.turn_order, before_repeat.turn_order);
        assert_eq!(after_repeat.next_step_index, before_repeat.next_step_index);
        assert_eq!(
            after_repeat.current_state_fingerprint,
            before_repeat.current_state_fingerprint
        );
        assert_eq!(after_repeat.combat_log, before_repeat.combat_log);
        assert_eq!(after_repeat.audit_log, before_repeat.audit_log);
        assert_eq!(
            after_repeat.lifecycle_transition_log,
            before_repeat.lifecycle_transition_log
        );
    }

    #[test]
    fn session_runtime_explicit_start_after_end_is_noop() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.start_combat();
        session.end_combat();
        let before_start_attempt = session.snapshot();

        session.start_combat();
        let after_start_attempt = session.snapshot();

        assert_eq!(
            before_start_attempt.lifecycle.phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(
            after_start_attempt.lifecycle,
            before_start_attempt.lifecycle
        );
        assert_eq!(
            after_start_attempt.turn_order,
            before_start_attempt.turn_order
        );
        assert_eq!(
            after_start_attempt.next_step_index,
            before_start_attempt.next_step_index
        );
        assert_eq!(
            after_start_attempt.current_state_fingerprint,
            before_start_attempt.current_state_fingerprint
        );
        assert_eq!(
            after_start_attempt.combat_log,
            before_start_attempt.combat_log
        );
        assert_eq!(
            after_start_attempt.audit_log,
            before_start_attempt.audit_log
        );
        assert_eq!(
            after_start_attempt.lifecycle_transition_log,
            before_start_attempt.lifecycle_transition_log
        );
    }

    #[test]
    fn session_runtime_command_after_explicit_start_does_not_duplicate_lifecycle_transition() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.start_combat();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::InProgress);
        assert_eq!(session.lifecycle().started_at_step, Some(0));
        assert_eq!(session.lifecycle().ended_at_step, None);
        assert_eq!(session.next_step_index(), 1);
        assert_eq!(session.lifecycle_transition_log().len(), 1);
        assert_eq!(
            session.lifecycle_transition_log()[0].trigger,
            LifecycleTransitionTrigger::ExplicitStart
        );
        assert_eq!(session.lifecycle_transition_log()[0].step_index, 0);
    }

    #[test]
    fn session_runtime_control_command_ends_combat_and_rejects_repeated_end() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_end = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_end.current_state);

        let accepted = session.submit_control_command(CombatControlCommandSpec::explicit_end());
        let after_end = session.snapshot();

        assert!(accepted.accepted);
        assert_eq!(accepted.command_kind, CombatControlCommandKind::ExplicitEnd);
        assert_eq!(accepted.command_kind.code(), "explicitEnd");
        assert_eq!(accepted.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(accepted.previous_lifecycle, before_end.lifecycle);
        assert_eq!(accepted.next_lifecycle, after_end.lifecycle);
        assert_eq!(accepted.previous_turn_order, before_end.turn_order);
        assert_eq!(accepted.next_turn_order, before_end.turn_order);
        assert_eq!(
            accepted.lifecycle_transition,
            Some(after_end.lifecycle_transition_log[0].clone())
        );
        assert_eq!(accepted.turn_advance, None);
        assert_eq!(accepted.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(accepted.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(accepted.reason, "Combat explicitly ended.");
        assert_eq!(after_end.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(after_end.lifecycle.started_at_step, Some(0));
        assert_eq!(after_end.lifecycle.ended_at_step, Some(0));
        assert_eq!(session.control_history().len(), 1);
        assert_eq!(session.control_history()[0].sequence, 0);
        assert_eq!(
            session.control_history()[0].command_kind,
            CombatControlCommandKind::ExplicitEnd
        );
        assert_eq!(
            session.control_history()[0].lifecycle_transition_sequence,
            Some(0)
        );

        let before_repeat = session.snapshot();
        let before_repeat_state_fingerprint =
            fingerprint_projected_state(&before_repeat.current_state);
        let rejected = session.submit_control_command(CombatControlCommandSpec::explicit_end());
        let after_repeat = session.snapshot();

        assert!(!rejected.accepted);
        assert_eq!(rejected.command_kind, CombatControlCommandKind::ExplicitEnd);
        assert_eq!(
            rejected.decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
        assert_eq!(rejected.reason, "Combat is already ended.");
        assert_eq!(rejected.lifecycle_transition, None);
        assert_eq!(rejected.turn_advance, None);
        assert_eq!(rejected.previous_lifecycle, before_repeat.lifecycle);
        assert_eq!(rejected.next_lifecycle, before_repeat.lifecycle);
        assert_eq!(
            rejected.state_before_fingerprint,
            before_repeat_state_fingerprint
        );
        assert_eq!(
            rejected.state_after_fingerprint,
            before_repeat_state_fingerprint
        );
        assert_eq!(
            after_repeat.lifecycle_transition_log,
            before_repeat.lifecycle_transition_log
        );
        assert_eq!(session.control_history().len(), 2);
        assert_eq!(session.control_history()[1].sequence, 1);
        assert_eq!(
            session.control_history()[1].decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
        assert_eq!(
            session.control_history()[1].lifecycle_transition_sequence,
            None
        );
    }

    #[test]
    fn session_runtime_control_command_conditionally_ends_when_end_condition_is_met() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let mut session = CombatSessionState::new("runtime-conditional-end", scenario);
        session.start_combat();
        let before_end = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_end.current_state);

        let readout =
            session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
        let after_end = session.snapshot();

        assert!(before_end.combat_end_condition.combat_should_end);
        assert!(readout.accepted);
        assert_eq!(
            readout.command_kind,
            CombatControlCommandKind::EndIfConditionMet
        );
        assert_eq!(readout.command_kind.code(), "endIfConditionMet");
        assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(readout.decision_kind.code(), "accepted");
        assert_eq!(readout.previous_lifecycle, before_end.lifecycle);
        assert_eq!(readout.next_lifecycle, after_end.lifecycle);
        assert_eq!(readout.previous_turn_order, before_end.turn_order);
        assert_eq!(readout.next_turn_order, before_end.turn_order);
        assert_eq!(
            readout.lifecycle_transition,
            Some(after_end.lifecycle_transition_log[1].clone())
        );
        assert_eq!(readout.turn_advance, None);
        assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(
            readout.reason,
            "Combat conditionally ended. Combat should end because no active enemies remain."
        );
        assert_eq!(after_end.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(after_end.lifecycle.started_at_step, Some(0));
        assert_eq!(after_end.lifecycle.ended_at_step, Some(0));
        assert_eq!(after_end.lifecycle_transition_log.len(), 2);
        assert_eq!(
            after_end.lifecycle_transition_log[1].trigger,
            LifecycleTransitionTrigger::ConditionalEnd
        );
        assert_eq!(
            after_end.lifecycle_transition_log[1].trigger.code(),
            "conditionalEnd"
        );
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(
            history.command_kind,
            CombatControlCommandKind::EndIfConditionMet
        );
        assert!(history.accepted);
        assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(history.lifecycle_transition_sequence, Some(1));
        assert_eq!(history.turn_transition_sequence, None);
        assert_eq!(history.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(history.state_after_fingerprint, before_state_fingerprint);
    }

    #[test]
    fn session_runtime_control_command_rejects_conditional_end_while_combat_can_continue() {
        let mut session =
            CombatSessionState::new("runtime-conditional-end", hexing_bolt_fixture_scenario());
        session.start_combat();
        let before_attempt = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_attempt.current_state);

        let readout =
            session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
        let after_attempt = session.snapshot();

        assert!(!before_attempt.combat_end_condition.combat_should_end);
        assert!(!readout.accepted);
        assert_eq!(
            readout.command_kind,
            CombatControlCommandKind::EndIfConditionMet
        );
        assert_eq!(
            readout.decision_kind,
            CombatControlDecisionKind::RejectedByEndCondition
        );
        assert_eq!(readout.decision_kind.code(), "rejectedByEndCondition");
        assert_eq!(
            readout.reason,
            "Combat end condition is not met. Combat can continue because both sides have active combatants."
        );
        assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
        assert_eq!(readout.next_turn_order, before_attempt.turn_order);
        assert_eq!(readout.lifecycle_transition, None);
        assert_eq!(readout.turn_advance, None);
        assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
        assert_eq!(
            after_attempt.lifecycle_transition_log,
            before_attempt.lifecycle_transition_log
        );
        assert_eq!(session.control_history().len(), 1);
        assert_eq!(
            session.control_history()[0].decision_kind,
            CombatControlDecisionKind::RejectedByEndCondition
        );
        assert_eq!(
            session.control_history()[0].lifecycle_transition_sequence,
            None
        );
    }

    #[test]
    fn session_runtime_control_command_rejects_conditional_end_after_combat_already_ended() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let mut session = CombatSessionState::new("runtime-conditional-end", scenario);
        session.end_combat();
        let before_attempt = session.snapshot();
        let before_state_fingerprint = fingerprint_projected_state(&before_attempt.current_state);

        let readout =
            session.submit_control_command(CombatControlCommandSpec::end_if_condition_met());
        let after_attempt = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(
            readout.command_kind,
            CombatControlCommandKind::EndIfConditionMet
        );
        assert_eq!(
            readout.decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
        assert_eq!(readout.reason, "Combat is already ended.");
        assert_eq!(readout.lifecycle_transition, None);
        assert_eq!(readout.turn_advance, None);
        assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.state_before_fingerprint, before_state_fingerprint);
        assert_eq!(readout.state_after_fingerprint, before_state_fingerprint);
        assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
        assert_eq!(
            after_attempt.lifecycle_transition_log,
            before_attempt.lifecycle_transition_log
        );
        assert_eq!(session.control_history().len(), 1);
        assert_eq!(
            session.control_history()[0].decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
    }

    #[test]
    fn session_runtime_script_runs_conditional_end_control_step() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let mut session = CombatSessionState::new("runtime-conditional-end-script", scenario);

        let readout = session.run_script(CombatSessionScriptSpec::new(
            "conditional-end-script",
            "Conditional end script",
            "Script runs the Rust conditional end control.",
            vec![CombatSessionScriptStepSpec::control(
                "script-conditional-end",
                "Conditionally end combat",
                "Ends only because the Rust end condition is met.",
                CombatControlCommandSpec::end_if_condition_met(),
            )],
        ));

        assert_eq!(readout.steps.len(), 1);
        let step = &readout.steps[0];
        assert_eq!(step.command_kind, CombatSessionScriptCommandKind::Control);
        assert!(step.accepted);
        assert_eq!(
            step.decision_kind,
            CombatSessionScriptDecisionKind::Control(CombatControlDecisionKind::Accepted)
        );
        assert_eq!(
            step.reason,
            "Combat conditionally ended. Combat should end because no active enemies remain."
        );
        assert_eq!(step.control_history_sequence, Some(0));
        assert_eq!(step.command_audit_sequence, None);
        assert_eq!(step.runtime_step_id, None);
        assert_eq!(
            readout.final_snapshot.lifecycle.phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(
            session.lifecycle_transition_log()[0].trigger,
            LifecycleTransitionTrigger::ConditionalEnd
        );
    }

    #[test]
    fn session_runtime_direct_control_methods_do_not_record_control_history() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.start_combat();
        session.advance_turn();
        session.end_combat();

        assert!(session.control_history().is_empty());
        assert_eq!(session.lifecycle_transition_log().len(), 2);
        assert_eq!(session.turn_transition_log().len(), 1);
    }

    #[test]
    fn combat_lifecycle_preserves_first_end_marker() {
        let mut lifecycle = CombatLifecycle::ready();

        lifecycle.end_at_step(3);
        lifecycle.end_at_step(9);

        assert_eq!(lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(lifecycle.started_at_step, Some(3));
        assert_eq!(lifecycle.ended_at_step, Some(3));

        let mut in_progress_lifecycle = CombatLifecycle::ready();
        in_progress_lifecycle.start_at_step(1);
        in_progress_lifecycle.end_at_step(4);
        in_progress_lifecycle.end_at_step(9);

        assert_eq!(in_progress_lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(in_progress_lifecycle.started_at_step, Some(1));
        assert_eq!(in_progress_lifecycle.ended_at_step, Some(4));
    }

    #[test]
    fn session_runtime_end_from_ready_records_lifecycle_transition() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.end_combat();

        assert_eq!(session.lifecycle_transition_log().len(), 1);
        let transition = &session.lifecycle_transition_log()[0];
        assert_eq!(transition.sequence, 0);
        assert_eq!(transition.trigger, LifecycleTransitionTrigger::ExplicitEnd);
        assert_eq!(transition.trigger.code(), "explicitEnd");
        assert_eq!(transition.step_index, 0);
        assert_eq!(transition.previous_phase, CombatLifecyclePhase::Ready);
        assert_eq!(transition.next_phase, CombatLifecyclePhase::Ended);
        assert_eq!(transition.started_at_step, Some(0));
        assert_eq!(transition.ended_at_step, Some(0));
    }

    #[test]
    fn session_runtime_repeated_end_combat_preserves_first_end_snapshot() {
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
        session.end_combat();
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-post-end",
            "Runtime post-end command",
            "A command submitted after combat ended.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let before_repeat = session.snapshot();

        session.end_combat();
        let after_repeat = session.snapshot();

        assert_eq!(before_repeat.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(before_repeat.lifecycle.started_at_step, Some(0));
        assert_eq!(before_repeat.lifecycle.ended_at_step, Some(1));
        assert_eq!(before_repeat.next_step_index, 2);
        assert_eq!(after_repeat.lifecycle, before_repeat.lifecycle);
        assert_eq!(after_repeat.turn_order, before_repeat.turn_order);
        assert_eq!(after_repeat.next_step_index, before_repeat.next_step_index);
        assert_eq!(
            after_repeat.current_state_fingerprint,
            before_repeat.current_state_fingerprint
        );
        assert_eq!(after_repeat.combat_log, before_repeat.combat_log);
        assert_eq!(after_repeat.audit_log, before_repeat.audit_log);
        assert_eq!(
            after_repeat.lifecycle_transition_log,
            before_repeat.lifecycle_transition_log
        );
    }

    #[test]
    fn session_runtime_rejects_commands_after_combat_end() {
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
        session.end_combat();
        let ended_at_step = session.lifecycle().ended_at_step;
        let state_before_attempt = session.snapshot().current_state_fingerprint;

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-post-end",
            "Runtime post-end command",
            "A command submitted after combat ended.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let state_after_attempt = session.snapshot().current_state_fingerprint;

        assert_eq!(readout.step.index, 1);
        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert!(readout.receipt.events.is_empty());
        assert!(readout.receipt.attack_roll.is_none());
        assert!(readout.receipt.damage.is_none());
        assert!(readout.receipt.modifier.is_none());
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 9);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 9);
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
        assert_eq!(state_before_attempt, state_after_attempt);
        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
        assert_eq!(session.lifecycle().ended_at_step, ended_at_step);
        assert_eq!(session.next_step_index(), 2);
    }

    #[test]
    fn session_runtime_records_post_end_attempt_in_log_and_audit() {
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
        session.end_combat();

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-post-end",
            "Runtime post-end command",
            "A command submitted after combat ended.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(readout.combat_log[0].id, "log-runtime-post-end");
        assert!(readout.combat_log[0].event_types.is_empty());
        assert_eq!(session.combat_log().len(), 2);
        assert_eq!(session.audit_log().len(), 2);
        assert_eq!(readout.audit_entry.id, "audit-runtime-post-end");
        assert_eq!(readout.audit_entry.sequence, 1);
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByLifecycle
        );
        assert_eq!(
            readout.audit_entry.decision_kind.code(),
            "rejectedByLifecycle"
        );
        assert!(!readout.audit_entry.accepted);
        assert_eq!(
            readout.audit_entry.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(readout.audit_entry.event_count, 0);
        assert_eq!(readout.audit_entry.trace_count, 2);
        assert_eq!(session.audit_log()[1], readout.audit_entry);
    }

    #[test]
    fn session_runtime_post_end_command_does_not_record_action_usage() {
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
        session.end_combat();
        let before_post_end = session.action_usage_log().to_vec();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-post-end",
            "Runtime post-end command",
            "A command submitted after combat ended.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(session.action_usage_log(), before_post_end.as_slice());
    }

    #[test]
    fn session_runtime_rejects_commands_for_non_current_actor() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        let state_before_attempt = session.snapshot().current_state_fingerprint;

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-wrong-actor",
            "Runtime wrong actor",
            "Adept attempts to act during Raider's turn.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let snapshot = session.snapshot();

        assert_eq!(readout.step.index, 0);
        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert!(readout.receipt.events.is_empty());
        assert!(readout.receipt.target_legality.is_none());
        assert!(readout.receipt.attack_roll.is_none());
        assert!(readout.receipt.damage.is_none());
        assert!(readout.receipt.modifier.is_none());
        assert_eq!(
            readout.receipt.trace[1].message,
            "Command rejected by turn order."
        );
        assert_eq!(readout.state_before.combatants[1].hit_points.current, 18);
        assert_eq!(readout.state_after.combatants[1].hit_points.current, 18);
        assert_eq!(
            readout.audit_entry.state_before_fingerprint,
            readout.audit_entry.state_after_fingerprint
        );
        assert_eq!(snapshot.current_state_fingerprint, state_before_attempt);
        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
        assert_eq!(
            snapshot.turn_order.current_actor_id,
            Some("entity-raider".to_string())
        );
        assert_eq!(snapshot.next_step_index, 1);
    }

    #[test]
    fn session_runtime_records_non_current_actor_attempt_in_log_and_audit() {
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
        session.advance_turn();
        let after_hit_fingerprint = session.snapshot().current_state_fingerprint;

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-wrong-actor",
            "Runtime wrong actor",
            "Adept attempts to act during Raider's turn.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let snapshot = session.snapshot();

        assert_eq!(readout.combat_log[0].id, "log-runtime-wrong-actor");
        assert!(readout.combat_log[0].event_types.is_empty());
        assert_eq!(session.combat_log().len(), 2);
        assert_eq!(session.audit_log().len(), 2);
        assert_eq!(readout.audit_entry.id, "audit-runtime-wrong-actor");
        assert_eq!(readout.audit_entry.sequence, 1);
        assert_eq!(
            readout.audit_entry.decision_kind,
            CommandDecisionKind::RejectedByTurnOrder
        );
        assert_eq!(
            readout.audit_entry.decision_kind.code(),
            "rejectedByTurnOrder"
        );
        assert!(!readout.audit_entry.accepted);
        assert_eq!(
            readout.audit_entry.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(readout.audit_entry.event_count, 0);
        assert_eq!(readout.audit_entry.trace_count, 2);
        assert_eq!(session.audit_log()[1], readout.audit_entry);
        assert_eq!(snapshot.current_state_fingerprint, after_hit_fingerprint);
        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::InProgress);
        assert_eq!(snapshot.lifecycle.started_at_step, Some(0));
        assert_eq!(snapshot.lifecycle.ended_at_step, None);
        assert_eq!(snapshot.turn_order.round_number, 1);
        assert_eq!(snapshot.turn_order.current_turn_index, 1);
        assert_eq!(
            snapshot.turn_order.current_actor_id,
            Some("entity-raider".to_string())
        );
        assert_eq!(snapshot.turn_transition_log.len(), 1);
        assert_eq!(snapshot.turn_transition_log[0].previous_turn_index, 0);
        assert_eq!(snapshot.turn_transition_log[0].next_turn_index, 1);
        assert!(!snapshot.turn_transition_log[0].wrapped_round);
    }

    #[test]
    fn session_runtime_non_current_actor_command_does_not_record_action_usage() {
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
        session.advance_turn();
        let before_wrong_actor = session.action_usage_log().to_vec();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-wrong-actor",
            "Runtime wrong actor",
            "Adept attempts to act during Raider's turn.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(session.action_usage_log(), before_wrong_actor.as_slice());
    }

    #[test]
    fn session_runtime_current_turn_action_usage_filters_after_turn_advance() {
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

        session.advance_turn();
        let summary = session.current_turn_action_usage();

        assert_eq!(summary.round_number, 1);
        assert_eq!(summary.turn_index, 1);
        assert_eq!(summary.current_actor_id, Some("entity-raider".to_string()));
        assert_eq!(summary.used_action_count, 0);
        assert!(summary.used_action_ids.is_empty());
        assert!(summary.used_ability_ids.is_empty());
    }

    #[test]
    fn session_runtime_turn_transition_history_is_empty_initially() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        assert!(session.turn_transition_log().is_empty());
        assert!(session.snapshot().turn_transition_log.is_empty());
    }

    #[test]
    fn session_runtime_ended_combat_gate_takes_precedence_over_actor_gate() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        session.end_combat();

        let readout = session.submit_command(CombatSessionCommandSpec::new(
            "runtime-post-end-wrong-actor",
            "Runtime post-end wrong actor",
            "Adept attempts to act during Raider's turn after combat ended.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert!(!readout.receipt.accepted);
        assert_eq!(
            readout.receipt.rejection,
            Some(RulebenchRejection::InvalidAction)
        );
        assert_eq!(
            readout.receipt.trace[1].message,
            "Command rejected by lifecycle."
        );
        assert_eq!(session.lifecycle().phase, CombatLifecyclePhase::Ended);
        assert_eq!(session.lifecycle().ended_at_step, Some(0));
        assert_eq!(
            session.turn_order().current_actor_id,
            Some("entity-raider".to_string())
        );
    }

    #[test]
    fn session_runtime_initializes_turn_order_from_scenario_combatants() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        assert_eq!(session.turn_order().round_number, 1);
        assert_eq!(session.turn_order().current_turn_index, 0);
        assert_eq!(
            session.turn_order().participant_order,
            vec!["entity-adept".to_string(), "entity-raider".to_string()]
        );
        assert_eq!(
            session.turn_order().current_actor_id,
            Some("entity-adept".to_string())
        );
    }

    #[test]
    fn session_runtime_advances_turns_and_rounds() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.advance_turn();

        assert_eq!(session.turn_order().round_number, 1);
        assert_eq!(session.turn_order().current_turn_index, 1);
        assert_eq!(
            session.turn_order().current_actor_id,
            Some("entity-raider".to_string())
        );

        session.advance_turn();

        assert_eq!(session.turn_order().round_number, 2);
        assert_eq!(session.turn_order().current_turn_index, 0);
        assert_eq!(
            session.turn_order().current_actor_id,
            Some("entity-adept".to_string())
        );
    }

    #[test]
    fn session_runtime_records_successful_turn_transition() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let previous_turn_order = session.turn_order().clone();

        let readout = session.advance_turn();

        assert_eq!(session.turn_transition_log().len(), 1);
        let transition = &session.turn_transition_log()[0];
        assert!(readout.accepted);
        assert_eq!(readout.decision_kind, TurnAdvanceDecisionKind::Advanced);
        assert_eq!(readout.previous_turn_order, previous_turn_order);
        assert_eq!(readout.next_turn_order, session.turn_order().clone());
        assert_eq!(readout.transition, Some(transition.clone()));
        assert_eq!(
            readout.state_before_fingerprint,
            readout.state_after_fingerprint
        );
        assert_eq!(readout.reason, "Turn advanced to the next participant.");
        assert_eq!(transition.sequence, 0);
        assert_eq!(transition.previous_round_number, 1);
        assert_eq!(transition.previous_turn_index, 0);
        assert_eq!(
            transition.previous_actor_id,
            Some("entity-adept".to_string())
        );
        assert_eq!(transition.next_round_number, 1);
        assert_eq!(transition.next_turn_index, 1);
        assert_eq!(transition.next_actor_id, Some("entity-raider".to_string()));
        assert!(!transition.wrapped_round);
    }

    #[test]
    fn session_runtime_control_command_advances_turn_with_readout() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let before_advance = session.snapshot();

        let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
        let after_advance = session.snapshot();

        assert!(readout.accepted);
        assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert_eq!(readout.command_kind.code(), "advanceTurn");
        assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(readout.lifecycle_transition, None);
        let turn_advance = readout
            .turn_advance
            .as_ref()
            .expect("advance turn control returns turn readout");
        assert!(turn_advance.accepted);
        assert_eq!(
            turn_advance.decision_kind,
            TurnAdvanceDecisionKind::Advanced
        );
        assert_eq!(
            turn_advance.transition,
            Some(after_advance.turn_transition_log[0].clone())
        );
        assert_eq!(readout.previous_lifecycle, before_advance.lifecycle);
        assert_eq!(readout.next_lifecycle, before_advance.lifecycle);
        assert_eq!(readout.previous_turn_order, before_advance.turn_order);
        assert_eq!(readout.next_turn_order, after_advance.turn_order);
        assert_eq!(
            readout.state_before_fingerprint,
            turn_advance.state_before_fingerprint
        );
        assert_eq!(
            readout.state_after_fingerprint,
            turn_advance.state_after_fingerprint
        );
        assert_eq!(readout.reason, "Turn advanced to the next participant.");
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(history.sequence, 0);
        assert_eq!(history.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert!(history.accepted);
        assert_eq!(history.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(
            history.previous_lifecycle_phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(history.next_lifecycle_phase, CombatLifecyclePhase::Ready);
        assert_eq!(
            history.previous_round_number,
            before_advance.turn_order.round_number
        );
        assert_eq!(
            history.previous_turn_index,
            before_advance.turn_order.current_turn_index
        );
        assert_eq!(
            history.previous_actor_id,
            before_advance.turn_order.current_actor_id
        );
        assert_eq!(
            history.next_round_number,
            after_advance.turn_order.round_number
        );
        assert_eq!(
            history.next_turn_index,
            after_advance.turn_order.current_turn_index
        );
        assert_eq!(
            history.next_actor_id,
            after_advance.turn_order.current_actor_id
        );
        assert_eq!(history.lifecycle_transition_sequence, None);
        assert_eq!(history.turn_transition_sequence, Some(0));
        assert_eq!(
            history.state_before_fingerprint,
            turn_advance.state_before_fingerprint
        );
        assert_eq!(
            history.state_after_fingerprint,
            turn_advance.state_after_fingerprint
        );
        assert_eq!(history.reason, "Turn advanced to the next participant.");
    }

    #[test]
    fn session_runtime_records_turn_transition_round_wrap() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.advance_turn();
        let readout = session.advance_turn();

        assert_eq!(session.turn_transition_log().len(), 2);
        let transition = &session.turn_transition_log()[1];
        assert!(readout.accepted);
        assert_eq!(readout.decision_kind, TurnAdvanceDecisionKind::Advanced);
        assert_eq!(readout.next_turn_order, session.turn_order().clone());
        assert_eq!(readout.transition, Some(transition.clone()));
        assert_eq!(
            readout.state_before_fingerprint,
            readout.state_after_fingerprint
        );
        assert_eq!(transition.sequence, 1);
        assert_eq!(transition.previous_round_number, 1);
        assert_eq!(transition.previous_turn_index, 1);
        assert_eq!(
            transition.previous_actor_id,
            Some("entity-raider".to_string())
        );
        assert_eq!(transition.next_round_number, 2);
        assert_eq!(transition.next_turn_index, 0);
        assert_eq!(transition.next_actor_id, Some("entity-adept".to_string()));
        assert!(transition.wrapped_round);
    }

    #[test]
    fn session_runtime_turn_wrap_refreshes_spent_current_actor_action_resource() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-round-one-action",
            "Runtime round one action",
            "Adept spends the standard action in round one.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
        assert!(
            !session
                .preflight_command(UseActionIntent::new(
                    "entity-adept",
                    "hexing_bolt",
                    "entity-raider"
                ))
                .accepted
        );

        session.advance_turn();
        let readout = session.advance_turn();

        assert!(readout.accepted);
        assert_eq!(
            readout.next_turn_order.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::standard_action_available())
        );
        assert!(
            session
                .preflight_command(UseActionIntent::new(
                    "entity-adept",
                    "hexing_bolt",
                    "entity-raider"
                ))
                .accepted
        );
        assert!(session
            .current_actor_command_candidates()
            .candidates
            .iter()
            .any(|candidate| candidate.accepted));
    }

    #[test]
    fn session_runtime_rejected_turn_advance_does_not_refresh_action_resource() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-spend-before-end",
            "Runtime spend before end",
            "Adept spends the standard action before combat ends.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        session.end_combat();
        let before = session.action_resource_ledger();

        let readout = session.advance_turn();
        let after = session.action_resource_ledger();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            TurnAdvanceDecisionKind::RejectedByLifecycle
        );
        assert_eq!(after, before);
        assert_eq!(
            after
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::new(
                ActionResourceKind::StandardAction,
                0,
                1
            ))
        );
    }

    #[test]
    fn session_runtime_control_turn_advance_refreshes_action_resource_on_round_wrap() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_intent_command(CombatSessionIntentCommandSpec::new(
            "runtime-spend-before-control-wrap",
            "Runtime spend before control wrap",
            "Adept spends the standard action before control turn advancement.",
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        session.submit_control_command(CombatControlCommandSpec::advance_turn());
        let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());

        assert!(readout.accepted);
        assert_eq!(readout.decision_kind, CombatControlDecisionKind::Accepted);
        assert_eq!(session.control_history().len(), 2);
        assert_eq!(
            session
                .action_resource_ledger()
                .combatants
                .iter()
                .find(|combatant| combatant.combatant_id == "entity-adept")
                .and_then(|combatant| combatant.resources.first())
                .cloned(),
            Some(ActionResourceState::standard_action_available())
        );
    }

    #[test]
    fn session_runtime_action_usage_records_current_turn_context() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.advance_turn();
        session.advance_turn();

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-round-two-hit",
            "Runtime round two hit",
            "Adept acts after turn order wraps into round two.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        assert_eq!(session.action_usage_log().len(), 1);
        let usage = &session.action_usage_log()[0];
        assert_eq!(usage.step_id, "runtime-round-two-hit");
        assert_eq!(usage.step_index, 0);
        assert_eq!(usage.round_number, 2);
        assert_eq!(usage.turn_index, 0);
        assert_eq!(usage.actor_id, "entity-adept");
        assert_eq!(usage.outcome_class, CommandOutcomeClass::AcceptedHit);
    }

    #[test]
    fn session_runtime_current_turn_action_usage_filters_after_round_wrap() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-round-one-hit",
            "Runtime round one hit",
            "Adept acts in round one.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        session.advance_turn();
        session.advance_turn();

        let before_round_two_action = session.current_turn_action_usage();
        assert_eq!(before_round_two_action.round_number, 2);
        assert_eq!(before_round_two_action.turn_index, 0);
        assert_eq!(
            before_round_two_action.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert_eq!(before_round_two_action.used_action_count, 0);

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-round-two-hit",
            "Runtime round two hit",
            "Adept acts in round two.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));
        let after_round_two_action = session.current_turn_action_usage();

        assert_eq!(after_round_two_action.used_action_count, 1);
        assert_eq!(
            after_round_two_action.used_action_ids,
            vec!["hexing_bolt".to_string()]
        );
    }

    #[test]
    fn session_runtime_does_not_advance_turn_after_combat_end() {
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
        session.advance_turn();
        session.end_combat();
        let before_attempt = session.snapshot();
        let before_attempt_state_fingerprint =
            fingerprint_projected_state(&before_attempt.current_state);

        let readout = session.advance_turn();
        let after_attempt = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            TurnAdvanceDecisionKind::RejectedByLifecycle
        );
        assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
        assert_eq!(readout.next_turn_order, before_attempt.turn_order);
        assert_eq!(readout.transition, None);
        assert_eq!(
            readout.state_before_fingerprint,
            before_attempt_state_fingerprint
        );
        assert_eq!(
            readout.state_after_fingerprint,
            before_attempt_state_fingerprint
        );
        assert_eq!(readout.reason, "Combat is already ended.");
        assert_eq!(before_attempt.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(before_attempt.lifecycle.ended_at_step, Some(1));
        assert_eq!(after_attempt.lifecycle, before_attempt.lifecycle);
        assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
        assert_eq!(
            after_attempt.turn_transition_log,
            before_attempt.turn_transition_log
        );
        assert_eq!(
            after_attempt.next_step_index,
            before_attempt.next_step_index
        );
        assert_eq!(
            after_attempt.current_state_fingerprint,
            before_attempt.current_state_fingerprint
        );
        assert_eq!(after_attempt.combat_log, before_attempt.combat_log);
        assert_eq!(after_attempt.audit_log, before_attempt.audit_log);
    }

    #[test]
    fn session_runtime_control_command_rejects_turn_advance_after_end() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        session.end_combat();
        let before_attempt = session.snapshot();

        let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
        let after_attempt = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert_eq!(
            readout.decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
        assert_eq!(readout.lifecycle_transition, None);
        let turn_advance = readout
            .turn_advance
            .as_ref()
            .expect("advance turn control returns turn readout");
        assert!(!turn_advance.accepted);
        assert_eq!(
            turn_advance.decision_kind,
            TurnAdvanceDecisionKind::RejectedByLifecycle
        );
        assert_eq!(turn_advance.transition, None);
        assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
        assert_eq!(readout.next_turn_order, before_attempt.turn_order);
        assert_eq!(readout.reason, "Combat is already ended.");
        assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
        assert_eq!(
            after_attempt.turn_transition_log,
            before_attempt.turn_transition_log
        );
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(history.sequence, 0);
        assert_eq!(history.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert!(!history.accepted);
        assert_eq!(
            history.decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );
        assert_eq!(
            history.previous_lifecycle_phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(history.next_lifecycle_phase, CombatLifecyclePhase::Ended);
        assert_eq!(history.lifecycle_transition_sequence, None);
        assert_eq!(history.turn_transition_sequence, None);
        assert_eq!(history.reason, "Combat is already ended.");
    }

    #[test]
    fn session_runtime_turn_order_represents_empty_participants() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants.clear();
        let mut session = CombatSessionState::new("runtime-empty", scenario);

        assert_eq!(session.turn_order().round_number, 0);
        assert_eq!(session.turn_order().current_turn_index, 0);
        assert!(session.turn_order().participant_order.is_empty());
        assert_eq!(session.turn_order().current_actor_id, None);
        assert_eq!(
            session.current_turn_action_usage(),
            ActionUsageSummary {
                round_number: 0,
                turn_index: 0,
                current_actor_id: None,
                used_action_count: 0,
                used_action_ids: Vec::new(),
                used_ability_ids: Vec::new(),
            }
        );
        assert_eq!(
            session.combatant_vitality(),
            CombatantVitalitySummary {
                combatants: Vec::new(),
                active_combatant_ids: Vec::new(),
                defeated_combatant_ids: Vec::new(),
                active_count: 0,
                defeated_count: 0,
            }
        );
        assert_eq!(
            session.current_actor_options(),
            CurrentActorOptionSummary {
                round_number: 0,
                turn_index: 0,
                lifecycle_phase: CombatLifecyclePhase::Ready,
                current_actor_id: None,
                current_actor_defeated: false,
                available: false,
                unavailable_reason: Some(CurrentActorOptionsUnavailableReason::NoCurrentActor),
                actions: Vec::new(),
            }
        );
        assert!(session.turn_transition_log().is_empty());
        let before_attempt = session.snapshot();
        let before_attempt_state_fingerprint =
            fingerprint_projected_state(&before_attempt.current_state);

        let readout = session.advance_turn();
        let after_attempt = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder
        );
        assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
        assert_eq!(readout.next_turn_order, before_attempt.turn_order);
        assert_eq!(readout.transition, None);
        assert_eq!(
            readout.state_before_fingerprint,
            before_attempt_state_fingerprint
        );
        assert_eq!(
            readout.state_after_fingerprint,
            before_attempt_state_fingerprint
        );
        assert_eq!(readout.reason, "Turn order has no participants.");
        assert_eq!(session.turn_order().round_number, 0);
        assert_eq!(session.turn_order().current_turn_index, 0);
        assert_eq!(session.turn_order().current_actor_id, None);
        assert!(session.turn_transition_log().is_empty());
        assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
        assert_eq!(
            after_attempt.turn_transition_log,
            before_attempt.turn_transition_log
        );
        assert_eq!(
            after_attempt.current_state_fingerprint,
            before_attempt.current_state_fingerprint
        );
    }

    #[test]
    fn session_runtime_control_command_rejects_turn_advance_with_empty_participants() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants.clear();
        let mut session = CombatSessionState::new("runtime-empty", scenario);
        let before_attempt = session.snapshot();

        let readout = session.submit_control_command(CombatControlCommandSpec::advance_turn());
        let after_attempt = session.snapshot();

        assert!(!readout.accepted);
        assert_eq!(readout.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert_eq!(
            readout.decision_kind,
            CombatControlDecisionKind::RejectedByEmptyTurnOrder
        );
        assert_eq!(readout.lifecycle_transition, None);
        let turn_advance = readout
            .turn_advance
            .as_ref()
            .expect("advance turn control returns turn readout");
        assert!(!turn_advance.accepted);
        assert_eq!(
            turn_advance.decision_kind,
            TurnAdvanceDecisionKind::RejectedByEmptyTurnOrder
        );
        assert_eq!(turn_advance.transition, None);
        assert_eq!(readout.previous_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.next_lifecycle, before_attempt.lifecycle);
        assert_eq!(readout.previous_turn_order, before_attempt.turn_order);
        assert_eq!(readout.next_turn_order, before_attempt.turn_order);
        assert_eq!(readout.reason, "Turn order has no participants.");
        assert_eq!(after_attempt.turn_order, before_attempt.turn_order);
        assert!(after_attempt.turn_transition_log.is_empty());
        assert_eq!(session.control_history().len(), 1);
        let history = &session.control_history()[0];
        assert_eq!(history.sequence, 0);
        assert_eq!(history.command_kind, CombatControlCommandKind::AdvanceTurn);
        assert!(!history.accepted);
        assert_eq!(
            history.decision_kind,
            CombatControlDecisionKind::RejectedByEmptyTurnOrder
        );
        assert_eq!(history.previous_round_number, 0);
        assert_eq!(history.next_round_number, 0);
        assert_eq!(history.previous_actor_id, None);
        assert_eq!(history.next_actor_id, None);
        assert_eq!(history.lifecycle_transition_sequence, None);
        assert_eq!(history.turn_transition_sequence, None);
        assert_eq!(history.reason, "Turn order has no participants.");
    }

    #[test]
    fn session_runtime_snapshot_reads_initial_state() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let snapshot = session.snapshot();

        assert_eq!(snapshot.session_id, "runtime-hexing-bolt");
        assert_eq!(snapshot.next_step_index, 0);
        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
        assert!(snapshot.lifecycle_transition_log.is_empty());
        assert!(snapshot.current_actor_options.available);
        assert_eq!(
            snapshot.current_actor_options.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert_eq!(
            snapshot.turn_order.current_actor_id,
            Some("entity-adept".to_string())
        );
        assert!(snapshot.combat_log.is_empty());
        assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 18);
    }

    #[test]
    fn session_runtime_snapshot_fingerprint_is_stable_for_unchanged_state() {
        let session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        let first_snapshot = session.snapshot();
        let second_snapshot = session.snapshot();

        assert_eq!(
            first_snapshot.current_state_fingerprint.algorithm,
            PROJECTION_FINGERPRINT_ALGORITHM
        );
        assert_eq!(
            first_snapshot.current_state_fingerprint,
            second_snapshot.current_state_fingerprint
        );
        assert_eq!(
            first_snapshot.current_state_fingerprint,
            fingerprint_projection(&first_snapshot.current_state)
        );
    }

    #[test]
    fn session_runtime_snapshot_reads_command_updates() {
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

        let snapshot = session.snapshot();

        assert_eq!(snapshot.next_step_index, 1);
        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::InProgress);
        assert_eq!(snapshot.lifecycle_transition_log.len(), 1);
        assert_eq!(
            snapshot.lifecycle_transition_log[0].trigger,
            LifecycleTransitionTrigger::CommandStart
        );
        assert_eq!(snapshot.lifecycle_transition_log[0].step_index, 0);
        assert_eq!(
            snapshot.lifecycle_transition_log[0].previous_phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(
            snapshot.lifecycle_transition_log[0].next_phase,
            CombatLifecyclePhase::InProgress
        );
        assert_eq!(snapshot.combat_log.len(), 1);
        assert_eq!(snapshot.combat_log[0].step_id, "runtime-hit");
        assert_eq!(snapshot.audit_log.len(), 1);
        assert_eq!(snapshot.audit_log[0].step_id, "runtime-hit");
        assert_eq!(
            snapshot.audit_log[0].decision_kind,
            CommandDecisionKind::AcceptedByResolver
        );
        assert!(snapshot.audit_log[0].accepted);
        assert_eq!(snapshot.audit_log[0].rejection, None);
        assert_eq!(snapshot.action_usage_log.len(), 1);
        assert_eq!(snapshot.action_usage_log[0].step_id, "runtime-hit");
        assert_eq!(
            snapshot.action_usage_log[0].ability_id,
            "ability.hexing-bolt"
        );
        assert!(snapshot.turn_transition_log.is_empty());
        assert_eq!(snapshot.current_turn_action_usage.used_action_count, 1);
        assert_eq!(
            snapshot.current_turn_action_usage.used_action_ids,
            vec!["hexing_bolt".to_string()]
        );
        assert_eq!(snapshot.combatant_vitality.active_count, 2);
        assert_eq!(snapshot.combatant_vitality.defeated_count, 0);
        assert_eq!(
            snapshot.combatant_vitality.active_combatant_ids,
            vec!["entity-adept".to_string(), "entity-raider".to_string()]
        );
        assert!(snapshot.current_actor_options.available);
        assert_eq!(
            snapshot.current_actor_options.actions[0].target_options[0].current_hit_points,
            9
        );
        assert_eq!(snapshot.current_state.combatants[1].hit_points.current, 9);
        assert_eq!(
            snapshot.current_state.combatants[1].conditions,
            vec!["rattled".to_string()]
        );
    }

    #[test]
    fn session_runtime_vitality_summary_marks_zero_hp_defeated() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[1].hit_points.current = 0;
        let session = CombatSessionState::new("runtime-zero-hp", scenario);

        let summary = session.combatant_vitality();

        assert_eq!(summary.active_count, 1);
        assert_eq!(summary.defeated_count, 1);
        assert_eq!(
            summary.active_combatant_ids,
            vec!["entity-adept".to_string()]
        );
        assert_eq!(
            summary.defeated_combatant_ids,
            vec!["entity-raider".to_string()]
        );
        assert!(summary.combatants[1].defeated);
    }

    #[test]
    fn session_runtime_vitality_summary_marks_negative_hp_defeated() {
        let mut scenario = hexing_bolt_fixture_scenario();
        scenario.combatants[0].hit_points.current = -3;
        let session = CombatSessionState::new("runtime-negative-hp", scenario);

        let summary = session.combatant_vitality();

        assert_eq!(summary.active_count, 1);
        assert_eq!(summary.defeated_count, 1);
        assert_eq!(
            summary.active_combatant_ids,
            vec!["entity-raider".to_string()]
        );
        assert_eq!(
            summary.defeated_combatant_ids,
            vec!["entity-adept".to_string()]
        );
        assert_eq!(summary.combatants[0].current_hit_points, -3);
        assert!(summary.combatants[0].defeated);
    }

    #[test]
    fn session_runtime_snapshot_fingerprint_changes_after_accepted_hit() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());
        let initial_fingerprint = session.snapshot().current_state_fingerprint;

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-hit",
            "Runtime hit",
            "Adept hits Raider through the command runtime.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ));

        let hit_fingerprint = session.snapshot().current_state_fingerprint;

        assert_ne!(initial_fingerprint, hit_fingerprint);
    }

    #[test]
    fn session_runtime_snapshot_fingerprint_is_preserved_after_rejected_command() {
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
        let after_hit_fingerprint = session.snapshot().current_state_fingerprint;

        session.submit_command(CombatSessionCommandSpec::new(
            "runtime-rejected",
            "Runtime rejected",
            "Adept targets themself through the command runtime.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ));
        let after_rejection_snapshot = session.snapshot();

        assert_eq!(after_rejection_snapshot.next_step_index, 2);
        assert_eq!(
            after_rejection_snapshot.current_state_fingerprint,
            after_hit_fingerprint
        );
    }

    #[test]
    fn session_runtime_snapshot_reads_turn_and_end_state() {
        let mut session =
            CombatSessionState::new("runtime-hexing-bolt", hexing_bolt_fixture_scenario());

        session.advance_turn();
        session.end_combat();

        let snapshot = session.snapshot();

        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(snapshot.lifecycle.started_at_step, Some(0));
        assert_eq!(snapshot.lifecycle.ended_at_step, Some(0));
        assert_eq!(snapshot.lifecycle_transition_log.len(), 1);
        assert_eq!(
            snapshot.lifecycle_transition_log[0].trigger,
            LifecycleTransitionTrigger::ExplicitEnd
        );
        assert_eq!(snapshot.lifecycle_transition_log[0].step_index, 0);
        assert_eq!(
            snapshot.lifecycle_transition_log[0].previous_phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(
            snapshot.lifecycle_transition_log[0].next_phase,
            CombatLifecyclePhase::Ended
        );
        assert_eq!(
            snapshot.lifecycle_transition_log[0].started_at_step,
            Some(0)
        );
        assert_eq!(snapshot.lifecycle_transition_log[0].ended_at_step, Some(0));
        assert_eq!(snapshot.turn_order.round_number, 1);
        assert_eq!(snapshot.turn_order.current_turn_index, 1);
        assert_eq!(
            snapshot.turn_order.current_actor_id,
            Some("entity-raider".to_string())
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
