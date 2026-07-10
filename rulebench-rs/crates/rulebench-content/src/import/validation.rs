use std::collections::BTreeSet;

use super::{
    AuthoredContentPack, ContentImportDiagnostic, ContentImportDiagnosticCode, ContentImportLimits,
    ContentImportReport,
};
use crate::{ContentDefinitionKind, ContentPackDiagnostic, CONTENT_PACK_FINGERPRINT_ALGORITHM};

pub(super) fn validate_authored_pack(
    authored: &AuthoredContentPack,
    limits: ContentImportLimits,
) -> Vec<ContentImportDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_required_string(&mut diagnostics, "identity.id", &authored.identity.id);
    validate_required_string(
        &mut diagnostics,
        "identity.version",
        &authored.identity.version,
    );
    validate_required_string(&mut diagnostics, "title", &authored.title);
    validate_required_string(&mut diagnostics, "summary", &authored.summary);
    validate_required_string(
        &mut diagnostics,
        "provenance.sourceId",
        &authored.provenance.source_id,
    );
    validate_required_string(
        &mut diagnostics,
        "ruleset.rulesetId",
        &authored.ruleset.ruleset_id,
    );
    validate_required_string(
        &mut diagnostics,
        "ruleset.rulesetVersion",
        &authored.ruleset.ruleset_version,
    );

    for (path, value) in pack_strings(authored) {
        if value.len() > limits.maximum_string_bytes {
            diagnostics.push(ContentImportDiagnostic {
                code: ContentImportDiagnosticCode::LimitExceeded,
                path: path.to_string(),
                definition_kind: None,
                definition_id: None,
                message: format!(
                    "Content field {path} contains {} bytes; the limit is {}.",
                    value.len(),
                    limits.maximum_string_bytes
                ),
            });
        }
    }

    if authored.dependencies.len() > limits.maximum_dependencies {
        diagnostics.push(limit_diagnostic(
            "dependencies",
            authored.dependencies.len(),
            limits.maximum_dependencies,
        ));
    }

    let catalogs = catalog_identities(authored);
    let total_definitions = catalogs.iter().map(|(_, ids)| ids.len()).sum::<usize>();
    if total_definitions > limits.maximum_total_definitions {
        diagnostics.push(limit_diagnostic(
            "catalogs",
            total_definitions,
            limits.maximum_total_definitions,
        ));
    }
    for (kind, ids) in catalogs {
        validate_catalog(&mut diagnostics, kind, &ids, limits);
    }

    for (index, dependency) in authored.dependencies.iter().enumerate() {
        validate_fingerprint(
            &mut diagnostics,
            &format!("dependencies[{index}].fingerprint"),
            &dependency.fingerprint.algorithm,
            &dependency.fingerprint.value,
        );
    }
    diagnostics
}

fn validate_catalog(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    kind: ContentDefinitionKind,
    ids: &[String],
    limits: ContentImportLimits,
) {
    if ids.len() > limits.maximum_definitions_per_catalog {
        diagnostics.push(ContentImportDiagnostic {
            code: ContentImportDiagnosticCode::LimitExceeded,
            path: format!("catalogs.{}", kind.code()),
            definition_kind: Some(kind),
            definition_id: None,
            message: format!(
                "Content {} catalog contains {} definitions; the limit is {}.",
                kind.code(),
                ids.len(),
                limits.maximum_definitions_per_catalog
            ),
        });
    }
    validate_duplicate_ids(diagnostics, kind, ids);
    for id in ids {
        if id.is_empty() {
            diagnostics.push(ContentImportDiagnostic {
                code: ContentImportDiagnosticCode::EmptyField,
                path: format!("catalogs.{}[].id", kind.code()),
                definition_kind: Some(kind),
                definition_id: None,
                message: format!("Content {} id must not be empty.", kind.code()),
            });
        }
    }
}

fn validate_required_string(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    path: &str,
    value: &str,
) {
    if value.is_empty() {
        diagnostics.push(ContentImportDiagnostic {
            code: ContentImportDiagnosticCode::EmptyField,
            path: path.to_string(),
            definition_kind: None,
            definition_id: None,
            message: format!("Content import field {path} must not be empty."),
        });
    }
}

fn validate_fingerprint(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    path: &str,
    algorithm: &str,
    value: &str,
) {
    let valid_value = value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if algorithm != CONTENT_PACK_FINGERPRINT_ALGORITHM || !valid_value {
        diagnostics.push(ContentImportDiagnostic {
            code: ContentImportDiagnosticCode::InvalidFingerprint,
            path: path.to_string(),
            definition_kind: None,
            definition_id: None,
            message: format!(
                "Content fingerprint at {path} must use {CONTENT_PACK_FINGERPRINT_ALGORITHM} with 16 lowercase hexadecimal characters."
            ),
        });
    }
}

fn limit_diagnostic(path: &str, actual: usize, maximum: usize) -> ContentImportDiagnostic {
    ContentImportDiagnostic {
        code: ContentImportDiagnosticCode::LimitExceeded,
        path: path.to_string(),
        definition_kind: None,
        definition_id: None,
        message: format!("Content field {path} contains {actual} entries; the limit is {maximum}."),
    }
}

fn validate_duplicate_ids(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    kind: ContentDefinitionKind,
    ids: &[String],
) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            diagnostics.push(ContentImportDiagnostic {
                code: ContentImportDiagnosticCode::DuplicateDefinition,
                path: format!("catalogs.{}", kind.code()),
                definition_kind: Some(kind),
                definition_id: Some(id.clone()),
                message: format!(
                    "Content {} id {id} is declared more than once.",
                    kind.code()
                ),
            });
        }
    }
}

fn pack_strings(authored: &AuthoredContentPack) -> Vec<(&'static str, &str)> {
    let mut strings = vec![
        ("identity.id", authored.identity.id.as_str()),
        ("identity.version", authored.identity.version.as_str()),
        ("title", authored.title.as_str()),
        ("summary", authored.summary.as_str()),
        (
            "provenance.sourceId",
            authored.provenance.source_id.as_str(),
        ),
        ("ruleset.rulesetId", authored.ruleset.ruleset_id.as_str()),
        (
            "ruleset.rulesetVersion",
            authored.ruleset.ruleset_version.as_str(),
        ),
    ];
    if let Some(authored_by) = authored.provenance.authored_by.as_deref() {
        strings.push(("provenance.authoredBy", authored_by));
    }
    strings
}

fn catalog_identities(authored: &AuthoredContentPack) -> Vec<(ContentDefinitionKind, Vec<String>)> {
    vec![
        (
            ContentDefinitionKind::Ruleset,
            ids(&authored.catalogs.rulesets, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Entity,
            ids(&authored.catalogs.entities, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Ability,
            ids(&authored.catalogs.abilities, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Class,
            ids(&authored.catalogs.classes, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Stat,
            ids(&authored.catalogs.stat_definitions, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Modifier,
            ids(&authored.catalogs.modifiers, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Item,
            ids(&authored.catalogs.items, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Action,
            ids(&authored.catalogs.actions, |value| &value.id),
        ),
    ]
}

fn ids<T>(values: &[T], id: impl Fn(&T) -> &String) -> Vec<String> {
    values.iter().map(|value| id(value).clone()).collect()
}

pub(super) fn import_pack_diagnostic(diagnostic: ContentPackDiagnostic) -> ContentImportDiagnostic {
    let path = diagnostic
        .definition_kind
        .map(|kind| format!("catalogs.{}", kind.code()))
        .unwrap_or_else(|| "pack".to_string());
    ContentImportDiagnostic {
        code: ContentImportDiagnosticCode::PackValidation(diagnostic.code),
        path,
        definition_kind: diagnostic.definition_kind,
        definition_id: diagnostic.reference_id,
        message: diagnostic.message,
    }
}

pub(super) fn sort_diagnostics(diagnostics: &mut [ContentImportDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            &left.path,
            left.code,
            left.definition_kind,
            &left.definition_id,
            &left.message,
        )
            .cmp(&(
                &right.path,
                right.code,
                right.definition_kind,
                &right.definition_id,
                &right.message,
            ))
    });
}

pub(super) fn rejected(diagnostics: Vec<ContentImportDiagnostic>) -> ContentImportReport {
    ContentImportReport {
        accepted: false,
        diagnostics,
    }
}
