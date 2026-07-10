//! Canonical authored content, references, validation, and diagnostics.
//!
//! This crate owns content catalogs and structural validation. It does not
//! execute rule behavior, mutate combat state, or depend on fixtures or UI.

#![forbid(unsafe_code)]

mod active_modifier;
mod diagnostics;
mod scenario;
mod stats;
mod validation;

pub use active_modifier::ActiveModifier;
pub use diagnostics::{
    ContentDiagnostic, ContentDiagnosticCode, ContentDiagnosticSeverity, ContentValidationReport,
    ScenarioMetadata,
};
pub use scenario::{
    ClassDefinition, Combatant, CombatantEffectiveStatReadout,
    CombatantModifierStatAdjustmentReadout, EffectiveStatReadout, EntityDefinition, Grid, GridCell,
    ItemDefinition, ModifierDefinition, ModifierStatAdjustment, ModifierStatAdjustmentContribution,
    RulebenchScenario, UseActionIntent,
};
pub use stats::{DerivedStatFormula, StatBlock, StatDefinition, StatDefinitionKind};
pub use validation::{validate_scenario_content, validate_scenario_content_report};

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_core::GridPosition;
    use rulebench_ruleset::{
        AbilityDefinition, ActionDefinition, AttackCheckDeclaration, CheckDeclaration,
        DefenseReference, HitEffect, TargetKind, TargetSelection, TargetTeamConstraint,
        TargetingDeclaration, VisibilityRequirement,
    };

    #[test]
    fn validation_runs_on_authored_content_without_combat_session_construction() {
        let scenario = incomplete_scenario();

        let diagnostics = validate_scenario_content(&scenario);
        let report = validate_scenario_content_report(&scenario);

        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ContentDiagnosticCode::MissingActionActor }));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ContentDiagnosticCode::MissingActionAbility }));
        assert!(!report.accepted);
        assert_eq!(report.error_count, diagnostics.len());
    }

    fn incomplete_scenario() -> RulebenchScenario {
        let selected_action = test_action();
        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "content-validation".to_string(),
                title: "Content Validation".to_string(),
                summary: "An intentionally incomplete authored content graph.".to_string(),
                seed_label: "content-validation".to_string(),
            },
            rulesets: Vec::new(),
            selected_ruleset_id: "missing-ruleset".to_string(),
            grid: Grid {
                width: 1,
                height: 1,
                cells: vec![GridCell {
                    position: GridPosition { x: 0, y: 0 },
                    terrain_tags: Vec::new(),
                }],
            },
            combatants: Vec::new(),
            entities: Vec::new(),
            abilities: Vec::<AbilityDefinition>::new(),
            selected_ability_id: None,
            classes: Vec::new(),
            selected_class_id: None,
            stat_definitions: Vec::new(),
            modifiers: Vec::new(),
            items: Vec::new(),
            selected_item_id: None,
            actions: vec![selected_action.clone()],
            selected_action,
        }
    }

    fn test_action() -> ActionDefinition {
        ActionDefinition {
            id: "test-action".to_string(),
            ruleset_id: "missing-ruleset".to_string(),
            ability_id: "missing-ability".to_string(),
            name: "Test Action".to_string(),
            actor_id: "missing-actor".to_string(),
            targeting: TargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 1,
                visibility_requirement: VisibilityRequirement::Ignored,
                target_ids: Vec::new(),
                visible_target_ids: Vec::new(),
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 0,
                modifier_stat_id: "attack".to_string(),
                defense: DefenseReference {
                    id: "defense".to_string(),
                    label: "Defense".to_string(),
                },
            }),
            hit: HitEffect {
                damage_bonus: 0,
                damage_type: "test".to_string(),
                modifier_id: "test-modifier".to_string(),
                modifier_label: "Test Modifier".to_string(),
                modifier_duration: "test".to_string(),
                operations: Vec::new(),
            },
            action_text: "Test action.".to_string(),
            effect_text: "Test effect.".to_string(),
        }
    }
}
