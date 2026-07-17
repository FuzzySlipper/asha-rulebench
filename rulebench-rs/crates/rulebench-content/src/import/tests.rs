use super::*;
use crate::{
    AuthoredActionDefinition, AuthoredEffectOperation, AuthoredModifierEffectOperation,
    AuthoredReactionHookEffectOperation, AuthoredTargetingDeclaration, ContentPackCanonicalVersion,
    ContentPackCatalogs, ContentPackCollisionPolicy, ContentPackIdentity, ContentPackProvenance,
    ContentPackSourceKind, EntityDefinition, ModifierDefinition, ModifierDurationPolicy,
    ModifierStackingPolicy, ModifierStatAdjustment, ReactionParticipantSelector,
};
use rulebench_ruleset::{
    AbilityDefinition, AbilityDefinitionKind, ActionResolutionModuleConfiguration,
    ActionResourceCost, AttackCheckDeclaration, CheckDeclaration, DamageEffectOperation,
    DefenseReference, EffectOperationId, HealingEffectOperation, ModifierTenure,
    OperationPipelineV2, ReactionWindow, RuleModuleDeclaration, RulesetMetadata,
    RulesetProviderCapability, RulesetProviderCatalog, RulesetProviderDescriptor,
    SavingThrowCheckDeclaration, TargetKind, TargetSelection, TargetTeamConstraint,
    VisibilityRequirement,
};

#[test]
fn valid_authored_pack_imports_reproducibly() {
    let ruleset = ruleset();
    let first = import_content_pack(
        authored_pack(&ruleset, false),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("first import should succeed");
    let second = import_content_pack(
        authored_pack(&ruleset, true),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("second import should succeed");

    assert_eq!(first.pack, second.pack);
    assert_eq!(first.pack.exact_reference(), first.resolved_set.root);
}

#[test]
fn malformed_and_duplicate_authored_data_is_rejected_stably() {
    let ruleset = ruleset();
    let mut authored = authored_pack(&ruleset, false);
    authored.identity.id.clear();
    authored
        .catalogs
        .entities
        .push(authored.catalogs.entities[0].clone());
    authored.dependencies.push(crate::ContentPackReference {
        id: "dependency".to_string(),
        version: "1.0.0".to_string(),
        fingerprint: crate::ContentFingerprint {
            algorithm: "unknown".to_string(),
            value: "NOT-A-FINGERPRINT".to_string(),
        },
    });

    let first = import_content_pack(
        authored.clone(),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("malformed import should fail");
    authored.catalogs.entities.reverse();
    let second = import_content_pack(
        authored,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("reordered malformed import should fail");

    assert_eq!(first, second);
    assert!(first
        .diagnostics
        .iter()
        .any(|value| value.code == ContentImportDiagnosticCode::DuplicateDefinition));
    assert!(first
        .diagnostics
        .iter()
        .any(|value| value.code == ContentImportDiagnosticCode::InvalidFingerprint));
}

#[test]
fn structural_limits_reject_before_canonicalization() {
    let ruleset = ruleset();
    let report = import_content_pack(
        authored_pack(&ruleset, false),
        ContentImportLimits {
            maximum_total_definitions: 1,
            ..ContentImportLimits::default()
        },
        ContentImportContext::empty(),
    )
    .expect_err("oversized import should fail");

    assert!(report
        .diagnostics
        .iter()
        .any(|value| value.code == ContentImportDiagnosticCode::LimitExceeded));
}

#[test]
fn duplicate_tags_are_canonicalized_with_a_stable_warning() {
    let ruleset = ruleset();
    let mut authored = authored_pack(&ruleset, false);
    authored.tags = vec!["test".to_string(), "test".to_string()];

    let imported = import_content_pack(
        authored,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("warning-only import should succeed");

    assert_eq!(imported.pack.tags, vec!["test"]);
    assert_eq!(imported.diagnostics.len(), 1);
    assert_eq!(
        imported.diagnostics[0].severity,
        ContentImportDiagnosticSeverity::Warning
    );
    assert_eq!(
        imported.diagnostics[0].code,
        ContentImportDiagnosticCode::DuplicateTagCanonicalized
    );
}

#[test]
fn authored_ability_validation_reports_stable_duplicate_and_field_paths() {
    let ruleset = ruleset();
    let mut authored = authored_pack(&ruleset, false);
    let ability = AbilityDefinition {
        id: "ability.binding-glyph".to_string(),
        name: "Binding Glyph".to_string(),
        kind: AbilityDefinitionKind::Spell,
        summary: String::new(),
        tags: vec!["control".to_string()],
    };
    authored.catalogs.abilities = vec![ability.clone(), ability];

    let report = import_content_pack(
        authored,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("duplicate incomplete abilities fail before storage");

    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentImportDiagnosticCode::DuplicateDefinition
            && diagnostic.definition_kind == Some(ContentDefinitionKind::Ability)
            && diagnostic.definition_id.as_deref() == Some("ability.binding-glyph")
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentImportDiagnosticCode::EmptyField
            && diagnostic.path == "catalogs.abilities[0].summary"
            && diagnostic.definition_kind == Some(ContentDefinitionKind::Ability)
            && diagnostic.definition_id.as_deref() == Some("ability.binding-glyph")
    }));
}

#[test]
fn incompatible_ruleset_is_rejected_without_an_accepted_pack() {
    let ruleset = ruleset();
    let mut authored = authored_pack(&ruleset, false);
    authored.ruleset.ruleset_version = "2.0.0".to_string();

    let report = import_content_pack(
        authored,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("incompatible ruleset should fail");

    assert!(report.diagnostics.iter().any(|value| {
        value.code
            == ContentImportDiagnosticCode::PackValidation(
                ContentPackDiagnosticCode::IncompatibleRuleset,
            )
    }));
}

#[test]
fn exact_dependency_must_be_available() {
    let ruleset = ruleset();
    let dependency = import_content_pack(
        authored_pack_with_id(&ruleset, "dependency", "entity.dependency"),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("dependency should import")
    .pack;
    let mut root = authored_pack_with_id(&ruleset, "root", "entity.root");
    root.catalogs.rulesets.clear();
    root.dependencies.push(dependency.exact_reference());

    let missing = import_content_pack(
        root.clone(),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("missing dependency should fail");
    assert!(missing.diagnostics.iter().any(|value| {
        value.code
            == ContentImportDiagnosticCode::PackValidation(
                ContentPackDiagnosticCode::MissingDependency,
            )
    }));

    let imported = import_content_pack(
        root,
        ContentImportLimits::default(),
        ContentImportContext {
            available_packs: &[dependency],
            rulesets: &[ruleset],
            provider_catalog: None,
        },
    )
    .expect("available dependency should resolve");
    assert_eq!(imported.resolved_set.packs.len(), 2);
}

#[test]
fn portable_actions_resolve_ability_and_modifier_from_the_exact_dependency_set() {
    let ruleset = ruleset();
    let mut dependency_definition =
        authored_pack_with_id(&ruleset, "pack.action-dependency", "entity.dependency");
    dependency_definition.canonical_version = ContentPackCanonicalVersion::V1;
    dependency_definition.catalogs.abilities = vec![portable_ability()];
    dependency_definition.catalogs.modifiers = vec![portable_modifier()];
    let dependency = import_content_pack(
        dependency_definition,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("dependency definitions import");

    let mut root = authored_pack_with_id(&ruleset, "pack.action-root", "entity.root");
    root.canonical_version = ContentPackCanonicalVersion::V1;
    root.dependencies = vec![dependency.pack.exact_reference()];
    root.catalogs.actions = vec![portable_action("ability.portable", "modifier.portable")];
    let provider_catalog = provider_catalog(
        &ruleset,
        &[
            ("check.attackVsDefense", "1"),
            (
                "targeting.singleCombatant",
                OperationPipelineV2::VOCABULARY_VERSION,
            ),
            (
                "operation.applyModifier",
                EffectOperationId::VOCABULARY_VERSION,
            ),
            ("operation.damage", EffectOperationId::VOCABULARY_VERSION),
        ],
    );
    let imported = import_content_pack(
        root,
        ContentImportLimits::default(),
        ContentImportContext {
            available_packs: std::slice::from_ref(&dependency.pack),
            rulesets: &[],
            provider_catalog: Some(&provider_catalog),
        },
    )
    .expect("exact dependency definitions satisfy portable action references");

    assert_eq!(imported.resolved_set.packs.len(), 2);
    assert!(imported.diagnostics.is_empty());
}

#[test]
fn provider_capabilities_reject_unsupported_check_targeting_and_effect() {
    let ruleset = ruleset();
    let mut root = authored_pack_with_id(&ruleset, "pack.provider-gap", "entity.root");
    root.canonical_version = ContentPackCanonicalVersion::V1;
    root.catalogs.abilities = vec![portable_ability()];
    root.catalogs.modifiers = vec![portable_modifier()];
    root.catalogs.actions = vec![portable_action("ability.portable", "modifier.portable")];
    let provider_catalog = provider_catalog(&ruleset, &[]);

    let report = import_content_pack(
        root,
        ContentImportLimits::default(),
        ContentImportContext {
            available_packs: &[],
            rulesets: &[],
            provider_catalog: Some(&provider_catalog),
        },
    )
    .expect_err("provider capability omissions fail before persistence");

    for (code, path) in [
        (
            ContentImportDiagnosticCode::UnsupportedActionCheck,
            "resolvedPacks[pack.provider-gap@1.0.0].catalogs.actions[0].check",
        ),
        (
            ContentImportDiagnosticCode::UnsupportedActionTargeting,
            "resolvedPacks[pack.provider-gap@1.0.0].catalogs.actions[0].targeting",
        ),
        (
            ContentImportDiagnosticCode::UnsupportedActionEffect,
            "resolvedPacks[pack.provider-gap@1.0.0].catalogs.actions[0].effects[0]",
        ),
    ] {
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == code
                && diagnostic.path == path
                && diagnostic.definition_kind == Some(ContentDefinitionKind::Action)
                && diagnostic.definition_id.as_deref() == Some("action.portable")
        }));
    }
}

#[test]
fn portable_action_missing_references_report_stable_definition_paths() {
    let ruleset = ruleset();
    let mut root = authored_pack_with_id(&ruleset, "pack.action-root", "entity.root");
    root.canonical_version = ContentPackCanonicalVersion::V1;
    root.catalogs.actions = vec![portable_action("ability.missing", "modifier.missing")];

    let report = import_content_pack(
        root,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("unresolved portable definitions fail closed");

    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentImportDiagnosticCode::MissingActionAbility
            && diagnostic.path
                == "resolvedPacks[pack.action-root@1.0.0].catalogs.actions[0].abilityId"
            && diagnostic.definition_kind == Some(ContentDefinitionKind::Action)
            && diagnostic.definition_id.as_deref() == Some("action.portable")
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ContentImportDiagnosticCode::MissingActionModifier
            && diagnostic.path
                == "resolvedPacks[pack.action-root@1.0.0].catalogs.actions[0].effects[1].modifierId"
            && diagnostic.definition_kind == Some(ContentDefinitionKind::Action)
            && diagnostic.definition_id.as_deref() == Some("action.portable")
    }));
}

#[test]
fn malformed_portable_action_catalog_reports_stable_semantic_diagnostics() {
    let ruleset = ruleset();
    let mut root = authored_pack_with_id(&ruleset, "pack.action-invalid", "entity.root");
    root.canonical_version = ContentPackCanonicalVersion::V1;
    root.catalogs.abilities = vec![portable_ability()];
    let mut modifier = portable_modifier();
    modifier.duration_policy = ModifierDurationPolicy::Permanent;
    root.catalogs.modifiers = vec![modifier];
    let mut action = portable_action("ability.portable", "modifier.portable");
    action.check = CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
        save_stat_id: "body".to_string(),
        difficulty_class: 12,
    });
    action.resource_costs = vec![
        ActionResourceCost::standard_action(),
        ActionResourceCost::standard_action(),
    ];
    action.effects = vec![AuthoredEffectOperation::OpenReactionWindow(
        AuthoredReactionHookEffectOperation {
            hook_id: "response".to_string(),
            window: ReactionWindow::AfterEffect,
            eligible_reactors: vec![ReactionParticipantSelector::DeclaredTargets],
            options: Vec::new(),
            maximum_nested_depth: 0,
        },
    )];
    root.catalogs.actions = vec![action];

    let report = import_content_pack(
        root,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect_err("malformed portable declarations fail before canonical import");

    for (code, path) in [
        (
            ContentImportDiagnosticCode::InvalidModifierDeclaration,
            "catalogs.modifiers[0].durationPolicy",
        ),
        (
            ContentImportDiagnosticCode::UnsupportedActionCheck,
            "catalogs.actions[0].check",
        ),
        (
            ContentImportDiagnosticCode::DuplicateActionResourceCost,
            "catalogs.actions[0].resourceCosts[1].resourceId",
        ),
        (
            ContentImportDiagnosticCode::InvalidReactionDeclaration,
            "catalogs.actions[0].effects[0]",
        ),
    ] {
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code && diagnostic.path == path));
    }
}

#[test]
fn non_executable_effect_sequences_fail_before_canonical_import() {
    let ruleset = ruleset();
    let damage = || {
        AuthoredEffectOperation::Damage(DamageEffectOperation {
            damage_bonus: 1,
            damage_type: "force".to_string(),
        })
    };
    let heal = || {
        AuthoredEffectOperation::Heal(HealingEffectOperation {
            healing_bonus: 2,
            healing_type: "restoration".to_string(),
        })
    };

    for (case, effects, expected_path) in [
        ("heal-only", vec![heal()], "catalogs.actions[0].effects"),
        (
            "repeated-damage",
            vec![damage(), damage()],
            "catalogs.actions[0].effects[1]",
        ),
        (
            "reordered-heal-before-damage",
            vec![heal(), damage()],
            "catalogs.actions[0].effects[1]",
        ),
    ] {
        let mut pack = authored_pack_with_id(
            &ruleset,
            &format!("pack.effect-program.{case}"),
            &format!("entity.{case}"),
        );
        pack.canonical_version = ContentPackCanonicalVersion::V1;
        pack.catalogs.abilities = vec![portable_ability()];
        pack.catalogs.modifiers = vec![portable_modifier()];
        let mut action = portable_action("ability.portable", "modifier.portable");
        action.effects = effects;
        pack.catalogs.actions = vec![action];

        let report = import_content_pack(
            pack,
            ContentImportLimits::default(),
            ContentImportContext::empty(),
        )
        .expect_err("non-executable authored effect programs must fail before persistence");

        assert!(
            report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == ContentImportDiagnosticCode::UnsupportedActionEffect
                    && diagnostic.path == expected_path
                    && diagnostic.definition_id.as_deref() == Some("action.portable")
            }),
            "missing exact rejection for {case}: {report:?}"
        );
    }
}

fn authored_pack(ruleset: &RulesetMetadata, reverse: bool) -> AuthoredContentPack {
    let mut pack = authored_pack_with_id(ruleset, "test.pack", "entity.one");
    pack.tags = if reverse {
        vec!["beta".to_string(), "alpha".to_string()]
    } else {
        vec!["alpha".to_string(), "beta".to_string()]
    };
    if reverse {
        pack.catalogs.entities.reverse();
    }
    pack
}

fn authored_pack_with_id(
    ruleset: &RulesetMetadata,
    pack_id: &str,
    entity_id: &str,
) -> AuthoredContentPack {
    ContentPackDefinition {
        canonical_version: ContentPackCanonicalVersion::V0,
        identity: ContentPackIdentity::new(pack_id, "1.0.0"),
        title: "Test Pack".to_string(),
        summary: "A pack submitted through the canonical import boundary.".to_string(),
        tags: Vec::new(),
        provenance: ContentPackProvenance {
            source_kind: ContentPackSourceKind::Embedded,
            source_id: format!("fixture:{pack_id}"),
            authored_by: None,
        },
        ruleset: ruleset.artifact_provenance(),
        dependencies: Vec::new(),
        collision_policy: ContentPackCollisionPolicy::Reject,
        catalogs: ContentPackCatalogs {
            rulesets: vec![ruleset.clone()],
            entities: vec![EntityDefinition {
                id: entity_id.to_string(),
                name: "Entity".to_string(),
                summary: "Imported entity".to_string(),
                tags: Vec::new(),
                damage_adjustments: Vec::new(),
            }],
            ..ContentPackCatalogs::default()
        },
    }
}

fn portable_ability() -> AbilityDefinition {
    AbilityDefinition {
        id: "ability.portable".to_string(),
        name: "Portable Ability".to_string(),
        kind: AbilityDefinitionKind::Ability,
        summary: "Dependency-owned ability.".to_string(),
        tags: Vec::new(),
    }
}

fn portable_modifier() -> ModifierDefinition {
    ModifierDefinition {
        id: "modifier.portable".to_string(),
        label: "Portable Modifier".to_string(),
        summary: "Dependency-owned modifier.".to_string(),
        default_tenure: ModifierTenure::Temporary,
        stacking_group: "portable".to_string(),
        stacking_policy: ModifierStackingPolicy::Refresh,
        duration_policy: ModifierDurationPolicy::Turns(1),
        stat_adjustments: vec![ModifierStatAdjustment {
            stat_id: "guard".to_string(),
            stat_label: "Guard".to_string(),
            delta: -1,
        }],
    }
}

fn portable_action(ability_id: &str, modifier_id: &str) -> AuthoredActionDefinition {
    AuthoredActionDefinition {
        id: "action.portable".to_string(),
        ability_id: ability_id.to_string(),
        name: "Portable Action".to_string(),
        targeting: AuthoredTargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 6,
            visibility_requirement: VisibilityRequirement::Required,
            operation_pipeline: None,
        },
        check: CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 2,
            modifier_stat_id: "focus".to_string(),
            defense: DefenseReference {
                id: "guard".to_string(),
                label: "Guard".to_string(),
            },
        }),
        effects: vec![
            AuthoredEffectOperation::Damage(DamageEffectOperation {
                damage_bonus: 1,
                damage_type: "force".to_string(),
            }),
            AuthoredEffectOperation::ApplyModifier(AuthoredModifierEffectOperation {
                modifier_id: modifier_id.to_string(),
            }),
        ],
        resource_costs: Vec::new(),
        movement: None,
        action_text: "Use the portable action.".to_string(),
        effect_text: "Apply the portable modifier.".to_string(),
    }
}

fn ruleset() -> RulesetMetadata {
    RulesetMetadata {
        id: "rules.test".to_string(),
        name: "Test Rules".to_string(),
        version: "1.0.0".to_string(),
        summary: "Test ruleset".to_string(),
        modules: vec![RuleModuleDeclaration::action_resolution(
            ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
        )],
    }
}

fn provider_catalog(
    ruleset: &RulesetMetadata,
    capabilities: &[(&str, &str)],
) -> RulesetProviderCatalog {
    RulesetProviderCatalog::new(vec![RulesetProviderDescriptor {
        provider_id: "provider.rules.test".to_string(),
        provider_version: "1".to_string(),
        ruleset: ruleset.clone(),
        operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        effect_operation_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        capabilities: capabilities
            .iter()
            .map(|(id, version)| RulesetProviderCapability {
                id: (*id).to_string(),
                version: (*version).to_string(),
            })
            .collect(),
    }])
    .expect("test provider catalog is valid")
}
