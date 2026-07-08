use crate::model::{
    CombatantEffectiveStatReadout, CombatantModifierStatAdjustmentReadout, EffectiveStatReadout,
    ModifierStatAdjustmentContribution, RulebenchScenario,
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

pub fn effective_stats_for_combatant(
    scenario: &RulebenchScenario,
    combatant_id: &str,
) -> Option<CombatantEffectiveStatReadout> {
    let combatant = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == combatant_id)?;
    let modifier_readout = active_modifier_stat_adjustments_for_combatant(scenario, combatant_id)?;

    let stats = combatant
        .stats
        .base_stats
        .iter()
        .chain(combatant.stats.derived_stats.iter())
        .map(|stat| {
            let contributions = modifier_readout
                .contributions
                .iter()
                .filter(|contribution| contribution.stat_id == stat.id)
                .cloned()
                .collect::<Vec<_>>();
            let total_modifier_delta = contributions
                .iter()
                .map(|contribution| contribution.delta)
                .sum();

            EffectiveStatReadout {
                stat_id: stat.id.clone(),
                stat_label: stat.label.clone(),
                base_value: stat.value,
                total_modifier_delta,
                effective_value: stat.value + total_modifier_delta,
                contributions,
            }
        })
        .collect();

    Some(CombatantEffectiveStatReadout {
        combatant_id: combatant.id.clone(),
        stats,
    })
}
