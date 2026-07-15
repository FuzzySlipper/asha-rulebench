use rulebench_rules::{
    ContentDefinitionChangeKind, ContentImportDiagnostic, ContentImportDiagnosticSeverity,
    ContentImportReport, ContentPackCatalogs, ContentPackCollisionPolicy, ContentPackDefinition,
    ContentPackDiffReadout, ContentPackIdentity, ContentPackMetadataChangeKind,
    ContentPackProvenance, ContentPackReference, ContentPackSourceKind, DamageAdjustment,
    DamageAdjustmentPolicy, EntityDefinition, ImportedContentPack,
};
use serde::{Deserialize, Serialize};

use crate::{validate_ruleset_definition, RulesetDefinitionDto};

pub const AUTHORED_CONTENT_PACK_FORMAT: &str = "asha-rulebench.content-pack";
pub const AUTHORED_CONTENT_PACK_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentFingerprintDto {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackIdentityDto {
    pub id: String,
    pub version: String,
    pub fingerprint: Option<ContentFingerprintDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportDiagnosticDto {
    pub severity: String,
    pub code: String,
    pub path: String,
    pub reference_id: Option<String>,
    pub definition_kind: Option<String>,
    pub message: String,
}

impl From<&ContentImportDiagnostic> for ContentImportDiagnosticDto {
    fn from(diagnostic: &ContentImportDiagnostic) -> Self {
        Self {
            severity: diagnostic.severity.code().to_string(),
            code: diagnostic.code.code().to_string(),
            path: diagnostic.path.clone(),
            reference_id: diagnostic.definition_id.clone(),
            definition_kind: diagnostic
                .definition_kind
                .map(|kind| kind.code().to_string()),
            message: diagnostic.message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportReadoutDto {
    pub example_id: String,
    pub pack: ContentPackIdentityDto,
    pub accepted: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
}

impl ContentImportReadoutDto {
    pub fn accepted(example_id: impl Into<String>, imported: &ImportedContentPack) -> Self {
        let diagnostics = imported
            .diagnostics
            .iter()
            .map(ContentImportDiagnosticDto::from)
            .collect::<Vec<_>>();
        Self {
            example_id: example_id.into(),
            pack: ContentPackIdentityDto {
                id: imported.pack.identity.id.clone(),
                version: imported.pack.identity.version.clone(),
                fingerprint: Some(ContentFingerprintDto {
                    algorithm: imported.pack.fingerprint.algorithm.clone(),
                    value: imported.pack.fingerprint.value.clone(),
                }),
            },
            accepted: true,
            error_count: count_severity(
                &imported.diagnostics,
                ContentImportDiagnosticSeverity::Error,
            ),
            warning_count: count_severity(
                &imported.diagnostics,
                ContentImportDiagnosticSeverity::Warning,
            ),
            diagnostics,
        }
    }

    pub fn rejected(
        example_id: impl Into<String>,
        identity: &ContentPackIdentity,
        report: &ContentImportReport,
    ) -> Self {
        Self {
            example_id: example_id.into(),
            pack: ContentPackIdentityDto {
                id: identity.id.clone(),
                version: identity.version.clone(),
                fingerprint: None,
            },
            accepted: false,
            error_count: count_severity(
                &report.diagnostics,
                ContentImportDiagnosticSeverity::Error,
            ),
            warning_count: count_severity(
                &report.diagnostics,
                ContentImportDiagnosticSeverity::Warning,
            ),
            diagnostics: report
                .diagnostics
                .iter()
                .map(ContentImportDiagnosticDto::from)
                .collect(),
        }
    }
}

fn count_severity(
    diagnostics: &[ContentImportDiagnostic],
    severity: ContentImportDiagnosticSeverity,
) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == severity)
        .count()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentPackDocumentDto {
    pub format: String,
    pub format_version: u32,
    pub pack: AuthoredContentPackDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentPackDto {
    pub id: String,
    pub version: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub provenance: AuthoredContentProvenanceDto,
    pub ruleset_id: String,
    #[serde(default)]
    pub dependencies: Vec<ContentPackReferenceDto>,
    pub catalogs: AuthoredContentCatalogsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentProvenanceDto {
    pub source_kind: AuthoredContentSourceKindDto,
    pub source_id: String,
    #[serde(default)]
    pub authored_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredContentSourceKindDto {
    AuthoredFile,
    BridgeSubmission,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentCatalogsDto {
    #[serde(default)]
    pub rulesets: Vec<RulesetDefinitionDto>,
    #[serde(default)]
    pub entities: Vec<AuthoredEntityDefinitionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredEntityDefinitionDto {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub damage_adjustments: Vec<AuthoredDamageAdjustmentDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredDamageAdjustmentDto {
    pub damage_type: String,
    pub policy: AuthoredDamageAdjustmentPolicyDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredDamageAdjustmentPolicyDto {
    Resistance,
    Vulnerability,
    Immunity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredContentDecodeError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl AuthoredContentPackDocumentDto {
    pub fn to_authority(&self) -> Result<ContentPackDefinition, AuthoredContentDecodeError> {
        if self.format != AUTHORED_CONTENT_PACK_FORMAT {
            return Err(AuthoredContentDecodeError {
                code: "unsupportedAuthoredContentFormat",
                path: "format".to_string(),
                message: format!("Unsupported authored content format: {}", self.format),
            });
        }
        if self.format_version != AUTHORED_CONTENT_PACK_VERSION {
            return Err(AuthoredContentDecodeError {
                code: "unsupportedAuthoredContentVersion",
                path: "formatVersion".to_string(),
                message: format!(
                    "Authored content format version {} is unsupported; expected {}.",
                    self.format_version, AUTHORED_CONTENT_PACK_VERSION
                ),
            });
        }
        let rulesets = self
            .pack
            .catalogs
            .rulesets
            .iter()
            .enumerate()
            .map(|(index, definition)| {
                validate_ruleset_definition(definition).map_err(|error| {
                    AuthoredContentDecodeError {
                        code: error.code(),
                        path: format!("pack.catalogs.rulesets[{index}]"),
                        message: format!("Authored ruleset was rejected: {error:?}"),
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let selected_ruleset = rulesets
            .iter()
            .find(|ruleset| ruleset.id == self.pack.ruleset_id)
            .ok_or_else(|| AuthoredContentDecodeError {
                code: "missingAuthoredPackRuleset",
                path: "pack.rulesetId".to_string(),
                message: format!(
                    "The authored pack does not declare its selected ruleset {}.",
                    self.pack.ruleset_id
                ),
            })?;
        let entities = self
            .pack
            .catalogs
            .entities
            .iter()
            .map(AuthoredEntityDefinitionDto::to_authority)
            .collect();
        Ok(ContentPackDefinition {
            identity: ContentPackIdentity::new(&self.pack.id, &self.pack.version),
            title: self.pack.title.clone(),
            summary: self.pack.summary.clone(),
            tags: self.pack.tags.clone(),
            provenance: ContentPackProvenance {
                source_kind: match self.pack.provenance.source_kind {
                    AuthoredContentSourceKindDto::AuthoredFile => {
                        ContentPackSourceKind::AuthoredFile
                    }
                    AuthoredContentSourceKindDto::BridgeSubmission => {
                        ContentPackSourceKind::BridgeSubmission
                    }
                },
                source_id: self.pack.provenance.source_id.clone(),
                authored_by: self.pack.provenance.authored_by.clone(),
            },
            ruleset: selected_ruleset.artifact_provenance(),
            dependencies: self
                .pack
                .dependencies
                .iter()
                .map(ContentPackReferenceDto::to_authority)
                .collect(),
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                rulesets,
                entities,
                ..ContentPackCatalogs::default()
            },
        })
    }
}

impl AuthoredEntityDefinitionDto {
    fn to_authority(&self) -> EntityDefinition {
        EntityDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            damage_adjustments: self
                .damage_adjustments
                .iter()
                .map(|adjustment| DamageAdjustment {
                    damage_type: adjustment.damage_type.clone(),
                    policy: match adjustment.policy {
                        AuthoredDamageAdjustmentPolicyDto::Resistance => {
                            DamageAdjustmentPolicy::Resistance
                        }
                        AuthoredDamageAdjustmentPolicyDto::Vulnerability => {
                            DamageAdjustmentPolicy::Vulnerability
                        }
                        AuthoredDamageAdjustmentPolicyDto::Immunity => {
                            DamageAdjustmentPolicy::Immunity
                        }
                    },
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackReferenceDto {
    pub id: String,
    pub version: String,
    pub fingerprint: ContentFingerprintDto,
}

impl ContentPackReferenceDto {
    pub fn to_authority(&self) -> ContentPackReference {
        ContentPackReference {
            id: self.id.clone(),
            version: self.version.clone(),
            fingerprint: rulebench_rules::ContentFingerprint {
                algorithm: self.fingerprint.algorithm.clone(),
                value: self.fingerprint.value.clone(),
            },
        }
    }
}

impl From<&ContentPackReference> for ContentPackReferenceDto {
    fn from(reference: &ContentPackReference) -> Self {
        Self {
            id: reference.id.clone(),
            version: reference.version.clone(),
            fingerprint: ContentFingerprintDto {
                algorithm: reference.fingerprint.algorithm.clone(),
                value: reference.fingerprint.value.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentReplacementPolicyDto {
    Reject,
    ReplaceSameIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportRequestDto {
    pub authored_payload: String,
    pub replacement_policy: ContentReplacementPolicyDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPayloadRequestDto {
    pub authored_payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentReferenceRequestDto {
    pub reference: ContentPackReferenceDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDefinitionSummaryDto {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredContentPackSummaryDto {
    pub reference: ContentPackReferenceDto,
    pub title: String,
    pub summary: String,
    pub source_kind: String,
    pub source_id: String,
    pub authored_by: Option<String>,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub dependencies: Vec<ContentPackReferenceDto>,
    pub definitions: Vec<ContentDefinitionSummaryDto>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackReviewDto {
    pub pack: StoredContentPackSummaryDto,
    pub authored_payload: String,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDefinitionChangeDto {
    pub kind: String,
    pub id: String,
    pub change: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackDiffDto {
    pub before: ContentPackReferenceDto,
    pub after: ContentPackReferenceDto,
    pub changed: bool,
    pub fingerprint_changed: bool,
    pub ruleset_compatibility_changed: bool,
    pub dependency_set_changed: bool,
    pub metadata_changes: Vec<String>,
    pub definition_changes: Vec<ContentDefinitionChangeDto>,
}

impl From<&ContentPackDiffReadout> for ContentPackDiffDto {
    fn from(diff: &ContentPackDiffReadout) -> Self {
        Self {
            before: ContentPackReferenceDto::from(&diff.before),
            after: ContentPackReferenceDto::from(&diff.after),
            changed: diff.changed,
            fingerprint_changed: diff.fingerprint_changed,
            ruleset_compatibility_changed: diff.ruleset_compatibility_changed,
            dependency_set_changed: diff.dependency_set_changed,
            metadata_changes: diff
                .metadata_changes
                .iter()
                .map(|change| metadata_change_code(*change).to_string())
                .collect(),
            definition_changes: diff
                .definition_changes
                .iter()
                .map(|change| ContentDefinitionChangeDto {
                    kind: change.kind.code().to_string(),
                    id: change.id.clone(),
                    change: definition_change_code(change.change).to_string(),
                })
                .collect(),
        }
    }
}

fn metadata_change_code(change: ContentPackMetadataChangeKind) -> &'static str {
    change.code()
}

fn definition_change_code(change: ContentDefinitionChangeKind) -> &'static str {
    change.code()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportOutcomeDto {
    pub review: ContentPackReviewDto,
    pub replaced: Option<ContentPackReferenceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportAttemptDto {
    pub accepted: bool,
    pub pack: ContentPackIdentityDto,
    pub outcome: Option<ContentImportOutcomeDto>,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentAuditEntryDto {
    pub sequence: u64,
    pub operation: String,
    pub reference: ContentPackReferenceDto,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentWorkspaceDto {
    pub packs: Vec<StoredContentPackSummaryDto>,
    pub audit: Vec<ContentAuditEntryDto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_rules::{ContentImportDiagnosticCode, ContentPackDiagnosticCode};

    #[test]
    fn rejected_import_mapping_preserves_rust_diagnostic_identity() {
        let identity = ContentPackIdentity::new("pack.invalid", "1.0.0");
        let report = ContentImportReport {
            accepted: false,
            diagnostics: vec![ContentImportDiagnostic {
                severity: ContentImportDiagnosticSeverity::Error,
                code: ContentImportDiagnosticCode::PackValidation(
                    ContentPackDiagnosticCode::MissingDependency,
                ),
                path: "pack".to_string(),
                definition_kind: None,
                definition_id: Some("pack.missing".to_string()),
                message: "Missing exact dependency.".to_string(),
            }],
        };

        let dto = ContentImportReadoutDto::rejected("error", &identity, &report);

        assert!(!dto.accepted);
        assert_eq!(dto.error_count, 1);
        assert_eq!(dto.diagnostics[0].code, "missingContentPackDependency");
        assert_eq!(
            dto.diagnostics[0].reference_id.as_deref(),
            Some("pack.missing")
        );
    }
}
