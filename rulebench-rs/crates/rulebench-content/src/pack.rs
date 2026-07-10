use crate::{
    ClassDefinition, EntityDefinition, ItemDefinition, ModifierDefinition, StatDefinition,
};
use rulebench_ruleset::{
    AbilityDefinition, ActionDefinition, RulesetArtifactProvenance, RulesetMetadata,
};

pub const CONTENT_PACK_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-content-pack.v0";
pub const CONTENT_PACK_SET_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-content-pack-set.v0";

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
        }
    }
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
    pub actions: Vec<ActionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPackDefinition {
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
}
