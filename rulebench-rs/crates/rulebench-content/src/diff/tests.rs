use super::*;
use crate::{
    canonicalize_content_pack, ContentPackCanonicalVersion, ContentPackCatalogs,
    ContentPackCollisionPolicy, ContentPackDefinition, ContentPackIdentity, ContentPackProvenance,
    ContentPackSourceKind, EntityDefinition,
};
use rpg_ir::RulesetMetadata;

#[test]
fn ordering_independent_canonical_packs_have_no_diff() {
    let mut before_definition = definition();
    before_definition.tags = vec!["alpha".to_string(), "beta".to_string()];
    let mut after_definition = before_definition.clone();
    after_definition.tags.reverse();
    after_definition.catalogs.entities.reverse();

    let diff = compare_content_packs(
        &canonicalize_content_pack(before_definition),
        &canonicalize_content_pack(after_definition),
    );

    assert!(!diff.changed);
    assert!(diff.metadata_changes.is_empty());
    assert!(diff.definition_changes.is_empty());
}

#[test]
fn definition_diff_reports_added_removed_and_changed_entries() {
    let before = canonicalize_content_pack(definition());
    let mut after_definition = definition();
    after_definition.catalogs.entities.remove(0);
    after_definition.catalogs.entities[0].summary = "Changed summary".to_string();
    after_definition
        .catalogs
        .entities
        .push(entity("entity.added"));
    let after = canonicalize_content_pack(after_definition);

    let diff = compare_content_packs(&before, &after);

    assert_eq!(
        diff.definition_changes,
        vec![
            ContentDefinitionChange {
                kind: ContentDefinitionKind::Entity,
                id: "entity.added".to_string(),
                change: ContentDefinitionChangeKind::Added,
            },
            ContentDefinitionChange {
                kind: ContentDefinitionKind::Entity,
                id: "entity.changed".to_string(),
                change: ContentDefinitionChangeKind::Changed,
            },
            ContentDefinitionChange {
                kind: ContentDefinitionKind::Entity,
                id: "entity.removed".to_string(),
                change: ContentDefinitionChangeKind::Removed,
            },
        ]
    );
}

#[test]
fn metadata_diff_reports_compatibility_dependencies_and_fingerprint_changes() {
    let before = canonicalize_content_pack(definition());
    let mut after_definition = definition();
    after_definition.identity.version = "2.0.0".to_string();
    after_definition.ruleset.ruleset_version = "2.0.0".to_string();
    after_definition.dependencies.push(before.exact_reference());
    let after = canonicalize_content_pack(after_definition);

    let diff = compare_content_packs(&before, &after);

    assert!(diff.changed);
    assert!(diff.fingerprint_changed);
    assert!(diff.ruleset_compatibility_changed);
    assert!(diff.dependency_set_changed);
    assert!(diff
        .metadata_changes
        .contains(&ContentPackMetadataChangeKind::Identity));
    assert!(diff
        .metadata_changes
        .contains(&ContentPackMetadataChangeKind::RulesetCompatibility));
    assert!(diff
        .metadata_changes
        .contains(&ContentPackMetadataChangeKind::Dependencies));
    assert!(diff
        .metadata_changes
        .contains(&ContentPackMetadataChangeKind::Fingerprint));
}

fn definition() -> ContentPackDefinition {
    let ruleset = RulesetMetadata {
        id: "rules.diff".to_string(),
        name: "Diff Rules".to_string(),
        version: "1.0.0".to_string(),
        summary: "Rules for content diff tests".to_string(),
        modules: Vec::new(),
    };
    ContentPackDefinition {
        canonical_version: ContentPackCanonicalVersion::V0,
        identity: ContentPackIdentity::new("pack.diff", "1.0.0"),
        title: "Diff Pack".to_string(),
        summary: "Content diff fixture".to_string(),
        tags: Vec::new(),
        provenance: ContentPackProvenance {
            source_kind: ContentPackSourceKind::Embedded,
            source_id: "fixture:diff".to_string(),
            authored_by: None,
        },
        ruleset: ruleset.artifact_provenance(),
        dependencies: Vec::new(),
        collision_policy: ContentPackCollisionPolicy::Reject,
        catalogs: ContentPackCatalogs {
            rulesets: vec![ruleset],
            entities: vec![entity("entity.removed"), entity("entity.changed")],
            ..ContentPackCatalogs::default()
        },
    }
}

fn entity(id: &str) -> EntityDefinition {
    EntityDefinition {
        id: id.to_string(),
        name: "Diff Entity".to_string(),
        summary: "Original summary".to_string(),
        tags: Vec::new(),
        damage_adjustments: Vec::new(),
    }
}
