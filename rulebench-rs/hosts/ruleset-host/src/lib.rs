#![forbid(unsafe_code)]

use std::sync::Mutex;

use rpg_compiler::{
    compile_prepared_ruleset_json, load_compiled_ruleset_artifact_json, CompiledRulesetBundle,
    RpgCompileFailure, RpgDiagnostic, RpgDiagnosticSeverity, RpgDiagnosticStage,
};
use rpg_ir::{
    CompiledRulesetArtifact, MaterializedRulesetDefinitionKind, MaterializedRulesetVisibility,
    RulesetDependencyRelationship, RulesetExtensionPolicy, RulesetRelationshipKind,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum RulesetLifecycleStatus {
    NoActiveRuleset,
    CompiledCandidate,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDiagnosticDto {
    pub stage: String,
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetIdentityDto {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetSourcePackageDto {
    pub id: String,
    pub version: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetLockEntryDto {
    pub requester: String,
    pub package_id: String,
    pub requested_version: String,
    pub resolved_version: String,
    pub source_fingerprint: String,
    pub import_as: String,
    pub relationship: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetRequirementDto {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetDefinitionDto {
    pub id: String,
    pub label: Option<String>,
    pub kind: String,
    pub visibility: String,
    pub extension_policy: String,
    pub references: Vec<String>,
    pub package_id: String,
    pub package_version: String,
    pub source_module: String,
    pub source_declaration: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetRelationshipDto {
    pub kind: String,
    pub source: String,
    pub target: String,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetFingerprintDto {
    pub source: String,
    pub semantic: String,
    pub presentation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetArtifactSummaryDto {
    pub schema: RulesetIdentityDto,
    pub artifact_id: String,
    pub composition: RulesetIdentityDto,
    pub language: RulesetIdentityDto,
    pub source_packages: Vec<RulesetSourcePackageDto>,
    pub dependency_lock: Vec<RulesetLockEntryDto>,
    pub required_operations: Vec<RulesetRequirementDto>,
    pub required_capabilities: Vec<RulesetRequirementDto>,
    pub exported_roots: Vec<String>,
    pub definitions: Vec<RulesetDefinitionDto>,
    pub policy_binding_ids: Vec<String>,
    pub relationships: Vec<RulesetRelationshipDto>,
    pub derivation_slots: usize,
    pub overlay_slots: usize,
    pub fingerprints: RulesetFingerprintDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RulesetWorkspaceResponseDto {
    pub ok: bool,
    pub status: RulesetLifecycleStatus,
    pub active_artifact: Option<RulesetArtifactSummaryDto>,
    pub candidate_artifact: Option<RulesetArtifactSummaryDto>,
    pub activation_revision: u32,
    pub gameplay_available: bool,
    pub diagnostics: Vec<RulesetDiagnosticDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct RulesetCompileRequestDto {
    pub prepared_source: String,
}

#[derive(Debug, Clone)]
struct ActivationSlots<Bundle> {
    candidate: Option<Bundle>,
    active: Option<Bundle>,
    activation_revision: u32,
}

impl<Bundle> Default for ActivationSlots<Bundle> {
    fn default() -> Self {
        Self {
            candidate: None,
            active: None,
            activation_revision: 0,
        }
    }
}

impl<Bundle> ActivationSlots<Bundle> {
    fn stage(&mut self, candidate: Bundle) {
        self.candidate = Some(candidate);
    }

    fn activate(&mut self) -> bool {
        let Some(candidate) = self.candidate.take() else {
            return false;
        };
        self.active = Some(candidate);
        self.activation_revision += 1;
        true
    }

    fn status(&self) -> RulesetLifecycleStatus {
        if self.candidate.is_some() {
            RulesetLifecycleStatus::CompiledCandidate
        } else if self.active.is_some() {
            RulesetLifecycleStatus::Active
        } else {
            RulesetLifecycleStatus::NoActiveRuleset
        }
    }
}

pub struct RulesetHost {
    slots: Mutex<ActivationSlots<CompiledRulesetBundle>>,
}

impl RulesetHost {
    pub fn new() -> Self {
        Self {
            slots: Mutex::new(ActivationSlots::default()),
        }
    }

    pub fn status(&self) -> RulesetWorkspaceResponseDto {
        let slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        response_from_slots(true, &slots, Vec::new())
    }

    pub fn compile_candidate(&self, prepared_source: &str) -> RulesetWorkspaceResponseDto {
        let compilation = compile_prepared_ruleset_json(prepared_source.as_bytes());
        match compilation {
            Ok(bundle) => match close_portable_artifact(bundle) {
                Ok(loaded) => {
                    let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                    slots.stage(loaded);
                    response_from_slots(true, &slots, Vec::new())
                }
                Err(diagnostics) => {
                    let slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                    response_from_slots(false, &slots, diagnostics)
                }
            },
            Err(failure) => {
                let slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
                response_from_slots(false, &slots, diagnostics_from_failure(failure))
            }
        }
    }

    pub fn activate_candidate(&self) -> RulesetWorkspaceResponseDto {
        let mut slots = self.slots.lock().unwrap_or_else(|error| error.into_inner());
        if slots.activate() {
            response_from_slots(true, &slots, Vec::new())
        } else {
            response_from_slots(
                false,
                &slots,
                vec![RulesetDiagnosticDto {
                    stage: "activation".to_owned(),
                    severity: "error".to_owned(),
                    code: "RULESET_ACTIVATION_CANDIDATE_REQUIRED".to_owned(),
                    path: "$.candidateArtifact".to_owned(),
                    message: "compile an accepted artifact before activation".to_owned(),
                }],
            )
        }
    }
}

fn close_portable_artifact(
    bundle: CompiledRulesetBundle,
) -> Result<CompiledRulesetBundle, Vec<RulesetDiagnosticDto>> {
    let encoded = serde_json::to_vec(bundle.artifact()).map_err(|error| {
        vec![RulesetDiagnosticDto {
            stage: "artifact".to_owned(),
            severity: "error".to_owned(),
            code: "RULESET_ARTIFACT_ENCODING_FAILED".to_owned(),
            path: "$".to_owned(),
            message: error.to_string(),
        }]
    })?;
    load_compiled_ruleset_artifact_json(&encoded).map_err(diagnostics_from_failure)
}

fn response_from_slots(
    ok: bool,
    slots: &ActivationSlots<CompiledRulesetBundle>,
    diagnostics: Vec<RulesetDiagnosticDto>,
) -> RulesetWorkspaceResponseDto {
    RulesetWorkspaceResponseDto {
        ok,
        status: slots.status(),
        active_artifact: slots
            .active
            .as_ref()
            .map(|bundle| artifact_summary(bundle.artifact())),
        candidate_artifact: slots
            .candidate
            .as_ref()
            .map(|bundle| artifact_summary(bundle.artifact())),
        activation_revision: slots.activation_revision,
        gameplay_available: false,
        diagnostics,
    }
}

fn diagnostics_from_failure(failure: RpgCompileFailure) -> Vec<RulesetDiagnosticDto> {
    failure
        .diagnostics
        .into_iter()
        .map(diagnostic_dto)
        .collect()
}

fn diagnostic_dto(diagnostic: RpgDiagnostic) -> RulesetDiagnosticDto {
    RulesetDiagnosticDto {
        stage: diagnostic_stage(diagnostic.stage).to_owned(),
        severity: diagnostic_severity(diagnostic.severity).to_owned(),
        code: diagnostic.code,
        path: diagnostic.path,
        message: diagnostic.message,
    }
}

fn diagnostic_stage(stage: RpgDiagnosticStage) -> &'static str {
    match stage {
        RpgDiagnosticStage::Decode => "decode",
        RpgDiagnosticStage::Compatibility => "compatibility",
        RpgDiagnosticStage::Requirements => "requirements",
        RpgDiagnosticStage::References => "references",
        RpgDiagnosticStage::Semantics => "semantics",
        RpgDiagnosticStage::Artifact => "artifact",
    }
}

fn diagnostic_severity(severity: RpgDiagnosticSeverity) -> &'static str {
    match severity {
        RpgDiagnosticSeverity::Error => "error",
    }
}

fn artifact_summary(artifact: &CompiledRulesetArtifact) -> RulesetArtifactSummaryDto {
    RulesetArtifactSummaryDto {
        schema: RulesetIdentityDto {
            id: artifact.artifact_schema.identity.clone(),
            version: artifact.artifact_schema.major.to_string(),
        },
        artifact_id: artifact.artifact_id.clone(),
        composition: RulesetIdentityDto {
            id: artifact.composition_identity.id.clone(),
            version: artifact.composition_identity.version.clone(),
        },
        language: RulesetIdentityDto {
            id: artifact.language_identity.id.clone(),
            version: artifact.language_identity.version.clone(),
        },
        source_packages: artifact
            .source_packages
            .iter()
            .map(|source| RulesetSourcePackageDto {
                id: source.id.clone(),
                version: source.version.clone(),
                source_fingerprint: source.source_fingerprint.clone(),
            })
            .collect(),
        dependency_lock: artifact
            .dependency_lock
            .iter()
            .map(|entry| RulesetLockEntryDto {
                requester: entry.requester.clone(),
                package_id: entry.package_id.clone(),
                requested_version: entry.requested_version.clone(),
                resolved_version: entry.resolved_version.clone(),
                source_fingerprint: entry.source_fingerprint.clone(),
                import_as: entry.import_as.clone(),
                relationship: dependency_relationship(entry.relationship).to_owned(),
            })
            .collect(),
        required_operations: artifact
            .required_operations
            .iter()
            .map(|entry| RulesetRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        required_capabilities: artifact
            .required_capabilities
            .iter()
            .map(|entry| RulesetRequirementDto {
                id: entry.id.clone(),
                version: entry.version,
            })
            .collect(),
        exported_roots: artifact.exported_roots.clone(),
        definitions: artifact
            .materialized_definitions
            .iter()
            .map(|definition| RulesetDefinitionDto {
                id: definition.id.clone(),
                label: definition
                    .presentation
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                kind: definition_kind(definition.kind).to_owned(),
                visibility: definition_visibility(definition.visibility).to_owned(),
                extension_policy: extension_policy(definition.extension_policy).to_owned(),
                references: definition.references.clone(),
                package_id: definition.provenance.package_id.clone(),
                package_version: definition.provenance.package_version.clone(),
                source_module: definition.provenance.source.module.clone(),
                source_declaration: definition.provenance.source.declaration.clone(),
            })
            .collect(),
        policy_binding_ids: artifact
            .compiled_policy_bindings
            .iter()
            .map(|binding| binding.id.clone())
            .collect(),
        relationships: artifact
            .relationships
            .iter()
            .map(|relationship| RulesetRelationshipDto {
                kind: relationship_kind(relationship.kind).to_owned(),
                source: relationship.source.clone(),
                target: relationship.target.clone(),
                order: relationship.order,
            })
            .collect(),
        derivation_slots: artifact.derivation_provenance.len(),
        overlay_slots: artifact.overlay_provenance.len(),
        fingerprints: RulesetFingerprintDto {
            source: artifact.fingerprints.source.clone(),
            semantic: artifact.fingerprints.semantic.clone(),
            presentation: artifact.fingerprints.presentation.clone(),
        },
    }
}

fn dependency_relationship(relationship: RulesetDependencyRelationship) -> &'static str {
    match relationship {
        RulesetDependencyRelationship::DependsOn => "dependsOn",
        RulesetDependencyRelationship::Contributes => "contributes",
        RulesetDependencyRelationship::Patches => "patches",
    }
}

fn definition_kind(kind: MaterializedRulesetDefinitionKind) -> &'static str {
    match kind {
        MaterializedRulesetDefinitionKind::Action => "action",
        MaterializedRulesetDefinitionKind::Support => "support",
    }
}

fn definition_visibility(visibility: MaterializedRulesetVisibility) -> &'static str {
    match visibility {
        MaterializedRulesetVisibility::Exported => "exported",
        MaterializedRulesetVisibility::Support => "support",
    }
}

fn extension_policy(policy: RulesetExtensionPolicy) -> &'static str {
    match policy {
        RulesetExtensionPolicy::Sealed => "sealed",
        RulesetExtensionPolicy::Derivable => "derivable",
        RulesetExtensionPolicy::Patchable => "patchable",
        RulesetExtensionPolicy::Configurable => "configurable",
    }
}

fn relationship_kind(kind: RulesetRelationshipKind) -> &'static str {
    match kind {
        RulesetRelationshipKind::DependsOn => "dependsOn",
        RulesetRelationshipKind::Contributes => "contributes",
        RulesetRelationshipKind::DerivesFrom => "derivesFrom",
        RulesetRelationshipKind::Patches => "patches",
        RulesetRelationshipKind::Configures => "configures",
        RulesetRelationshipKind::Exports => "exports",
    }
}

pub fn generated_protocol() -> String {
    let declarations = [
        RulesetLifecycleStatus::decl(),
        RulesetDiagnosticDto::decl(),
        RulesetIdentityDto::decl(),
        RulesetSourcePackageDto::decl(),
        RulesetLockEntryDto::decl(),
        RulesetRequirementDto::decl(),
        RulesetDefinitionDto::decl(),
        RulesetRelationshipDto::decl(),
        RulesetFingerprintDto::decl(),
        RulesetArtifactSummaryDto::decl(),
        RulesetWorkspaceResponseDto::decl(),
        RulesetCompileRequestDto::decl(),
    ];
    let exports = declarations
        .into_iter()
        .map(|declaration| format!("export {declaration}"))
        .collect::<Vec<_>>();
    format!(
        "// @generated by rulebench-ruleset-host. Do not edit.\n\n{}\n",
        exports.join("\n\n")
    )
}

#[cfg(test)]
mod tests {
    use super::{ActivationSlots, RulesetHost, RulesetLifecycleStatus};

    #[test]
    fn activation_replaces_the_whole_candidate_and_never_partial_state() {
        let mut slots = ActivationSlots::default();
        assert_eq!(slots.status(), RulesetLifecycleStatus::NoActiveRuleset);
        assert!(!slots.activate());
        assert_eq!(slots.activation_revision, 0);

        slots.stage("artifact-one".to_owned());
        assert_eq!(slots.status(), RulesetLifecycleStatus::CompiledCandidate);
        assert!(slots.activate());
        assert_eq!(slots.active.as_deref(), Some("artifact-one"));
        assert_eq!(slots.candidate, None);
        assert_eq!(slots.activation_revision, 1);

        slots.stage("artifact-two".to_owned());
        assert_eq!(slots.active.as_deref(), Some("artifact-one"));
        assert_eq!(slots.activation_revision, 1);
        assert!(slots.activate());
        assert_eq!(slots.active.as_deref(), Some("artifact-two"));
        assert_eq!(slots.activation_revision, 2);
    }

    #[test]
    fn failed_compilation_cannot_create_a_candidate_or_active_artifact() {
        let host = RulesetHost::new();

        let compilation = host.compile_candidate(r#"{"unexpected":true}"#);
        assert!(!compilation.ok);
        assert_eq!(compilation.status, RulesetLifecycleStatus::NoActiveRuleset);
        assert!(compilation.candidate_artifact.is_none());
        assert!(compilation.active_artifact.is_none());
        assert_eq!(compilation.activation_revision, 0);
        assert_eq!(
            compilation.diagnostics[0].code,
            "RULESET_PREPARED_DECODE_FAILED"
        );

        let activation = host.activate_candidate();
        assert!(!activation.ok);
        assert_eq!(activation.status, RulesetLifecycleStatus::NoActiveRuleset);
        assert_eq!(activation.activation_revision, 0);
    }
}
