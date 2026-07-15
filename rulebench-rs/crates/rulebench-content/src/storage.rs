use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    CanonicalContentPack, ContentDefinitionKind, ContentDefinitionReference, ContentFingerprint,
    ContentPackProvenance, ContentPackReference, ImportedContentPack,
};

mod codec;

use codec::{decode_record, encode_record, fingerprint_payload, record_file_stem};

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
    definition_index: BTreeMap<(ContentDefinitionKind, String), Vec<ContentPackReference>>,
}

impl ContentPackStorage {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ContentStorageError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| ContentStorageError::Io {
            operation: "createStorageDirectory",
            path: root.clone(),
        })?;

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

        let definition_index = build_definition_index(records.values());
        Ok(Self {
            root,
            records,
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
        let entries = fs::read_dir(&root).map_err(|_| ContentStorageError::Io {
            operation: "readStorageDirectory",
            path: root.clone(),
        })?;
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        let mut issues = paths
            .iter()
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
            .map(|path| ContentStorageStartupIssue {
                code: "partialContentCommitQuarantined".to_string(),
                path: path.clone(),
                reason: "An uncommitted content temporary file was ignored.".to_string(),
            })
            .collect::<Vec<_>>();
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
        let definition_index = build_definition_index(records.values());
        Ok((
            Self {
                root,
                records,
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
        let staged_replacements = stage_replacements(&self.root, &same_identity)?;
        if let Err(error) = write_record(&self.root, &record, authored_payload) {
            discard_staged_replacements(&staged_replacements);
            return Err(error);
        }
        if let Err(error) = remove_staged_originals(&staged_replacements) {
            remove_committed_files(&self.root, &record.reference);
            restore_staged_replacements(&staged_replacements);
            discard_staged_replacements(&staged_replacements);
            return Err(error);
        }
        discard_staged_replacements(&staged_replacements);
        for replaced in same_identity {
            self.records.remove(&replaced);
        }
        self.records
            .insert(record.reference.clone(), record.clone());
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
}

fn record_from_pack(pack: &CanonicalContentPack, payload: &[u8]) -> ContentStorageRecord {
    ContentStorageRecord {
        reference: pack.exact_reference(),
        title: pack.title.clone(),
        summary: pack.summary.clone(),
        provenance: pack.provenance.clone(),
        ruleset_id: pack.ruleset.ruleset_id.clone(),
        ruleset_version: pack.ruleset.ruleset_version.clone(),
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
    for reference in references {
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
