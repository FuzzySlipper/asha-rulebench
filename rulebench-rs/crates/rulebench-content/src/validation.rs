use std::collections::{HashMap, HashSet};

use crate::{
    Combatant, ContentDiagnostic, ContentDiagnosticCode, ContentDiagnosticSeverity,
    ContentValidationReport, ModifierDurationPolicy, RulebenchScenario, StatDefinitionKind,
};
use rulebench_ruleset::{
    ActionDefinition, ActionResourceRefreshPolicy, AttackCheckDeclaration, CheckDeclaration,
    ContestedCheckDeclaration, ModifierTenure, RuleModuleValidationError,
    SavingThrowCheckDeclaration, TargetKind, TargetSelection,
};

pub fn validate_scenario_content_report(scenario: &RulebenchScenario) -> ContentValidationReport {
    ContentValidationReport::from_diagnostics(validate_scenario_content(scenario))
}

pub fn validate_scenario_content(scenario: &RulebenchScenario) -> Vec<ContentDiagnostic> {
    let mut diagnostics = Vec::new();

    validate_rulesets(scenario, &mut diagnostics);
    validate_entities(scenario, &mut diagnostics);
    validate_abilities(scenario, &mut diagnostics);
    validate_classes(scenario, &mut diagnostics);
    validate_stat_definitions(scenario, &mut diagnostics);
    validate_modifiers(scenario, &mut diagnostics);
    validate_combatant_class_and_stat_references(scenario, &mut diagnostics);
    validate_items(scenario, &mut diagnostics);

    let mut seen_action_ids = HashSet::new();
    for action in &scenario.actions {
        if action.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyActionId,
                None,
                "Action catalog contains an action with an empty id.",
            ));
            continue;
        }

        if !seen_action_ids.insert(action.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateActionId,
                Some(action.id.clone()),
                format!("Action id {} appears more than once.", action.id),
            ));
        }

        validate_action_references(scenario, action, &mut diagnostics);
    }

    if !scenario
        .actions
        .iter()
        .any(|action| action.id == scenario.selected_action.id)
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::SelectedActionMissingFromCatalog,
            Some(scenario.selected_action.id.clone()),
            format!(
                "Selected action {} is not present in the scenario action catalog.",
                scenario.selected_action.id
            ),
        ));
    }

    diagnostics
}

fn validate_rulesets(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_ruleset_ids = HashSet::new();
    for ruleset in &scenario.rulesets {
        if ruleset.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyRulesetId,
                None,
                "Ruleset catalog contains a ruleset with an empty id.",
            ));
            continue;
        }

        if !seen_ruleset_ids.insert(ruleset.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateRulesetId,
                Some(ruleset.id.clone()),
                format!("Ruleset id {} appears more than once.", ruleset.id),
            ));
        }

        if let Err(error) = ruleset.validate_modules() {
            diagnostics.push(ContentDiagnostic::error(
                ruleset_module_validation_code(&error),
                Some(ruleset.id.clone()),
                format!(
                    "Ruleset {} has an invalid behavior-module declaration: {}.",
                    ruleset.id,
                    error.code()
                ),
            ));
        }
    }

    if scenario
        .ruleset_by_id(&scenario.selected_ruleset_id)
        .is_none()
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog,
            Some(scenario.selected_ruleset_id.clone()),
            format!(
                "Selected ruleset {} is not present in the scenario ruleset catalog.",
                scenario.selected_ruleset_id
            ),
        ));
    }
}

fn ruleset_module_validation_code(error: &RuleModuleValidationError) -> ContentDiagnosticCode {
    match error {
        RuleModuleValidationError::UnknownModuleCode { .. } => {
            ContentDiagnosticCode::UnknownRulesetModule
        }
        RuleModuleValidationError::MissingRequiredModule { .. } => {
            ContentDiagnosticCode::MissingRequiredRulesetModule
        }
        RuleModuleValidationError::DuplicateModuleDeclaration { .. } => {
            ContentDiagnosticCode::DuplicateRulesetModule
        }
        RuleModuleValidationError::IncompatibleModuleVersion { .. } => {
            ContentDiagnosticCode::IncompatibleRulesetModuleVersion
        }
        RuleModuleValidationError::ConfigurationDoesNotMatchModule { .. } => {
            ContentDiagnosticCode::RulesetModuleConfigurationMismatch
        }
    }
}

fn validate_entities(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_entity_ids = HashSet::new();
    for entity in &scenario.entities {
        if entity.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyEntityId,
                None,
                "Entity catalog contains an entity with an empty id.",
            ));
            continue;
        }

        if !seen_entity_ids.insert(entity.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateEntityId,
                Some(entity.id.clone()),
                format!("Entity id {} appears more than once.", entity.id),
            ));
        }

        let mut adjustments_by_type = HashMap::new();
        for adjustment in &entity.damage_adjustments {
            if adjustment.damage_type.is_empty() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::EmptyDamageAdjustmentType,
                    Some(entity.id.clone()),
                    format!(
                        "Entity {} declares a damage adjustment with an empty type.",
                        entity.id
                    ),
                ));
                continue;
            }
            if adjustments_by_type
                .insert(adjustment.damage_type.clone(), adjustment.policy)
                .is_some()
            {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::ConflictingDamageAdjustment,
                    Some(adjustment.damage_type.clone()),
                    format!(
                        "Entity {} declares more than one adjustment for damage type {}.",
                        entity.id, adjustment.damage_type
                    ),
                ));
            }
        }
    }
}

fn validate_abilities(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_ability_ids = HashSet::new();
    for ability in &scenario.abilities {
        if ability.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyAbilityId,
                None,
                "Ability catalog contains an ability with an empty id.",
            ));
            continue;
        }

        if !seen_ability_ids.insert(ability.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateAbilityId,
                Some(ability.id.clone()),
                format!("Ability id {} appears more than once.", ability.id),
            ));
        }
    }

    if let Some(selected_ability_id) = &scenario.selected_ability_id {
        if scenario.ability_by_id(selected_ability_id).is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::SelectedAbilityMissingFromCatalog,
                Some(selected_ability_id.clone()),
                format!(
                    "Selected ability {} is not present in the scenario ability catalog.",
                    selected_ability_id
                ),
            ));
        }
    }
}

fn validate_classes(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_class_ids = HashSet::new();
    for class in &scenario.classes {
        if class.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyClassId,
                None,
                "Class catalog contains a class with an empty id.",
            ));
            continue;
        }

        if !seen_class_ids.insert(class.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateClassId,
                Some(class.id.clone()),
                format!("Class id {} appears more than once.", class.id),
            ));
        }
    }

    if let Some(selected_class_id) = &scenario.selected_class_id {
        if scenario.class_by_id(selected_class_id).is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::SelectedClassMissingFromCatalog,
                Some(selected_class_id.clone()),
                format!(
                    "Selected class {} is not present in the scenario class catalog.",
                    selected_class_id
                ),
            ));
        }
    }
}

fn validate_stat_definitions(
    scenario: &RulebenchScenario,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let mut seen_stat_ids = HashSet::new();
    for stat in &scenario.stat_definitions {
        if stat.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyStatDefinitionId,
                None,
                "Stat catalog contains a stat definition with an empty id.",
            ));
            continue;
        }

        if !seen_stat_ids.insert(stat.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateStatDefinitionId,
                Some(stat.id.clone()),
                format!("Stat definition id {} appears more than once.", stat.id),
            ));
        }
    }

    let definitions = scenario
        .stat_definitions
        .iter()
        .filter(|stat| !stat.id.is_empty())
        .map(|stat| (stat.id.as_str(), stat))
        .collect::<HashMap<_, _>>();

    for stat in &scenario.stat_definitions {
        match (&stat.kind, &stat.formula) {
            (StatDefinitionKind::Base, Some(_)) => diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::BaseStatFormulaNotAllowed,
                Some(stat.id.clone()),
                format!("Base stat {} must not declare a derived formula.", stat.id),
            )),
            (StatDefinitionKind::Derived, None) => diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingDerivedStatFormula,
                Some(stat.id.clone()),
                format!("Derived stat {} must declare a formula.", stat.id),
            )),
            (_, Some(formula)) => {
                if !formula.shape_is_valid() {
                    diagnostics.push(ContentDiagnostic::error(
                        ContentDiagnosticCode::InvalidDerivedStatFormula,
                        Some(stat.id.clone()),
                        format!(
                            "Derived stat {} uses malformed {} formula operands.",
                            stat.id,
                            formula.code()
                        ),
                    ));
                }

                for referenced_stat_id in formula.referenced_stat_ids() {
                    if !definitions.contains_key(referenced_stat_id) {
                        diagnostics.push(ContentDiagnostic::error(
                            ContentDiagnosticCode::UnknownDerivedStatReference,
                            Some(referenced_stat_id.to_string()),
                            format!(
                                "Derived stat {} references unknown stat {}.",
                                stat.id, referenced_stat_id
                            ),
                        ));
                    }
                }
            }
            (StatDefinitionKind::Base, None) => {}
        }
    }

    let mut visiting = Vec::new();
    let mut visited = HashSet::new();
    for stat in &scenario.stat_definitions {
        if stat.kind == StatDefinitionKind::Derived {
            validate_derived_formula_cycles(
                stat.id.as_str(),
                &definitions,
                &mut visiting,
                &mut visited,
                diagnostics,
            );
        }
    }
}

fn validate_derived_formula_cycles(
    stat_id: &str,
    definitions: &HashMap<&str, &crate::StatDefinition>,
    visiting: &mut Vec<String>,
    visited: &mut HashSet<String>,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if visited.contains(stat_id) {
        return;
    }
    if let Some(cycle_start) = visiting.iter().position(|entry| entry == stat_id) {
        let mut cycle = visiting[cycle_start..].to_vec();
        cycle.push(stat_id.to_string());
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::DerivedStatFormulaCycle,
            Some(stat_id.to_string()),
            format!(
                "Derived stat formula cycle detected: {}.",
                cycle.join(" -> ")
            ),
        ));
        return;
    }

    let Some(stat) = definitions.get(stat_id) else {
        return;
    };
    let Some(formula) = stat.formula.as_ref() else {
        return;
    };

    visiting.push(stat_id.to_string());
    for referenced_stat_id in formula.referenced_stat_ids() {
        if definitions
            .get(referenced_stat_id)
            .is_some_and(|definition| definition.kind == StatDefinitionKind::Derived)
        {
            validate_derived_formula_cycles(
                referenced_stat_id,
                definitions,
                visiting,
                visited,
                diagnostics,
            );
        }
    }
    visiting.pop();
    visited.insert(stat_id.to_string());
}

fn validate_modifiers(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_modifier_ids = HashSet::new();
    for modifier in &scenario.modifiers {
        if modifier.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyModifierId,
                None,
                "Modifier catalog contains a modifier with an empty id.",
            ));
            continue;
        }

        if !seen_modifier_ids.insert(modifier.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateModifierId,
                Some(modifier.id.clone()),
                format!("Modifier id {} appears more than once.", modifier.id),
            ));
        }

        if modifier.stacking_group.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyModifierStackingGroup,
                Some(modifier.id.clone()),
                format!(
                    "Modifier {} must declare a non-empty stacking group.",
                    modifier.id
                ),
            ));
        }
        match (&modifier.default_tenure, &modifier.duration_policy) {
            (_, ModifierDurationPolicy::Turns(0)) => diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::InvalidModifierTurnDuration,
                Some(modifier.id.clone()),
                format!("Modifier {} must declare at least one turn.", modifier.id),
            )),
            (_, ModifierDurationPolicy::Rounds(0)) => diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::InvalidModifierRoundDuration,
                Some(modifier.id.clone()),
                format!("Modifier {} must declare at least one round.", modifier.id),
            )),
            (_, ModifierDurationPolicy::UntilEvent(event)) if event.is_empty() => {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::EmptyModifierDurationEvent,
                    Some(modifier.id.clone()),
                    format!(
                        "Modifier {} must declare a non-empty duration event.",
                        modifier.id
                    ),
                ));
            }
            (ModifierTenure::Permanent, ModifierDurationPolicy::Permanent)
            | (ModifierTenure::Temporary, ModifierDurationPolicy::Turns(_))
            | (ModifierTenure::Temporary, ModifierDurationPolicy::Rounds(_))
            | (ModifierTenure::Temporary, ModifierDurationPolicy::UntilEvent(_)) => {}
            _ => diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::ModifierTenureDurationMismatch,
                Some(modifier.id.clone()),
                format!(
                    "Modifier {} has incompatible tenure and duration policy.",
                    modifier.id
                ),
            )),
        }

        for adjustment in &modifier.stat_adjustments {
            if scenario
                .stat_definition_by_id(&adjustment.stat_id)
                .is_none()
            {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingModifierStatAdjustmentTarget,
                    Some(adjustment.stat_id.clone()),
                    format!(
                        "Modifier {} adjusts stat {} that is not present in the scenario stat catalog.",
                        modifier.id, adjustment.stat_id
                    ),
                ));
            }
        }
    }
}

fn validate_combatant_class_and_stat_references(
    scenario: &RulebenchScenario,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let mut seen_combatant_ids = HashSet::new();
    for combatant in &scenario.combatants {
        validate_combatant_resource_pools(combatant, diagnostics);
        if combatant.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyCombatantId,
                None,
                "Scenario contains a combatant with an empty id.",
            ));
        } else if !seen_combatant_ids.insert(combatant.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateCombatantId,
                Some(combatant.id.clone()),
                format!("Combatant id {} appears more than once.", combatant.id),
            ));
        }
        if scenario.entity_by_id(&combatant.entity_id).is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingCombatantEntity,
                Some(combatant.entity_id.clone()),
                format!(
                    "Combatant {} references entity {} that is not present in the scenario entity catalog.",
                    combatant.id, combatant.entity_id
                ),
            ));
        }

        for class_id in &combatant.class_ids {
            if scenario.class_by_id(class_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingCombatantClass,
                    Some(class_id.clone()),
                    format!(
                        "Combatant {} references class {} that is not present in the scenario class catalog.",
                        combatant.id, class_id
                    ),
                ));
            }
        }

        let mut seen_base_stat_ids = HashSet::new();
        for stat in &combatant.stats.base_stats {
            if scenario.stat_definition_by_id(&stat.id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingCombatantStatDefinition,
                    Some(stat.id.clone()),
                    format!(
                        "Combatant {} has stat {} that is not present in the scenario stat catalog.",
                        combatant.id, stat.id
                    ),
                ));
            }
            if !seen_base_stat_ids.insert(stat.id.clone()) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::DuplicateCombatantBaseStat,
                    Some(stat.id.clone()),
                    format!(
                        "Combatant {} declares base stat {} more than once.",
                        combatant.id, stat.id
                    ),
                ));
            }
        }

        for definition in scenario.stat_definitions.iter().filter(|definition| {
            definition.kind == StatDefinitionKind::Base && !definition.id.is_empty()
        }) {
            if !seen_base_stat_ids.contains(&definition.id) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingCombatantBaseStat,
                    Some(definition.id.clone()),
                    format!(
                        "Combatant {} is missing base stat {}.",
                        combatant.id, definition.id
                    ),
                ));
            }
        }

        for stat in &combatant.stats.derived_stats {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::AuthoredDerivedStatValue,
                Some(stat.id.clone()),
                format!(
                    "Combatant {} declares derived stat {} as input; derived values come from formulas.",
                    combatant.id, stat.id
                ),
            ));
        }

        for modifier in &combatant.active_modifiers {
            if scenario.modifier_by_id(&modifier.modifier_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingActiveModifierDefinition,
                    Some(modifier.modifier_id.clone()),
                    format!(
                        "Combatant {} has active modifier {} that is not present in the scenario modifier catalog.",
                        combatant.id, modifier.modifier_id
                    ),
                ));
            }
        }
    }
}

fn validate_items(scenario: &RulebenchScenario, diagnostics: &mut Vec<ContentDiagnostic>) {
    let mut seen_item_ids = HashSet::new();
    for item in &scenario.items {
        if item.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyItemId,
                None,
                "Item catalog contains an item with an empty id.",
            ));
            continue;
        }

        if !seen_item_ids.insert(item.id.clone()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateItemId,
                Some(item.id.clone()),
                format!("Item id {} appears more than once.", item.id),
            ));
        }
        if item.equipment_slot.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyItemEquipmentSlot,
                Some(item.id.clone()),
                format!("Item {} has an empty equipment slot.", item.id),
            ));
        }
        for requirement in &item.requirements {
            if scenario
                .stat_definition_by_id(&requirement.stat_id)
                .is_none()
            {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingItemRequirementStat,
                    Some(item.id.clone()),
                    format!(
                        "Item {} requires stat {} that is not present in the stat catalog.",
                        item.id, requirement.stat_id
                    ),
                ));
            }
        }
        for modifier_id in &item.granted_modifier_ids {
            if scenario.modifier_by_id(modifier_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingItemGrantedModifier,
                    Some(item.id.clone()),
                    format!("Item {} grants missing modifier {}.", item.id, modifier_id),
                ));
            }
        }
        for ability_id in &item.granted_ability_ids {
            if scenario.ability_by_id(ability_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingItemGrantedAbility,
                    Some(item.id.clone()),
                    format!("Item {} grants missing ability {}.", item.id, ability_id),
                ));
            }
        }
        let mut granted_resource_ids = HashSet::new();
        for pool in &item.granted_resource_pools {
            if pool.id.is_empty() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::EmptyActionResourcePoolId,
                    Some(item.id.clone()),
                    format!("Item {} grants a resource pool with an empty id.", item.id),
                ));
                continue;
            }
            if !granted_resource_ids.insert(pool.id.clone()) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::DuplicateActionResourcePoolId,
                    Some(pool.id.clone()),
                    format!(
                        "Item {} grants resource pool {} more than once.",
                        item.id, pool.id
                    ),
                ));
            }
            if pool.maximum == 0 || i32::try_from(pool.maximum).is_err() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::InvalidActionResourcePoolMaximum,
                    Some(pool.id.clone()),
                    format!(
                        "Item {} resource pool {} has unsupported maximum {}.",
                        item.id, pool.id, pool.maximum
                    ),
                ));
            }
            if pool.refresh_policy == ActionResourceRefreshPolicy::Turns(0) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::InvalidActionResourceRefreshPolicy,
                    Some(pool.id.clone()),
                    format!(
                        "Item {} resource pool {} declares a zero-turn refresh clock.",
                        item.id, pool.id
                    ),
                ));
            }
        }
    }

    if let Some(selected_item_id) = &scenario.selected_item_id {
        if scenario.item_by_id(selected_item_id).is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::SelectedItemMissingFromCatalog,
                Some(selected_item_id.clone()),
                format!(
                    "Selected item {} is not present in the scenario item catalog.",
                    selected_item_id
                ),
            ));
        }
    }

    for combatant in &scenario.combatants {
        let mut inventory_ids = HashSet::new();
        for item_id in &combatant.inventory_item_ids {
            if !inventory_ids.insert(item_id.clone()) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::DuplicateInventoryItem,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} owns item {} more than once.",
                        combatant.id, item_id
                    ),
                ));
            }
            if scenario.item_by_id(item_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingInventoryItem,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} owns item {} that is not present in the item catalog.",
                        combatant.id, item_id
                    ),
                ));
            }
        }
        let mut equipped_ids = HashSet::new();
        let mut equipped_slots = HashMap::new();
        let mut resource_pool_ids = combatant
            .resource_pools
            .iter()
            .map(|pool| pool.id.clone())
            .collect::<HashSet<_>>();
        for item_id in &combatant.equipped_item_ids {
            if !equipped_ids.insert(item_id.clone()) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::DuplicateEquippedItem,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} equips item {} more than once.",
                        combatant.id, item_id
                    ),
                ));
            }
            let Some(item) = scenario.item_by_id(item_id) else {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingEquippedItem,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} equips item {} that is not present in the scenario item catalog.",
                        combatant.id, item_id
                    ),
                ));
                continue;
            };
            if !inventory_ids.contains(item_id) {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::EquippedItemNotOwned,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} equips unowned item {}.",
                        combatant.id, item_id
                    ),
                ));
            }
            if let Some(other_item_id) =
                equipped_slots.insert(item.equipment_slot.clone(), item.id.clone())
            {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::EquipmentSlotConflict,
                    Some(item.id.clone()),
                    format!(
                        "Combatant {} equips items {} and {} in slot {}.",
                        combatant.id, other_item_id, item.id, item.equipment_slot
                    ),
                ));
            }
            for requirement in &item.requirements {
                let meets_requirement = combatant
                    .stat_by_id(&requirement.stat_id)
                    .is_some_and(|stat| stat.value >= requirement.minimum);
                if !meets_requirement {
                    diagnostics.push(ContentDiagnostic::error(
                        ContentDiagnosticCode::EquipmentRequirementNotMet,
                        Some(item.id.clone()),
                        format!(
                            "Combatant {} does not meet item {} requirement {} >= {}.",
                            combatant.id, item.id, requirement.stat_id, requirement.minimum
                        ),
                    ));
                }
            }
            for pool in &item.granted_resource_pools {
                if !resource_pool_ids.insert(pool.id.clone()) {
                    diagnostics.push(ContentDiagnostic::error(
                        ContentDiagnosticCode::EquipmentResourcePoolConflict,
                        Some(item.id.clone()),
                        format!(
                            "Combatant {} receives duplicate resource pool {} from item {}.",
                            combatant.id, pool.id, item.id
                        ),
                    ));
                }
            }
        }
        for ability_id in &combatant.base_ability_ids {
            if scenario.ability_by_id(ability_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingBaseAbility,
                    Some(ability_id.clone()),
                    format!(
                        "Combatant {} has missing base ability {}.",
                        combatant.id, ability_id
                    ),
                ));
            }
        }
    }
}

fn validate_action_references(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if scenario.ability_by_id(&action.ability_id).is_none() {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingActionAbility,
            Some(action.ability_id.clone()),
            format!(
                "Action {} references ability {} that is not present in the scenario ability catalog.",
                action.id, action.ability_id
            ),
        ));
    }

    if !scenario.selected_ruleset_id.is_empty()
        && scenario
            .selected_ruleset()
            .is_some_and(|ruleset| !ruleset.id.is_empty())
        && action.ruleset_id != scenario.selected_ruleset_id
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::CrossRulesetActionReference,
            Some(action.id.clone()),
            format!(
                "Action {} belongs to ruleset {} but scenario selected ruleset is {}.",
                action.id, action.ruleset_id, scenario.selected_ruleset_id
            ),
        ));
    }

    let actor = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == action.actor_id);
    if actor.is_none() {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingActionActor,
            Some(action.actor_id.clone()),
            format!(
                "Action {} references actor {} that is not present in combatants.",
                action.id, action.actor_id
            ),
        ));
    }

    if action.targeting.target_kind != TargetKind::Combatant
        || action.targeting.selection != TargetSelection::Single
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::UnsupportedTargetingDeclaration,
            Some(action.id.clone()),
            format!(
                "Action {} declares targeting that the current action-resolution module does not support.",
                action.id
            ),
        ));
    }

    let check_is_supported = scenario
        .ruleset_by_id(&action.ruleset_id)
        .and_then(|ruleset| ruleset.validate_modules().ok())
        .map(|registry| registry.action_resolution().supports_check(&action.check));
    if check_is_supported == Some(false) {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::UnsupportedCheckDeclaration,
            Some(action.id.clone()),
            format!(
                "Action {} declares a check that the current action-resolution module does not support.",
                action.id
            ),
        ));
    }

    let target_ids = action
        .targeting
        .target_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for target_id in &action.targeting.target_ids {
        let target = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == *target_id);
        if target.is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingActionTarget,
                Some(target_id.clone()),
                format!(
                    "Action {} references target {} that is not present in combatants.",
                    action.id, target_id
                ),
            ));
        }
    }

    for visible_target_id in &action.targeting.visible_target_ids {
        if !target_ids.contains(visible_target_id.as_str()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::VisibleTargetOutsideTargetIds,
                Some(visible_target_id.clone()),
                format!(
                    "Action {} marks {} visible but does not list it as a target.",
                    action.id, visible_target_id
                ),
            ));
        }
    }

    if let Some(actor) = actor {
        let base_ability = actor.base_ability_ids.contains(&action.ability_id);
        let item_ability = actor.inventory_item_ids.iter().any(|item_id| {
            scenario
                .item_by_id(item_id)
                .is_some_and(|item| item.granted_ability_ids.contains(&action.ability_id))
        });
        if scenario.ability_by_id(&action.ability_id).is_some() && !base_ability && !item_ability {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingActionAbilityGrant,
                Some(action.id.clone()),
                format!(
                    "Action {} ability {} is not granted to actor {} by base content or inventory.",
                    action.id, action.ability_id, actor.id
                ),
            ));
        }
        match &action.check {
            CheckDeclaration::Attack(attack) => {
                validate_actor_attack_stat(scenario, action, attack, actor, diagnostics);
            }
            CheckDeclaration::SavingThrow(save) => {
                validate_saving_throw(scenario, action, save, diagnostics);
            }
            CheckDeclaration::Contested(contested) => {
                validate_contested_actor_stat(scenario, action, contested, actor, diagnostics);
            }
        }
        validate_action_resource_costs(scenario, action, actor, diagnostics);
    }
    validate_hit_modifier(scenario, action, diagnostics);
    validate_effect_operations(action, diagnostics);

    for target_id in &action.targeting.target_ids {
        if let Some(target) = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == *target_id)
        {
            match &action.check {
                CheckDeclaration::Attack(attack) => {
                    validate_target_defense(action, attack, target, diagnostics);
                }
                CheckDeclaration::SavingThrow(save) => {
                    validate_combatant_check_stat(
                        scenario,
                        action,
                        target,
                        &save.save_stat_id,
                        ContentDiagnosticCode::MissingSavingThrowStat,
                        "saving throw",
                        diagnostics,
                    );
                }
                CheckDeclaration::Contested(contested) => {
                    validate_combatant_check_stat(
                        scenario,
                        action,
                        target,
                        &contested.target_stat_id,
                        ContentDiagnosticCode::MissingContestedTargetStat,
                        "contested target",
                        diagnostics,
                    );
                }
            }
        }
    }
}

fn validate_action_resource_costs(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    actor: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let mut seen_resource_ids = HashSet::new();
    for cost in &action.resource_costs {
        if cost.resource_id.is_empty() || cost.amount == 0 {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::InvalidActionResourceCost,
                Some(action.id.clone()),
                format!(
                    "Action {} declares a zero {} resource cost.",
                    action.id, cost.resource_id
                ),
            ));
        }

        let base_pool_exists = actor
            .resource_pools
            .iter()
            .any(|pool| pool.id == cost.resource_id);
        let inventory_pool_exists = actor.inventory_item_ids.iter().any(|item_id| {
            scenario.item_by_id(item_id).is_some_and(|item| {
                item.granted_resource_pools
                    .iter()
                    .any(|pool| pool.id == cost.resource_id)
            })
        });
        if !base_pool_exists && !inventory_pool_exists {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingActionResourcePool,
                Some(cost.resource_id.clone()),
                format!(
                    "Action {} references resource pool {} that actor {} does not own.",
                    action.id, cost.resource_id, actor.id
                ),
            ));
        }

        if !seen_resource_ids.insert(cost.resource_id.as_str()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateActionResourceCost,
                Some(action.id.clone()),
                format!(
                    "Action {} declares {} more than once in its resource costs.",
                    action.id, cost.resource_id
                ),
            ));
        }
    }
}

fn validate_combatant_resource_pools(
    combatant: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let mut seen_resource_ids = HashSet::new();
    for pool in &combatant.resource_pools {
        if pool.id.is_empty() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::EmptyActionResourcePoolId,
                Some(combatant.id.clone()),
                format!(
                    "Combatant {} declares a resource pool with an empty id.",
                    combatant.id
                ),
            ));
            continue;
        }
        if !seen_resource_ids.insert(pool.id.as_str()) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::DuplicateActionResourcePoolId,
                Some(pool.id.clone()),
                format!(
                    "Combatant {} declares resource pool {} more than once.",
                    combatant.id, pool.id
                ),
            ));
        }
        if pool.maximum == 0 || i32::try_from(pool.maximum).is_err() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::InvalidActionResourcePoolMaximum,
                Some(pool.id.clone()),
                format!(
                    "Combatant {} resource pool {} has unsupported maximum {}.",
                    combatant.id, pool.id, pool.maximum
                ),
            ));
        }
        if matches!(pool.refresh_policy, ActionResourceRefreshPolicy::Turns(0)) {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::InvalidActionResourceRefreshPolicy,
                Some(pool.id.clone()),
                format!(
                    "Combatant {} resource pool {} declares a zero-turn refresh clock.",
                    combatant.id, pool.id
                ),
            ));
        }
    }
}

fn validate_saving_throw(
    _scenario: &RulebenchScenario,
    action: &ActionDefinition,
    save: &SavingThrowCheckDeclaration,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if save.difficulty_class < 0 {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::InvalidSavingThrowDifficultyClass,
            Some(action.id.clone()),
            format!(
                "Action {} declares negative saving throw difficulty class {}.",
                action.id, save.difficulty_class
            ),
        ));
    }
}

fn validate_contested_actor_stat(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    contested: &ContestedCheckDeclaration,
    actor: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    validate_combatant_check_stat(
        scenario,
        action,
        actor,
        &contested.actor_stat_id,
        ContentDiagnosticCode::MissingContestedActorStat,
        "contested actor",
        diagnostics,
    );
}

fn validate_combatant_check_stat(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    combatant: &Combatant,
    stat_id: &str,
    diagnostic_code: ContentDiagnosticCode,
    role: &str,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let Some(definition) = scenario.stat_definition_by_id(stat_id) else {
        diagnostics.push(ContentDiagnostic::error(
            diagnostic_code,
            Some(stat_id.to_string()),
            format!(
                "Action {} references {} stat {} that combatant {} does not have.",
                action.id, role, stat_id, combatant.id
            ),
        ));
        return;
    };

    if definition.kind == StatDefinitionKind::Base && combatant.stat_by_id(stat_id).is_none() {
        diagnostics.push(ContentDiagnostic::error(
            diagnostic_code,
            Some(stat_id.to_string()),
            format!(
                "Action {} references {} base stat {} that combatant {} does not have.",
                action.id, role, stat_id, combatant.id
            ),
        ));
    }
}

fn validate_effect_operations(action: &ActionDefinition, diagnostics: &mut Vec<ContentDiagnostic>) {
    for operation in &action.hit.operations {
        if !operation.is_currently_supported() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::UnsupportedEffectOperation,
                Some(action.id.clone()),
                format!(
                    "Action {} declares effect operation {} without a Rust runtime handler.",
                    action.id,
                    operation.id().code()
                ),
            ));
        }
    }
}

fn validate_hit_modifier(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if let Some(modifier) = action.hit.modifier_operation() {
        if scenario.modifier_by_id(&modifier.modifier_id).is_none() {
            diagnostics.push(ContentDiagnostic::error(
                ContentDiagnosticCode::MissingHitModifierDefinition,
                Some(modifier.modifier_id.clone()),
                format!(
                    "Action {} applies modifier {} that is not present in the scenario modifier catalog.",
                    action.id, modifier.modifier_id
                ),
            ));
        }
    }
}

fn validate_actor_attack_stat(
    scenario: &RulebenchScenario,
    action: &ActionDefinition,
    attack: &AttackCheckDeclaration,
    actor: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    let Some(definition) = scenario.stat_definition_by_id(&attack.modifier_stat_id) else {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingAttackModifierStat,
            Some(attack.modifier_stat_id.clone()),
            format!(
                "Action {} references attack modifier stat {} that actor {} does not have.",
                action.id, attack.modifier_stat_id, actor.id
            ),
        ));
        return;
    };

    if definition.kind == StatDefinitionKind::Base
        && actor.stat_by_id(&attack.modifier_stat_id).is_none()
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingAttackModifierStat,
            Some(attack.modifier_stat_id.clone()),
            format!(
                "Action {} references base attack modifier stat {} that actor {} does not have.",
                action.id, attack.modifier_stat_id, actor.id
            ),
        ));
    }
}

fn validate_target_defense(
    action: &ActionDefinition,
    attack: &AttackCheckDeclaration,
    target: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if !target
        .defenses
        .iter()
        .any(|defense| defense.id == attack.defense.id)
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingTargetDefense,
            Some(attack.defense.id.clone()),
            format!(
                "Action {} references defense {} that target {} does not have.",
                action.id, attack.defense.id, target.id
            ),
        ));
    }
}

impl ContentDiagnostic {
    fn error(
        code: ContentDiagnosticCode,
        content_id: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: ContentDiagnosticSeverity::Error,
            code,
            content_id,
            message: message.into(),
        }
    }
}
