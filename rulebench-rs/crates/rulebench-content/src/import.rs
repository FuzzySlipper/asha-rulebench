use crate::{
    canonicalize_content_pack, resolve_content_pack_set, CanonicalContentPack,
    ContentDefinitionKind, ContentPackDefinition, ContentPackDiagnosticCode,
    ResolvedContentPackSet,
};
use rulebench_ruleset::RulesetMetadata;

mod validation;

use validation::{import_pack_diagnostic, rejected, sort_diagnostics, validate_authored_pack};

pub type AuthoredContentPack = ContentPackDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentImportLimits {
    pub maximum_dependencies: usize,
    pub maximum_definitions_per_catalog: usize,
    pub maximum_total_definitions: usize,
    pub maximum_string_bytes: usize,
}

impl Default for ContentImportLimits {
    fn default() -> Self {
        Self {
            maximum_dependencies: 64,
            maximum_definitions_per_catalog: 10_000,
            maximum_total_definitions: 50_000,
            maximum_string_bytes: 16_384,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContentImportContext<'a> {
    pub available_packs: &'a [CanonicalContentPack],
    pub rulesets: &'a [RulesetMetadata],
}

impl ContentImportContext<'_> {
    pub const fn empty() -> Self {
        Self {
            available_packs: &[],
            rulesets: &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentImportDiagnosticCode {
    EmptyField,
    InvalidFingerprint,
    LimitExceeded,
    DuplicateDefinition,
    PackValidation(ContentPackDiagnosticCode),
}

impl ContentImportDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            ContentImportDiagnosticCode::EmptyField => "emptyContentImportField",
            ContentImportDiagnosticCode::InvalidFingerprint => "invalidContentFingerprint",
            ContentImportDiagnosticCode::LimitExceeded => "contentImportLimitExceeded",
            ContentImportDiagnosticCode::DuplicateDefinition => "duplicateContentImportDefinition",
            ContentImportDiagnosticCode::PackValidation(code) => code.code(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentImportDiagnostic {
    pub code: ContentImportDiagnosticCode,
    pub path: String,
    pub definition_kind: Option<ContentDefinitionKind>,
    pub definition_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentImportReport {
    pub accepted: bool,
    pub diagnostics: Vec<ContentImportDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedContentPack {
    pub pack: CanonicalContentPack,
    pub resolved_set: ResolvedContentPackSet,
}

pub fn import_content_pack(
    authored: AuthoredContentPack,
    limits: ContentImportLimits,
    context: ContentImportContext<'_>,
) -> Result<ImportedContentPack, ContentImportReport> {
    let mut diagnostics = validate_authored_pack(&authored, limits);
    sort_diagnostics(&mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(rejected(diagnostics));
    }

    let pack = canonicalize_content_pack(authored);
    let root = pack.exact_reference();
    let mut available_packs = context.available_packs.to_vec();
    available_packs.push(pack.clone());
    let mut rulesets = context.rulesets.to_vec();
    rulesets.extend(pack.catalogs.rulesets.clone());

    match resolve_content_pack_set(&root, &available_packs, &rulesets) {
        Ok(resolved_set) => Ok(ImportedContentPack { pack, resolved_set }),
        Err(report) => {
            let mut diagnostics = report
                .diagnostics
                .into_iter()
                .map(import_pack_diagnostic)
                .collect::<Vec<_>>();
            sort_diagnostics(&mut diagnostics);
            Err(rejected(diagnostics))
        }
    }
}

#[cfg(test)]
mod tests;
