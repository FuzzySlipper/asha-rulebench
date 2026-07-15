use std::collections::BTreeSet;

use rulebench_combat::{
    COMBAT_AUTOMATION_POLICY_REGISTRY, GOVERNED_ASHA_REVISION, RUNTIME_EFFECT_OPERATION_REGISTRY,
    RUNTIME_TARGETING_OPERATION_REGISTRY,
};
use rulebench_ruleset::{EffectOperationId, OperationPipelineV2, TargetingOperationId};

pub const CAPABILITY_MANIFEST_ID: &str = "asha-rulebench.capabilities";
pub const CAPABILITY_MANIFEST_VERSION: u32 = 2;
pub const CAPABILITY_ARTIFACT_SCHEMA: &str = "asha-rulebench.capabilities.ts@2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityKind {
    Operation,
    Targeting,
    Policy,
    Content,
    Replay,
    Session,
}

impl CapabilityKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::Targeting => "targeting",
            Self::Policy => "policy",
            Self::Content => "content",
            Self::Replay => "replay",
            Self::Session => "session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySupport {
    pub declared: bool,
    pub validation_supported: bool,
    pub runtime_executable: bool,
    pub protocol_exposed: bool,
    pub live_host_exposed: bool,
    pub ui_exposed: bool,
    pub regression_covered: bool,
    pub durable_across_restart: bool,
}

impl CapabilitySupport {
    pub fn validate(&self, capability_id: &str) -> Result<(), CapabilityManifestError> {
        let dependency_error = if self.validation_supported && !self.declared {
            Some("validation support requires a declaration")
        } else if self.runtime_executable && !self.validation_supported {
            Some("runtime execution requires validation support")
        } else if self.protocol_exposed && !self.runtime_executable {
            Some("protocol exposure requires runtime execution")
        } else if self.live_host_exposed && !self.protocol_exposed {
            Some("live-host exposure requires protocol exposure")
        } else if self.ui_exposed && !self.live_host_exposed {
            Some("UI exposure requires live-host exposure")
        } else if self.durable_across_restart && !self.runtime_executable {
            Some("restart durability requires runtime execution")
        } else {
            None
        };
        match dependency_error {
            Some(reason) => Err(CapabilityManifestError::InvalidSupport {
                capability_id: capability_id.to_string(),
                reason: reason.to_string(),
            }),
            None => Ok(()),
        }
    }
}

/// Executable operation, targeting, and automation-policy identities that
/// require authority-level conformance evidence.
///
/// This is assembled from the same owner registries as the manifest. Fixture
/// runners use it to reject missing, renamed, or version-drifted conformance
/// registrations without maintaining a parallel capability checklist.
pub fn executable_conformance_capabilities() -> Vec<CapabilityIdentity> {
    let mut identities = EffectOperationId::ALL
        .iter()
        .copied()
        .filter(|operation| RUNTIME_EFFECT_OPERATION_REGISTRY.contains(operation))
        .map(|operation| CapabilityIdentity {
            id: format!("operation.{}", operation.code()),
            version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        })
        .chain(
            TargetingOperationId::ALL
                .iter()
                .copied()
                .filter(|targeting| RUNTIME_TARGETING_OPERATION_REGISTRY.contains(targeting))
                .map(|targeting| CapabilityIdentity {
                    id: format!("targeting.{}", targeting.code()),
                    version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
                }),
        )
        .chain(
            COMBAT_AUTOMATION_POLICY_REGISTRY
                .iter()
                .map(|policy| CapabilityIdentity {
                    id: format!("policy.{}", policy.id),
                    version: policy.version.to_string(),
                }),
        )
        .collect::<Vec<_>>();
    identities.sort();
    identities
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityEntry {
    pub id: String,
    pub kind: CapabilityKind,
    pub version: String,
    pub support: CapabilitySupport,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapabilityIdentity {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetProviderManifestEntry {
    pub provider: CapabilityIdentity,
    pub ruleset: CapabilityIdentity,
    pub operation_vocabulary_version: String,
    pub effect_operation_vocabulary_version: String,
    pub capabilities: Vec<CapabilityIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCapabilityProfile {
    pub adapter_id: String,
    pub storage_mode: String,
    pub content_storage_adapter: String,
    pub replay_storage_adapter: String,
    pub replay_recovery_mode: String,
    pub session_recovery_mode: String,
    pub authored_content_enabled: bool,
    pub exposes_capabilities_through_protocol: bool,
    pub exposes_capabilities_through_live_host: bool,
    pub exposes_capabilities_in_ui: bool,
    pub durable_content: bool,
    pub durable_finalized_replays: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRegistryInput {
    pub providers: Vec<RulesetProviderManifestEntry>,
    pub rulesets: Vec<CapabilityIdentity>,
    pub packages: Vec<CapabilityIdentity>,
    pub scenarios: Vec<CapabilityIdentity>,
    pub regression_capability_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchCapabilityManifest {
    pub manifest_id: String,
    pub manifest_version: u32,
    pub generated_artifact_schema: String,
    pub governed_asha_revision: String,
    pub operation_vocabulary_version: String,
    pub effect_vocabulary_version: String,
    pub host: HostCapabilityProfile,
    pub providers: Vec<RulesetProviderManifestEntry>,
    pub rulesets: Vec<CapabilityIdentity>,
    pub packages: Vec<CapabilityIdentity>,
    pub scenarios: Vec<CapabilityIdentity>,
    pub capabilities: Vec<CapabilityEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityManifestError {
    DuplicateCapability {
        capability_id: String,
    },
    InvalidSupport {
        capability_id: String,
        reason: String,
    },
}

pub fn assemble_capability_manifest(
    mut input: CapabilityRegistryInput,
    host: HostCapabilityProfile,
) -> Result<RulebenchCapabilityManifest, CapabilityManifestError> {
    sort_and_deduplicate_identities(&mut input.rulesets);
    sort_and_deduplicate_identities(&mut input.packages);
    sort_and_deduplicate_identities(&mut input.scenarios);
    input.providers.sort_by(|left, right| {
        left.provider
            .cmp(&right.provider)
            .then_with(|| left.ruleset.cmp(&right.ruleset))
    });
    for provider in &mut input.providers {
        sort_and_deduplicate_identities(&mut provider.capabilities);
    }
    let regression = input
        .regression_capability_ids
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut capabilities = operation_capabilities(&host, &regression);
    capabilities.extend(targeting_capabilities(&host, &regression));
    capabilities.extend(policy_capabilities(&host, &regression));
    capabilities.extend(host_capabilities(&host, &regression));
    capabilities.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.version.cmp(&right.version))
    });

    let mut seen = BTreeSet::new();
    for capability in &capabilities {
        if !seen.insert(capability.id.clone()) {
            return Err(CapabilityManifestError::DuplicateCapability {
                capability_id: capability.id.clone(),
            });
        }
        capability.support.validate(&capability.id)?;
    }

    Ok(RulebenchCapabilityManifest {
        manifest_id: CAPABILITY_MANIFEST_ID.to_string(),
        manifest_version: CAPABILITY_MANIFEST_VERSION,
        generated_artifact_schema: CAPABILITY_ARTIFACT_SCHEMA.to_string(),
        governed_asha_revision: GOVERNED_ASHA_REVISION.to_string(),
        operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        effect_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        host,
        providers: input.providers,
        rulesets: input.rulesets,
        packages: input.packages,
        scenarios: input.scenarios,
        capabilities,
    })
}

fn operation_capabilities(
    host: &HostCapabilityProfile,
    regression: &BTreeSet<String>,
) -> Vec<CapabilityEntry> {
    EffectOperationId::ALL
        .iter()
        .copied()
        .map(|operation| {
            let id = format!("operation.{}", operation.code());
            let validation_supported = operation.validation_supported();
            let runtime_executable = RUNTIME_EFFECT_OPERATION_REGISTRY.contains(&operation);
            CapabilityEntry {
                id: id.clone(),
                kind: CapabilityKind::Operation,
                version: EffectOperationId::VOCABULARY_VERSION.to_string(),
                support: CapabilitySupport {
                    declared: true,
                    validation_supported,
                    runtime_executable,
                    protocol_exposed: runtime_executable
                        && host.exposes_capabilities_through_protocol,
                    live_host_exposed: runtime_executable
                        && host.exposes_capabilities_through_live_host,
                    ui_exposed: runtime_executable && host.exposes_capabilities_in_ui,
                    regression_covered: regression.contains(&id),
                    durable_across_restart: runtime_executable && host.durable_finalized_replays,
                },
                evidence: vec![
                    "rulebench-ruleset.effect-operation-registry".to_string(),
                    "rulebench-combat.runtime-effect-operation-registry".to_string(),
                ],
            }
        })
        .collect()
}

fn targeting_capabilities(
    host: &HostCapabilityProfile,
    regression: &BTreeSet<String>,
) -> Vec<CapabilityEntry> {
    TargetingOperationId::ALL
        .iter()
        .copied()
        .map(|targeting| {
            let id = format!("targeting.{}", targeting.code());
            let validation_supported = targeting.validation_supported();
            let runtime_executable = RUNTIME_TARGETING_OPERATION_REGISTRY.contains(&targeting);
            CapabilityEntry {
                id: id.clone(),
                kind: CapabilityKind::Targeting,
                version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
                support: CapabilitySupport {
                    declared: true,
                    validation_supported,
                    runtime_executable,
                    protocol_exposed: runtime_executable
                        && host.exposes_capabilities_through_protocol,
                    live_host_exposed: runtime_executable
                        && host.exposes_capabilities_through_live_host,
                    ui_exposed: runtime_executable && host.exposes_capabilities_in_ui,
                    regression_covered: regression.contains(&id),
                    durable_across_restart: runtime_executable && host.durable_finalized_replays,
                },
                evidence: vec![
                    "rulebench-ruleset.targeting-operation-registry".to_string(),
                    "rulebench-combat.runtime-targeting-operation-registry".to_string(),
                ],
            }
        })
        .collect()
}

fn policy_capabilities(
    host: &HostCapabilityProfile,
    regression: &BTreeSet<String>,
) -> Vec<CapabilityEntry> {
    COMBAT_AUTOMATION_POLICY_REGISTRY
        .iter()
        .map(|policy| {
            let id = format!("policy.{}", policy.id);
            CapabilityEntry {
                id: id.clone(),
                kind: CapabilityKind::Policy,
                version: policy.version.to_string(),
                support: CapabilitySupport {
                    declared: true,
                    validation_supported: true,
                    runtime_executable: true,
                    protocol_exposed: host.exposes_capabilities_through_protocol,
                    live_host_exposed: host.exposes_capabilities_through_live_host,
                    ui_exposed: host.exposes_capabilities_in_ui,
                    regression_covered: regression.contains(&id),
                    durable_across_restart: host.durable_finalized_replays,
                },
                evidence: vec!["rulebench-combat.automation-policy-registry".to_string()],
            }
        })
        .collect()
}

fn host_capabilities(
    host: &HostCapabilityProfile,
    regression: &BTreeSet<String>,
) -> Vec<CapabilityEntry> {
    vec![
        host_capability(
            "content.authored-pack",
            "1",
            CapabilityKind::Content,
            host.authored_content_enabled,
            host.durable_content,
            host,
            regression,
        ),
        host_capability(
            "replay.finalized-archive",
            "1",
            CapabilityKind::Replay,
            true,
            host.durable_finalized_replays,
            host,
            regression,
        ),
        CapabilityEntry {
            id: "session.active-recovery".to_string(),
            kind: CapabilityKind::Session,
            version: "none".to_string(),
            support: CapabilitySupport {
                declared: false,
                validation_supported: false,
                runtime_executable: false,
                protocol_exposed: false,
                live_host_exposed: false,
                ui_exposed: false,
                regression_covered: false,
                durable_across_restart: false,
            },
            evidence: vec![format!(
                "rulebench-process-host.session-recovery-mode:{}",
                host.session_recovery_mode
            )],
        },
    ]
}

fn host_capability(
    id: &str,
    version: &str,
    kind: CapabilityKind,
    executable: bool,
    durable: bool,
    host: &HostCapabilityProfile,
    regression: &BTreeSet<String>,
) -> CapabilityEntry {
    CapabilityEntry {
        id: id.to_string(),
        kind,
        version: version.to_string(),
        support: CapabilitySupport {
            declared: true,
            validation_supported: true,
            runtime_executable: executable,
            protocol_exposed: executable && host.exposes_capabilities_through_protocol,
            live_host_exposed: executable && host.exposes_capabilities_through_live_host,
            ui_exposed: executable && host.exposes_capabilities_in_ui,
            regression_covered: regression.contains(id),
            durable_across_restart: executable && durable,
        },
        evidence: vec![format!(
            "rulebench-process-host.storage-mode:{}",
            host.storage_mode
        )],
    }
}

fn sort_and_deduplicate_identities(identities: &mut Vec<CapabilityIdentity>) {
    identities.sort();
    identities.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host() -> HostCapabilityProfile {
        HostCapabilityProfile {
            adapter_id: "test-host".to_string(),
            storage_mode: "memory".to_string(),
            content_storage_adapter: "memory".to_string(),
            replay_storage_adapter: "memory".to_string(),
            replay_recovery_mode: "finalizedArchive".to_string(),
            session_recovery_mode: "none".to_string(),
            authored_content_enabled: false,
            exposes_capabilities_through_protocol: true,
            exposes_capabilities_through_live_host: true,
            exposes_capabilities_in_ui: true,
            durable_content: false,
            durable_finalized_replays: false,
        }
    }

    #[test]
    fn declared_but_unimplemented_operation_cannot_be_reported_executable() {
        let support = CapabilitySupport {
            declared: true,
            validation_supported: false,
            runtime_executable: true,
            protocol_exposed: false,
            live_host_exposed: false,
            ui_exposed: false,
            regression_covered: false,
            durable_across_restart: false,
        };
        assert!(matches!(
            support.validate("operation.future"),
            Err(CapabilityManifestError::InvalidSupport { .. })
        ));
    }

    #[test]
    fn generated_or_ui_evidence_cannot_substitute_for_live_runtime_support() {
        let support = CapabilitySupport {
            declared: true,
            validation_supported: true,
            runtime_executable: true,
            protocol_exposed: true,
            live_host_exposed: false,
            ui_exposed: true,
            regression_covered: true,
            durable_across_restart: false,
        };
        assert!(matches!(
            support.validate("targeting.fixture-only"),
            Err(CapabilityManifestError::InvalidSupport { .. })
        ));
    }

    #[test]
    fn manifest_uses_the_owner_operation_and_policy_registries() {
        let manifest = assemble_capability_manifest(
            CapabilityRegistryInput {
                providers: Vec::new(),
                rulesets: Vec::new(),
                packages: Vec::new(),
                scenarios: Vec::new(),
                regression_capability_ids: vec![
                    "operation.damage".to_string(),
                    "policy.firstAcceptedCandidate".to_string(),
                ],
            },
            host(),
        )
        .expect("owner registries form a valid manifest");

        assert_eq!(
            manifest
                .capabilities
                .iter()
                .filter(|entry| entry.kind == CapabilityKind::Operation)
                .count(),
            EffectOperationId::ALL.len()
        );
        assert!(manifest
            .capabilities
            .iter()
            .any(|entry| entry.id == "policy.firstAcceptedCandidate"));
    }

    #[test]
    fn executable_conformance_identities_are_derived_from_owner_registries() {
        let identities = executable_conformance_capabilities();

        assert_eq!(
            identities.len(),
            RUNTIME_EFFECT_OPERATION_REGISTRY.len()
                + RUNTIME_TARGETING_OPERATION_REGISTRY.len()
                + COMBAT_AUTOMATION_POLICY_REGISTRY.len()
        );
        assert!(identities.iter().any(|identity| {
            identity.id == "operation.heal"
                && identity.version == EffectOperationId::VOCABULARY_VERSION
        }));
        assert!(identities.iter().any(|identity| {
            identity.id == "targeting.multipleCombatants"
                && identity.version == OperationPipelineV2::VOCABULARY_VERSION
        }));
    }
}
