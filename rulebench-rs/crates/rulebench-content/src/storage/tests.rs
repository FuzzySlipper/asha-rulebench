use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::{
    import_content_pack, ContentImportContext, ContentImportLimits, ContentPackCatalogs,
    ContentPackCollisionPolicy, ContentPackDefinition, ContentPackIdentity, ContentPackSourceKind,
    EntityDefinition,
};
use rulebench_ruleset::{
    ActionResolutionModuleConfiguration, RuleModuleDeclaration, RulesetMetadata,
};

static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn restart_reload_preserves_payload_receipt_and_definition_index() {
    let directory = test_directory("reload");
    let imported = imported_pack("pack.one", "entity.one");
    let reference = imported.pack.exact_reference();
    let payload = br#"{"id":"pack.one"}"#;

    {
        let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
        storage
            .store(&imported, payload, StorageReplacementPolicy::Reject)
            .expect("validated pack should store");
    }

    let reopened = ContentPackStorage::open(&directory).expect("storage should reload");
    assert_eq!(reopened.list().len(), 1);
    assert_eq!(
        reopened
            .retrieve(&reference)
            .expect("payload should load")
            .bytes,
        payload
    );
    assert_eq!(
        reopened.references_for_definition(ContentDefinitionKind::Entity, "entity.one"),
        &[reference.clone()]
    );
    assert_eq!(
        reopened
            .activate_reimported(&reference, imported)
            .expect("exact reimport should activate")
            .exact_reference(),
        reference
    );
    cleanup(&directory);
}

#[test]
fn corrupted_payload_is_rejected_on_restart() {
    let directory = test_directory("corruption");
    let imported = imported_pack("pack.corrupt", "entity.corrupt");
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&imported, b"original", StorageReplacementPolicy::Reject)
        .expect("pack should store");
    let payload_path = fs::read_dir(&directory)
        .expect("directory should list")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("payload"))
        .expect("payload file should exist");
    fs::write(payload_path, b"tampered").expect("test should corrupt payload");
    drop(storage);

    let error = match ContentPackStorage::open(&directory) {
        Ok(_) => panic!("corrupt storage should not open"),
        Err(error) => error,
    };
    assert!(matches!(error, ContentStorageError::CorruptPayload { .. }));
    cleanup(&directory);
}

#[test]
fn replacement_is_explicit_and_drifted_reimport_cannot_activate() {
    let directory = test_directory("replacement");
    let original = imported_pack("pack.replace", "entity.original");
    let original_reference = original.pack.exact_reference();
    let replacement = imported_pack("pack.replace", "entity.replacement");
    let replacement_reference = replacement.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&original, b"original", StorageReplacementPolicy::Reject)
        .expect("original should store");

    let denied = storage
        .store(
            &replacement,
            b"replacement",
            StorageReplacementPolicy::Reject,
        )
        .expect_err("implicit replacement should fail");
    assert!(matches!(
        denied,
        ContentStorageError::ReplacementDenied { .. }
    ));
    let mismatch = storage
        .activate_reimported(&original_reference, replacement.clone())
        .expect_err("drifted reimport should not activate");
    assert!(matches!(
        mismatch,
        ContentStorageError::CandidateMismatch { .. }
    ));

    storage
        .store(
            &replacement,
            b"replacement",
            StorageReplacementPolicy::ReplaceSameIdentity,
        )
        .expect("explicit replacement should succeed");
    assert!(matches!(
        storage.retrieve(&original_reference),
        Err(ContentStorageError::NotFound { .. })
    ));
    assert!(storage.retrieve(&replacement_reference).is_ok());
    cleanup(&directory);
}

fn imported_pack(pack_id: &str, entity_id: &str) -> ImportedContentPack {
    let ruleset = ruleset();
    import_content_pack(
        ContentPackDefinition {
            identity: ContentPackIdentity::new(pack_id, "1.0.0"),
            title: "Stored Pack".to_string(),
            summary: "Storage integration fixture".to_string(),
            tags: Vec::new(),
            provenance: ContentPackProvenance {
                source_kind: ContentPackSourceKind::AuthoredFile,
                source_id: format!("fixture:{pack_id}"),
                authored_by: Some("storage-test".to_string()),
            },
            ruleset: ruleset.artifact_provenance(),
            dependencies: Vec::new(),
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                rulesets: vec![ruleset],
                entities: vec![EntityDefinition {
                    id: entity_id.to_string(),
                    name: "Stored Entity".to_string(),
                    summary: "Entity in persisted pack".to_string(),
                    tags: Vec::new(),
                    damage_adjustments: Vec::new(),
                }],
                ..ContentPackCatalogs::default()
            },
        },
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("storage fixture should import")
}

fn ruleset() -> RulesetMetadata {
    RulesetMetadata {
        id: "rules.storage".to_string(),
        name: "Storage Rules".to_string(),
        version: "1.0.0".to_string(),
        summary: "Rules for storage tests".to_string(),
        modules: vec![RuleModuleDeclaration::action_resolution(
            ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
        )],
    }
}

fn test_directory(label: &str) -> PathBuf {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "asha-rulebench-content-storage-{}-{label}-{sequence}",
        std::process::id()
    ))
}

fn cleanup(directory: &Path) {
    fs::remove_dir_all(directory).expect("test storage should clean up");
}
