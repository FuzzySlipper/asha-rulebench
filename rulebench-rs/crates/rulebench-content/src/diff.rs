use std::collections::{BTreeMap, BTreeSet};

use crate::{CanonicalContentPack, ContentDefinitionKind, ContentPackReference};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentDefinitionChangeKind {
    Added,
    Removed,
    Changed,
}

impl ContentDefinitionChangeKind {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDefinitionChangeKind::Added => "added",
            ContentDefinitionChangeKind::Removed => "removed",
            ContentDefinitionChangeKind::Changed => "changed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentDefinitionChange {
    pub kind: ContentDefinitionKind,
    pub id: String,
    pub change: ContentDefinitionChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentPackMetadataChangeKind {
    Identity,
    Title,
    Summary,
    Tags,
    Provenance,
    RulesetCompatibility,
    Dependencies,
    CollisionPolicy,
    Fingerprint,
}

impl ContentPackMetadataChangeKind {
    pub const fn code(self) -> &'static str {
        match self {
            ContentPackMetadataChangeKind::Identity => "identity",
            ContentPackMetadataChangeKind::Title => "title",
            ContentPackMetadataChangeKind::Summary => "summary",
            ContentPackMetadataChangeKind::Tags => "tags",
            ContentPackMetadataChangeKind::Provenance => "provenance",
            ContentPackMetadataChangeKind::RulesetCompatibility => "rulesetCompatibility",
            ContentPackMetadataChangeKind::Dependencies => "dependencies",
            ContentPackMetadataChangeKind::CollisionPolicy => "collisionPolicy",
            ContentPackMetadataChangeKind::Fingerprint => "fingerprint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackDiffReadout {
    pub before: ContentPackReference,
    pub after: ContentPackReference,
    pub changed: bool,
    pub fingerprint_changed: bool,
    pub ruleset_compatibility_changed: bool,
    pub dependency_set_changed: bool,
    pub metadata_changes: Vec<ContentPackMetadataChangeKind>,
    pub definition_changes: Vec<ContentDefinitionChange>,
}

pub fn compare_content_packs(
    before: &CanonicalContentPack,
    after: &CanonicalContentPack,
) -> ContentPackDiffReadout {
    let mut metadata_changes = Vec::new();
    push_metadata_change(
        &mut metadata_changes,
        before.identity != after.identity,
        ContentPackMetadataChangeKind::Identity,
    );
    push_metadata_change(
        &mut metadata_changes,
        before.title != after.title,
        ContentPackMetadataChangeKind::Title,
    );
    push_metadata_change(
        &mut metadata_changes,
        before.summary != after.summary,
        ContentPackMetadataChangeKind::Summary,
    );
    push_metadata_change(
        &mut metadata_changes,
        before.tags != after.tags,
        ContentPackMetadataChangeKind::Tags,
    );
    push_metadata_change(
        &mut metadata_changes,
        before.provenance != after.provenance,
        ContentPackMetadataChangeKind::Provenance,
    );
    let ruleset_compatibility_changed = before.ruleset != after.ruleset;
    push_metadata_change(
        &mut metadata_changes,
        ruleset_compatibility_changed,
        ContentPackMetadataChangeKind::RulesetCompatibility,
    );
    let dependency_set_changed = before.dependencies != after.dependencies;
    push_metadata_change(
        &mut metadata_changes,
        dependency_set_changed,
        ContentPackMetadataChangeKind::Dependencies,
    );
    push_metadata_change(
        &mut metadata_changes,
        before.collision_policy != after.collision_policy,
        ContentPackMetadataChangeKind::CollisionPolicy,
    );
    let fingerprint_changed = before.fingerprint != after.fingerprint;
    push_metadata_change(
        &mut metadata_changes,
        fingerprint_changed,
        ContentPackMetadataChangeKind::Fingerprint,
    );

    let mut definition_changes = Vec::new();
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Ruleset,
        &before.catalogs.rulesets,
        &after.catalogs.rulesets,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Entity,
        &before.catalogs.entities,
        &after.catalogs.entities,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Ability,
        &before.catalogs.abilities,
        &after.catalogs.abilities,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Class,
        &before.catalogs.classes,
        &after.catalogs.classes,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Stat,
        &before.catalogs.stat_definitions,
        &after.catalogs.stat_definitions,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Modifier,
        &before.catalogs.modifiers,
        &after.catalogs.modifiers,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Item,
        &before.catalogs.items,
        &after.catalogs.items,
        |value| &value.id,
    );
    compare_definitions(
        &mut definition_changes,
        ContentDefinitionKind::Action,
        &before.catalogs.actions,
        &after.catalogs.actions,
        |value| &value.id,
    );
    definition_changes.sort();

    ContentPackDiffReadout {
        before: before.exact_reference(),
        after: after.exact_reference(),
        changed: !metadata_changes.is_empty() || !definition_changes.is_empty(),
        fingerprint_changed,
        ruleset_compatibility_changed,
        dependency_set_changed,
        metadata_changes,
        definition_changes,
    }
}

fn push_metadata_change(
    changes: &mut Vec<ContentPackMetadataChangeKind>,
    changed: bool,
    kind: ContentPackMetadataChangeKind,
) {
    if changed {
        changes.push(kind);
    }
}

fn compare_definitions<T: Eq>(
    changes: &mut Vec<ContentDefinitionChange>,
    kind: ContentDefinitionKind,
    before: &[T],
    after: &[T],
    id: impl Fn(&T) -> &String,
) {
    let before_by_id = before
        .iter()
        .map(|value| (id(value).as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let after_by_id = after
        .iter()
        .map(|value| (id(value).as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let ids = before_by_id
        .keys()
        .chain(after_by_id.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    for definition_id in ids {
        let change = match (
            before_by_id.get(definition_id),
            after_by_id.get(definition_id),
        ) {
            (None, Some(_)) => Some(ContentDefinitionChangeKind::Added),
            (Some(_), None) => Some(ContentDefinitionChangeKind::Removed),
            (Some(left), Some(right)) if left != right => {
                Some(ContentDefinitionChangeKind::Changed)
            }
            _ => None,
        };
        if let Some(change) = change {
            changes.push(ContentDefinitionChange {
                kind,
                id: definition_id.to_string(),
                change,
            });
        }
    }
}

#[cfg(test)]
mod tests;
