use super::super::test_support::*;

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
    assert!(adept.stat_by_id("initiative").is_none());
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
            .modifier
            .as_ref()
            .map(|modifier| modifier.source_id.as_str()),
        Some("hexing_bolt")
    );
    assert_eq!(
        receipt
            .roll_consumption
            .iter()
            .map(|entry| (
                entry.sequence,
                entry.request_kind,
                entry.supplied_value,
                entry.consumed
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, RollRequestKind::AttackRoll, Some(17), true),
            (1, RollRequestKind::DamageRoll, Some(5), true),
        ]
    );
    assert_eq!(
        receipt
            .projection
            .as_ref()
            .map(|projection| projection.combatants[1].hit_points.current),
        Some(9)
    );
}

#[test]
fn second_ruleset_selects_turn_control_without_forking_resolution() {
    let scenario = turn_control_fixture_scenario();
    let ruleset = scenario
        .selected_ruleset()
        .expect("second ruleset is selected");

    assert_eq!(ruleset.id, "asha-rulebench.turn-control.v0");
    assert_eq!(ruleset.modules.len(), 2);
    assert!(ruleset
        .validate_modules()
        .expect("valid modules")
        .turn_control()
        .is_some());
    assert_eq!(
        ruleset.validate_artifact_provenance(&ruleset.artifact_provenance()),
        Ok(())
    );

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );

    assert!(receipt.accepted);
    assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
}

#[test]
fn ruleset_catalog_exposes_both_minimal_rulesets_by_identity() {
    let catalog = ruleset_catalog_readout();

    assert_eq!(catalog.selected_ruleset_id, "asha-rulebench.hexing-bolt.v0");
    assert_eq!(
        catalog
            .rulesets
            .iter()
            .map(|ruleset| ruleset.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "asha-rulebench.hexing-bolt.v0",
            "asha-rulebench.turn-control.v0",
        ]
    );
}

#[test]
fn resolver_rejects_rulesets_with_invalid_module_declarations() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.rulesets[0].modules.clear();

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );

    assert!(!receipt.accepted);
    assert_eq!(
        receipt.rejection,
        Some(RulebenchRejection::InvalidRulesetModules)
    );
    assert_eq!(
        RulebenchRejection::InvalidRulesetModules.code(),
        "invalidRulesetModules"
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
    scenario.actions[0]
        .attack_check_mut()
        .expect("fixture uses an attack check")
        .modifier = 99;

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
    assert_eq!(
        receipt
            .roll_consumption
            .iter()
            .map(|entry| (
                entry.sequence,
                entry.request_kind,
                entry.supplied_value,
                entry.consumed
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, RollRequestKind::AttackRoll, Some(9), true),
            (1, RollRequestKind::DamageRoll, Some(5), false),
        ]
    );
}

#[test]
fn resolver_uses_derived_actor_stat_for_attack_modifier() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0]
        .attack_check_mut()
        .expect("fixture uses an attack check")
        .modifier_stat_id = "initiative".to_string();

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[12, 5],
    );

    assert!(receipt.accepted);
    assert_eq!(
        receipt.attack_roll.as_ref().map(|roll| roll.modifier),
        Some(3)
    );
    assert_eq!(
        receipt.attack_roll.as_ref().map(|roll| roll.total),
        Some(15)
    );
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.message == "Derived attack stat evaluated."));
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.detail.contains("total 15 beats Nerve 13")));
}

#[test]
fn resolver_applies_effects_after_failed_saving_throw_and_records_the_decision() {
    let mut scenario = hexing_bolt_fixture_scenario();
    enable_check_handlers(&mut scenario, vec![CheckHandlerKind::SavingThrow]);
    scenario.actions[0].check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "mind".to_string(),
        difficulty_class: 12,
    });

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[10, 5],
    );

    assert!(receipt.accepted);
    assert!(receipt.attack_roll.is_none());
    assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
    assert!(matches!(
        receipt.events.as_slice(),
        [
            DomainEvent::ActionUsed { .. },
            DomainEvent::SavingThrowResolved {
                outcome: SavingThrowOutcome::Failed,
                ..
            },
            DomainEvent::DamageApplied { .. },
            DomainEvent::ModifierApplied { .. },
        ]
    ));
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.message == "Damage vitality resolved."));
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.detail.contains("ties save")));
}

#[test]
fn resolver_rejects_check_handlers_not_enabled_by_the_ruleset() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "mind".to_string(),
        difficulty_class: 12,
    });

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[10, 5],
    );

    assert!(!receipt.accepted);
    assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
    assert!(receipt.events.is_empty());
}

#[test]
fn resolver_tie_saving_throw_avoids_effects_with_one_consumed_roll() {
    let mut scenario = hexing_bolt_fixture_scenario();
    enable_check_handlers(&mut scenario, vec![CheckHandlerKind::SavingThrow]);
    scenario.actions[0].check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "mind".to_string(),
        difficulty_class: 12,
    });

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[11],
    );

    assert!(receipt.accepted);
    assert!(receipt.damage.is_none());
    assert_eq!(receipt.roll_consumption.len(), 1);
    assert!(matches!(
        receipt.events.as_slice(),
        [
            DomainEvent::ActionUsed { .. },
            DomainEvent::SavingThrowResolved {
                outcome: SavingThrowOutcome::Saved,
                ..
            },
        ]
    ));
}

#[test]
fn resolver_contested_tie_favors_target_and_actor_win_applies_effects() {
    let mut scenario = hexing_bolt_fixture_scenario();
    enable_check_handlers(&mut scenario, vec![CheckHandlerKind::Contested]);
    scenario.actions[0].check = CheckDeclaration::Contested(ContestedCheckDeclaration {
        actor_stat_id: "mind".to_string(),
        target_stat_id: "body".to_string(),
    });

    let tie_receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[7, 8],
    );
    assert!(tie_receipt.accepted);
    assert!(tie_receipt.damage.is_none());
    assert!(matches!(
        tie_receipt.events.as_slice(),
        [
            DomainEvent::ActionUsed { .. },
            DomainEvent::ContestedCheckResolved {
                outcome: ContestedCheckOutcome::TargetWins,
                ..
            },
        ]
    ));

    let win_receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[8, 1, 5],
    );
    assert!(win_receipt.accepted);
    assert_eq!(
        win_receipt.damage.as_ref().map(|damage| damage.amount),
        Some(9)
    );
    assert!(matches!(
        win_receipt.events.as_slice(),
        [
            DomainEvent::ActionUsed { .. },
            DomainEvent::ContestedCheckResolved {
                outcome: ContestedCheckOutcome::ActorWins,
                ..
            },
            DomainEvent::DamageApplied { .. },
            DomainEvent::ModifierApplied { .. },
        ]
    ));
    assert!(win_receipt
        .trace
        .iter()
        .any(|entry| entry.detail.contains("ties favor the target")));
}

#[test]
fn resolver_applies_typed_damage_adjustments_before_temporary_vitality() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].temporary_vitality = 3;
    scenario.entities[1].damage_adjustments = vec![DamageAdjustment {
        damage_type: "psychic".to_string(),
        policy: DamageAdjustmentPolicy::Resistance,
    }];

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );

    let damage = receipt.damage.expect("hit resolves damage");
    assert_eq!(damage.requested_amount, 9);
    assert_eq!(damage.amount, 4);
    assert_eq!(damage.temporary_vitality_absorbed, 3);
    assert_eq!(damage.after.current, 17);
    assert_eq!(damage.temporary_vitality_after, 0);
    assert_eq!(
        receipt
            .projection
            .map(|projection| projection.combatants[1].temporary_vitality),
        Some(0)
    );
}

#[test]
fn resolver_applies_immunity_and_vulnerability_to_terminal_damage() {
    let mut immune_scenario = hexing_bolt_fixture_scenario();
    immune_scenario.entities[1].damage_adjustments = vec![DamageAdjustment {
        damage_type: "psychic".to_string(),
        policy: DamageAdjustmentPolicy::Immunity,
    }];
    let immune_receipt = resolve_use_action(
        &immune_scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );
    assert_eq!(
        immune_receipt.damage.as_ref().map(|damage| damage.amount),
        Some(0)
    );
    assert_eq!(
        immune_receipt
            .projection
            .as_ref()
            .map(|projection| projection.combatants[1].hit_points.current),
        Some(18)
    );

    let mut vulnerable_scenario = hexing_bolt_fixture_scenario();
    vulnerable_scenario.combatants[1].hit_points.current = 10;
    vulnerable_scenario.entities[1].damage_adjustments = vec![DamageAdjustment {
        damage_type: "psychic".to_string(),
        policy: DamageAdjustmentPolicy::Vulnerability,
    }];
    let vulnerable_receipt = resolve_use_action(
        &vulnerable_scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );
    assert_eq!(
        vulnerable_receipt
            .damage
            .as_ref()
            .map(|damage| damage.amount),
        Some(18)
    );
    assert_eq!(
        vulnerable_receipt
            .projection
            .as_ref()
            .map(|projection| projection.combatants[1].hit_points.current),
        Some(0)
    );
}

#[test]
fn resolver_applies_capped_healing_and_replace_only_temporary_vitality() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[1].hit_points.current = 16;
    scenario.combatants[1].temporary_vitality = 4;
    scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::Heal(HealingEffectOperation {
            healing_bonus: 99,
            healing_type: "vitality".to_string(),
        }));
    scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::GrantTemporaryVitality(
            TemporaryVitalityEffectOperation { vitality_bonus: 10 },
        ));

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );

    assert_eq!(
        receipt.damage.as_ref().map(|damage| damage.after.current),
        Some(11)
    );
    assert_eq!(
        receipt.healing.as_ref().map(|healing| healing.amount),
        Some(7)
    );
    assert_eq!(
        receipt
            .temporary_vitality
            .as_ref()
            .map(|vitality| (vitality.before, vitality.after)),
        Some((0, 10))
    );
    assert!(matches!(
        receipt.events.as_slice(),
        [
            DomainEvent::ActionUsed { .. },
            DomainEvent::AttackRolled { .. },
            DomainEvent::DamageApplied { .. },
            DomainEvent::HealingApplied { .. },
            DomainEvent::TemporaryVitalityGranted { .. },
            DomainEvent::ModifierApplied { .. },
        ]
    ));
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.message == "Healing vitality resolved."));
    assert!(receipt
        .trace
        .iter()
        .any(|entry| entry.message == "Temporary vitality resolved."));
    assert_eq!(
        receipt.projection.map(|projection| (
            projection.combatants[1].hit_points.current,
            projection.combatants[1].temporary_vitality,
        )),
        Some((18, 10))
    );
}

fn enable_check_handlers(scenario: &mut RulebenchScenario, handlers: Vec<CheckHandlerKind>) {
    let ruleset = scenario
        .rulesets
        .first_mut()
        .expect("fixture has a ruleset");
    let declaration = ruleset
        .modules
        .iter_mut()
        .find(|declaration| declaration.module == RuleModuleId::ActionResolution)
        .expect("fixture has action resolution");
    let RuleModuleConfiguration::ActionResolution(configuration) = &mut declaration.configuration
    else {
        panic!("fixture action resolution declaration has the right configuration");
    };
    configuration.supported_check_handlers = handlers;
}

#[test]
fn resolver_rejects_missing_attack_modifier_stat_source() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0]
        .attack_check_mut()
        .expect("fixture uses an attack check")
        .modifier_stat_id = "missing_mind".to_string();

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
    assert_eq!(
        receipt
            .roll_consumption
            .iter()
            .map(|entry| (
                entry.sequence,
                entry.request_kind,
                entry.supplied_value,
                entry.consumed
            ))
            .collect::<Vec<_>>(),
        vec![(0, RollRequestKind::AttackRoll, None, false)]
    );
}

#[test]
fn resolver_rejects_missing_damage_roll_with_consumption_evidence() {
    let receipt = resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17],
    );

    assert!(!receipt.accepted);
    assert_eq!(
        receipt.rejection,
        Some(RulebenchRejection::MissingDamageRoll)
    );
    assert!(receipt.events.is_empty());
    assert!(receipt.damage.is_none());
    assert_eq!(
        receipt
            .roll_consumption
            .iter()
            .map(|entry| (
                entry.sequence,
                entry.request_kind,
                entry.supplied_value,
                entry.consumed
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, RollRequestKind::AttackRoll, Some(17), true),
            (1, RollRequestKind::DamageRoll, None, false),
        ]
    );
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
fn typescript_authored_action_executes_without_a_parallel_legacy_catalog_entry() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions.clear();

    let receipt = resolve_use_action(
        &scenario,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    );

    assert!(receipt.accepted);
    assert_eq!(receipt.authority_surface, ASHA_RPG_AUTHORITY_SURFACE);
    assert_eq!(receipt.attack_roll.map(|roll| roll.total), Some(25));
    assert!(receipt.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageApplied {
            target_id,
            amount: 9,
            ..
        } if target_id == "entity-raider"
    )));
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
            "hexing-bolt-reaction",
            "hexing-bolt-miss",
            "hexing-bolt-self-target-rejected",
            "hexing-bolt-veteran-hit",
            "binding-glyph-failed-save",
            "binding-glyph-saved",
            "watchtower-skirmish",
            "watchtower-storm-pulse-area",
            "watchtower-storm-pulse-multiple",
            "watchtower-vitality-operations"
        ]
    );
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.outcome_class.code())
            .collect::<Vec<_>>(),
        vec![
            "acceptedHit",
            "acceptedHit",
            "acceptedMiss",
            "rejectedTargetLegality",
            "acceptedHit",
            "acceptedHit",
            "acceptedMiss",
            "acceptedHit",
            "acceptedHit",
            "acceptedHit",
            "acceptedHit"
        ]
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

    assert_eq!(summaries.len(), 3);
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
    assert_eq!(summaries[1].id, "hexing-bolt-veteran-opening");
    assert_eq!(summaries[1].steps.len(), 1);
    assert_eq!(summaries[1].steps[0].id, "veteran-hexing-bolt-hit");
    assert_eq!(summaries[2].id, "objective-turn-control-opening");
    assert_eq!(summaries[2].steps.len(), 1);
    assert_eq!(summaries[2].steps[0].id, "warden-binding-glyph");
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
    let state = CombatState::from_scenario(&scenario);

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
    let mut state = CombatState::from_scenario(&scenario);

    assert_eq!(state.active_modifiers_for("entity-raider"), Some(&[][..]));
    state.apply_hit(damage, Some(modifier));
    state.apply_hit(damage, Some(modifier));
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
    let state = CombatState::from_projection(&first_step.state_after);

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
    let state = CombatState::from_projection(&miss_step.state_after);

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

    let next_scenario = CombatState::from_projection(projection).apply_to_scenario(scenario);

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
    let mut state = CombatState::from_scenario(&scenario);

    state.apply_hit(damage, Some(modifier));
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
