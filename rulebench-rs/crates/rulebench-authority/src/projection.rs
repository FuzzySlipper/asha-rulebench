use crate::model::*;

pub(crate) fn project_initial_state(
    scenario: &RulebenchScenario,
    summary: &str,
) -> ScenarioProjection {
    ScenarioProjection {
        summary: summary.to_string(),
        combatants: scenario
            .combatants
            .iter()
            .map(|combatant| FinalCombatantState {
                id: combatant.id.clone(),
                name: combatant.name.clone(),
                hit_points: combatant.hit_points,
                conditions: combatant.conditions.clone(),
            })
            .collect(),
    }
}

pub(crate) fn project_final_state(
    scenario: &RulebenchScenario,
    summary: &str,
    target_update: Option<(&DamageOutcome, &ModifierOutcome)>,
) -> ScenarioProjection {
    let mut projection = project_initial_state(scenario, summary);
    if let Some((damage, modifier)) = target_update {
        for combatant in &mut projection.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
            }
            if combatant.id == modifier.target_id && !combatant.conditions.contains(&modifier.label)
            {
                combatant.conditions.push(modifier.label.clone());
            }
        }
    }
    projection
}

pub(crate) fn scenario_with_projection(
    mut scenario: RulebenchScenario,
    projection: &ScenarioProjection,
) -> RulebenchScenario {
    for combatant in &mut scenario.combatants {
        if let Some(projected) = projection
            .combatants
            .iter()
            .find(|projected| projected.id == combatant.id)
        {
            combatant.hit_points = projected.hit_points;
            combatant.conditions = projected.conditions.clone();
        }
    }
    scenario
}
