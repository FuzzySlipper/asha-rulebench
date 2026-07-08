use std::collections::HashSet;

use crate::model::{
    ActionDefinition, Combatant, ContentDiagnostic, ContentDiagnosticCode,
    ContentDiagnosticSeverity, RulebenchScenario,
};

pub fn validate_scenario_content(scenario: &RulebenchScenario) -> Vec<ContentDiagnostic> {
    let mut diagnostics = Vec::new();

    if scenario.ruleset.id.is_empty() {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::EmptyRulesetId,
            None,
            "Scenario ruleset id is empty.",
        ));
    }

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
    }
}

fn validate_combatant_class_and_stat_references(
    scenario: &RulebenchScenario,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    for combatant in &scenario.combatants {
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

    let target_ids = action
        .target_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for target_id in &action.target_ids {
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

    for visible_target_id in &action.visible_target_ids {
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
        validate_actor_attack_stat(action, actor, diagnostics);
    }
    validate_hit_modifier(scenario, action, diagnostics);

    for target_id in &action.target_ids {
        if let Some(target) = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == *target_id)
        {
            validate_target_defense(action, target, diagnostics);
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
    actor: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if actor.stat_by_id(&action.attack.modifier_stat_id).is_none() {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingAttackModifierStat,
            Some(action.attack.modifier_stat_id.clone()),
            format!(
                "Action {} references attack modifier stat {} that actor {} does not have.",
                action.id, action.attack.modifier_stat_id, actor.id
            ),
        ));
    }
}

fn validate_target_defense(
    action: &ActionDefinition,
    target: &Combatant,
    diagnostics: &mut Vec<ContentDiagnostic>,
) {
    if !target
        .defenses
        .iter()
        .any(|defense| defense.id == action.attack.defense_id)
    {
        diagnostics.push(ContentDiagnostic::error(
            ContentDiagnosticCode::MissingTargetDefense,
            Some(action.attack.defense_id.clone()),
            format!(
                "Action {} references defense {} that target {} does not have.",
                action.id, action.attack.defense_id, target.id
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
