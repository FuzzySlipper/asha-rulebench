use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    CanonicalContentPack, ContentDefinitionKind, ContentDefinitionReference, ContentFingerprint,
    ContentPackProvenance, ContentPackReference, ImportedContentPack,
};

mod codec;

use codec::{
    decode_activation_index, decode_record, decode_replacement_transaction,
    encode_activation_index, encode_record, encode_replacement_transaction, fingerprint_payload,
    record_file_stem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageReplacementPolicy {
    Reject,
    ReplaceSameIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentStorageRecord {
    pub reference: ContentPackReference,
    pub title: String,
    pub summary: String,
    pub provenance: ContentPackProvenance,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub dependencies: Vec<ContentPackReference>,
    pub definitions: Vec<ContentDefinitionReference>,
    pub payload_fingerprint: ContentFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredContentPayload {
    pub record: ContentStorageRecord,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentStorageError {
    Io {
        operation: &'static str,
        path: PathBuf,
    },
    CorruptRecord {
        path: PathBuf,
        reason: String,
    },
    CorruptPayload {
        reference: ContentPackReference,
    },
    AlreadyStored {
        reference: ContentPackReference,
    },
    ReplacementDenied {
        id: String,
        version: String,
    },
    NotFound {
        reference: ContentPackReference,
    },
    CandidateMismatch {
        expected: ContentPackReference,
        actual: ContentPackReference,
    },
    ActivePack {
        reference: ContentPackReference,
    },
    RequiredBy {
        reference: ContentPackReference,
        dependent: ContentPackReference,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentStorageStartupIssue {
    pub code: String,
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug)]
pub struct ContentPackStorage {
    root: PathBuf,
    records: BTreeMap<ContentPackReference, ContentStorageRecord>,
    active: BTreeSet<ContentPackReference>,
    definition_index: BTreeMap<(ContentDefinitionKind, String), Vec<ContentPackReference>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContentReplacementTransaction {
    replacement: ContentPackReference,
    replaced: Vec<ContentPackReference>,
    previous_active: BTreeSet<ContentPackReference>,
    next_active: BTreeSet<ContentPackReference>,
}

impl ContentPackStorage {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ContentStorageError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| ContentStorageError::Io {
            operation: "createStorageDirectory",
            path: root.clone(),
        })?;
        recover_interrupted_replacement(&root)?;

        let mut records = BTreeMap::new();
        let entries = fs::read_dir(&root).map_err(|_| ContentStorageError::Io {
            operation: "readStorageDirectory",
            path: root.clone(),
        })?;
        let mut record_paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("record"))
            .collect::<Vec<_>>();
        record_paths.sort();

        let mut identities = BTreeSet::new();
        for path in record_paths {
            let bytes = fs::read(&path).map_err(|_| ContentStorageError::Io {
                operation: "readStorageRecord",
                path: path.clone(),
            })?;
            let record =
                decode_record(&bytes).map_err(|reason| ContentStorageError::CorruptRecord {
                    path: path.clone(),
                    reason,
                })?;
            let payload = read_payload(&root, &record)?;
            validate_payload(&record, &payload)?;
            if !identities.insert((
                record.reference.id.clone(),
                record.reference.version.clone(),
            )) {
                return Err(ContentStorageError::CorruptRecord {
                    path,
                    reason: "multiple fingerprints stored for one content identity".to_string(),
                });
            }
            if records.insert(record.reference.clone(), record).is_some() {
                return Err(ContentStorageError::CorruptRecord {
                    path,
                    reason: "duplicate exact content reference".to_string(),
                });
            }
        }

        let active = read_activation_index(&root)?;
        if let Some(reference) = active
            .iter()
            .find(|reference| !records.contains_key(*reference))
        {
            return Err(ContentStorageError::CorruptRecord {
                path: activation_index_path(&root),
                reason: format!("activation index references missing content: {reference:?}"),
            });
        }
        let definition_index = build_definition_index(records.values());
        Ok(Self {
            root,
            records,
            active,
            definition_index,
        })
    }

    pub fn open_quarantining(
        root: impl Into<PathBuf>,
    ) -> Result<(Self, Vec<ContentStorageStartupIssue>), ContentStorageError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| ContentStorageError::Io {
            operation: "createStorageDirectory",
            path: root.clone(),
        })?;
        let replacement_recovery_issue = recover_interrupted_replacement(&root)?;
        let entries = fs::read_dir(&root).map_err(|_| ContentStorageError::Io {
            operation: "readStorageDirectory",
            path: root.clone(),
        })?;
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        let mut issues = replacement_recovery_issue.into_iter().collect::<Vec<_>>();
        issues.extend(
            paths
                .iter()
                .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
                .map(|path| ContentStorageStartupIssue {
                    code: "partialContentCommitQuarantined".to_string(),
                    path: path.clone(),
                    reason: "An uncommitted content temporary file was ignored.".to_string(),
                })
                .collect::<Vec<_>>(),
        );
        let record_paths = paths
            .into_iter()
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("record"))
            .collect::<Vec<_>>();
        let mut records = BTreeMap::new();
        let mut identities = BTreeSet::new();
        for path in record_paths {
            let loaded = fs::read(&path)
                .map_err(|_| "content record could not be read".to_string())
                .and_then(|bytes| decode_record(&bytes))
                .and_then(|record| {
                    read_payload(&root, &record)
                        .map_err(|error| format!("{error:?}"))
                        .and_then(|payload| {
                            validate_payload(&record, &payload)
                                .map_err(|error| format!("{error:?}"))?;
                            Ok(record)
                        })
                });
            let record = match loaded {
                Ok(record) => record,
                Err(reason) => {
                    issues.push(ContentStorageStartupIssue {
                        code: "corruptContentArtifactQuarantined".to_string(),
                        path,
                        reason,
                    });
                    continue;
                }
            };
            let identity = (
                record.reference.id.clone(),
                record.reference.version.clone(),
            );
            if !identities.insert(identity) || records.contains_key(&record.reference) {
                issues.push(ContentStorageStartupIssue {
                    code: "duplicateContentIdentityQuarantined".to_string(),
                    path,
                    reason: "A duplicate content identity was ignored.".to_string(),
                });
                continue;
            }
            records.insert(record.reference.clone(), record);
        }
        let mut active = match read_activation_index(&root) {
            Ok(active) => active,
            Err(error) => {
                issues.push(ContentStorageStartupIssue {
                    code: "corruptContentActivationIndexQuarantined".to_string(),
                    path: activation_index_path(&root),
                    reason: format!("{error:?}"),
                });
                BTreeSet::new()
            }
        };
        let stale = active
            .iter()
            .filter(|reference| !records.contains_key(*reference))
            .cloned()
            .collect::<Vec<_>>();
        for reference in stale {
            active.remove(&reference);
            issues.push(ContentStorageStartupIssue {
                code: "staleContentActivationQuarantined".to_string(),
                path: activation_index_path(&root),
                reason: format!("Activation referenced unavailable content: {reference:?}"),
            });
        }
        let definition_index = build_definition_index(records.values());
        Ok((
            Self {
                root,
                records,
                active,
                definition_index,
            },
            issues,
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> Vec<&ContentStorageRecord> {
        self.records.values().collect()
    }

    pub fn active_references(&self) -> Vec<&ContentPackReference> {
        self.active.iter().collect()
    }

    pub fn is_active(&self, reference: &ContentPackReference) -> bool {
        self.active.contains(reference)
    }

    pub fn references_for_definition(
        &self,
        kind: ContentDefinitionKind,
        id: &str,
    ) -> &[ContentPackReference] {
        self.definition_index
            .get(&(kind, id.to_string()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn store(
        &mut self,
        imported: &ImportedContentPack,
        authored_payload: &[u8],
        policy: StorageReplacementPolicy,
    ) -> Result<ContentStorageRecord, ContentStorageError> {
        let reference = imported.pack.exact_reference();
        if self.records.contains_key(&reference) {
            return Err(ContentStorageError::AlreadyStored { reference });
        }

        let same_identity = self
            .records
            .keys()
            .filter(|stored| stored.id == reference.id && stored.version == reference.version)
            .cloned()
            .collect::<Vec<_>>();
        if !same_identity.is_empty() && policy == StorageReplacementPolicy::Reject {
            return Err(ContentStorageError::ReplacementDenied {
                id: reference.id,
                version: reference.version,
            });
        }

        let record = record_from_pack(&imported.pack, authored_payload);
        let mut next_active = self.active.clone();
        for replaced in &same_identity {
            next_active.remove(replaced);
        }
        let transaction = ContentReplacementTransaction {
            replacement: record.reference.clone(),
            replaced: same_identity.clone(),
            previous_active: self.active.clone(),
            next_active: next_active.clone(),
        };
        if !same_identity.is_empty() {
            write_replacement_transaction(&self.root, &transaction)?;
        }
        let staged_replacements = match stage_replacements(&self.root, &same_identity) {
            Ok(staged_replacements) => staged_replacements,
            Err(error) => {
                remove_replacement_transaction(&self.root);
                return Err(error);
            }
        };
        if let Err(error) = write_record(&self.root, &record, authored_payload) {
            discard_staged_replacements(&staged_replacements);
            remove_replacement_transaction(&self.root);
            return Err(error);
        }
        if let Err(error) = remove_staged_originals(&staged_replacements) {
            remove_committed_files(&self.root, &record.reference);
            restore_staged_replacements(&staged_replacements);
            discard_staged_replacements(&staged_replacements);
            remove_replacement_transaction(&self.root);
            return Err(error);
        }
        if !same_identity.is_empty() {
            if let Err(error) = write_activation_index(&self.root, &next_active) {
                remove_committed_files(&self.root, &record.reference);
                restore_staged_replacements(&staged_replacements);
                discard_staged_replacements(&staged_replacements);
                remove_replacement_transaction(&self.root);
                return Err(error);
            }
        }
        discard_staged_replacements(&staged_replacements);
        remove_replacement_transaction(&self.root);
        for replaced in same_identity {
            self.records.remove(&replaced);
        }
        self.records
            .insert(record.reference.clone(), record.clone());
        self.active = next_active;
        self.definition_index = build_definition_index(self.records.values());
        Ok(record)
    }

    pub fn retrieve(
        &self,
        reference: &ContentPackReference,
    ) -> Result<StoredContentPayload, ContentStorageError> {
        let record =
            self.records
                .get(reference)
                .cloned()
                .ok_or_else(|| ContentStorageError::NotFound {
                    reference: reference.clone(),
                })?;
        let bytes = read_payload(&self.root, &record)?;
        validate_payload(&record, &bytes)?;
        Ok(StoredContentPayload { record, bytes })
    }

    pub fn activate_reimported(
        &self,
        expected: &ContentPackReference,
        reimported: ImportedContentPack,
    ) -> Result<CanonicalContentPack, ContentStorageError> {
        self.retrieve(expected)?;
        let actual = reimported.pack.exact_reference();
        if &actual != expected {
            return Err(ContentStorageError::CandidateMismatch {
                expected: expected.clone(),
                actual,
            });
        }
        Ok(reimported.pack)
    }

    pub fn activate(
        &mut self,
        reference: &ContentPackReference,
    ) -> Result<(), ContentStorageError> {
        self.activate_set(std::slice::from_ref(reference))
    }

    pub fn activate_set(
        &mut self,
        references: &[ContentPackReference],
    ) -> Result<(), ContentStorageError> {
        for reference in references {
            self.retrieve(reference)?;
        }
        let mut active = self.active.clone();
        active.extend(references.iter().cloned());
        write_activation_index(&self.root, &active)?;
        self.active = active;
        Ok(())
    }

    pub fn deactivate(
        &mut self,
        reference: &ContentPackReference,
    ) -> Result<(), ContentStorageError> {
        self.retrieve(reference)?;
        let mut active = self.active.clone();
        active.remove(reference);
        write_activation_index(&self.root, &active)?;
        self.active = active;
        Ok(())
    }

    pub fn delete(&mut self, reference: &ContentPackReference) -> Result<(), ContentStorageError> {
        if self.active.contains(reference) {
            return Err(ContentStorageError::ActivePack {
                reference: reference.clone(),
            });
        }
        if let Some(dependent) = self.records.values().find(|record| {
            record.reference != *reference && record.dependencies.contains(reference)
        }) {
            return Err(ContentStorageError::RequiredBy {
                reference: reference.clone(),
                dependent: dependent.reference.clone(),
            });
        }
        self.retrieve(reference)?;
        let staged = stage_replacements(&self.root, std::slice::from_ref(reference))?;
        if let Err(error) = remove_staged_originals(&staged) {
            restore_staged_replacements(&staged);
            discard_staged_replacements(&staged);
            return Err(error);
        }
        discard_staged_replacements(&staged);
        self.records.remove(reference);
        self.definition_index = build_definition_index(self.records.values());
        Ok(())
    }
}

fn record_from_pack(pack: &CanonicalContentPack, payload: &[u8]) -> ContentStorageRecord {
    ContentStorageRecord {
        reference: pack.exact_reference(),
        title: pack.title.clone(),
        summary: pack.summary.clone(),
        provenance: pack.provenance.clone(),
        ruleset_id: pack.ruleset.ruleset_id.clone(),
        ruleset_version: pack.ruleset.ruleset_version.clone(),
        dependencies: pack.dependencies.clone(),
        definitions: pack.definition_references(),
        payload_fingerprint: fingerprint_payload(payload),
    }
}

fn write_record(
    root: &Path,
    record: &ContentStorageRecord,
    payload: &[u8],
) -> Result<(), ContentStorageError> {
    let stem = record_file_stem(&record.reference);
    let payload_path = root.join(format!("{stem}.payload"));
    let record_path = root.join(format!("{stem}.record"));
    let payload_temporary = root.join(format!("{stem}.payload.tmp"));
    let record_temporary = root.join(format!("{stem}.record.tmp"));

    fs::write(&payload_temporary, payload).map_err(|_| ContentStorageError::Io {
        operation: "writeContentPayload",
        path: payload_temporary.clone(),
    })?;
    fs::write(&record_temporary, encode_record(record)).map_err(|_| ContentStorageError::Io {
        operation: "writeContentRecord",
        path: record_temporary.clone(),
    })?;
    if fs::rename(&payload_temporary, &payload_path).is_err() {
        let _ = fs::remove_file(&payload_temporary);
        let _ = fs::remove_file(&record_temporary);
        return Err(ContentStorageError::Io {
            operation: "commitContentPayload",
            path: payload_path,
        });
    }
    if fs::rename(&record_temporary, &record_path).is_err() {
        let _ = fs::remove_file(&payload_path);
        let _ = fs::remove_file(&record_temporary);
        return Err(ContentStorageError::Io {
            operation: "commitContentRecord",
            path: record_path,
        });
    }
    Ok(())
}

fn read_payload(
    root: &Path,
    record: &ContentStorageRecord,
) -> Result<Vec<u8>, ContentStorageError> {
    let path = root.join(format!("{}.payload", record_file_stem(&record.reference)));
    fs::read(&path).map_err(|_| ContentStorageError::Io {
        operation: "readContentPayload",
        path,
    })
}

fn validate_payload(
    record: &ContentStorageRecord,
    payload: &[u8],
) -> Result<(), ContentStorageError> {
    if fingerprint_payload(payload) != record.payload_fingerprint {
        return Err(ContentStorageError::CorruptPayload {
            reference: record.reference.clone(),
        });
    }
    Ok(())
}

fn activation_index_path(root: &Path) -> PathBuf {
    root.join("activation.index")
}

fn replacement_transaction_path(root: &Path) -> PathBuf {
    root.join("content-replacement.transaction")
}

fn write_replacement_transaction(
    root: &Path,
    transaction: &ContentReplacementTransaction,
) -> Result<(), ContentStorageError> {
    let path = replacement_transaction_path(root);
    let temporary = root.join("content-replacement.transaction.tmp");
    fs::write(&temporary, encode_replacement_transaction(transaction)).map_err(|_| {
        ContentStorageError::Io {
            operation: "writeContentReplacementTransaction",
            path: temporary.clone(),
        }
    })?;
    fs::rename(&temporary, &path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        ContentStorageError::Io {
            operation: "commitContentReplacementTransaction",
            path,
        }
    })
}

fn remove_replacement_transaction(root: &Path) {
    let _ = fs::remove_file(replacement_transaction_path(root));
}

fn recover_interrupted_replacement(
    root: &Path,
) -> Result<Option<ContentStorageStartupIssue>, ContentStorageError> {
    let path = replacement_transaction_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|_| ContentStorageError::Io {
        operation: "readContentReplacementTransaction",
        path: path.clone(),
    })?;
    let transaction = decode_replacement_transaction(&bytes).map_err(|reason| {
        ContentStorageError::CorruptRecord {
            path: path.clone(),
            reason,
        }
    })?;
    let replacements = replacement_paths(root, &transaction.replaced);
    let current_active = read_activation_index(root)?;
    let replacement_complete = replacement_is_complete(root, &transaction.replacement);
    let originals_absent = replacements.iter().all(|replacement| {
        replacement
            .original_paths
            .iter()
            .all(|original| !original.exists())
    });
    let committed =
        replacement_complete && originals_absent && current_active == transaction.next_active;

    let (code, reason) = if committed {
        discard_staged_replacements(&replacements);
        remove_replacement_transaction(root);
        (
            "interruptedContentReplacementCompleted",
            "A committed content replacement was completed during startup.",
        )
    } else {
        remove_committed_files(root, &transaction.replacement);
        restore_staged_replacements(&replacements);
        let missing_original = replacements.iter().find_map(|replacement| {
            replacement
                .original_paths
                .iter()
                .find(|original| !original.exists())
        });
        if let Some(missing_original) = missing_original {
            return Err(ContentStorageError::CorruptRecord {
                path,
                reason: format!(
                    "interrupted replacement cannot restore missing artifact {}",
                    missing_original.display()
                ),
            });
        }
        write_activation_index(root, &transaction.previous_active)?;
        discard_staged_replacements(&replacements);
        remove_replacement_transaction(root);
        (
            "interruptedContentReplacementRolledBack",
            "An uncommitted content replacement was rolled back during startup.",
        )
    };
    Ok(Some(ContentStorageStartupIssue {
        code: code.to_string(),
        path,
        reason: reason.to_string(),
    }))
}

fn replacement_is_complete(root: &Path, reference: &ContentPackReference) -> bool {
    let stem = record_file_stem(reference);
    let record_path = root.join(format!("{stem}.record"));
    let Ok(bytes) = fs::read(record_path) else {
        return false;
    };
    let Ok(record) = decode_record(&bytes) else {
        return false;
    };
    if record.reference != *reference {
        return false;
    }
    let Ok(payload) = read_payload(root, &record) else {
        return false;
    };
    validate_payload(&record, &payload).is_ok()
}

fn read_activation_index(
    root: &Path,
) -> Result<BTreeSet<ContentPackReference>, ContentStorageError> {
    let path = activation_index_path(root);
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let bytes = fs::read(&path).map_err(|_| ContentStorageError::Io {
        operation: "readContentActivationIndex",
        path: path.clone(),
    })?;
    decode_activation_index(&bytes)
        .map_err(|reason| ContentStorageError::CorruptRecord { path, reason })
}

fn write_activation_index(
    root: &Path,
    active: &BTreeSet<ContentPackReference>,
) -> Result<(), ContentStorageError> {
    let path = activation_index_path(root);
    let temporary = root.join("activation.index.tmp");
    fs::write(&temporary, encode_activation_index(active)).map_err(|_| {
        ContentStorageError::Io {
            operation: "writeContentActivationIndex",
            path: temporary.clone(),
        }
    })?;
    fs::rename(&temporary, &path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        ContentStorageError::Io {
            operation: "commitContentActivationIndex",
            path,
        }
    })
}

#[derive(Debug)]
struct StagedContentReplacement {
    original_paths: Vec<PathBuf>,
    backup_paths: Vec<PathBuf>,
}

fn stage_replacements(
    root: &Path,
    references: &[ContentPackReference],
) -> Result<Vec<StagedContentReplacement>, ContentStorageError> {
    let mut staged = Vec::new();
    for replacement in replacement_paths(root, references) {
        let original_paths = replacement.original_paths;
        let backup_paths = replacement.backup_paths;
        for (original, backup) in original_paths.iter().zip(&backup_paths) {
            if fs::copy(original, backup).is_err() {
                for copied in &backup_paths {
                    let _ = fs::remove_file(copied);
                }
                discard_staged_replacements(&staged);
                return Err(ContentStorageError::Io {
                    operation: "stageReplacedContent",
                    path: original.clone(),
                });
            }
        }
        staged.push(StagedContentReplacement {
            original_paths,
            backup_paths,
        });
    }
    Ok(staged)
}

fn replacement_paths(
    root: &Path,
    references: &[ContentPackReference],
) -> Vec<StagedContentReplacement> {
    references
        .iter()
        .map(|reference| {
            let stem = record_file_stem(reference);
            let original_paths = ["record", "payload"]
                .into_iter()
                .map(|extension| root.join(format!("{stem}.{extension}")))
                .collect::<Vec<_>>();
            let backup_paths = original_paths
                .iter()
                .map(|path| {
                    path.with_extension(format!(
                        "{}.replace-backup.tmp",
                        path.extension()
                            .and_then(|value| value.to_str())
                            .unwrap_or("artifact")
                    ))
                })
                .collect::<Vec<_>>();
            StagedContentReplacement {
                original_paths,
                backup_paths,
            }
        })
        .collect()
}

fn restore_staged_replacements(replacements: &[StagedContentReplacement]) {
    for replacement in replacements.iter().rev() {
        for (original, backup) in replacement
            .original_paths
            .iter()
            .zip(&replacement.backup_paths)
        {
            let _ = fs::copy(backup, original);
        }
    }
}

fn remove_staged_originals(
    replacements: &[StagedContentReplacement],
) -> Result<(), ContentStorageError> {
    for replacement in replacements {
        for original in &replacement.original_paths {
            fs::remove_file(original).map_err(|_| ContentStorageError::Io {
                operation: "removeReplacedContent",
                path: original.clone(),
            })?;
        }
    }
    Ok(())
}

fn discard_staged_replacements(replacements: &[StagedContentReplacement]) {
    for replacement in replacements {
        for backup in &replacement.backup_paths {
            let _ = fs::remove_file(backup);
        }
    }
}

fn remove_committed_files(root: &Path, reference: &ContentPackReference) {
    let stem = record_file_stem(reference);
    for extension in ["record", "payload"] {
        let _ = fs::remove_file(root.join(format!("{stem}.{extension}")));
    }
}

fn build_definition_index<'a>(
    records: impl Iterator<Item = &'a ContentStorageRecord>,
) -> BTreeMap<(ContentDefinitionKind, String), Vec<ContentPackReference>> {
    let mut index = BTreeMap::<_, Vec<_>>::new();
    for record in records {
        for definition in &record.definitions {
            index
                .entry((definition.kind, definition.id.clone()))
                .or_default()
                .push(record.reference.clone());
        }
    }
    for references in index.values_mut() {
        references.sort();
    }
    index
}

#[cfg(test)]
mod tests;
