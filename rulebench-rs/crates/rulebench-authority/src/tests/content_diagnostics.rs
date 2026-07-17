use super::super::test_support::*;

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
fn content_diagnostics_report_invalid_ruleset_module_declarations() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.rulesets[0].modules.clear();

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::MissingRequiredRulesetModule
    );
    assert_eq!(
        ContentDiagnosticCode::MissingRequiredRulesetModule.code(),
        "missingRequiredRulesetModule"
    );
}

#[test]
fn content_diagnostics_reject_unimplemented_targeting_and_check_declarations() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].targeting.target_kind = TargetKind::Area;
    scenario.actions[0].targeting.selection = TargetSelection::Multiple;
    scenario.actions[0].check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "mind".to_string(),
        difficulty_class: 12,
    });

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            ContentDiagnosticCode::UnsupportedTargetingDeclaration,
            ContentDiagnosticCode::UnsupportedCheckDeclaration,
        ]
    );
    assert_eq!(
        ContentDiagnosticCode::UnsupportedTargetingDeclaration.code(),
        "unsupportedTargetingDeclaration"
    );
    assert_eq!(
        ContentDiagnosticCode::UnsupportedCheckDeclaration.code(),
        "unsupportedCheckDeclaration"
    );
}

#[test]
fn content_diagnostics_require_stateful_effects_to_use_operation_pipeline_v2() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::Move(MovementEffectOperation {
            maximum_distance: 3,
            movement_kind: MovementKind::Push,
        }));

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::InvalidEffectOperation
    );
    assert_eq!(
        ContentDiagnosticCode::InvalidEffectOperation.code(),
        "invalidEffectOperation"
    );
    assert!(diagnostics[0].message.contains("operation-pipeline v2"));
}

#[test]
fn content_diagnostics_reject_invalid_and_duplicate_action_resource_costs() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].resource_costs = vec![
        ActionResourceCost {
            resource_id: "standard-action".to_string(),
            amount: 0,
        },
        ActionResourceCost::standard_action(),
    ];

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::InvalidActionResourceCost
    );
    assert_eq!(
        diagnostics[1].code,
        ContentDiagnosticCode::DuplicateActionResourceCost
    );
    assert_eq!(
        ContentDiagnosticCode::InvalidActionResourceCost.code(),
        "invalidActionResourceCost"
    );
    assert_eq!(
        ContentDiagnosticCode::DuplicateActionResourceCost.code(),
        "duplicateActionResourceCost"
    );
}

#[test]
fn content_diagnostics_reject_malformed_resource_pools_and_missing_cost_targets() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0]
        .resource_pools
        .push(ActionResourcePool {
            id: String::new(),
            kind: ActionResourceKind::Charge,
            initial: 1,
            maximum: 1,
            refresh_policy: ActionResourceRefreshPolicy::Never,
        });
    scenario.combatants[0]
        .resource_pools
        .push(ActionResourcePool {
            id: "standard-action".to_string(),
            kind: ActionResourceKind::Cooldown,
            initial: 0,
            maximum: 0,
            refresh_policy: ActionResourceRefreshPolicy::Turns(0),
        });
    scenario.actions[0].resource_costs.push(ActionResourceCost {
        resource_id: "missing-pool".to_string(),
        amount: 1,
    });

    let diagnostics = validate_scenario_content(&scenario);
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert_eq!(
        codes,
        vec![
            ContentDiagnosticCode::EmptyActionResourcePoolId,
            ContentDiagnosticCode::DuplicateActionResourcePoolId,
            ContentDiagnosticCode::InvalidActionResourcePoolMaximum,
            ContentDiagnosticCode::InvalidActionResourceRefreshPolicy,
            ContentDiagnosticCode::MissingActionResourcePool,
        ]
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
        damage_adjustments: Vec::new(),
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
fn content_diagnostics_report_duplicate_combatant_ids() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants.push(scenario.combatants[0].clone());

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::DuplicateCombatantId
    );
    assert_eq!(diagnostics[0].content_id, Some("entity-adept".to_string()));
}

#[test]
fn content_diagnostics_reject_conflicting_damage_adjustments() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.entities[1].damage_adjustments = vec![
        DamageAdjustment {
            damage_type: "psychic".to_string(),
            policy: DamageAdjustmentPolicy::Resistance,
        },
        DamageAdjustment {
            damage_type: "psychic".to_string(),
            policy: DamageAdjustmentPolicy::Vulnerability,
        },
    ];

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::ConflictingDamageAdjustment
    );
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
    scenario.combatants[1].inventory_item_ids.clear();
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
        version: "1.0.0".to_string(),
        summary: "Invalid class fixture.".to_string(),
        tags: Vec::new(),
        prerequisites: Vec::new(),
        level_grants: Vec::new(),
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
    scenario.combatants[0].class_inputs = vec![ClassLevelInput {
        class_id: "class.missing".to_string(),
        version: "1.0.0".to_string(),
        level: 1,
    }];
    scenario.combatants[0].base_ability_ids = vec![
        "ability.hexing-bolt".to_string(),
        "ability.move".to_string(),
        "ability.basic-attack".to_string(),
    ];

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::MissingCombatantClass
    );
    assert_eq!(diagnostics[0].content_id, Some("class.missing".to_string()));
}

#[test]
fn content_diagnostics_reject_invalid_class_builds_and_grants() {
    let mut version = hexing_bolt_fixture_scenario();
    version.combatants[0].class_inputs[0].version = "2.0.0".to_string();
    assert!(validate_scenario_content(&version)
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::ClassVersionMismatch));

    let mut prerequisite = hexing_bolt_fixture_scenario();
    prerequisite.classes[0].prerequisites[0].minimum = 99;
    assert!(validate_scenario_content(&prerequisite)
        .iter()
        .any(|diagnostic| { diagnostic.code == ContentDiagnosticCode::ClassPrerequisiteNotMet }));

    let mut grants = hexing_bolt_fixture_scenario();
    let duplicate_grant = grants.classes[0].level_grants[0].clone();
    grants.classes[0].level_grants.push(duplicate_grant);
    grants.classes[0].level_grants[0]
        .granted_ability_ids
        .push("ability.missing".to_string());
    grants.classes[0].level_grants[0]
        .granted_modifier_ids
        .push("modifier.missing".to_string());
    let diagnostics = validate_scenario_content(&grants);
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::DuplicateClassGrantLevel));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::MissingClassGrantedAbility));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentDiagnosticCode::MissingClassGrantedModifier
    }));
}

#[test]
fn content_diagnostics_reject_malformed_reaction_windows() {
    let mut scenario = hexing_bolt_fixture_scenario();
    let hook = ReactionHookEffectOperation {
        hook_id: String::new(),
        window: ReactionWindow::BeforeEffect,
        eligible_reactor_ids: vec!["missing-reactor".to_string()],
        options: vec![
            ReactionOptionDeclaration {
                id: "duplicate".to_string(),
                reactor_id: "missing-reactor".to_string(),
                opens_nested_window: true,
            },
            ReactionOptionDeclaration {
                id: "duplicate".to_string(),
                reactor_id: "missing-reactor".to_string(),
                opens_nested_window: false,
            },
        ],
        maximum_nested_depth: 3,
    };
    scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::OpenReactionWindow(hook.clone()));
    scenario.actions[0]
        .hit
        .operations
        .push(HitEffectOperation::OpenReactionWindow(hook));

    let diagnostics = validate_scenario_content(&scenario);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::InvalidReactionHookId));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentDiagnosticCode::InvalidReactionEligibleReactor
    }));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::DuplicateReactionOption));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentDiagnosticCode::InvalidReactionNestedDepth
    }));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::DuplicateReactionHook));
}

#[test]
fn content_diagnostics_report_empty_stat_definition_id() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.stat_definitions.push(StatDefinition {
        id: String::new(),
        label: "Empty".to_string(),
        kind: StatDefinitionKind::Base,
        formula: None,
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
fn content_diagnostics_require_well_formed_derived_stat_formulas() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario
        .stat_definition_by_id("initiative")
        .expect("fixture has initiative");
    scenario
        .stat_definitions
        .iter_mut()
        .find(|definition| definition.id == "initiative")
        .expect("fixture has initiative")
        .formula = None;

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::MissingDerivedStatFormula
    );
    assert_eq!(
        ContentDiagnosticCode::MissingDerivedStatFormula.code(),
        "missingDerivedStatFormula"
    );
}

#[test]
fn content_diagnostics_reject_unknown_and_cyclic_derived_stat_references() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario
        .stat_definitions
        .iter_mut()
        .find(|definition| definition.id == "initiative")
        .expect("fixture has initiative")
        .formula = Some(DerivedStatFormula::StatReference {
        stat_id: "missing-stat".to_string(),
    });
    scenario.stat_definitions.push(StatDefinition {
        id: "focus".to_string(),
        label: "Focus".to_string(),
        kind: StatDefinitionKind::Derived,
        formula: Some(DerivedStatFormula::StatReference {
            stat_id: "focus".to_string(),
        }),
        summary: "Intentional cycle fixture.".to_string(),
    });

    let diagnostics = validate_scenario_content(&scenario);
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert_eq!(
        codes,
        vec![
            ContentDiagnosticCode::UnknownDerivedStatReference,
            ContentDiagnosticCode::DerivedStatFormulaCycle,
        ]
    );
}

#[test]
fn content_diagnostics_reject_malformed_and_authored_derived_values() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario
        .stat_definitions
        .iter_mut()
        .find(|definition| definition.id == "initiative")
        .expect("fixture has initiative")
        .formula = Some(DerivedStatFormula::Sum {
        operands: vec![DerivedStatFormula::Constant { value: 1 }],
    });
    scenario.combatants[0]
        .stats
        .derived_stats
        .push(NamedNumber {
            id: "initiative".to_string(),
            label: "Initiative".to_string(),
            value: 999,
        });

    let diagnostics = validate_scenario_content(&scenario);
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert_eq!(
        codes,
        vec![
            ContentDiagnosticCode::InvalidDerivedStatFormula,
            ContentDiagnosticCode::AuthoredDerivedStatValue,
        ]
    );
}

#[test]
fn content_diagnostics_reject_check_handlers_not_enabled_by_the_ruleset() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "mind".to_string(),
        difficulty_class: 12,
    });

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::UnsupportedCheckDeclaration
    );
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
        stacking_group: "invalid".to_string(),
        stacking_policy: ModifierStackingPolicy::Replace,
        duration_policy: ModifierDurationPolicy::Turns(1),
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
fn content_diagnostics_reject_invalid_modifier_duration_and_stacking_policies() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.modifiers[0].stacking_group.clear();
    scenario.modifiers[1].duration_policy = ModifierDurationPolicy::Rounds(0);
    scenario.modifiers[1].default_tenure = ModifierTenure::Temporary;

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::EmptyModifierStackingGroup
    );
    assert_eq!(
        diagnostics[1].code,
        ContentDiagnosticCode::InvalidModifierRoundDuration
    );
    assert_eq!(
        ContentDiagnosticCode::InvalidModifierRoundDuration.code(),
        "invalidModifierRoundDuration"
    );
}

#[test]
fn content_diagnostics_reject_empty_modifier_event_and_tenure_mismatch() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.modifiers[0].duration_policy = ModifierDurationPolicy::UntilEvent(String::new());
    scenario.modifiers[1].duration_policy = ModifierDurationPolicy::Turns(1);

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::EmptyModifierDurationEvent
    );
    assert_eq!(
        diagnostics[1].code,
        ContentDiagnosticCode::ModifierTenureDurationMismatch
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
    if let HitEffectOperation::ApplyModifier(modifier) = &mut scenario.actions[0].hit.operations[1]
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
fn content_diagnostics_reject_invalid_equipment_loadouts_and_grants() {
    let mut unowned = hexing_bolt_fixture_scenario();
    unowned.combatants[0].inventory_item_ids.clear();
    let unowned_diagnostics = validate_scenario_content(&unowned);
    assert!(unowned_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.code == ContentDiagnosticCode::EquippedItemNotOwned }));

    let mut slot_conflict = hexing_bolt_fixture_scenario();
    let mut second_focus = slot_conflict.items[0].clone();
    second_focus.id = "item.second-focus".to_string();
    slot_conflict.items.push(second_focus);
    slot_conflict.combatants[0]
        .inventory_item_ids
        .push("item.second-focus".to_string());
    slot_conflict.combatants[0]
        .equipped_item_ids
        .push("item.second-focus".to_string());
    let slot_diagnostics = validate_scenario_content(&slot_conflict);
    assert!(slot_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::EquipmentSlotConflict));

    let mut requirement = hexing_bolt_fixture_scenario();
    requirement.items[0].requirements[0].minimum = 99;
    let requirement_diagnostics = validate_scenario_content(&requirement);
    assert!(requirement_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentDiagnosticCode::EquipmentRequirementNotMet
    }));

    let mut grants = hexing_bolt_fixture_scenario();
    grants.items[0]
        .granted_modifier_ids
        .push("modifier.missing".to_string());
    grants.items[0]
        .granted_ability_ids
        .push("ability.missing".to_string());
    let grant_diagnostics = validate_scenario_content(&grants);
    assert!(grant_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentDiagnosticCode::MissingItemGrantedModifier
    }));
    assert!(grant_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.code == ContentDiagnosticCode::MissingItemGrantedAbility }));
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
    scenario.actions[0].targeting.target_ids = vec!["entity-missing-target".to_string()];
    scenario.actions[0].targeting.visible_target_ids = vec!["entity-missing-target".to_string()];

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
fn content_diagnostics_reject_cross_ruleset_action_references() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0].ruleset_id = "asha-rulebench.other.v0".to_string();

    let diagnostics = validate_scenario_content(&scenario);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ContentDiagnosticCode::CrossRulesetActionReference
    );
}

#[test]
fn content_diagnostics_report_visible_target_outside_target_ids() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.actions[0]
        .targeting
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
    scenario.actions[0]
        .attack_check_mut()
        .expect("fixture uses an attack check")
        .modifier_stat_id = "missing-mind".to_string();

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
    scenario.actions[0]
        .attack_check_mut()
        .expect("fixture uses an attack check")
        .defense
        .id = "missing-nerve".to_string();

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
fn content_diagnostics_reject_empty_combat_side_identity() {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.combatants[0].side_id.clear();

    let report = validate_scenario_content_report(&scenario);

    assert!(!report.accepted);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ContentDiagnosticCode::EmptyCombatantSide));
}

#[test]
fn content_diagnostics_reject_missing_objective_side() {
    let mut scenario = turn_control_fixture_scenario();
    scenario.rulesets[0].modules[1] = RuleModuleDeclaration::turn_control(
        TurnControlModuleConfiguration::explicit_turn_order_with_end_policy(
            CombatEndPolicy::ObjectiveSideVictory {
                side_id: "missing-side".to_string(),
            },
        ),
    );

    let report = validate_scenario_content_report(&scenario);

    assert!(!report.accepted);
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == ContentDiagnosticCode::MissingCombatEndObjectiveSide)
        .expect("missing objective side diagnostic");
    assert_eq!(diagnostic.content_id.as_deref(), Some("missing-side"));
}
