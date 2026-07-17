use crate::{
    AuthoredActionDefinition, AuthoredScenarioDefinition, ClassDefinition, EntityDefinition,
    ItemDefinition, ModifierDefinition, StatDefinition,
};
use rulebench_ruleset::{AbilityDefinition, RulesetArtifactProvenance, RulesetMetadata};

pub const CONTENT_PACK_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-content-pack.v0";
pub const CONTENT_PACK_FINGERPRINT_ALGORITHM_V1: &str = "fnv1a64.rulebench-content-pack.v1";
pub const CONTENT_PACK_FINGERPRINT_ALGORITHM_V2: &str = "fnv1a64.rulebench-content-pack.v2";
pub const CONTENT_PACK_SET_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-content-pack-set.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentPackCanonicalVersion {
    V0,
    V1,
    V2,
}

impl ContentPackCanonicalVersion {
    pub const fn fingerprint_algorithm(self) -> &'static str {
        match self {
            Self::V0 => CONTENT_PACK_FINGERPRINT_ALGORITHM,
            Self::V1 => CONTENT_PACK_FINGERPRINT_ALGORITHM_V1,
            Self::V2 => CONTENT_PACK_FINGERPRINT_ALGORITHM_V2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentPackIdentity {
    pub id: String,
    pub version: String,
}

impl ContentPackIdentity {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentPackSourceKind {
    Embedded,
    AuthoredFile,
    BridgeSubmission,
}

impl ContentPackSourceKind {
    pub const fn code(self) -> &'static str {
        match self {
            ContentPackSourceKind::Embedded => "embedded",
            ContentPackSourceKind::AuthoredFile => "authoredFile",
            ContentPackSourceKind::BridgeSubmission => "bridgeSubmission",
        }
    }
}

/// Stable source evidence. `source_id` is a logical authoring identity, never
/// an absolute host path or an import timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackProvenance {
    pub source_kind: ContentPackSourceKind,
    pub source_id: String,
    pub authored_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentFingerprint {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentPackReference {
    pub id: String,
    pub version: String,
    pub fingerprint: ContentFingerprint,
}

impl ContentPackReference {
    pub fn from_pack(pack: &CanonicalContentPack) -> Self {
        Self {
            id: pack.identity.id.clone(),
            version: pack.identity.version.clone(),
            fingerprint: pack.fingerprint.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackSetReference {
    pub root: ContentPackReference,
    pub packs: Vec<ContentPackReference>,
    pub fingerprint: ContentFingerprint,
}

impl ContentPackSetReference {
    pub fn is_self_consistent(&self) -> bool {
        self.packs.contains(&self.root)
            && self.fingerprint
                == crate::canonical::fingerprint_content_pack_set(&self.root, &self.packs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentPackCollisionPolicy {
    Reject,
}

impl ContentPackCollisionPolicy {
    pub const fn code(self) -> &'static str {
        match self {
            ContentPackCollisionPolicy::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentDefinitionKind {
    Ruleset,
    Entity,
    Ability,
    Class,
    Stat,
    Modifier,
    Item,
    Action,
    Scenario,
}

impl ContentDefinitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDefinitionKind::Ruleset => "ruleset",
            ContentDefinitionKind::Entity => "entity",
            ContentDefinitionKind::Ability => "ability",
            ContentDefinitionKind::Class => "class",
            ContentDefinitionKind::Stat => "stat",
            ContentDefinitionKind::Modifier => "modifier",
            ContentDefinitionKind::Item => "item",
            ContentDefinitionKind::Action => "action",
            ContentDefinitionKind::Scenario => "scenario",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "ruleset" => Some(ContentDefinitionKind::Ruleset),
            "entity" => Some(ContentDefinitionKind::Entity),
            "ability" => Some(ContentDefinitionKind::Ability),
            "class" => Some(ContentDefinitionKind::Class),
            "stat" => Some(ContentDefinitionKind::Stat),
            "modifier" => Some(ContentDefinitionKind::Modifier),
            "item" => Some(ContentDefinitionKind::Item),
            "action" => Some(ContentDefinitionKind::Action),
            "scenario" => Some(ContentDefinitionKind::Scenario),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentDefinitionReference {
    pub kind: ContentDefinitionKind,
    pub id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentPackCatalogs {
    pub rulesets: Vec<RulesetMetadata>,
    pub entities: Vec<EntityDefinition>,
    pub abilities: Vec<AbilityDefinition>,
    pub classes: Vec<ClassDefinition>,
    pub stat_definitions: Vec<StatDefinition>,
    pub modifiers: Vec<ModifierDefinition>,
    pub items: Vec<ItemDefinition>,
    pub actions: Vec<AuthoredActionDefinition>,
    pub scenarios: Vec<AuthoredScenarioDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackDefinition {
    pub canonical_version: ContentPackCanonicalVersion,
    pub identity: ContentPackIdentity,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub provenance: ContentPackProvenance,
    pub ruleset: RulesetArtifactProvenance,
    pub dependencies: Vec<ContentPackReference>,
    pub collision_policy: ContentPackCollisionPolicy,
    pub catalogs: ContentPackCatalogs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalContentPack {
    pub canonical_version: ContentPackCanonicalVersion,
    pub identity: ContentPackIdentity,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub provenance: ContentPackProvenance,
    pub ruleset: RulesetArtifactProvenance,
    pub dependencies: Vec<ContentPackReference>,
    pub collision_policy: ContentPackCollisionPolicy,
    pub catalogs: ContentPackCatalogs,
    pub fingerprint: ContentFingerprint,
}

impl CanonicalContentPack {
    pub fn exact_reference(&self) -> ContentPackReference {
        ContentPackReference::from_pack(self)
    }

    pub fn definition_references(&self) -> Vec<ContentDefinitionReference> {
        let mut references = Vec::new();
        references.extend(
            self.catalogs
                .rulesets
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Ruleset,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .scenarios
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Scenario,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .entities
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Entity,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .abilities
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Ability,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .classes
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Class,
                    id: value.id.clone(),
                }),
        );
        references.extend(self.catalogs.stat_definitions.iter().map(|value| {
            ContentDefinitionReference {
                kind: ContentDefinitionKind::Stat,
                id: value.id.clone(),
            }
        }));
        references.extend(
            self.catalogs
                .modifiers
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Modifier,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .items
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Item,
                    id: value.id.clone(),
                }),
        );
        references.extend(
            self.catalogs
                .actions
                .iter()
                .map(|value| ContentDefinitionReference {
                    kind: ContentDefinitionKind::Action,
                    id: value.id.clone(),
                }),
        );
        references.sort();
        references
    }
}
