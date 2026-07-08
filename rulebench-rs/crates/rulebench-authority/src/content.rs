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
