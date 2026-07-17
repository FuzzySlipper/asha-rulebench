use crate::{
    canonicalize_content_pack, resolve_content_pack_set, CanonicalContentPack,
    ContentDefinitionKind, ContentPackDefinition, ContentPackDiagnosticCode,
    ResolvedContentPackSet,
};
use rpg_ir::{RulesetMetadata, RulesetProviderCatalog};

mod validation;

use validation::{
    import_pack_diagnostic, rejected, scenario_materialization_diagnostic, sort_diagnostics,
    validate_authored_pack, validate_resolved_action_compatibility,
    validate_resolved_action_references,
};

pub type AuthoredContentPack = ContentPackDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentImportLimits {
    pub maximum_dependencies: usize,
    pub maximum_definitions_per_catalog: usize,
    pub maximum_total_definitions: usize,
    pub maximum_string_bytes: usize,
    pub maximum_operations_per_action: usize,
    pub maximum_reaction_selectors: usize,
    pub maximum_reaction_options: usize,
}

impl Default for ContentImportLimits {
    fn default() -> Self {
        Self {
            maximum_dependencies: 64,
            maximum_definitions_per_catalog: 10_000,
            maximum_total_definitions: 50_000,
            maximum_string_bytes: 16_384,
            maximum_operations_per_action: 64,
            maximum_reaction_selectors: 4,
            maximum_reaction_options: 16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContentImportContext<'a> {
    pub available_packs: &'a [CanonicalContentPack],
    pub rulesets: &'a [RulesetMetadata],
    pub provider_catalog: Option<&'a RulesetProviderCatalog>,
}

impl ContentImportContext<'_> {
    pub const fn empty() -> Self {
        Self {
            available_packs: &[],
            rulesets: &[],
            provider_catalog: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentImportDiagnosticSeverity {
    Error,
    Warning,
}

impl ContentImportDiagnosticSeverity {
    pub const fn code(self) -> &'static str {
        match self {
            ContentImportDiagnosticSeverity::Error => "error",
            ContentImportDiagnosticSeverity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentImportDiagnosticCode {
    EmptyField,
    InvalidFingerprint,
    LimitExceeded,
    DuplicateDefinition,
    InvalidActionDeclaration,
    InvalidModifierDeclaration,
    MissingActionAbility,
    MissingActionModifier,
    UnavailableActionRulesetProvider,
    IncompatibleActionRulesetProvider,
    UnsupportedActionCheck,
    UnsupportedActionTargeting,
    UnsupportedActionEffect,
    DuplicateActionResourceCost,
    InvalidReactionDeclaration,
    InvalidScenarioDeclaration,
    InvalidScenarioInitialState,
    DuplicateTagCanonicalized,
    PackValidation(ContentPackDiagnosticCode),
}

impl ContentImportDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            ContentImportDiagnosticCode::EmptyField => "emptyContentImportField",
            ContentImportDiagnosticCode::InvalidFingerprint => "invalidContentFingerprint",
            ContentImportDiagnosticCode::LimitExceeded => "contentImportLimitExceeded",
            ContentImportDiagnosticCode::DuplicateDefinition => "duplicateContentImportDefinition",
            ContentImportDiagnosticCode::InvalidActionDeclaration => {
                "invalidAuthoredActionDeclaration"
            }
            ContentImportDiagnosticCode::InvalidModifierDeclaration => {
                "invalidAuthoredModifierDeclaration"
            }
            ContentImportDiagnosticCode::MissingActionAbility => "missingAuthoredActionAbility",
            ContentImportDiagnosticCode::MissingActionModifier => "missingAuthoredActionModifier",
            ContentImportDiagnosticCode::UnavailableActionRulesetProvider => {
                "authoredActionRulesetProviderUnavailable"
            }
            ContentImportDiagnosticCode::IncompatibleActionRulesetProvider => {
                "authoredActionRulesetProviderIncompatible"
            }
            ContentImportDiagnosticCode::UnsupportedActionCheck => "unsupportedAuthoredActionCheck",
            ContentImportDiagnosticCode::UnsupportedActionTargeting => {
                "unsupportedAuthoredActionTargeting"
            }
            ContentImportDiagnosticCode::UnsupportedActionEffect => {
                "unsupportedAuthoredActionEffect"
            }
            ContentImportDiagnosticCode::DuplicateActionResourceCost => {
                "duplicateAuthoredActionResourceCost"
            }
            ContentImportDiagnosticCode::InvalidReactionDeclaration => {
                "invalidAuthoredReactionDeclaration"
            }
            ContentImportDiagnosticCode::InvalidScenarioDeclaration => {
                "invalidAuthoredScenarioDeclaration"
            }
            ContentImportDiagnosticCode::InvalidScenarioInitialState => {
                "invalidAuthoredScenarioInitialState"
            }
            ContentImportDiagnosticCode::DuplicateTagCanonicalized => {
                "duplicateContentTagCanonicalized"
            }
            ContentImportDiagnosticCode::PackValidation(code) => code.code(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentImportDiagnostic {
    pub severity: ContentImportDiagnosticSeverity,
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
    pub diagnostics: Vec<ContentImportDiagnostic>,
}

pub fn import_content_pack(
    authored: AuthoredContentPack,
    limits: ContentImportLimits,
    context: ContentImportContext<'_>,
) -> Result<ImportedContentPack, ContentImportReport> {
    let mut diagnostics = validate_authored_pack(&authored, limits);
    sort_diagnostics(&mut diagnostics);
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == ContentImportDiagnosticSeverity::Error)
    {
        return Err(rejected(diagnostics));
    }

    let pack = canonicalize_content_pack(authored);
    let root = pack.exact_reference();
    let mut available_packs = context.available_packs.to_vec();
    available_packs.push(pack.clone());
    let mut rulesets = context.rulesets.to_vec();
    rulesets.extend(pack.catalogs.rulesets.clone());

    match resolve_content_pack_set(&root, &available_packs, &rulesets) {
        Ok(resolved_set) => {
            diagnostics.extend(validate_resolved_action_references(&resolved_set));
            diagnostics.extend(validate_resolved_action_compatibility(
                &resolved_set,
                context.provider_catalog,
            ));
            let candidate = ImportedContentPack {
                pack: pack.clone(),
                resolved_set: resolved_set.clone(),
                diagnostics: diagnostics.clone(),
            };
            for scenario in &pack.catalogs.scenarios {
                if let Err(error) = crate::materialize_authored_scenario(&candidate, &scenario.id) {
                    diagnostics.push(scenario_materialization_diagnostic(&scenario.id, error));
                }
            }
            sort_diagnostics(&mut diagnostics);
            if diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == ContentImportDiagnosticSeverity::Error)
            {
                return Err(rejected(diagnostics));
            }
            Ok(ImportedContentPack {
                pack,
                resolved_set,
                diagnostics,
            })
        }
        Err(report) => {
            diagnostics.extend(report.diagnostics.into_iter().map(import_pack_diagnostic));
            sort_diagnostics(&mut diagnostics);
            Err(rejected(diagnostics))
        }
    }
}

#[cfg(test)]
mod tests;
