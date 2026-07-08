use crate::model::*;
use crate::state::CombatState;

pub(crate) fn project_initial_state(
    scenario: &RulebenchScenario,
    summary: &str,
) -> ScenarioProjection {
    CombatState::from_scenario(scenario).project(summary)
}

pub(crate) fn project_final_state(
    scenario: &RulebenchScenario,
    summary: &str,
    target_update: Option<(&DamageOutcome, &ModifierOutcome)>,
) -> ScenarioProjection {
    let mut state = CombatState::from_scenario(scenario);
    if let Some((damage, modifier)) = target_update {
        state.apply_hit(damage, modifier);
    }
    state.project(summary)
}

pub(crate) fn scenario_with_projection(
    scenario: RulebenchScenario,
    projection: &ScenarioProjection,
) -> RulebenchScenario {
    CombatState::from_projection(projection).apply_to_scenario(scenario)
}
