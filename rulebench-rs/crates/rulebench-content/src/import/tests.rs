use super::*;
use crate::{
    ContentPackCatalogs, ContentPackCollisionPolicy, ContentPackIdentity, ContentPackProvenance,
    ContentPackSourceKind, EntityDefinition,
};
use rulebench_ruleset::{
    ActionResolutionModuleConfiguration, RuleModuleDeclaration, RulesetMetadata,
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
        },
    )
    .expect("available dependency should resolve");
    assert_eq!(imported.resolved_set.packs.len(), 2);
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
