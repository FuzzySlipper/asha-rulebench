use std::collections::HashSet;

use crate::{
    Combatant, ContentDiagnostic, ContentDiagnosticCode, ContentDiagnosticSeverity,
    ContentValidationReport, RulebenchScenario,
};
use rulebench_ruleset::{
    ActionDefinition, AttackCheckDeclaration, RuleModuleValidationError, TargetKind,
    TargetSelection,
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
    for combatant in &scenario.combatants {
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

        for stat in combatant
            .stats
            .base_stats
            .iter()
            .chain(combatant.stats.derived_stats.iter())
        {
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
        for item_id in &combatant.equipped_item_ids {
            if scenario.item_by_id(item_id).is_none() {
                diagnostics.push(ContentDiagnostic::error(
                    ContentDiagnosticCode::MissingEquippedItem,
                    Some(item_id.clone()),
                    format!(
                        "Combatant {} equips item {} that is not present in the scenario item catalog.",
                        combatant.id, item_id
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

    if action.attack_check().is_none() {
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

    if let (Some(actor), Some(attack)) = (actor, action.attack_check()) {
        validate_actor_attack_stat(action, attack, actor, diagnostics);
    }
    validate_hit_modifier(scenario, action, diagnostics);
    validate_effect_operations(action, diagnostics);

    for target_id in &action.targeting.target_ids {
        if let Some(target) = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == *target_id)
        {
            if let Some(attack) = action.attack_check() {
                validate_target_defense(action, attack, target, diagnostics);
            }
        }
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
    action: &ActionDefinition,
    attack: &AttackCheckDeclaration,
    actor: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if actor.stat_by_id(&attack.modifier_stat_id).is_none() {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingAttackModifierStat,
            Some(attack.modifier_stat_id.clone()),
            format!(
                "Action {} references attack modifier stat {} that actor {} does not have.",
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
