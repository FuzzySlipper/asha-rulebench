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
        write_record(&self.root, &record, authored_payload)?;
        for replaced in same_identity {
            remove_files(&self.root, &replaced)?;
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
    fs::rename(&payload_temporary, &payload_path).map_err(|_| ContentStorageError::Io {
        operation: "commitContentPayload",
        path: payload_path.clone(),
    })?;
    fs::rename(&record_temporary, &record_path).map_err(|_| ContentStorageError::Io {
        operation: "commitContentRecord",
        path: record_path,
    })
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

fn remove_files(root: &Path, reference: &ContentPackReference) -> Result<(), ContentStorageError> {
    let stem = record_file_stem(reference);
    for extension in ["record", "payload"] {
        let path = root.join(format!("{stem}.{extension}"));
        fs::remove_file(&path).map_err(|_| ContentStorageError::Io {
            operation: "removeReplacedContent",
            path,
        })?;
    }
    Ok(())
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
