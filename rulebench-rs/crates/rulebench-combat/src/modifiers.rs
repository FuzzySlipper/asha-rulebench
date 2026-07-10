//! Modifier contribution and effective-stat resolution.

use std::collections::HashMap;

use crate::model::{
    CombatantEffectiveStatReadout, CombatantModifierStatAdjustmentReadout, EffectiveStatReadout,
    ModifierStatAdjustmentContribution, RulebenchScenario, StatDefinitionKind,
};
use rulebench_content::DerivedStatFormula;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectiveStatEvaluationError {
    MissingCombatant {
        combatant_id: String,
    },
    MissingBaseStat {
        combatant_id: String,
        stat_id: String,
    },
    MissingStatDefinition {
        stat_id: String,
    },
    MissingDerivedFormula {
        stat_id: String,
    },
    FormulaCycle {
        stat_ids: Vec<String>,
    },
    InvalidFormula {
        stat_id: String,
        operation: String,
    },
    ArithmeticOverflow {
        stat_id: String,
    },
}

impl EffectiveStatEvaluationError {
    pub const fn code(&self) -> &'static str {
        match self {
            EffectiveStatEvaluationError::MissingCombatant { .. } => "missingCombatant",
            EffectiveStatEvaluationError::MissingBaseStat { .. } => "missingBaseStat",
            EffectiveStatEvaluationError::MissingStatDefinition { .. } => "missingStatDefinition",
            EffectiveStatEvaluationError::MissingDerivedFormula { .. } => "missingDerivedFormula",
            EffectiveStatEvaluationError::FormulaCycle { .. } => "formulaCycle",
            EffectiveStatEvaluationError::InvalidFormula { .. } => "invalidFormula",
            EffectiveStatEvaluationError::ArithmeticOverflow { .. } => "arithmeticOverflow",
        }
    }
}

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
                source_id: active_modifier.source_id.clone(),
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
    evaluate_effective_stats_for_combatant(scenario, combatant_id).ok()
}

pub fn evaluate_effective_stats_for_combatant(
    scenario: &RulebenchScenario,
    combatant_id: &str,
) -> Result<CombatantEffectiveStatReadout, EffectiveStatEvaluationError> {
    let combatant = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == combatant_id)
        .ok_or_else(|| EffectiveStatEvaluationError::MissingCombatant {
            combatant_id: combatant_id.to_string(),
        })?;
    let modifier_readout = active_modifier_stat_adjustments_for_combatant(scenario, combatant_id)
        .ok_or_else(|| EffectiveStatEvaluationError::MissingCombatant {
        combatant_id: combatant_id.to_string(),
    })?;
    let base_stats = combatant
        .stats
        .base_stats
        .iter()
        .map(|stat| (stat.id.as_str(), stat))
        .collect::<HashMap<_, _>>();
    let definitions = scenario
        .stat_definitions
        .iter()
        .map(|definition| (definition.id.as_str(), definition))
        .collect::<HashMap<_, _>>();
    let mut context = EvaluationContext {
        combatant_id,
        definitions: &definitions,
        base_stats: &base_stats,
        contributions: &modifier_readout.contributions,
        evaluated: HashMap::new(),
        resolving: Vec::new(),
    };
    let mut stats = Vec::with_capacity(scenario.stat_definitions.len());

    for definition in &scenario.stat_definitions {
        let value = context.evaluate_stat(definition.id.as_str())?;
        let contributions = modifier_readout
            .contributions
            .iter()
            .filter(|contribution| contribution.stat_id == definition.id)
            .cloned()
            .collect::<Vec<_>>();
        let total_modifier_delta =
            contributions
                .iter()
                .try_fold(0_i32, |total, contribution| {
                    total.checked_add(contribution.delta).ok_or_else(|| {
                        EffectiveStatEvaluationError::ArithmeticOverflow {
                            stat_id: definition.id.clone(),
                        }
                    })
                })?;
        let base_value = value.checked_sub(total_modifier_delta).ok_or_else(|| {
            EffectiveStatEvaluationError::ArithmeticOverflow {
                stat_id: definition.id.clone(),
            }
        })?;

        stats.push(EffectiveStatReadout {
            stat_id: definition.id.clone(),
            stat_label: definition.label.clone(),
            kind: definition.kind,
            formula: definition.formula.clone(),
            base_value,
            total_modifier_delta,
            effective_value: value,
            contributions,
        });
    }

    Ok(CombatantEffectiveStatReadout {
        combatant_id: combatant.id.clone(),
        stats,
    })
}

struct EvaluationContext<'a> {
    combatant_id: &'a str,
    definitions: &'a HashMap<&'a str, &'a crate::model::StatDefinition>,
    base_stats: &'a HashMap<&'a str, &'a crate::model::NamedNumber>,
    contributions: &'a [ModifierStatAdjustmentContribution],
    evaluated: HashMap<String, i32>,
    resolving: Vec<String>,
}

impl EvaluationContext<'_> {
    fn evaluate_stat(&mut self, stat_id: &str) -> Result<i32, EffectiveStatEvaluationError> {
        if let Some(value) = self.evaluated.get(stat_id) {
            return Ok(*value);
        }
        if let Some(cycle_start) = self.resolving.iter().position(|entry| entry == stat_id) {
            let mut stat_ids = self.resolving[cycle_start..].to_vec();
            stat_ids.push(stat_id.to_string());
            return Err(EffectiveStatEvaluationError::FormulaCycle { stat_ids });
        }

        let definition = self.definitions.get(stat_id).ok_or_else(|| {
            EffectiveStatEvaluationError::MissingStatDefinition {
                stat_id: stat_id.to_string(),
            }
        })?;
        self.resolving.push(stat_id.to_string());
        let value_before_direct_modifiers = match definition.kind {
            StatDefinitionKind::Base => self
                .base_stats
                .get(stat_id)
                .map(|stat| stat.value)
                .ok_or_else(|| EffectiveStatEvaluationError::MissingBaseStat {
                    combatant_id: self.combatant_id.to_string(),
                    stat_id: stat_id.to_string(),
                })?,
            StatDefinitionKind::Derived => self.evaluate_formula(
                definition.formula.as_ref().ok_or_else(|| {
                    EffectiveStatEvaluationError::MissingDerivedFormula {
                        stat_id: stat_id.to_string(),
                    }
                })?,
                stat_id,
            )?,
        };
        let direct_modifier_delta = self.direct_modifier_delta(stat_id)?;
        let value = value_before_direct_modifiers
            .checked_add(direct_modifier_delta)
            .ok_or_else(|| EffectiveStatEvaluationError::ArithmeticOverflow {
                stat_id: stat_id.to_string(),
            })?;
        self.resolving.pop();
        self.evaluated.insert(stat_id.to_string(), value);
        Ok(value)
    }

    fn evaluate_formula(
        &mut self,
        formula: &DerivedStatFormula,
        owner_stat_id: &str,
    ) -> Result<i32, EffectiveStatEvaluationError> {
        match formula {
            DerivedStatFormula::Constant { value } => Ok(*value),
            DerivedStatFormula::StatReference { stat_id } => self.evaluate_stat(stat_id),
            DerivedStatFormula::Sum { operands } => {
                self.fold_formula_operands(operands, owner_stat_id, 0, i32::checked_add, "sum")
            }
            DerivedStatFormula::Product { operands } => {
                self.fold_formula_operands(operands, owner_stat_id, 1, i32::checked_mul, "product")
            }
            DerivedStatFormula::Difference {
                minuend,
                subtrahend,
            } => {
                let left = self.evaluate_formula(minuend, owner_stat_id)?;
                let right = self.evaluate_formula(subtrahend, owner_stat_id)?;
                left.checked_sub(right).ok_or_else(|| {
                    EffectiveStatEvaluationError::ArithmeticOverflow {
                        stat_id: owner_stat_id.to_string(),
                    }
                })
            }
        }
    }

    fn fold_formula_operands(
        &mut self,
        operands: &[DerivedStatFormula],
        owner_stat_id: &str,
        initial: i32,
        operation: fn(i32, i32) -> Option<i32>,
        operation_name: &str,
    ) -> Result<i32, EffectiveStatEvaluationError> {
        if operands.len() < 2 {
            return Err(EffectiveStatEvaluationError::InvalidFormula {
                stat_id: owner_stat_id.to_string(),
                operation: operation_name.to_string(),
            });
        }

        operands.iter().try_fold(initial, |total, operand| {
            let value = self.evaluate_formula(operand, owner_stat_id)?;
            operation(total, value).ok_or_else(|| {
                EffectiveStatEvaluationError::ArithmeticOverflow {
                    stat_id: owner_stat_id.to_string(),
                }
            })
        })
    }

    fn direct_modifier_delta(&self, stat_id: &str) -> Result<i32, EffectiveStatEvaluationError> {
        self.contributions
            .iter()
            .filter(|contribution| contribution.stat_id == stat_id)
            .try_fold(0_i32, |total, contribution| {
                total.checked_add(contribution.delta).ok_or_else(|| {
                    EffectiveStatEvaluationError::ArithmeticOverflow {
                        stat_id: stat_id.to_string(),
                    }
                })
            })
    }
}
