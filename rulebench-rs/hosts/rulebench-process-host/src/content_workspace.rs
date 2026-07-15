use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use rulebench_bridge::content_storage::{
    compare_content_packs, ContentPackReference, ContentPackSetReference, ContentPackStorage,
    ContentStorageError, ContentStorageRecord, ImportedContentPack, RulesetArtifactProvenance,
    StorageReplacementPolicy,
};
use rulebench_bridge::{import_authored_content, ContentInvocationError};
use rulebench_protocol::{
    AuthoredContentPackDocumentDto, ContentAuditEntryDto, ContentDefinitionSummaryDto,
    ContentImportAttemptDto, ContentImportDiagnosticDto, ContentImportOutcomeDto,
    ContentPackDiffDto, ContentPackIdentityDto, ContentPackReferenceDto, ContentPackReviewDto,
    ContentReplacementPolicyDto, ContentWorkspaceDto, StoredContentPackSummaryDto,
};

use crate::ArtifactRepositoryIssue;

const MAX_AUTHORED_PAYLOAD_BYTES: usize = 512 * 1024;
const AUDIT_FILE_NAME: &str = "content-audit-v1.jsonl";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentWorkspaceError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl ContentWorkspaceError {
    fn storage(error: ContentStorageError) -> Self {
        let (code, retryable) = match &error {
            ContentStorageError::NotFound { .. } => ("contentPackNotFound", false),
            ContentStorageError::AlreadyStored { .. } => ("contentPackAlreadyStored", false),
            ContentStorageError::ReplacementDenied { .. } => {
                ("contentReplacementConfirmationRequired", false)
            }
            ContentStorageError::ActivePack { .. } => ("activeContentPackCannotBeDeleted", false),
            ContentStorageError::RequiredBy { .. } => ("contentPackHasDependents", false),
            ContentStorageError::CorruptRecord { .. }
            | ContentStorageError::CorruptPayload { .. }
            | ContentStorageError::CandidateMismatch { .. } => ("corruptContentArtifact", false),
            ContentStorageError::Io { .. } => ("contentStorageUnavailable", true),
        };
        Self {
            code: code.to_string(),
            message: format!("Content repository operation failed: {error:?}"),
            retryable,
        }
    }
}

#[derive(Debug)]
pub struct ContentWorkspace {
    storage: ContentPackStorage,
    imported: BTreeMap<ContentPackReference, ImportedContentPack>,
    audit: Vec<ContentAuditEntryDto>,
    audit_path: PathBuf,
}

impl ContentWorkspace {
    pub fn open(mut storage: ContentPackStorage) -> (Self, Vec<ArtifactRepositoryIssue>) {
        let audit_path = storage.root().join(AUDIT_FILE_NAME);
        let (audit, mut issues) = load_audit(&audit_path);
        let mut imported: BTreeMap<ContentPackReference, ImportedContentPack> = BTreeMap::new();
        let mut pending = storage
            .list()
            .into_iter()
            .map(|record| record.reference.clone())
            .collect::<Vec<_>>();
        while !pending.is_empty() {
            let mut next = Vec::new();
            let mut progress = false;
            for reference in pending {
                let payload = match storage.retrieve(&reference) {
                    Ok(payload) => payload,
                    Err(error) => {
                        issues.push(issue(
                            "storedContentPayloadRejected",
                            storage.root(),
                            format!("{error:?}"),
                        ));
                        let _ = storage.deactivate(&reference);
                        continue;
                    }
                };
                let document = match decode_document(&payload.bytes) {
                    Ok(document) => document,
                    Err(error) => {
                        issues.push(issue(&error.code, storage.root(), error.message));
                        let _ = storage.deactivate(&reference);
                        continue;
                    }
                };
                let available = imported
                    .values()
                    .map(|value| value.pack.clone())
                    .collect::<Vec<_>>();
                match import_authored_content(&document, &available) {
                    Ok(candidate) if candidate.pack.exact_reference() == reference => {
                        imported.insert(reference, candidate);
                        progress = true;
                    }
                    Ok(candidate) => {
                        issues.push(issue(
                            "storedContentCanonicalMismatch",
                            storage.root(),
                            format!(
                                "Stored reference {reference:?} re-imported as {:?}.",
                                candidate.pack.exact_reference()
                            ),
                        ));
                        let _ = storage.deactivate(&reference);
                    }
                    Err(error) if error.code == "missingContentPackDependency" => {
                        next.push(reference);
                    }
                    Err(error) => {
                        issues.push(issue(&error.code, storage.root(), error.message));
                        let _ = storage.deactivate(&reference);
                    }
                }
            }
            if !progress {
                for reference in next.drain(..) {
                    issues.push(issue(
                        "storedContentDependencyUnavailable",
                        storage.root(),
                        format!("Stored content dependency chain did not resolve: {reference:?}"),
                    ));
                    let _ = storage.deactivate(&reference);
                }
            }
            pending = next;
        }
        (
            Self {
                storage,
                imported,
                audit,
                audit_path,
            },
            issues,
        )
    }

    pub fn storage(&self) -> &ContentPackStorage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut ContentPackStorage {
        &mut self.storage
    }

    pub fn snapshot(&self) -> ContentWorkspaceDto {
        ContentWorkspaceDto {
            packs: self
                .storage
                .list()
                .into_iter()
                .map(|record| self.summary(record))
                .collect(),
            audit: self.audit.clone(),
        }
    }

    pub fn import(
        &mut self,
        authored_payload: &str,
        replacement_policy: ContentReplacementPolicyDto,
    ) -> ContentImportAttemptDto {
        let document = match decode_document(authored_payload.as_bytes()) {
            Ok(document) => document,
            Err(error) => return rejected_attempt(None, error),
        };
        let identity = ContentPackIdentityDto {
            id: document.pack.id.clone(),
            version: document.pack.version.clone(),
            fingerprint: None,
        };
        let previous = self
            .imported
            .keys()
            .find(|reference| {
                reference.id == document.pack.id && reference.version == document.pack.version
            })
            .cloned();
        let available = self
            .imported
            .iter()
            .filter(|(reference, _)| Some(*reference) != previous.as_ref())
            .map(|(_, value)| value.pack.clone())
            .collect::<Vec<_>>();
        let imported = match import_authored_content(&document, &available) {
            Ok(imported) => imported,
            Err(error) => return rejected_invocation(identity, error),
        };
        let storage_policy = match replacement_policy {
            ContentReplacementPolicyDto::Reject => StorageReplacementPolicy::Reject,
            ContentReplacementPolicyDto::ReplaceSameIdentity => {
                StorageReplacementPolicy::ReplaceSameIdentity
            }
        };
        let record =
            match self
                .storage
                .store(&imported, authored_payload.as_bytes(), storage_policy)
            {
                Ok(record) => record,
                Err(error) => {
                    return rejected_attempt(Some(identity), ContentWorkspaceError::storage(error))
                }
            };
        if let Some(previous) = &previous {
            self.imported.remove(previous);
        }
        self.imported
            .insert(record.reference.clone(), imported.clone());
        let _ = self.append_audit(
            "authoredPayloadAccepted",
            &record.reference,
            format!(
                "Accepted versioned authored payload from {}.",
                record.provenance.source_id
            ),
        );
        let _ = self.append_audit(
            "canonicalReceiptStored",
            &record.reference,
            "Stored the exact canonical content receipt and payload fingerprint.".to_string(),
        );
        if let Some(replaced) = &previous {
            let _ = self.append_audit(
                "contentReplaced",
                &record.reference,
                format!("Replaced exact content reference {replaced:?}; activation was cleared."),
            );
        }
        let review = self
            .review_for(&record.reference)
            .expect("newly stored pack loads");
        ContentImportAttemptDto {
            accepted: true,
            pack: ContentPackIdentityDto {
                fingerprint: Some(review.pack.reference.fingerprint.clone()),
                ..identity
            },
            diagnostics: review.diagnostics.clone(),
            outcome: Some(ContentImportOutcomeDto {
                review,
                replaced: previous.as_ref().map(ContentPackReferenceDto::from),
            }),
            error_code: None,
            error_message: None,
        }
    }

    pub fn review(
        &self,
        reference: &ContentPackReference,
    ) -> Result<ContentPackReviewDto, ContentWorkspaceError> {
        self.review_for(reference)
    }

    pub fn compare(
        &self,
        authored_payload: &str,
    ) -> Result<ContentPackDiffDto, ContentWorkspaceError> {
        let document = decode_document(authored_payload.as_bytes())?;
        let previous = self
            .imported
            .iter()
            .find(|(reference, _)| {
                reference.id == document.pack.id && reference.version == document.pack.version
            })
            .map(|(_, value)| value)
            .ok_or_else(|| ContentWorkspaceError {
                code: "contentReplacementTargetNotFound".to_string(),
                message: "No stored pack has the authored payload's id and version.".to_string(),
                retryable: false,
            })?;
        let available = self
            .imported
            .values()
            .filter(|value| value.pack.exact_reference() != previous.pack.exact_reference())
            .map(|value| value.pack.clone())
            .collect::<Vec<_>>();
        let candidate = import_authored_content(&document, &available).map_err(invocation_error)?;
        Ok(ContentPackDiffDto::from(&compare_content_packs(
            &previous.pack,
            &candidate.pack,
        )))
    }

    pub fn activate(
        &mut self,
        reference: &ContentPackReference,
    ) -> Result<ContentWorkspaceDto, ContentWorkspaceError> {
        let imported = self
            .imported
            .get(reference)
            .ok_or_else(|| ContentWorkspaceError {
                code: "contentPackNotRevalidated".to_string(),
                message: "The exact stored content pack did not pass restart revalidation."
                    .to_string(),
                retryable: false,
            })?;
        self.storage
            .activate_set(&imported.resolved_set.reference.packs)
            .map_err(ContentWorkspaceError::storage)?;
        self.append_audit(
            "contentActivated",
            reference,
            format!(
                "Activated canonical pack set {} containing {} exact packs.",
                imported.resolved_set.reference.fingerprint.value,
                imported.resolved_set.reference.packs.len()
            ),
        )?;
        Ok(self.snapshot())
    }

    pub fn deactivate(
        &mut self,
        reference: &ContentPackReference,
    ) -> Result<ContentWorkspaceDto, ContentWorkspaceError> {
        self.storage
            .deactivate(reference)
            .map_err(ContentWorkspaceError::storage)?;
        self.append_audit(
            "contentDeactivated",
            reference,
            "Deactivated the exact canonical pack reference.".to_string(),
        )?;
        Ok(self.snapshot())
    }

    pub fn delete(
        &mut self,
        reference: &ContentPackReference,
    ) -> Result<ContentWorkspaceDto, ContentWorkspaceError> {
        self.storage
            .delete(reference)
            .map_err(ContentWorkspaceError::storage)?;
        self.imported.remove(reference);
        self.append_audit(
            "contentDeleted",
            reference,
            "Deleted an inactive content payload and canonical receipt.".to_string(),
        )?;
        Ok(self.snapshot())
    }

    pub fn active_pack_set(
        &self,
        reference: &ContentPackReference,
    ) -> Result<ContentPackSetReference, ContentWorkspaceError> {
        if !self.storage.is_active(reference) {
            return Err(ContentWorkspaceError {
                code: "contentPackNotActive".to_string(),
                message: "The requested exact content pack is not active.".to_string(),
                retryable: false,
            });
        }
        let imported = self
            .imported
            .get(reference)
            .ok_or_else(|| ContentWorkspaceError {
                code: "staleContentActivation".to_string(),
                message: "The activated content pack did not pass revalidation.".to_string(),
                retryable: false,
            })?;
        if imported
            .resolved_set
            .reference
            .packs
            .iter()
            .any(|dependency| !self.storage.is_active(dependency))
        {
            return Err(ContentWorkspaceError {
                code: "inactiveContentDependency".to_string(),
                message: "The activated content pack set has an inactive exact dependency."
                    .to_string(),
                retryable: false,
            });
        }
        Ok(imported.resolved_set.reference.clone())
    }

    pub fn ruleset_for(
        &self,
        reference: &ContentPackReference,
    ) -> Result<(&str, &str), ContentWorkspaceError> {
        let record = self
            .storage
            .list()
            .into_iter()
            .find(|record| &record.reference == reference)
            .ok_or_else(|| {
                ContentWorkspaceError::storage(ContentStorageError::NotFound {
                    reference: reference.clone(),
                })
            })?;
        Ok((&record.ruleset_id, &record.ruleset_version))
    }

    pub fn ruleset_provenance_for(
        &self,
        reference: &ContentPackReference,
    ) -> Result<RulesetArtifactProvenance, ContentWorkspaceError> {
        self.imported
            .get(reference)
            .map(|imported| imported.pack.ruleset.clone())
            .ok_or_else(|| ContentWorkspaceError {
                code: "contentPackNotRevalidated".to_string(),
                message: "The exact content pack has no revalidated ruleset provenance."
                    .to_string(),
                retryable: false,
            })
    }

    pub fn record_session_use(
        &mut self,
        reference: &ContentPackReference,
        session_id: &str,
    ) -> Result<(), ContentWorkspaceError> {
        self.append_audit(
            "contentBoundToSession",
            reference,
            format!("Bound exact activated content to live session {session_id}."),
        )
    }

    fn review_for(
        &self,
        reference: &ContentPackReference,
    ) -> Result<ContentPackReviewDto, ContentWorkspaceError> {
        let stored = self
            .storage
            .retrieve(reference)
            .map_err(ContentWorkspaceError::storage)?;
        let imported = self
            .imported
            .get(reference)
            .ok_or_else(|| ContentWorkspaceError {
                code: "contentPackNotRevalidated".to_string(),
                message: "The exact stored content pack is quarantined from authority use."
                    .to_string(),
                retryable: false,
            })?;
        let authored_payload =
            String::from_utf8(stored.bytes).map_err(|_| ContentWorkspaceError {
                code: "invalidStoredContentEncoding".to_string(),
                message: "Stored authored content is not UTF-8.".to_string(),
                retryable: false,
            })?;
        Ok(ContentPackReviewDto {
            pack: self.summary(&stored.record),
            authored_payload,
            diagnostics: imported
                .diagnostics
                .iter()
                .map(ContentImportDiagnosticDto::from)
                .collect(),
        })
    }

    fn summary(&self, record: &ContentStorageRecord) -> StoredContentPackSummaryDto {
        StoredContentPackSummaryDto {
            reference: ContentPackReferenceDto::from(&record.reference),
            title: record.title.clone(),
            summary: record.summary.clone(),
            source_kind: record.provenance.source_kind.code().to_string(),
            source_id: record.provenance.source_id.clone(),
            authored_by: record.provenance.authored_by.clone(),
            ruleset_id: record.ruleset_id.clone(),
            ruleset_version: record.ruleset_version.clone(),
            dependencies: record
                .dependencies
                .iter()
                .map(ContentPackReferenceDto::from)
                .collect(),
            definitions: record
                .definitions
                .iter()
                .map(|definition| ContentDefinitionSummaryDto {
                    kind: definition.kind.code().to_string(),
                    id: definition.id.clone(),
                })
                .collect(),
            active: self.storage.is_active(&record.reference),
        }
    }

    fn append_audit(
        &mut self,
        operation: &str,
        reference: &ContentPackReference,
        detail: String,
    ) -> Result<(), ContentWorkspaceError> {
        let entry = ContentAuditEntryDto {
            sequence: self.audit.last().map_or(1, |entry| entry.sequence + 1),
            operation: operation.to_string(),
            reference: ContentPackReferenceDto::from(reference),
            detail,
        };
        let encoded = serde_json::to_vec(&entry).map_err(|error| ContentWorkspaceError {
            code: "contentAuditSerializationFailed".to_string(),
            message: error.to_string(),
            retryable: false,
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_path)
            .map_err(|error| ContentWorkspaceError {
                code: "contentAuditStorageUnavailable".to_string(),
                message: error.to_string(),
                retryable: true,
            })?;
        file.write_all(&encoded)
            .and_then(|()| file.write_all(b"\n"))
            .map_err(|error| ContentWorkspaceError {
                code: "contentAuditStorageUnavailable".to_string(),
                message: error.to_string(),
                retryable: true,
            })?;
        self.audit.push(entry);
        Ok(())
    }
}

fn decode_document(bytes: &[u8]) -> Result<AuthoredContentPackDocumentDto, ContentWorkspaceError> {
    if bytes.len() > MAX_AUTHORED_PAYLOAD_BYTES {
        return Err(ContentWorkspaceError {
            code: "authoredContentPayloadTooLarge".to_string(),
            message: format!(
                "Authored payload exceeds the {} byte limit.",
                MAX_AUTHORED_PAYLOAD_BYTES
            ),
            retryable: false,
        });
    }
    serde_json::from_slice(bytes).map_err(|error| ContentWorkspaceError {
        code: "invalidAuthoredContentPayload".to_string(),
        message: format!("Authored payload did not match the Rust protocol DTO: {error}"),
        retryable: false,
    })
}

fn invocation_error(error: ContentInvocationError) -> ContentWorkspaceError {
    ContentWorkspaceError {
        code: error.code,
        message: error.message,
        retryable: false,
    }
}

fn rejected_invocation(
    identity: ContentPackIdentityDto,
    error: ContentInvocationError,
) -> ContentImportAttemptDto {
    ContentImportAttemptDto {
        accepted: false,
        pack: identity,
        outcome: None,
        diagnostics: error.diagnostics,
        error_code: Some(error.code),
        error_message: Some(error.message),
    }
}

fn rejected_attempt(
    identity: Option<ContentPackIdentityDto>,
    error: ContentWorkspaceError,
) -> ContentImportAttemptDto {
    ContentImportAttemptDto {
        accepted: false,
        pack: identity.unwrap_or(ContentPackIdentityDto {
            id: "unknown".to_string(),
            version: "unknown".to_string(),
            fingerprint: None,
        }),
        outcome: None,
        diagnostics: vec![ContentImportDiagnosticDto {
            severity: "error".to_string(),
            code: error.code.clone(),
            path: "payload".to_string(),
            reference_id: None,
            definition_kind: None,
            message: error.message.clone(),
        }],
        error_code: Some(error.code),
        error_message: Some(error.message),
    }
}

fn load_audit(path: &Path) -> (Vec<ContentAuditEntryDto>, Vec<ArtifactRepositoryIssue>) {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (Vec::new(), Vec::new())
        }
        Err(error) => {
            return (
                Vec::new(),
                vec![issue("contentAuditUnreadable", path, error.to_string())],
            )
        }
    };
    let source = match std::str::from_utf8(&bytes) {
        Ok(source) => source,
        Err(error) => {
            return (
                Vec::new(),
                vec![issue("contentAuditCorrupt", path, error.to_string())],
            )
        }
    };
    let mut audit = Vec::new();
    for (index, line) in source.lines().enumerate() {
        match serde_json::from_str::<ContentAuditEntryDto>(line) {
            Ok(entry) if entry.sequence == audit.len() as u64 + 1 => audit.push(entry),
            Ok(_) | Err(_) => {
                return (
                    audit,
                    vec![issue(
                        "contentAuditCorrupt",
                        path,
                        format!(
                            "Audit entry {} was invalid; later entries were ignored.",
                            index + 1
                        ),
                    )],
                )
            }
        }
    }
    (audit, Vec::new())
}

fn issue(code: &str, path: &Path, message: String) -> ArtifactRepositoryIssue {
    ArtifactRepositoryIssue {
        artifact_kind: "content".to_string(),
        code: code.to_string(),
        path: path.display().to_string(),
        message,
    }
}
