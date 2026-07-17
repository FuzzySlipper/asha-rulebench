use rulebench_rpg_adapter::{
    import_content_pack, ContentImportContext, ContentImportLimits, ContentImportReport,
    ContentPackCanonicalVersion, ContentPackCatalogs, ContentPackCollisionPolicy,
    ContentPackDefinition, ContentPackIdentity, ContentPackProvenance, ContentPackSourceKind,
    EntityDefinition, ImportedContentPack, RulesetMetadata,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentImportExample {
    pub id: String,
    pub outcome: ContentImportExampleOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentImportExampleOutcome {
    Accepted(ImportedContentPack),
    Rejected {
        identity: ContentPackIdentity,
        report: ContentImportReport,
    },
}

pub fn content_import_examples() -> Vec<ContentImportExample> {
    let ruleset = crate::hexing_bolt_fixture_scenario()
        .rulesets
        .into_iter()
        .next()
        .expect("fixture scenario has a ruleset");
    let valid = import_example(
        "content-import-valid",
        pack(&ruleset, "pack.valid", "entity.valid"),
    );

    let mut warning_pack = pack(&ruleset, "pack.warning", "entity.warning");
    warning_pack.tags = vec!["fixture".to_string(), "fixture".to_string()];
    let warning = import_example("content-import-warning", warning_pack);

    let dependency = import_content_pack(
        pack(&ruleset, "pack.dependency", "entity.dependency"),
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    )
    .expect("dependency fixture imports");
    let mut error_pack = pack(&ruleset, "pack.error", "entity.error");
    error_pack
        .dependencies
        .push(dependency.pack.exact_reference());
    let error = import_example("content-import-error", error_pack);

    vec![valid, warning, error]
}

fn import_example(id: &str, definition: ContentPackDefinition) -> ContentImportExample {
    let identity = definition.identity.clone();
    let outcome = match import_content_pack(
        definition,
        ContentImportLimits::default(),
        ContentImportContext::empty(),
    ) {
        Ok(imported) => ContentImportExampleOutcome::Accepted(imported),
        Err(report) => ContentImportExampleOutcome::Rejected { identity, report },
    };
    ContentImportExample {
        id: id.to_string(),
        outcome,
    }
}

fn pack(ruleset: &RulesetMetadata, pack_id: &str, entity_id: &str) -> ContentPackDefinition {
    ContentPackDefinition {
        canonical_version: ContentPackCanonicalVersion::V0,
        identity: ContentPackIdentity::new(pack_id, "1.0.0"),
        title: "Content Import Example".to_string(),
        summary: "Generated fixture for content import protocol evidence.".to_string(),
        tags: vec!["fixture".to_string()],
        provenance: ContentPackProvenance {
            source_kind: ContentPackSourceKind::Embedded,
            source_id: format!("fixture:{pack_id}"),
            authored_by: None,
        },
        ruleset: ruleset.artifact_provenance(),
        dependencies: Vec::new(),
        collision_policy: ContentPackCollisionPolicy::Reject,
        catalogs: ContentPackCatalogs {
            rulesets: vec![ruleset.clone()],
            entities: vec![EntityDefinition {
                id: entity_id.to_string(),
                name: "Example Entity".to_string(),
                summary: "Entity used by content import protocol evidence.".to_string(),
                tags: Vec::new(),
                damage_adjustments: Vec::new(),
            }],
            ..ContentPackCatalogs::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_cover_valid_warning_and_error_outcomes() {
        let examples = content_import_examples();
        let accepted = examples
            .iter()
            .filter_map(|example| match &example.outcome {
                ContentImportExampleOutcome::Accepted(imported) => Some(imported),
                ContentImportExampleOutcome::Rejected { .. } => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(accepted.len(), 2);
        assert!(accepted
            .iter()
            .any(|imported| imported.diagnostics.is_empty()));
        assert!(accepted
            .iter()
            .any(|imported| !imported.diagnostics.is_empty()));
        assert!(examples.iter().any(|example| matches!(
            example.outcome,
            ContentImportExampleOutcome::Rejected { .. }
        )));
    }
}
