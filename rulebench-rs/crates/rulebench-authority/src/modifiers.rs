use crate::model::{
    CombatantModifierStatAdjustmentReadout, ModifierStatAdjustmentContribution, RulebenchScenario,
};

pub fn active_modifier_stat_adjustments_for_combatant(
    scenario: &RulebenchScenario,
    combatant_id: &str,
) -> Option<CombatantModifierStatAdjustmentReadout> {
    let combatant = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == combatant_id)?;

    let mut contributions = Vec::new();
    for active_modifier in &combatant.active_modifiers {
        let Some(definition) = scenario.modifier_by_id(&active_modifier.modifier_id) else {
            continue;
        };

        for adjustment in &definition.stat_adjustments {
            contributions.push(ModifierStatAdjustmentContribution {
                modifier_id: active_modifier.modifier_id.clone(),
                modifier_label: active_modifier.label.clone(),
                tenure: active_modifier.tenure,
                stat_id: adjustment.stat_id.clone(),
                stat_label: adjustment.stat_label.clone(),
                delta: adjustment.delta,
            });
        }
    }

    Some(CombatantModifierStatAdjustmentReadout {
        combatant_id: combatant.id.clone(),
        contributions,
    })
}
