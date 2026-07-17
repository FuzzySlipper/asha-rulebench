use rpg_ir::{AbilityDefinition, AbilityDefinitionKind};
use rulebench_content::{
    ContentDefinitionChangeKind, ContentImportDiagnostic, ContentImportDiagnosticSeverity,
    ContentImportReport, ContentPackCanonicalVersion, ContentPackCatalogs,
    ContentPackCollisionPolicy, ContentPackDefinition, ContentPackDiffReadout, ContentPackIdentity,
    ContentPackMetadataChangeKind, ContentPackProvenance, ContentPackReference,
    ContentPackSourceKind, DamageAdjustment, DamageAdjustmentPolicy, EntityDefinition,
    ImportedContentPack,
};
use serde::{Deserialize, Serialize};

use crate::authored_action::{
    AuthoredActionDefinitionDto, AuthoredCheckDeclarationDto, AuthoredEffectOperationDto,
    AuthoredModifierDefinitionDto, AuthoredModifierDurationPolicyDto,
    AuthoredModifierStackingPolicyDto, AuthoredModifierTenureDto, AuthoredMovementKindDto,
    AuthoredTargetKindDto, AuthoredTargetSelectionDto, AuthoredTargetTeamConstraintDto,
    AuthoredVisibilityRequirementDto,
};
use crate::authored_scenario::{
    AuthoredClassDefinitionDto, AuthoredItemDefinitionDto, AuthoredScenarioDefinitionDto,
    AuthoredStatDefinitionDto,
};
use crate::{validate_ruleset_definition, RulesetDefinitionDto};

pub const AUTHORED_CONTENT_PACK_FORMAT: &str = "asha-rulebench.content-pack";
pub const AUTHORED_CONTENT_PACK_VERSION_V1: u32 = 1;
pub const AUTHORED_CONTENT_PACK_VERSION_V2: u32 = 2;
pub const AUTHORED_CONTENT_PACK_VERSION_V3: u32 = 3;
pub const AUTHORED_CONTENT_PACK_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentFingerprintDto {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackIdentityDto {
    pub id: String,
    pub version: String,
    pub fingerprint: Option<ContentFingerprintDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredContentPackDocumentDto {
    pub format: String,
    pub format_version: u32,
    pub pack: AuthoredContentPackDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentPackDto {
    pub id: String,
    pub version: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub provenance: AuthoredContentProvenanceDto,
    pub ruleset_id: String,
    #[serde(default)]
    pub dependencies: Vec<ContentPackReferenceDto>,
    pub catalogs: AuthoredContentCatalogsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentProvenanceDto {
    pub source_kind: AuthoredContentSourceKindDto,
    pub source_id: String,
    #[serde(default)]
    pub authored_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredContentSourceKindDto {
    AuthoredFile,
    BridgeSubmission,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredContentCatalogsDto {
    pub rulesets: Vec<RulesetDefinitionDto>,
    pub entities: Vec<AuthoredEntityDefinitionDto>,
    pub abilities: Vec<AuthoredAbilityDefinitionDto>,
    pub modifiers: Vec<AuthoredModifierDefinitionDto>,
    pub actions: Vec<AuthoredActionDefinitionDto>,
    pub classes: Vec<AuthoredClassDefinitionDto>,
    pub stat_definitions: Vec<AuthoredStatDefinitionDto>,
    pub items: Vec<AuthoredItemDefinitionDto>,
    pub scenarios: Vec<AuthoredScenarioDefinitionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAbilityDefinitionDto {
    pub id: String,
    pub name: String,
    pub kind: AuthoredAbilityDefinitionKindDto,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredAbilityDefinitionKindDto {
    Ability,
    Spell,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum AuthoredContentPackDocumentWireDto {
    V1(AuthoredContentPackDocumentV1Dto),
    V2(AuthoredContentPackDocumentV2Dto),
    V3(AuthoredContentPackDocumentV3Dto),
    V4(AuthoredContentPackDocumentV4Dto),
    Unsupported(UnsupportedAuthoredContentPackDocumentDto),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackDocumentV1Dto {
    format: String,
    format_version: u32,
    pack: AuthoredContentPackV1Dto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackDocumentV2Dto {
    format: String,
    format_version: u32,
    pack: AuthoredContentPackV2Dto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackDocumentV3Dto {
    format: String,
    format_version: u32,
    pack: AuthoredContentPackV3Dto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackDocumentV4Dto {
    format: String,
    format_version: u32,
    pack: AuthoredContentPackDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnsupportedAuthoredContentPackDocumentDto {
    format: String,
    format_version: u32,
    pack: UnsupportedAuthoredContentPackDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnsupportedAuthoredContentPackDocumentWireDto {
    format: String,
    format_version: u32,
    pack: UnsupportedAuthoredContentPackDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnsupportedAuthoredContentPackDto {
    #[serde(default)]
    id: String,
    #[serde(default)]
    version: String,
}

impl<'de> Deserialize<'de> for UnsupportedAuthoredContentPackDocumentDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let document = UnsupportedAuthoredContentPackDocumentWireDto::deserialize(deserializer)?;
        if matches!(
            document.format_version,
            AUTHORED_CONTENT_PACK_VERSION_V1
                | AUTHORED_CONTENT_PACK_VERSION_V2
                | AUTHORED_CONTENT_PACK_VERSION_V3
                | AUTHORED_CONTENT_PACK_VERSION
        ) {
            return Err(D::Error::custom(
                "a shipped authored content version must match its exact catalog shape",
            ));
        }
        Ok(Self {
            format: document.format,
            format_version: document.format_version,
            pack: document.pack,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackV1Dto {
    id: String,
    version: String,
    title: String,
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
    provenance: AuthoredContentProvenanceDto,
    ruleset_id: String,
    #[serde(default)]
    dependencies: Vec<ContentPackReferenceDto>,
    catalogs: AuthoredContentCatalogsV1Dto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackV2Dto {
    id: String,
    version: String,
    title: String,
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
    provenance: AuthoredContentProvenanceDto,
    ruleset_id: String,
    #[serde(default)]
    dependencies: Vec<ContentPackReferenceDto>,
    catalogs: AuthoredContentCatalogsV2Dto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentPackV3Dto {
    id: String,
    version: String,
    title: String,
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
    provenance: AuthoredContentProvenanceDto,
    ruleset_id: String,
    #[serde(default)]
    dependencies: Vec<ContentPackReferenceDto>,
    catalogs: AuthoredContentCatalogsV3Dto,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentCatalogsV1Dto {
    #[serde(default)]
    rulesets: Vec<RulesetDefinitionDto>,
    #[serde(default)]
    entities: Vec<AuthoredEntityDefinitionDto>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentCatalogsV2Dto {
    #[serde(default)]
    rulesets: Vec<RulesetDefinitionDto>,
    #[serde(default)]
    entities: Vec<AuthoredEntityDefinitionDto>,
    abilities: Vec<AuthoredAbilityDefinitionDto>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthoredContentCatalogsV3Dto {
    rulesets: Vec<RulesetDefinitionDto>,
    entities: Vec<AuthoredEntityDefinitionDto>,
    abilities: Vec<AuthoredAbilityDefinitionDto>,
    modifiers: Vec<AuthoredModifierDefinitionDto>,
    actions: Vec<AuthoredActionDefinitionDto>,
}

impl<'de> Deserialize<'de> for AuthoredContentPackDocumentDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        match AuthoredContentPackDocumentWireDto::deserialize(deserializer)? {
            AuthoredContentPackDocumentWireDto::V1(document) => {
                if matches!(
                    document.format_version,
                    AUTHORED_CONTENT_PACK_VERSION_V2
                        | AUTHORED_CONTENT_PACK_VERSION_V3
                        | AUTHORED_CONTENT_PACK_VERSION
                ) {
                    return Err(D::Error::custom(format!(
                        "authored content formatVersion {} requires its exact catalog shape",
                        document.format_version
                    )));
                }
                Ok(Self {
                    format: document.format,
                    format_version: document.format_version,
                    pack: document.pack.into_current(),
                })
            }
            AuthoredContentPackDocumentWireDto::V2(document) => {
                if matches!(
                    document.format_version,
                    AUTHORED_CONTENT_PACK_VERSION_V1
                        | AUTHORED_CONTENT_PACK_VERSION_V3
                        | AUTHORED_CONTENT_PACK_VERSION
                ) {
                    return Err(D::Error::custom(format!(
                        "authored content formatVersion {} requires its exact catalog shape",
                        document.format_version
                    )));
                }
                Ok(Self {
                    format: document.format,
                    format_version: document.format_version,
                    pack: document.pack.into_current(),
                })
            }
            AuthoredContentPackDocumentWireDto::V3(document) => {
                if matches!(
                    document.format_version,
                    AUTHORED_CONTENT_PACK_VERSION_V1
                        | AUTHORED_CONTENT_PACK_VERSION_V2
                        | AUTHORED_CONTENT_PACK_VERSION
                ) {
                    return Err(D::Error::custom(format!(
                        "authored content formatVersion {} requires its exact catalog shape",
                        document.format_version
                    )));
                }
                Ok(Self {
                    format: document.format,
                    format_version: document.format_version,
                    pack: document.pack.into_current(),
                })
            }
            AuthoredContentPackDocumentWireDto::V4(document) => {
                if matches!(
                    document.format_version,
                    AUTHORED_CONTENT_PACK_VERSION_V1
                        | AUTHORED_CONTENT_PACK_VERSION_V2
                        | AUTHORED_CONTENT_PACK_VERSION_V3
                ) {
                    return Err(D::Error::custom(format!(
                        "authored content formatVersion {} requires its exact catalog shape",
                        document.format_version
                    )));
                }
                Ok(Self {
                    format: document.format,
                    format_version: document.format_version,
                    pack: document.pack,
                })
            }
            AuthoredContentPackDocumentWireDto::Unsupported(document) => Ok(Self {
                format: document.format,
                format_version: document.format_version,
                pack: unsupported_authored_pack(document.pack),
            }),
        }
    }
}

impl Serialize for AuthoredContentPackDocumentDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V1 {
            if !self.pack.catalogs.abilities.is_empty()
                || !self.pack.catalogs.modifiers.is_empty()
                || !self.pack.catalogs.actions.is_empty()
                || !self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty()
            {
                return Err(S::Error::custom(
                    "authored v2/v3 definitions require a newer formatVersion",
                ));
            }
            return AuthoredContentPackDocumentV1Dto {
                format: self.format.clone(),
                format_version: self.format_version,
                pack: self.pack.to_v1(),
            }
            .serialize(serializer);
        }

        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V2 {
            if !self.pack.catalogs.modifiers.is_empty()
                || !self.pack.catalogs.actions.is_empty()
                || !self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty()
            {
                return Err(S::Error::custom(
                    "authored modifier and action definitions require formatVersion 3",
                ));
            }
            return AuthoredContentPackDocumentV2Dto {
                format: self.format.clone(),
                format_version: self.format_version,
                pack: self.pack.to_v2(),
            }
            .serialize(serializer);
        }

        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V3 {
            if !self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty()
            {
                return Err(S::Error::custom(
                    "authored class, stat, item, and scenario definitions require formatVersion 4",
                ));
            }
            return AuthoredContentPackDocumentV3Dto {
                format: self.format.clone(),
                format_version: self.format_version,
                pack: self.pack.to_v3(),
            }
            .serialize(serializer);
        }

        if self.format_version != AUTHORED_CONTENT_PACK_VERSION {
            return Err(S::Error::custom("unsupported authored content version"));
        }

        AuthoredContentPackDocumentV4Dto {
            format: self.format.clone(),
            format_version: self.format_version,
            pack: self.pack.clone(),
        }
        .serialize(serializer)
    }
}

fn unsupported_authored_pack(pack: UnsupportedAuthoredContentPackDto) -> AuthoredContentPackDto {
    AuthoredContentPackDto {
        id: pack.id,
        version: pack.version,
        title: String::new(),
        summary: String::new(),
        tags: Vec::new(),
        provenance: AuthoredContentProvenanceDto {
            source_kind: AuthoredContentSourceKindDto::AuthoredFile,
            source_id: String::new(),
            authored_by: None,
        },
        ruleset_id: String::new(),
        dependencies: Vec::new(),
        catalogs: AuthoredContentCatalogsDto::default(),
    }
}

impl AuthoredContentPackV1Dto {
    fn into_current(self) -> AuthoredContentPackDto {
        AuthoredContentPackDto {
            id: self.id,
            version: self.version,
            title: self.title,
            summary: self.summary,
            tags: self.tags,
            provenance: self.provenance,
            ruleset_id: self.ruleset_id,
            dependencies: self.dependencies,
            catalogs: AuthoredContentCatalogsDto {
                rulesets: self.catalogs.rulesets,
                entities: self.catalogs.entities,
                abilities: Vec::new(),
                modifiers: Vec::new(),
                actions: Vec::new(),
                classes: Vec::new(),
                stat_definitions: Vec::new(),
                items: Vec::new(),
                scenarios: Vec::new(),
            },
        }
    }
}

impl AuthoredContentPackV2Dto {
    fn into_current(self) -> AuthoredContentPackDto {
        AuthoredContentPackDto {
            id: self.id,
            version: self.version,
            title: self.title,
            summary: self.summary,
            tags: self.tags,
            provenance: self.provenance,
            ruleset_id: self.ruleset_id,
            dependencies: self.dependencies,
            catalogs: AuthoredContentCatalogsDto {
                rulesets: self.catalogs.rulesets,
                entities: self.catalogs.entities,
                abilities: self.catalogs.abilities,
                modifiers: Vec::new(),
                actions: Vec::new(),
                classes: Vec::new(),
                stat_definitions: Vec::new(),
                items: Vec::new(),
                scenarios: Vec::new(),
            },
        }
    }
}

impl AuthoredContentPackV3Dto {
    fn into_current(self) -> AuthoredContentPackDto {
        AuthoredContentPackDto {
            id: self.id,
            version: self.version,
            title: self.title,
            summary: self.summary,
            tags: self.tags,
            provenance: self.provenance,
            ruleset_id: self.ruleset_id,
            dependencies: self.dependencies,
            catalogs: AuthoredContentCatalogsDto {
                rulesets: self.catalogs.rulesets,
                entities: self.catalogs.entities,
                abilities: self.catalogs.abilities,
                modifiers: self.catalogs.modifiers,
                actions: self.catalogs.actions,
                classes: Vec::new(),
                stat_definitions: Vec::new(),
                items: Vec::new(),
                scenarios: Vec::new(),
            },
        }
    }
}

impl AuthoredContentPackDto {
    fn to_v1(&self) -> AuthoredContentPackV1Dto {
        AuthoredContentPackV1Dto {
            id: self.id.clone(),
            version: self.version.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            provenance: self.provenance.clone(),
            ruleset_id: self.ruleset_id.clone(),
            dependencies: self.dependencies.clone(),
            catalogs: AuthoredContentCatalogsV1Dto {
                rulesets: self.catalogs.rulesets.clone(),
                entities: self.catalogs.entities.clone(),
            },
        }
    }

    fn to_v2(&self) -> AuthoredContentPackV2Dto {
        AuthoredContentPackV2Dto {
            id: self.id.clone(),
            version: self.version.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            provenance: self.provenance.clone(),
            ruleset_id: self.ruleset_id.clone(),
            dependencies: self.dependencies.clone(),
            catalogs: AuthoredContentCatalogsV2Dto {
                rulesets: self.catalogs.rulesets.clone(),
                entities: self.catalogs.entities.clone(),
                abilities: self.catalogs.abilities.clone(),
            },
        }
    }

    fn to_v3(&self) -> AuthoredContentPackV3Dto {
        AuthoredContentPackV3Dto {
            id: self.id.clone(),
            version: self.version.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            provenance: self.provenance.clone(),
            ruleset_id: self.ruleset_id.clone(),
            dependencies: self.dependencies.clone(),
            catalogs: AuthoredContentCatalogsV3Dto {
                rulesets: self.catalogs.rulesets.clone(),
                entities: self.catalogs.entities.clone(),
                abilities: self.catalogs.abilities.clone(),
                modifiers: self.catalogs.modifiers.clone(),
                actions: self.catalogs.actions.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredEntityDefinitionDto {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub damage_adjustments: Vec<AuthoredDamageAdjustmentDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredDamageAdjustmentDto {
    pub damage_type: String,
    pub policy: AuthoredDamageAdjustmentPolicyDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredDamageAdjustmentPolicyDto {
    Resistance,
    Vulnerability,
    Immunity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredContentDecodeError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl AuthoredContentPackDocumentDto {
    pub fn to_authority(&self) -> Result<ContentPackDefinition, AuthoredContentDecodeError> {
        if self.format != AUTHORED_CONTENT_PACK_FORMAT {
            return Err(AuthoredContentDecodeError {
                code: "unsupportedAuthoredContentFormat",
                path: "format".to_string(),
                message: format!("Unsupported authored content format: {}", self.format),
            });
        }
        if !matches!(
            self.format_version,
            AUTHORED_CONTENT_PACK_VERSION_V1
                | AUTHORED_CONTENT_PACK_VERSION_V2
                | AUTHORED_CONTENT_PACK_VERSION_V3
                | AUTHORED_CONTENT_PACK_VERSION
        ) {
            return Err(AuthoredContentDecodeError {
                code: "unsupportedAuthoredContentVersion",
                path: "formatVersion".to_string(),
                message: format!(
                    "Authored content format version {} is unsupported; supported versions are {}, {}, {}, and {}.",
                    self.format_version,
                    AUTHORED_CONTENT_PACK_VERSION_V1,
                    AUTHORED_CONTENT_PACK_VERSION_V2,
                    AUTHORED_CONTENT_PACK_VERSION_V3,
                    AUTHORED_CONTENT_PACK_VERSION
                ),
            });
        }
        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V1
            && (!self.pack.catalogs.abilities.is_empty()
                || !self.pack.catalogs.modifiers.is_empty()
                || !self.pack.catalogs.actions.is_empty()
                || !self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty())
        {
            return Err(AuthoredContentDecodeError {
                code: "authoredDefinitionRequiresV2",
                path: "pack.catalogs.abilities".to_string(),
                message: "Authored ability definitions require formatVersion 2.".to_string(),
            });
        }
        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V2
            && (!self.pack.catalogs.modifiers.is_empty()
                || !self.pack.catalogs.actions.is_empty()
                || !self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty())
        {
            return Err(AuthoredContentDecodeError {
                code: "authoredDefinitionRequiresV3",
                path: "pack.catalogs".to_string(),
                message: "Authored modifier and action definitions require formatVersion 3."
                    .to_string(),
            });
        }
        if self.format_version == AUTHORED_CONTENT_PACK_VERSION_V3
            && (!self.pack.catalogs.classes.is_empty()
                || !self.pack.catalogs.stat_definitions.is_empty()
                || !self.pack.catalogs.items.is_empty()
                || !self.pack.catalogs.scenarios.is_empty())
        {
            return Err(AuthoredContentDecodeError {
                code: "authoredDefinitionRequiresV4",
                path: "pack.catalogs".to_string(),
                message:
                    "Authored class, stat, item, and scenario definitions require formatVersion 4."
                        .to_string(),
            });
        }
        let catalogs = self.pack.catalogs.to_authority()?;
        let selected_ruleset = catalogs
            .rulesets
            .iter()
            .find(|ruleset| ruleset.id == self.pack.ruleset_id)
            .ok_or_else(|| AuthoredContentDecodeError {
                code: "missingAuthoredPackRuleset",
                path: "pack.rulesetId".to_string(),
                message: format!(
                    "The authored pack does not declare its selected ruleset {}.",
                    self.pack.ruleset_id
                ),
            })?;
        Ok(ContentPackDefinition {
            canonical_version: match self.format_version {
                AUTHORED_CONTENT_PACK_VERSION => ContentPackCanonicalVersion::V2,
                AUTHORED_CONTENT_PACK_VERSION_V3 => ContentPackCanonicalVersion::V1,
                _ => ContentPackCanonicalVersion::V0,
            },
            identity: ContentPackIdentity::new(&self.pack.id, &self.pack.version),
            title: self.pack.title.clone(),
            summary: self.pack.summary.clone(),
            tags: self.pack.tags.clone(),
            provenance: ContentPackProvenance {
                source_kind: match self.pack.provenance.source_kind {
                    AuthoredContentSourceKindDto::AuthoredFile => {
                        ContentPackSourceKind::AuthoredFile
                    }
                    AuthoredContentSourceKindDto::BridgeSubmission => {
                        ContentPackSourceKind::BridgeSubmission
                    }
                },
                source_id: self.pack.provenance.source_id.clone(),
                authored_by: self.pack.provenance.authored_by.clone(),
            },
            ruleset: selected_ruleset.artifact_provenance(),
            dependencies: self
                .pack
                .dependencies
                .iter()
                .map(ContentPackReferenceDto::to_authority)
                .collect(),
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs,
        })
    }
}

impl AuthoredContentCatalogsDto {
    fn to_authority(&self) -> Result<ContentPackCatalogs, AuthoredContentDecodeError> {
        let rulesets = self
            .rulesets
            .iter()
            .enumerate()
            .map(|(index, definition)| {
                validate_ruleset_definition(definition).map_err(|error| {
                    AuthoredContentDecodeError {
                        code: error.code(),
                        path: format!("pack.catalogs.rulesets[{index}]"),
                        message: format!("Authored ruleset was rejected: {error:?}"),
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let entities = self
            .entities
            .iter()
            .map(AuthoredEntityDefinitionDto::to_authority)
            .collect();
        let abilities = self
            .abilities
            .iter()
            .map(AuthoredAbilityDefinitionDto::to_authority)
            .collect();
        let modifiers = self
            .modifiers
            .iter()
            .map(AuthoredModifierDefinitionDto::to_authority)
            .collect();
        let actions = self
            .actions
            .iter()
            .map(AuthoredActionDefinitionDto::to_authority)
            .collect();
        let classes = self
            .classes
            .iter()
            .map(AuthoredClassDefinitionDto::to_authority)
            .collect();
        let stat_definitions = self
            .stat_definitions
            .iter()
            .map(AuthoredStatDefinitionDto::to_authority)
            .collect();
        let items = self
            .items
            .iter()
            .map(AuthoredItemDefinitionDto::to_authority)
            .collect();
        let scenarios = self
            .scenarios
            .iter()
            .map(AuthoredScenarioDefinitionDto::to_authority)
            .collect();

        Ok(ContentPackCatalogs {
            rulesets,
            entities,
            abilities,
            modifiers,
            actions,
            classes,
            stat_definitions,
            items,
            scenarios,
        })
    }
}

impl AuthoredAbilityDefinitionDto {
    fn to_authority(&self) -> AbilityDefinition {
        AbilityDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: match self.kind {
                AuthoredAbilityDefinitionKindDto::Ability => AbilityDefinitionKind::Ability,
                AuthoredAbilityDefinitionKindDto::Spell => AbilityDefinitionKind::Spell,
            },
            summary: self.summary.clone(),
            tags: self.tags.clone(),
        }
    }
}

impl AuthoredEntityDefinitionDto {
    fn to_authority(&self) -> EntityDefinition {
        EntityDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            damage_adjustments: self
                .damage_adjustments
                .iter()
                .map(|adjustment| DamageAdjustment {
                    damage_type: adjustment.damage_type.clone(),
                    policy: match adjustment.policy {
                        AuthoredDamageAdjustmentPolicyDto::Resistance => {
                            DamageAdjustmentPolicy::Resistance
                        }
                        AuthoredDamageAdjustmentPolicyDto::Vulnerability => {
                            DamageAdjustmentPolicy::Vulnerability
                        }
                        AuthoredDamageAdjustmentPolicyDto::Immunity => {
                            DamageAdjustmentPolicy::Immunity
                        }
                    },
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackReferenceDto {
    pub id: String,
    pub version: String,
    pub fingerprint: ContentFingerprintDto,
}

impl ContentPackReferenceDto {
    pub fn to_authority(&self) -> ContentPackReference {
        ContentPackReference {
            id: self.id.clone(),
            version: self.version.clone(),
            fingerprint: rulebench_content::ContentFingerprint {
                algorithm: self.fingerprint.algorithm.clone(),
                value: self.fingerprint.value.clone(),
            },
        }
    }
}

impl From<&ContentPackReference> for ContentPackReferenceDto {
    fn from(reference: &ContentPackReference) -> Self {
        Self {
            id: reference.id.clone(),
            version: reference.version.clone(),
            fingerprint: ContentFingerprintDto {
                algorithm: reference.fingerprint.algorithm.clone(),
                value: reference.fingerprint.value.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentReplacementPolicyDto {
    Reject,
    ReplaceSameIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportRequestDto {
    pub authored_payload: String,
    pub replacement_policy: ContentReplacementPolicyDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPayloadRequestDto {
    pub authored_payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentReferenceRequestDto {
    pub reference: ContentPackReferenceDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDraftIdentityDto {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentTemplateDraftRequestDto {
    pub identity: ContentDraftIdentityDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentCloneDraftRequestDto {
    pub reference: ContentPackReferenceDto,
    pub identity: ContentDraftIdentityDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentAuthoringDraftDto {
    pub authored_payload: String,
    pub source_kind: String,
    pub source_label: String,
    pub identity: ContentDraftIdentityDto,
    pub identity_expectation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDefinitionSummaryDto {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredContentPackSummaryDto {
    pub reference: ContentPackReferenceDto,
    pub title: String,
    pub summary: String,
    pub source_kind: String,
    pub source_id: String,
    pub authored_by: Option<String>,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub dependencies: Vec<ContentPackReferenceDto>,
    pub definitions: Vec<ContentDefinitionSummaryDto>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackReviewDto {
    pub pack: StoredContentPackSummaryDto,
    pub authored_payload: String,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
    pub abilities: Vec<ContentAbilityDeclarationSummaryDto>,
    pub modifiers: Vec<ContentModifierDeclarationSummaryDto>,
    pub actions: Vec<ContentActionDeclarationSummaryDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentAbilityDeclarationSummaryDto {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub tags: Vec<String>,
}

impl From<&AuthoredAbilityDefinitionDto> for ContentAbilityDeclarationSummaryDto {
    fn from(value: &AuthoredAbilityDefinitionDto) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            kind: match value.kind {
                AuthoredAbilityDefinitionKindDto::Ability => "ability",
                AuthoredAbilityDefinitionKindDto::Spell => "spell",
            }
            .to_string(),
            summary: value.summary.clone(),
            tags: value.tags.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentModifierDeclarationSummaryDto {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub tenure: String,
    pub stacking: String,
    pub duration: String,
    pub stat_adjustments: Vec<String>,
}

impl From<&AuthoredModifierDefinitionDto> for ContentModifierDeclarationSummaryDto {
    fn from(value: &AuthoredModifierDefinitionDto) -> Self {
        let tenure = match value.default_tenure {
            AuthoredModifierTenureDto::Temporary => "temporary",
            AuthoredModifierTenureDto::Permanent => "permanent",
        };
        let stacking_policy = match value.stacking_policy {
            AuthoredModifierStackingPolicyDto::Stack => "stack",
            AuthoredModifierStackingPolicyDto::Replace => "replace",
            AuthoredModifierStackingPolicyDto::Refresh => "refresh",
        };
        let duration = match &value.duration_policy {
            AuthoredModifierDurationPolicyDto::Permanent => "permanent".to_string(),
            AuthoredModifierDurationPolicyDto::Turns { turns } => format!("{turns} turns"),
            AuthoredModifierDurationPolicyDto::Rounds { rounds } => format!("{rounds} rounds"),
            AuthoredModifierDurationPolicyDto::UntilEvent { event } => {
                format!("until event {event}")
            }
        };
        Self {
            id: value.id.clone(),
            label: value.label.clone(),
            summary: value.summary.clone(),
            tenure: tenure.to_string(),
            stacking: format!("{} · {stacking_policy}", value.stacking_group),
            duration,
            stat_adjustments: value
                .stat_adjustments
                .iter()
                .map(|adjustment| {
                    format!(
                        "{} ({}) {:+}",
                        adjustment.stat_label, adjustment.stat_id, adjustment.delta
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentActionDeclarationSummaryDto {
    pub id: String,
    pub name: String,
    pub ability_id: String,
    pub targeting: String,
    pub check: String,
    pub effects: Vec<String>,
    pub resource_costs: Vec<String>,
    pub action_text: String,
    pub effect_text: String,
}

impl From<&AuthoredActionDefinitionDto> for ContentActionDeclarationSummaryDto {
    fn from(value: &AuthoredActionDefinitionDto) -> Self {
        let target_kind = match value.targeting.target_kind {
            AuthoredTargetKindDto::Combatant => "combatant",
            AuthoredTargetKindDto::Area => "area",
        };
        let selection = match value.targeting.selection {
            AuthoredTargetSelectionDto::Single => "single",
            AuthoredTargetSelectionDto::Multiple => "multiple",
        };
        let team = match value.targeting.team_constraint {
            AuthoredTargetTeamConstraintDto::Hostile => "hostile",
            AuthoredTargetTeamConstraintDto::Ally => "ally",
            AuthoredTargetTeamConstraintDto::Any => "any team",
        };
        let visibility = match value.targeting.visibility_requirement {
            AuthoredVisibilityRequirementDto::Required => "visible",
            AuthoredVisibilityRequirementDto::Ignored => "visibility ignored",
        };
        let check = match &value.check {
            AuthoredCheckDeclarationDto::Attack {
                modifier,
                modifier_stat_id,
                defense,
            } => format!(
                "attack {modifier:+} + {modifier_stat_id} vs {} ({})",
                defense.label, defense.id
            ),
            AuthoredCheckDeclarationDto::SavingThrow {
                save_stat_id,
                difficulty_class,
            } => format!("{save_stat_id} saving throw vs DC {difficulty_class}"),
            AuthoredCheckDeclarationDto::Contested {
                actor_stat_id,
                target_stat_id,
            } => format!("contested {actor_stat_id} vs {target_stat_id}"),
        };
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            ability_id: value.ability_id.clone(),
            targeting: format!(
                "{selection} {target_kind} · {team} · range {} · {visibility}",
                value.targeting.maximum_range
            ),
            check,
            effects: value.effects.iter().map(effect_operation_summary).collect(),
            resource_costs: value
                .resource_costs
                .iter()
                .map(|cost| format!("{} × {}", cost.resource_id, cost.amount))
                .collect(),
            action_text: value.action_text.clone(),
            effect_text: value.effect_text.clone(),
        }
    }
}

fn effect_operation_summary(value: &AuthoredEffectOperationDto) -> String {
    match value {
        AuthoredEffectOperationDto::Damage {
            damage_bonus,
            damage_type,
        } => format!("damage · {damage_type} {damage_bonus:+}"),
        AuthoredEffectOperationDto::Heal {
            healing_bonus,
            healing_type,
        } => format!("heal · {healing_type} {healing_bonus:+}"),
        AuthoredEffectOperationDto::GrantTemporaryVitality { vitality_bonus } => {
            format!("grant temporary vitality · {vitality_bonus:+}")
        }
        AuthoredEffectOperationDto::ApplyModifier { modifier_id } => {
            format!("apply modifier · {modifier_id}")
        }
        AuthoredEffectOperationDto::Move {
            maximum_distance,
            movement_kind,
        } => {
            let movement_kind = match movement_kind {
                AuthoredMovementKindDto::Push => "push",
                AuthoredMovementKindDto::Pull => "pull",
                AuthoredMovementKindDto::Shift => "shift",
            };
            format!("move · {movement_kind} · {maximum_distance}")
        }
        AuthoredEffectOperationDto::ChangeResource { resource_id, delta } => {
            format!("change resource · {resource_id} {delta:+}")
        }
        AuthoredEffectOperationDto::OpenReactionWindow {
            hook_id,
            options,
            maximum_nested_depth,
            ..
        } => format!(
            "open reaction window · {hook_id} · {} options · depth {maximum_nested_depth}",
            options.len()
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentActionBindingActorDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentActionBindingScenarioDto {
    pub id: String,
    pub title: String,
    pub actors: Vec<ContentActionBindingActorDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentActionBindingCandidateDto {
    pub content_pack: ContentPackReferenceDto,
    pub action_id: String,
    pub action_name: String,
    pub ability_id: String,
    pub scenarios: Vec<ContentActionBindingScenarioDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentActionBindingCatalogDto {
    pub actions: Vec<ContentActionBindingCandidateDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentDefinitionChangeDto {
    pub kind: String,
    pub id: String,
    pub change: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPackDiffDto {
    pub before: ContentPackReferenceDto,
    pub after: ContentPackReferenceDto,
    pub changed: bool,
    pub fingerprint_changed: bool,
    pub ruleset_compatibility_changed: bool,
    pub dependency_set_changed: bool,
    pub metadata_changes: Vec<String>,
    pub definition_changes: Vec<ContentDefinitionChangeDto>,
}

impl From<&ContentPackDiffReadout> for ContentPackDiffDto {
    fn from(diff: &ContentPackDiffReadout) -> Self {
        Self {
            before: ContentPackReferenceDto::from(&diff.before),
            after: ContentPackReferenceDto::from(&diff.after),
            changed: diff.changed,
            fingerprint_changed: diff.fingerprint_changed,
            ruleset_compatibility_changed: diff.ruleset_compatibility_changed,
            dependency_set_changed: diff.dependency_set_changed,
            metadata_changes: diff
                .metadata_changes
                .iter()
                .map(|change| metadata_change_code(*change).to_string())
                .collect(),
            definition_changes: diff
                .definition_changes
                .iter()
                .map(|change| ContentDefinitionChangeDto {
                    kind: change.kind.code().to_string(),
                    id: change.id.clone(),
                    change: definition_change_code(change.change).to_string(),
                })
                .collect(),
        }
    }
}

fn metadata_change_code(change: ContentPackMetadataChangeKind) -> &'static str {
    change.code()
}

fn definition_change_code(change: ContentDefinitionChangeKind) -> &'static str {
    change.code()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportOutcomeDto {
    pub review: ContentPackReviewDto,
    pub replaced: Option<ContentPackReferenceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImportAttemptDto {
    pub accepted: bool,
    pub pack: ContentPackIdentityDto,
    pub outcome: Option<ContentImportOutcomeDto>,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentAuditEntryDto {
    pub sequence: u64,
    pub operation: String,
    pub reference: ContentPackReferenceDto,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentWorkspaceDto {
    pub packs: Vec<StoredContentPackSummaryDto>,
    pub audit: Vec<ContentAuditEntryDto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_content::{ContentImportDiagnosticCode, ContentPackDiagnosticCode};
    use serde_json::json;

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

    #[test]
    fn authored_v1_reader_remains_strict_and_read_compatible() {
        let document: AuthoredContentPackDocumentDto =
            serde_json::from_value(authored_document(AUTHORED_CONTENT_PACK_VERSION_V1, None))
                .expect("committed v1 shape remains readable");

        let authority = document.to_authority().expect("v1 remains importable");
        assert!(authority.catalogs.abilities.is_empty());
        let encoded = serde_json::to_value(&document).expect("v1 remains writable");
        assert_eq!(encoded["formatVersion"], AUTHORED_CONTENT_PACK_VERSION_V1);
        assert!(encoded["pack"]["catalogs"].get("abilities").is_none());
        serde_json::from_value::<AuthoredContentPackDocumentDto>(encoded)
            .expect("written v1 remains readable");

        let v1_with_v2_field = authored_document(
            AUTHORED_CONTENT_PACK_VERSION_V1,
            Some(json!([authored_ability()])),
        );
        serde_json::from_value::<AuthoredContentPackDocumentDto>(v1_with_v2_field)
            .expect_err("v1 must not accept a v2 catalog field");
    }

    #[test]
    fn authored_v2_converts_ability_definitions_without_ui_semantics() {
        let document: AuthoredContentPackDocumentDto = serde_json::from_value(authored_document(
            AUTHORED_CONTENT_PACK_VERSION_V2,
            Some(json!([authored_ability()])),
        ))
        .expect("v2 shape decodes");
        let authority = document.to_authority().expect("v2 converts in Rust");

        assert_eq!(authority.catalogs.abilities.len(), 1);
        assert_eq!(authority.catalogs.abilities[0].id, "ability.binding-glyph");
        assert_eq!(
            authority.catalogs.abilities[0].kind,
            AbilityDefinitionKind::Spell
        );
    }

    #[test]
    fn unknown_authored_version_reaches_the_classified_version_error() {
        let mut documents = [None, Some(json!([authored_ability()]))]
            .into_iter()
            .map(|abilities| authored_document(99, abilities))
            .collect::<Vec<_>>();
        let mut v3_shape = authored_v3_document();
        v3_shape["formatVersion"] = json!(99);
        documents.push(v3_shape);
        for value in documents {
            let document: AuthoredContentPackDocumentDto = serde_json::from_value(value)
                .expect("unknown version shape remains classifiable without conversion");
            let error = document
                .to_authority()
                .expect_err("unknown versions fail closed");

            assert_eq!(error.code, "unsupportedAuthoredContentVersion");
            assert_eq!(error.path, "formatVersion");
        }
    }

    #[test]
    fn authored_v2_requires_the_complete_v2_catalog_shape() {
        serde_json::from_value::<AuthoredContentPackDocumentDto>(authored_document(
            AUTHORED_CONTENT_PACK_VERSION_V2,
            None,
        ))
        .expect_err("v2 must not silently default its ability catalog");

        let mut v2_with_v3_fields = authored_document(
            AUTHORED_CONTENT_PACK_VERSION_V2,
            Some(json!([authored_ability()])),
        );
        v2_with_v3_fields["pack"]["catalogs"]["modifiers"] = json!([]);
        v2_with_v3_fields["pack"]["catalogs"]["actions"] = json!([]);
        serde_json::from_value::<AuthoredContentPackDocumentDto>(v2_with_v3_fields)
            .expect_err("v2 must reject v3 catalog fields even when they are empty");
    }

    #[test]
    fn authored_v3_is_strict_and_converts_only_portable_action_fields() {
        let value = authored_v3_document();
        let document: AuthoredContentPackDocumentDto =
            serde_json::from_value(value.clone()).expect("complete v3 shape decodes");
        let authority = document.to_authority().expect("v3 converts in Rust");

        assert_eq!(authority.canonical_version, ContentPackCanonicalVersion::V1);
        assert_eq!(authority.catalogs.modifiers.len(), 1);
        assert_eq!(authority.catalogs.actions.len(), 1);
        assert_eq!(
            authority.catalogs.actions[0].ability_id,
            "ability.binding-glyph"
        );
        assert_eq!(
            authority.catalogs.actions[0].effects[1].code(),
            "applyModifier"
        );
        let encoded = serde_json::to_value(&document).expect("v3 remains writable");
        serde_json::from_value::<AuthoredContentPackDocumentDto>(encoded)
            .expect("written v3 remains readable");

        let mut missing_actions = value.clone();
        missing_actions["pack"]["catalogs"]
            .as_object_mut()
            .expect("catalogs is an object")
            .remove("actions");
        serde_json::from_value::<AuthoredContentPackDocumentDto>(missing_actions)
            .expect_err("v3 must not default its action catalog");

        let mut runtime_bound = value;
        runtime_bound["pack"]["catalogs"]["actions"][0]["actorId"] = json!("entity.actor");
        serde_json::from_value::<AuthoredContentPackDocumentDto>(runtime_bound)
            .expect_err("v3 must reject scenario-bound runtime ids");
    }

    fn authored_document(
        format_version: u32,
        abilities: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let mut catalogs = json!({
            "rulesets": [{
                "id": "asha-rulebench.turn-control.v0",
                "name": "Turn Control",
                "version": "0.1.0",
                "summary": "Authored v2 fixture.",
                "modules": [{
                    "module": "actionResolution",
                    "version": "1",
                    "configuration": {
                        "module": "actionResolution",
                        "targetingPolicy": "declaredTargetsAndLineOfSight",
                        "supportedCheckHandlers": ["attackVsDefense"]
                    }
                }]
            }],
            "entities": []
        });
        if let Some(abilities) = abilities {
            catalogs
                .as_object_mut()
                .expect("catalog fixture is an object")
                .insert("abilities".to_string(), abilities);
        }
        json!({
            "format": AUTHORED_CONTENT_PACK_FORMAT,
            "formatVersion": format_version,
            "pack": {
                "id": "pack.authored.v2",
                "version": "2.0.0",
                "title": "Authored v2",
                "summary": "Cross-version reader fixture.",
                "tags": ["authored"],
                "provenance": {
                    "sourceKind": "authoredFile",
                    "sourceId": "fixture:authored-v2"
                },
                "rulesetId": "asha-rulebench.turn-control.v0",
                "dependencies": [],
                "catalogs": catalogs
            }
        })
    }

    fn authored_ability() -> serde_json::Value {
        json!({
            "id": "ability.binding-glyph",
            "name": "Binding Glyph",
            "kind": "spell",
            "summary": "Second-ruleset control spell authored through v2.",
            "tags": ["control", "spell"]
        })
    }

    fn authored_v3_document() -> serde_json::Value {
        let mut document = authored_document(
            AUTHORED_CONTENT_PACK_VERSION_V3,
            Some(json!([authored_ability()])),
        );
        let catalogs = document["pack"]["catalogs"]
            .as_object_mut()
            .expect("catalog fixture is an object");
        catalogs.insert(
            "modifiers".to_string(),
            json!([{
                "id": "modifier.binding-glyph.anchored",
                "label": "Anchored",
                "summary": "Temporary movement penalty.",
                "defaultTenure": "temporary",
                "stackingGroup": "binding-glyph-anchor",
                "stackingPolicy": "refresh",
                "durationPolicy": { "kind": "turns", "turns": 1 },
                "statAdjustments": [{
                    "statId": "mobility",
                    "statLabel": "Mobility",
                    "delta": -1
                }]
            }]),
        );
        catalogs.insert(
            "actions".to_string(),
            json!([{
                "id": "action.binding-glyph",
                "abilityId": "ability.binding-glyph",
                "name": "Binding Glyph",
                "targeting": {
                    "targetKind": "combatant",
                    "selection": "single",
                    "teamConstraint": "hostile",
                    "maximumRange": 6,
                    "visibilityRequirement": "required",
                    "operationPipeline": null
                },
                "check": {
                    "kind": "attack",
                    "modifier": 2,
                    "modifierStatId": "focus",
                    "defense": { "id": "guard", "label": "Guard" }
                },
                "effects": [
                    {
                        "operation": "damage",
                        "damageBonus": 4,
                        "damageType": "arcane"
                    },
                    {
                        "operation": "applyModifier",
                        "modifierId": "modifier.binding-glyph.anchored"
                    }
                ],
                "resourceCosts": [{ "resourceId": "standard-action", "amount": 1 }],
                "movement": null,
                "actionText": "Inscribe the glyph.",
                "effectText": "Damage and anchor the target."
            }]),
        );
        document
    }
}
