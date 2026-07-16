use std::collections::{BTreeMap, BTreeSet};

use crate::{
    fingerprint_content_pack_set, CanonicalContentPack, ContentDefinitionKind, ContentPackIdentity,
    ContentPackReference, ContentPackSetReference,
};
use rulebench_ruleset::{RulesetCompatibilityError, RulesetMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentPackDiagnosticCode {
    EmptyPackId,
    EmptyPackVersion,
    EmptyPackTitle,
    EmptyPackSummary,
    EmptyProvenanceSource,
    DuplicateAvailablePackIdentity,
    DuplicateDependency,
    MissingDependency,
    IncompatibleDependencyVersion,
    IncompatibleDependencyFingerprint,
    CyclicDependency,
    MissingRuleset,
    IncompatibleRuleset,
    DefinitionCollision,
}

impl ContentPackDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            ContentPackDiagnosticCode::EmptyPackId => "emptyContentPackId",
            ContentPackDiagnosticCode::EmptyPackVersion => "emptyContentPackVersion",
            ContentPackDiagnosticCode::EmptyPackTitle => "emptyContentPackTitle",
            ContentPackDiagnosticCode::EmptyPackSummary => "emptyContentPackSummary",
            ContentPackDiagnosticCode::EmptyProvenanceSource => "emptyContentPackProvenanceSource",
            ContentPackDiagnosticCode::DuplicateAvailablePackIdentity => {
                "duplicateAvailableContentPackIdentity"
            }
            ContentPackDiagnosticCode::DuplicateDependency => "duplicateContentPackDependency",
            ContentPackDiagnosticCode::MissingDependency => "missingContentPackDependency",
            ContentPackDiagnosticCode::IncompatibleDependencyVersion => {
                "incompatibleContentPackDependencyVersion"
            }
            ContentPackDiagnosticCode::IncompatibleDependencyFingerprint => {
                "incompatibleContentPackDependencyFingerprint"
            }
            ContentPackDiagnosticCode::CyclicDependency => "cyclicContentPackDependency",
            ContentPackDiagnosticCode::MissingRuleset => "missingContentPackRuleset",
            ContentPackDiagnosticCode::IncompatibleRuleset => "incompatibleContentPackRuleset",
            ContentPackDiagnosticCode::DefinitionCollision => "contentPackDefinitionCollision",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackDiagnostic {
    pub code: ContentPackDiagnosticCode,
    pub pack_id: String,
    pub reference_id: Option<String>,
    pub definition_kind: Option<ContentDefinitionKind>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackValidationReport {
    pub accepted: bool,
    pub diagnostics: Vec<ContentPackDiagnostic>,
}

impl ContentPackValidationReport {
    fn rejected(diagnostics: Vec<ContentPackDiagnostic>) -> Self {
        Self {
            accepted: false,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedContentPackSet {
    pub root: ContentPackReference,
    pub packs: Vec<CanonicalContentPack>,
    pub reference: ContentPackSetReference,
}

pub fn resolve_content_pack_set(
    root: &ContentPackReference,
    available: &[CanonicalContentPack],
    rulesets: &[RulesetMetadata],
) -> Result<ResolvedContentPackSet, ContentPackValidationReport> {
    let mut resolver = ContentPackResolver::new(available, rulesets);
    resolver.validate_available_identities();
    resolver.visit(root);
    resolver.validate_collisions();

    if !resolver.diagnostics.is_empty() {
        return Err(ContentPackValidationReport::rejected(resolver.diagnostics));
    }

    let mut packs = resolver.resolved;
    packs.sort_by(|left, right| left.identity.cmp(&right.identity));
    let references = packs
        .iter()
        .map(CanonicalContentPack::exact_reference)
        .collect::<Vec<_>>();
    let reference = ContentPackSetReference {
        root: root.clone(),
        fingerprint: fingerprint_content_pack_set(root, &references),
        packs: references,
    };

    Ok(ResolvedContentPackSet {
        root: root.clone(),
        packs,
        reference,
    })
}

struct ContentPackResolver<'a> {
    available: &'a [CanonicalContentPack],
    rulesets: &'a [RulesetMetadata],
    visiting: BTreeSet<ContentPackIdentity>,
    visited: BTreeSet<ContentPackIdentity>,
    resolved: Vec<CanonicalContentPack>,
    diagnostics: Vec<ContentPackDiagnostic>,
}

impl<'a> ContentPackResolver<'a> {
    fn new(available: &'a [CanonicalContentPack], rulesets: &'a [RulesetMetadata]) -> Self {
        Self {
            available,
            rulesets,
            visiting: BTreeSet::new(),
            visited: BTreeSet::new(),
            resolved: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn validate_available_identities(&mut self) {
        let mut identities = BTreeSet::new();
        for pack in self.available {
            if !identities.insert(pack.identity.clone()) {
                self.push_diagnostic(
                    ContentPackDiagnosticCode::DuplicateAvailablePackIdentity,
                    &pack.identity.id,
                    Some(pack.identity.version.clone()),
                    None,
                    format!(
                        "Multiple available packs declare {}@{}; load order is not authoritative.",
                        pack.identity.id, pack.identity.version
                    ),
                );
            }
        }
    }

    fn visit(&mut self, reference: &ContentPackReference) {
        let Some(pack) = self.pack_for_reference(reference) else {
            return;
        };
        let identity = pack.identity.clone();
        if self.visited.contains(&identity) {
            return;
        }
        if !self.visiting.insert(identity.clone()) {
            self.push_diagnostic(
                ContentPackDiagnosticCode::CyclicDependency,
                &identity.id,
                Some(reference.id.clone()),
                None,
                format!(
                    "Content pack {}@{} participates in a dependency cycle.",
                    identity.id, identity.version
                ),
            );
            return;
        }

        self.validate_pack_metadata(pack);
        self.validate_ruleset(pack);
        let mut seen_dependencies = BTreeSet::new();
        for dependency in &pack.dependencies {
            let dependency_key = (
                dependency.id.clone(),
                dependency.version.clone(),
                dependency.fingerprint.clone(),
            );
            if !seen_dependencies.insert(dependency_key) {
                self.push_diagnostic(
                    ContentPackDiagnosticCode::DuplicateDependency,
                    &pack.identity.id,
                    Some(dependency.id.clone()),
                    None,
                    format!(
                        "Content pack {} repeats dependency {}@{}.",
                        pack.identity.id, dependency.id, dependency.version
                    ),
                );
                continue;
            }
            self.visit(dependency);
        }

        self.visiting.remove(&identity);
        self.visited.insert(identity);
        self.resolved.push(pack.clone());
    }

    fn pack_for_reference(
        &mut self,
        reference: &ContentPackReference,
    ) -> Option<&'a CanonicalContentPack> {
        let candidates = self
            .available
            .iter()
            .filter(|pack| pack.identity.id == reference.id)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            self.push_diagnostic(
                ContentPackDiagnosticCode::MissingDependency,
                &reference.id,
                Some(reference.id.clone()),
                None,
                format!(
                    "Content pack {}@{} is not available.",
                    reference.id, reference.version
                ),
            );
            return None;
        }

        let Some(pack) = candidates
            .into_iter()
            .find(|pack| pack.identity.version == reference.version)
        else {
            self.push_diagnostic(
                ContentPackDiagnosticCode::IncompatibleDependencyVersion,
                &reference.id,
                Some(reference.id.clone()),
                None,
                format!(
                    "Content pack {} does not provide required version {}.",
                    reference.id, reference.version
                ),
            );
            return None;
        };

        if pack.fingerprint != reference.fingerprint {
            self.push_diagnostic(
                ContentPackDiagnosticCode::IncompatibleDependencyFingerprint,
                &reference.id,
                Some(reference.id.clone()),
                None,
                format!(
                    "Content pack {}@{} does not match required fingerprint {}.",
                    reference.id, reference.version, reference.fingerprint.value
                ),
            );
            return None;
        }
        Some(pack)
    }

    fn validate_pack_metadata(&mut self, pack: &CanonicalContentPack) {
        let checks = [
            (
                pack.identity.id.is_empty(),
                ContentPackDiagnosticCode::EmptyPackId,
                "Content pack id must not be empty.",
            ),
            (
                pack.identity.version.is_empty(),
                ContentPackDiagnosticCode::EmptyPackVersion,
                "Content pack version must not be empty.",
            ),
            (
                pack.title.is_empty(),
                ContentPackDiagnosticCode::EmptyPackTitle,
                "Content pack title must not be empty.",
            ),
            (
                pack.summary.is_empty(),
                ContentPackDiagnosticCode::EmptyPackSummary,
                "Content pack summary must not be empty.",
            ),
            (
                pack.provenance.source_id.is_empty(),
                ContentPackDiagnosticCode::EmptyProvenanceSource,
                "Content pack provenance source must not be empty.",
            ),
        ];
        for (failed, code, message) in checks {
            if failed {
                self.push_diagnostic(code, &pack.identity.id, None, None, message.to_string());
            }
        }
    }

    fn validate_ruleset(&mut self, pack: &CanonicalContentPack) {
        let Some(ruleset) = self
            .rulesets
            .iter()
            .find(|ruleset| ruleset.id == pack.ruleset.ruleset_id)
        else {
            self.push_diagnostic(
                ContentPackDiagnosticCode::MissingRuleset,
                &pack.identity.id,
                Some(pack.ruleset.ruleset_id.clone()),
                Some(ContentDefinitionKind::Ruleset),
                format!(
                    "Content pack {} requires missing ruleset {}@{}.",
                    pack.identity.id, pack.ruleset.ruleset_id, pack.ruleset.ruleset_version
                ),
            );
            return;
        };

        if let Err(error) = ruleset.validate_artifact_provenance(&pack.ruleset) {
            self.push_diagnostic(
                ContentPackDiagnosticCode::IncompatibleRuleset,
                &pack.identity.id,
                Some(pack.ruleset.ruleset_id.clone()),
                Some(ContentDefinitionKind::Ruleset),
                incompatible_ruleset_message(pack, &error),
            );
        }
    }

    fn validate_collisions(&mut self) {
        let mut owners = BTreeMap::<
            (ContentDefinitionKind, String),
            (ContentPackIdentity, Option<RulesetMetadata>),
        >::new();
        for pack in self.resolved.clone() {
            for (kind, id) in definition_identities(&pack) {
                let key = (kind, id.clone());
                let ruleset = (kind == ContentDefinitionKind::Ruleset)
                    .then(|| {
                        pack.catalogs
                            .rulesets
                            .iter()
                            .find(|ruleset| ruleset.id == id)
                            .cloned()
                    })
                    .flatten();
                if let Some((owner, owner_ruleset)) = owners.get(&key) {
                    // An exact repeated ruleset declaration is compatibility
                    // metadata, not load-order override authority. This lets
                    // exact dependency packs share the selected ruleset while
                    // every differing or non-ruleset definition still fails
                    // closed as a collision.
                    if kind == ContentDefinitionKind::Ruleset
                        && owner_ruleset.as_ref() == ruleset.as_ref()
                    {
                        continue;
                    }
                    self.push_diagnostic(
                        ContentPackDiagnosticCode::DefinitionCollision,
                        &pack.identity.id,
                        Some(id.clone()),
                        Some(kind),
                        format!(
                            "Content {} {} from {}@{} collides with {}@{}; override precedence is not supported.",
                            kind.code(),
                            id,
                            pack.identity.id,
                            pack.identity.version,
                            owner.id,
                            owner.version
                        ),
                    );
                } else {
                    owners.insert(key, (pack.identity.clone(), ruleset));
                }
            }
        }
    }

    fn push_diagnostic(
        &mut self,
        code: ContentPackDiagnosticCode,
        pack_id: &str,
        reference_id: Option<String>,
        definition_kind: Option<ContentDefinitionKind>,
        message: String,
    ) {
        self.diagnostics.push(ContentPackDiagnostic {
            code,
            pack_id: pack_id.to_string(),
            reference_id,
            definition_kind,
            message,
        });
    }
}

fn definition_identities(pack: &CanonicalContentPack) -> Vec<(ContentDefinitionKind, String)> {
    let mut definitions = Vec::new();
    definitions.extend(
        pack.catalogs
            .rulesets
            .iter()
            .map(|definition| (ContentDefinitionKind::Ruleset, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .entities
            .iter()
            .map(|definition| (ContentDefinitionKind::Entity, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .abilities
            .iter()
            .map(|definition| (ContentDefinitionKind::Ability, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .classes
            .iter()
            .map(|definition| (ContentDefinitionKind::Class, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .stat_definitions
            .iter()
            .map(|definition| (ContentDefinitionKind::Stat, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .modifiers
            .iter()
            .map(|definition| (ContentDefinitionKind::Modifier, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .items
            .iter()
            .map(|definition| (ContentDefinitionKind::Item, definition.id.clone())),
    );
    definitions.extend(
        pack.catalogs
            .actions
            .iter()
            .map(|definition| (ContentDefinitionKind::Action, definition.id.clone())),
    );
    definitions
}

fn incompatible_ruleset_message(
    pack: &CanonicalContentPack,
    error: &RulesetCompatibilityError,
) -> String {
    format!(
        "Content pack {} requires incompatible ruleset provenance {} ({}).",
        pack.identity.id,
        pack.ruleset.ruleset_id,
        error.code()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        canonicalize_content_pack, ContentPackCanonicalVersion, ContentPackCatalogs,
        ContentPackCollisionPolicy, ContentPackDefinition, ContentPackProvenance,
        ContentPackSourceKind, EntityDefinition,
    };
    use rulebench_ruleset::{
        ActionResolutionModuleConfiguration, RuleModuleDeclaration, RulesetMetadata,
    };

    #[test]
    fn exact_dependencies_resolve_into_a_deterministic_pack_set_reference() {
        let ruleset = ruleset();
        let base = pack("base", Vec::new(), "entity.base", &ruleset);
        let root = pack(
            "root",
            vec![base.exact_reference()],
            "entity.root",
            &ruleset,
        );

        let first = resolve_content_pack_set(
            &root.exact_reference(),
            &[root.clone(), base.clone()],
            std::slice::from_ref(&ruleset),
        )
        .expect("exact dependencies resolve");
        let second = resolve_content_pack_set(
            &root.exact_reference(),
            &[base, root],
            std::slice::from_ref(&ruleset),
        )
        .expect("available ordering is irrelevant");

        assert_eq!(first.reference, second.reference);
        assert!(first.reference.is_self_consistent());
        assert_eq!(first.packs.len(), 2);
        assert_eq!(first.reference.root.id, "root");
        assert_eq!(first.reference.packs[0].id, "base");
        assert_eq!(first.reference.packs[1].id, "root");
    }

    #[test]
    fn dependency_lookup_fails_closed_for_missing_version_and_fingerprint() {
        let ruleset = ruleset();
        let base = pack("base", Vec::new(), "entity.base", &ruleset);
        let missing = reference("missing", "1.0.0", "0000");
        let root = pack("root", vec![missing], "entity.root", &ruleset);
        let missing_report = resolve_content_pack_set(
            &root.exact_reference(),
            &[root.clone(), base.clone()],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("missing dependency fails");
        assert_eq!(
            missing_report.diagnostics[0].code,
            ContentPackDiagnosticCode::MissingDependency
        );

        let wrong_version = pack(
            "version-root",
            vec![reference("base", "2.0.0", &base.fingerprint.value)],
            "entity.version-root",
            &ruleset,
        );
        let version_report = resolve_content_pack_set(
            &wrong_version.exact_reference(),
            &[wrong_version.clone(), base.clone()],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("version mismatch fails");
        assert_eq!(
            version_report.diagnostics[0].code,
            ContentPackDiagnosticCode::IncompatibleDependencyVersion
        );

        let wrong_fingerprint = pack(
            "fingerprint-root",
            vec![reference("base", "1.0.0", "ffffffffffffffff")],
            "entity.fingerprint-root",
            &ruleset,
        );
        let fingerprint_report = resolve_content_pack_set(
            &wrong_fingerprint.exact_reference(),
            &[wrong_fingerprint, base],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("fingerprint mismatch fails");
        assert_eq!(
            fingerprint_report.diagnostics[0].code,
            ContentPackDiagnosticCode::IncompatibleDependencyFingerprint
        );
    }

    #[test]
    fn reject_only_collision_policy_has_no_load_order_precedence() {
        let ruleset = ruleset();
        let base = pack("base", Vec::new(), "entity.shared", &ruleset);
        let root = pack(
            "root",
            vec![base.exact_reference()],
            "entity.shared",
            &ruleset,
        );

        let report = resolve_content_pack_set(
            &root.exact_reference(),
            &[root, base],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("definition override is rejected");

        let collision = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == ContentPackDiagnosticCode::DefinitionCollision)
            .expect("collision diagnostic");
        assert_eq!(
            collision.definition_kind,
            Some(ContentDefinitionKind::Entity)
        );
        assert!(collision
            .message
            .contains("override precedence is not supported"));
    }

    #[test]
    fn ruleset_provenance_is_exact_and_diagnostic_codes_are_stable() {
        let ruleset = ruleset();
        let root_pack = pack("root", Vec::new(), "entity.root", &ruleset);
        let report = resolve_content_pack_set(&root_pack.exact_reference(), &[root_pack], &[])
            .expect_err("missing ruleset fails");

        assert_eq!(
            report.diagnostics[0].code.code(),
            "missingContentPackRuleset"
        );
        assert_eq!(
            ContentPackDiagnosticCode::IncompatibleDependencyFingerprint.code(),
            "incompatibleContentPackDependencyFingerprint"
        );
        assert_eq!(
            ContentPackDiagnosticCode::DefinitionCollision.code(),
            "contentPackDefinitionCollision"
        );

        let mut incompatible_pack = pack("incompatible", Vec::new(), "entity.other", &ruleset);
        incompatible_pack.ruleset.ruleset_version = "2.0.0".to_string();
        let incompatible_report = resolve_content_pack_set(
            &incompatible_pack.exact_reference(),
            &[incompatible_pack],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("incompatible provenance fails");
        assert_eq!(
            incompatible_report.diagnostics[0].code,
            ContentPackDiagnosticCode::IncompatibleRuleset
        );
    }

    #[test]
    fn duplicate_available_identity_is_rejected_without_load_order_selection() {
        let ruleset = ruleset();
        let pack = pack("root", Vec::new(), "entity.root", &ruleset);

        let report = resolve_content_pack_set(
            &pack.exact_reference(),
            &[pack.clone(), pack],
            std::slice::from_ref(&ruleset),
        )
        .expect_err("duplicate available identity is ambiguous");

        assert_eq!(
            report.diagnostics[0].code,
            ContentPackDiagnosticCode::DuplicateAvailablePackIdentity
        );
    }

    fn pack(
        id: &str,
        dependencies: Vec<ContentPackReference>,
        entity_id: &str,
        ruleset: &RulesetMetadata,
    ) -> CanonicalContentPack {
        canonicalize_content_pack(ContentPackDefinition {
            canonical_version: ContentPackCanonicalVersion::V0,
            identity: ContentPackIdentity::new(id, "1.0.0"),
            title: format!("{id} pack"),
            summary: format!("Canonical content for {id}."),
            tags: vec!["test".to_string()],
            provenance: ContentPackProvenance {
                source_kind: ContentPackSourceKind::Embedded,
                source_id: format!("tests/{id}"),
                authored_by: Some("rulebench-tests".to_string()),
            },
            ruleset: ruleset.artifact_provenance(),
            dependencies,
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                entities: vec![EntityDefinition {
                    id: entity_id.to_string(),
                    name: entity_id.to_string(),
                    summary: "Test entity.".to_string(),
                    tags: Vec::new(),
                    damage_adjustments: Vec::new(),
                }],
                ..ContentPackCatalogs::default()
            },
        })
    }

    fn reference(id: &str, version: &str, fingerprint: &str) -> ContentPackReference {
        ContentPackReference {
            id: id.to_string(),
            version: version.to_string(),
            fingerprint: crate::ContentFingerprint {
                algorithm: crate::CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
                value: fingerprint.to_string(),
            },
        }
    }

    fn ruleset() -> RulesetMetadata {
        RulesetMetadata {
            id: "rules.test".to_string(),
            name: "Test Rules".to_string(),
            version: "1.0.0".to_string(),
            summary: "Content pack test ruleset.".to_string(),
            modules: vec![RuleModuleDeclaration::action_resolution(
                ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
            )],
        }
    }
}
