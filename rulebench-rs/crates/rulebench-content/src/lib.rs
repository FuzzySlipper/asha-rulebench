//! Canonical authored content, references, validation, and diagnostics.
//!
//! This crate owns content catalogs and structural validation. It does not
//! execute rule behavior, mutate combat state, or depend on fixtures or UI.

#![forbid(unsafe_code)]

mod active_modifier;
mod canonical;
mod diagnostics;
mod diff;
mod import;
mod pack;
mod pack_validation;
mod scenario;
mod stats;
mod storage;
mod validation;

pub use active_modifier::ActiveModifier;
pub use canonical::{canonicalize_content_pack, fingerprint_content_pack_set};
pub use diagnostics::{
    ContentDiagnostic, ContentDiagnosticCode, ContentDiagnosticSeverity, ContentValidationReport,
    ScenarioMetadata,
};
pub use diff::{
    compare_content_packs, ContentDefinitionChange, ContentDefinitionChangeKind,
    ContentPackDiffReadout, ContentPackMetadataChangeKind,
};
pub use import::{
    import_content_pack, AuthoredContentPack, ContentImportContext, ContentImportDiagnostic,
    ContentImportDiagnosticCode, ContentImportDiagnosticSeverity, ContentImportLimits,
    ContentImportReport, ImportedContentPack,
};
pub use pack::{
    CanonicalContentPack, ContentDefinitionKind, ContentDefinitionReference, ContentFingerprint,
    ContentPackCatalogs, ContentPackCollisionPolicy, ContentPackDefinition, ContentPackIdentity,
    ContentPackProvenance, ContentPackReference, ContentPackSetReference, ContentPackSourceKind,
    CONTENT_PACK_FINGERPRINT_ALGORITHM, CONTENT_PACK_SET_FINGERPRINT_ALGORITHM,
};
pub use pack_validation::{
    resolve_content_pack_set, ContentPackDiagnostic, ContentPackDiagnosticCode,
    ContentPackValidationReport, ResolvedContentPackSet,
};
pub use scenario::{
    ClassDefinition, ClassLevelGrant, ClassLevelInput, Combatant, CombatantEffectiveStatReadout,
    CombatantModifierStatAdjustmentReadout, DamageAdjustment, DamageAdjustmentPolicy,
    EffectiveStatReadout, EntityDefinition, Grid, GridCell, ItemDefinition, ModifierDefinition,
    ModifierDurationPolicy, ModifierStackingPolicy, ModifierStatAdjustment,
    ModifierStatAdjustmentContribution, RulebenchScenario, StatRequirement, UseActionIntent,
};
pub use stats::{DerivedStatFormula, StatBlock, StatDefinition, StatDefinitionKind};
pub use storage::{
    ContentPackStorage, ContentStorageError, ContentStorageRecord, ContentStorageStartupIssue,
    StorageReplacementPolicy, StoredContentPayload,
};
pub use validation::{validate_scenario_content, validate_scenario_content_report};

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_core::GridPosition;
    use rulebench_ruleset::{
        AbilityDefinition, ActionDefinition, ActionResourceCost, AttackCheckDeclaration,
        CheckDeclaration, DefenseReference, HitEffect, TargetKind, TargetSelection,
        TargetTeamConstraint, TargetingDeclaration, VisibilityRequirement,
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

    #[test]
    fn scenario_validation_rejects_tampered_content_pack_set_reference() {
        let mut scenario = incomplete_scenario();
        let pack_reference = ContentPackReference {
            id: "rules.core".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: ContentFingerprint {
                algorithm: CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
                value: "0000000000000000".to_string(),
            },
        };
        scenario.content_pack_set = Some(ContentPackSetReference {
            root: pack_reference.clone(),
            packs: vec![pack_reference],
            fingerprint: ContentFingerprint {
                algorithm: CONTENT_PACK_SET_FINGERPRINT_ALGORITHM.to_string(),
                value: "tampered".to_string(),
            },
        });

        let diagnostics = validate_scenario_content(&scenario);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ContentDiagnosticCode::InvalidContentPackSetReference
        }));
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
            content_pack_set: None,
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
            resource_costs: vec![ActionResourceCost::standard_action()],
            movement: None,
            action_text: "Test action.".to_string(),
            effect_text: "Test effect.".to_string(),
        }
    }
}
