use super::super::test_support::*;

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
fn action_resource_transition_kind_codes_are_stable() {
    assert_eq!(ActionResourceTransitionKind::Spent.code(), "spent");
    assert_eq!(ActionResourceTransitionKind::Refreshed.code(), "refreshed");
    assert_eq!(
        ActionResourceTransitionKind::ChangedByEffect.code(),
        "changedByEffect"
    );
}

#[test]
fn modifier_duration_expiration_decision_kind_codes_are_stable() {
    assert_eq!(
        ModifierDurationExpirationDecisionKind::Expired.code(),
        "expired"
    );
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
    assert_eq!(scenario.actions.len(), 5);
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
        vec![
            "ability.hexing-bolt",
            "ability.move",
            "ability.basic-attack"
        ]
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
        scenario.combatants[0].class_inputs,
        vec![ClassLevelInput {
            class_id: "class.hex-adept".to_string(),
            version: "1.0.0".to_string(),
            level: 1,
        }]
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
            source_id: "legacy".to_string(),
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
            source_id: "legacy".to_string(),
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
            source_id: "legacy".to_string(),
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
            source_id: "legacy".to_string(),
            modifier_label: "battle drilled".to_string(),
            tenure: ModifierTenure::Permanent,
            stat_id: "initiative".to_string(),
            stat_label: "Initiative".to_string(),
            delta: 1,
        }]
    );
}

#[test]
fn derived_stats_evaluate_from_effective_inputs_and_explain_the_formula() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0]
        .active_modifiers
        .push(ActiveModifier::temporary("rattled", "rattled", "one turn"));

    let readout = evaluate_effective_stats_for_combatant(&scenario, "entity-adept")
        .expect("fixture formula evaluates");
    let initiative = readout
        .stats
        .iter()
        .find(|stat| stat.stat_id == "initiative")
        .expect("derived initiative is evaluated");

    assert_eq!(initiative.kind, StatDefinitionKind::Derived);
    assert!(matches!(
        initiative.formula,
        Some(DerivedStatFormula::Difference { .. })
    ));
    assert_eq!(initiative.base_value, 2);
    assert_eq!(initiative.total_modifier_delta, 0);
    assert_eq!(initiative.effective_value, 2);
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
