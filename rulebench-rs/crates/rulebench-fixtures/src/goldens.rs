/// Package-owned regression manifest.
///
/// The expectation remains Rust data; generated TypeScript is a projection of
/// that evidence and is checked separately through the named command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureGoldenManifest {
    pub package_id: String,
    pub artifacts: Vec<FixtureGoldenArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureGoldenArtifact {
    pub id: String,
    pub kind: FixtureGoldenArtifactKind,
    pub check_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureGoldenContentReviewAttachment {
    pub artifact_id: String,
    pub comparison: rulebench_rules::ContentPackDiffReadout,
}

impl FixtureGoldenContentReviewAttachment {
    pub fn compare(
        artifact_id: impl Into<String>,
        before: &rulebench_rules::CanonicalContentPack,
        after: &rulebench_rules::CanonicalContentPack,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            comparison: rulebench_rules::compare_content_packs(before, after),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureGoldenArtifactKind {
    Receipt,
    ScenarioCatalog,
    SessionTranscript,
    ControlHistory,
    ScriptReadout,
    AutomaticRun,
    ReplayVerification,
    ContentPackReview,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_rules::{
        canonicalize_content_pack, ContentPackCatalogs, ContentPackCollisionPolicy,
        ContentPackDefinition, ContentPackIdentity, ContentPackProvenance, ContentPackSourceKind,
        RulesetMetadata,
    };

    #[test]
    fn golden_workflow_can_attach_structured_content_comparison() {
        let before = canonicalize_content_pack(definition("Before"));
        let after = canonicalize_content_pack(definition("After"));

        let attachment =
            FixtureGoldenContentReviewAttachment::compare("content-pack-review", &before, &after);

        assert_eq!(attachment.artifact_id, "content-pack-review");
        assert!(attachment.comparison.changed);
        assert!(attachment.comparison.definition_changes.is_empty());
    }

    fn definition(title: &str) -> ContentPackDefinition {
        let ruleset = RulesetMetadata {
            id: "rules.golden".to_string(),
            name: "Golden Rules".to_string(),
            version: "1.0.0".to_string(),
            summary: "Golden review rules".to_string(),
            modules: Vec::new(),
        };
        ContentPackDefinition {
            identity: ContentPackIdentity::new("pack.golden", "1.0.0"),
            title: title.to_string(),
            summary: "Golden review pack".to_string(),
            tags: Vec::new(),
            provenance: ContentPackProvenance {
                source_kind: ContentPackSourceKind::Embedded,
                source_id: "fixture:golden".to_string(),
                authored_by: None,
            },
            ruleset: ruleset.artifact_provenance(),
            dependencies: Vec::new(),
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                rulesets: vec![ruleset],
                ..ContentPackCatalogs::default()
            },
        }
    }
}
