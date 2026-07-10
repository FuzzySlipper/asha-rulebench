use rulebench_rules::{
    ContentImportDiagnostic, ContentImportDiagnosticSeverity, ContentImportReport,
    ContentPackIdentity, ImportedContentPack,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentFingerprintDto {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackIdentityDto {
    pub id: String,
    pub version: String,
    pub fingerprint: Option<ContentFingerprintDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
