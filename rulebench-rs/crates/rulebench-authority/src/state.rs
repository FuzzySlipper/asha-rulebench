use crate::model::{
    ActiveModifier, BoundedValue, Combatant, DamageOutcome, FinalCombatantState, ModifierOutcome,
    RulebenchScenario, ScenarioProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatState {
    combatants: Vec<CombatantState>,
}

impl CombatState {
    pub(crate) fn from_scenario(scenario: &RulebenchScenario) -> Self {
        Self {
            combatants: scenario
                .combatants
                .iter()
                .map(CombatantState::from_combatant)
                .collect(),
        }
    }

    pub(crate) fn from_projection(projection: &ScenarioProjection) -> Self {
        Self {
            combatants: projection
                .combatants
                .iter()
                .map(CombatantState::from_final_state)
                .collect(),
        }
    }

    pub(crate) fn project(&self, summary: &str) -> ScenarioProjection {
        ScenarioProjection {
            summary: summary.to_string(),
            combatants: self
                .combatants
                .iter()
                .map(CombatantState::to_final_state)
                .collect(),
        }
    }

    pub(crate) fn apply_hit(&mut self, damage: &DamageOutcome, modifier: &ModifierOutcome) {
        for combatant in &mut self.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
            }
            if combatant.id == modifier.target_id {
                combatant.apply_modifier(modifier);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn active_modifiers_for(&self, combatant_id: &str) -> Option<&[ActiveModifier]> {
        self.combatants
            .iter()
            .find(|combatant| combatant.id == combatant_id)
            .map(|combatant| combatant.active_modifiers.as_slice())
    }

    pub(crate) fn apply_to_scenario(&self, mut scenario: RulebenchScenario) -> RulebenchScenario {
        for combatant in &mut scenario.combatants {
            if let Some(state) = self
                .combatants
                .iter()
                .find(|state| state.id == combatant.id)
            {
                combatant.hit_points = state.hit_points;
                combatant.conditions = state.conditions.clone();
            }
        }
        scenario
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CombatantState {
    id: String,
    name: String,
    hit_points: BoundedValue,
    active_modifiers: Vec<ActiveModifier>,
    conditions: Vec<String>,
}

impl CombatantState {
    fn from_combatant(combatant: &Combatant) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            active_modifiers: combatant.active_modifiers.clone(),
            conditions: combatant.conditions.clone(),
        }
    }

    fn to_final_state(&self) -> FinalCombatantState {
        FinalCombatantState {
            id: self.id.clone(),
            name: self.name.clone(),
            hit_points: self.hit_points,
            conditions: self.condition_labels(),
        }
    }

    fn from_final_state(combatant: &FinalCombatantState) -> Self {
        Self {
            id: combatant.id.clone(),
            name: combatant.name.clone(),
            hit_points: combatant.hit_points,
            active_modifiers: Vec::new(),
            conditions: combatant.conditions.clone(),
        }
    }

    fn apply_modifier(&mut self, modifier: &ModifierOutcome) {
        if self.active_modifiers.iter().any(|active| {
            active.modifier_id == modifier.modifier_id || active.label == modifier.label
        }) {
            return;
        }

        self.active_modifiers.push(ActiveModifier::temporary(
            modifier.modifier_id.clone(),
            modifier.label.clone(),
            modifier.duration.clone(),
        ));
    }

    fn condition_labels(&self) -> Vec<String> {
        let mut labels = self.conditions.clone();
        for modifier in &self.active_modifiers {
            if !labels.contains(&modifier.label) {
                labels.push(modifier.label.clone());
            }
        }
        labels
    }
}
