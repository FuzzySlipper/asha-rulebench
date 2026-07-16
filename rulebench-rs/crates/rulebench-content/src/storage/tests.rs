use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::{
    import_content_pack, ContentImportContext, ContentImportLimits, ContentPackCanonicalVersion,
    ContentPackCatalogs, ContentPackCollisionPolicy, ContentPackDefinition, ContentPackIdentity,
    ContentPackSourceKind, EntityDefinition,
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
fn quarantining_open_reports_corrupt_and_partial_content_without_hiding_good_records() {
    let directory = test_directory("quarantine");
    let good = imported_pack("pack.good", "entity.good");
    let corrupt = imported_pack("pack.bad", "entity.bad");
    let good_reference = good.pack.exact_reference();
    let corrupt_reference = corrupt.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&good, b"good", StorageReplacementPolicy::Reject)
        .expect("good pack stores");
    storage
        .store(&corrupt, b"bad", StorageReplacementPolicy::Reject)
        .expect("second pack stores");
    let corrupt_stem = record_file_stem(&corrupt_reference);
    fs::write(
        directory.join(format!("{corrupt_stem}.payload")),
        b"tampered",
    )
    .expect("payload corruption writes");
    fs::write(directory.join("interrupted.record.tmp"), b"partial")
        .expect("partial fixture writes");
    drop(storage);

    let (reopened, issues) =
        ContentPackStorage::open_quarantining(&directory).expect("quarantining open succeeds");
    assert_eq!(reopened.list().len(), 1);
    assert!(reopened.retrieve(&good_reference).is_ok());
    assert!(matches!(
        reopened.retrieve(&corrupt_reference),
        Err(ContentStorageError::NotFound { .. })
    ));
    assert!(issues
        .iter()
        .any(|issue| issue.code == "corruptContentArtifactQuarantined"));
    assert!(issues
        .iter()
        .any(|issue| issue.code == "partialContentCommitQuarantined"));
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
    storage
        .activate(&original_reference)
        .expect("original activation persists");

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
    assert!(!storage.is_active(&original_reference));
    assert!(!storage.is_active(&replacement_reference));
    cleanup(&directory);
}

#[test]
fn activation_sets_and_safe_deletion_survive_restart_and_honor_dependencies() {
    let directory = test_directory("activation-dependencies");
    let dependency = imported_pack("pack.dependency", "entity.dependency");
    let dependency_reference = dependency.pack.exact_reference();
    let root = imported_pack_with_dependency("pack.root", "entity.root", dependency.pack.clone());
    let root_reference = root.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&dependency, b"dependency", StorageReplacementPolicy::Reject)
        .expect("dependency stores");
    storage
        .store(&root, b"root", StorageReplacementPolicy::Reject)
        .expect("root stores");
    storage
        .activate_set(&root.resolved_set.reference.packs)
        .expect("exact resolved set activates atomically");
    drop(storage);

    let mut reopened = ContentPackStorage::open(&directory).expect("activation index reloads");
    assert!(reopened.is_active(&dependency_reference));
    assert!(reopened.is_active(&root_reference));
    assert!(matches!(
        reopened.delete(&root_reference),
        Err(ContentStorageError::ActivePack { .. })
    ));
    reopened
        .deactivate(&root_reference)
        .expect("root deactivates");
    reopened
        .deactivate(&dependency_reference)
        .expect("dependency deactivates");
    assert!(matches!(
        reopened.delete(&dependency_reference),
        Err(ContentStorageError::RequiredBy { .. })
    ));
    reopened
        .delete(&root_reference)
        .expect("root deletes first");
    reopened
        .delete(&dependency_reference)
        .expect("unreferenced dependency deletes");
    cleanup(&directory);
}

#[test]
fn failed_replacement_restores_the_last_known_good_content() {
    let directory = test_directory("replacement-rollback");
    let original = imported_pack("pack.rollback", "entity.original");
    let original_reference = original.pack.exact_reference();
    let replacement = imported_pack("pack.rollback", "entity.replacement");
    let replacement_reference = replacement.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&original, b"original", StorageReplacementPolicy::Reject)
        .expect("original should store");
    let replacement_stem = record_file_stem(&replacement_reference);
    let blocked_temporary = directory.join(format!("{replacement_stem}.payload.tmp"));
    fs::create_dir(&blocked_temporary).expect("fault fixture directory creates");

    assert!(storage
        .store(
            &replacement,
            b"replacement",
            StorageReplacementPolicy::ReplaceSameIdentity,
        )
        .is_err());
    assert_eq!(
        storage
            .retrieve(&original_reference)
            .expect("original remains readable")
            .bytes,
        b"original"
    );
    fs::remove_dir(blocked_temporary).expect("fault fixture removes");
    drop(storage);

    let reopened = ContentPackStorage::open(&directory).expect("storage should reopen");
    assert_eq!(
        reopened
            .retrieve(&original_reference)
            .expect("original survives restart")
            .bytes,
        b"original"
    );
    cleanup(&directory);
}

#[test]
fn restart_rolls_back_replacement_interrupted_before_activation_commit() {
    let directory = test_directory("replacement-crash-before-activation");
    let original = imported_pack("pack.interrupted", "entity.original");
    let original_reference = original.pack.exact_reference();
    let replacement = imported_pack("pack.interrupted", "entity.replacement");
    let replacement_reference = replacement.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&original, b"original", StorageReplacementPolicy::Reject)
        .expect("original stores");
    storage
        .activate(&original_reference)
        .expect("original activates");

    prepare_interrupted_replacement(&storage, &original_reference, &replacement, b"replacement");
    drop(storage);

    let (reopened, issues) = ContentPackStorage::open_quarantining(&directory)
        .expect("startup should roll the interrupted replacement back");
    assert_eq!(
        reopened
            .retrieve(&original_reference)
            .expect("last good artifact survives")
            .bytes,
        b"original"
    );
    assert!(reopened.is_active(&original_reference));
    assert!(matches!(
        reopened.retrieve(&replacement_reference),
        Err(ContentStorageError::NotFound { .. })
    ));
    assert!(issues
        .iter()
        .any(|issue| issue.code == "interruptedContentReplacementRolledBack"));
    cleanup(&directory);
}

#[test]
fn restart_finishes_replacement_interrupted_after_activation_commit() {
    let directory = test_directory("replacement-crash-after-activation");
    let original = imported_pack("pack.committed", "entity.original");
    let original_reference = original.pack.exact_reference();
    let replacement = imported_pack("pack.committed", "entity.replacement");
    let replacement_reference = replacement.pack.exact_reference();
    let mut storage = ContentPackStorage::open(&directory).expect("storage should open");
    storage
        .store(&original, b"original", StorageReplacementPolicy::Reject)
        .expect("original stores");
    storage
        .activate(&original_reference)
        .expect("original activates");

    let next_active = prepare_interrupted_replacement(
        &storage,
        &original_reference,
        &replacement,
        b"replacement",
    );
    write_activation_index(&directory, &next_active).expect("activation commit succeeds");
    drop(storage);

    let (reopened, issues) = ContentPackStorage::open_quarantining(&directory)
        .expect("startup should finish the committed replacement");
    assert!(matches!(
        reopened.retrieve(&original_reference),
        Err(ContentStorageError::NotFound { .. })
    ));
    assert_eq!(
        reopened
            .retrieve(&replacement_reference)
            .expect("committed replacement survives")
            .bytes,
        b"replacement"
    );
    assert!(!reopened.is_active(&original_reference));
    assert!(!reopened.is_active(&replacement_reference));
    assert!(issues
        .iter()
        .any(|issue| issue.code == "interruptedContentReplacementCompleted"));
    assert!(!replacement_transaction_path(&directory).exists());
    cleanup(&directory);
}

fn prepare_interrupted_replacement(
    storage: &ContentPackStorage,
    original_reference: &ContentPackReference,
    replacement: &ImportedContentPack,
    payload: &[u8],
) -> BTreeSet<ContentPackReference> {
    let replacement_record = record_from_pack(&replacement.pack, payload);
    let mut next_active = storage.active.clone();
    next_active.remove(original_reference);
    let transaction = ContentReplacementTransaction {
        replacement: replacement_record.reference.clone(),
        replaced: vec![original_reference.clone()],
        previous_active: storage.active.clone(),
        next_active: next_active.clone(),
    };
    write_replacement_transaction(storage.root(), &transaction)
        .expect("replacement transaction persists");
    let staged = stage_replacements(storage.root(), std::slice::from_ref(original_reference))
        .expect("original artifacts stage");
    write_record(storage.root(), &replacement_record, payload).expect("replacement commits");
    remove_staged_originals(&staged).expect("old originals are removed");
    next_active
}

fn imported_pack(pack_id: &str, entity_id: &str) -> ImportedContentPack {
    let ruleset = ruleset();
    import_content_pack(
        ContentPackDefinition {
            canonical_version: ContentPackCanonicalVersion::V0,
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

fn imported_pack_with_dependency(
    pack_id: &str,
    entity_id: &str,
    dependency: CanonicalContentPack,
) -> ImportedContentPack {
    let ruleset = ruleset();
    import_content_pack(
        ContentPackDefinition {
            canonical_version: ContentPackCanonicalVersion::V0,
            identity: ContentPackIdentity::new(pack_id, "1.0.0"),
            title: "Dependent Stored Pack".to_string(),
            summary: "Storage dependency fixture".to_string(),
            tags: Vec::new(),
            provenance: ContentPackProvenance {
                source_kind: ContentPackSourceKind::AuthoredFile,
                source_id: format!("fixture:{pack_id}"),
                authored_by: Some("storage-test".to_string()),
            },
            ruleset: ruleset.artifact_provenance(),
            dependencies: vec![dependency.exact_reference()],
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                rulesets: Vec::new(),
                entities: vec![EntityDefinition {
                    id: entity_id.to_string(),
                    name: "Dependent Entity".to_string(),
                    summary: "Entity in dependent pack".to_string(),
                    tags: Vec::new(),
                    damage_adjustments: Vec::new(),
                }],
                ..ContentPackCatalogs::default()
            },
        },
        ContentImportLimits::default(),
        ContentImportContext {
            available_packs: std::slice::from_ref(&dependency),
            rulesets: std::slice::from_ref(&ruleset),
            provider_catalog: None,
        },
    )
    .expect("dependent fixture should import")
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
