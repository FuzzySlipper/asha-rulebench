use rulebench_rules::{
    CapabilityEntry, CapabilityIdentity, HostCapabilityProfile, RulebenchCapabilityManifest,
    RulesetProviderManifestEntry,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySupportDto {
    pub declared: bool,
    pub validation_supported: bool,
    pub runtime_executable: bool,
    pub protocol_exposed: bool,
    pub live_host_exposed: bool,
    pub ui_exposed: bool,
    pub regression_covered: bool,
    pub durable_across_restart: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntryDto {
    pub id: String,
    pub kind: String,
    pub version: String,
    pub support: CapabilitySupportDto,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityIdentityDto {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesetProviderDto {
    pub provider: CapabilityIdentityDto,
    pub ruleset: CapabilityIdentityDto,
    pub operation_vocabulary_version: String,
    pub effect_operation_vocabulary_version: String,
    pub capabilities: Vec<CapabilityIdentityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostCapabilityProfileDto {
    pub adapter_id: String,
    pub storage_mode: String,
    pub content_storage_adapter: String,
    pub replay_storage_adapter: String,
    pub replay_recovery_mode: String,
    pub session_recovery_mode: String,
    pub authority_viewer_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulebenchCapabilityManifestDto {
    pub manifest_id: String,
    pub manifest_version: u32,
    pub generated_artifact_schema: String,
    pub governed_asha_revision: String,
    pub operation_vocabulary_version: String,
    pub effect_vocabulary_version: String,
    pub protocol_id: String,
    pub protocol_version: u32,
    pub host: HostCapabilityProfileDto,
    pub providers: Vec<RulesetProviderDto>,
    pub rulesets: Vec<CapabilityIdentityDto>,
    pub packages: Vec<CapabilityIdentityDto>,
    pub scenarios: Vec<CapabilityIdentityDto>,
    pub capabilities: Vec<CapabilityEntryDto>,
}

impl From<&RulebenchCapabilityManifest> for RulebenchCapabilityManifestDto {
    fn from(value: &RulebenchCapabilityManifest) -> Self {
        Self {
            manifest_id: value.manifest_id.clone(),
            manifest_version: value.manifest_version,
            generated_artifact_schema: value.generated_artifact_schema.clone(),
            governed_asha_revision: value.governed_asha_revision.clone(),
            operation_vocabulary_version: value.operation_vocabulary_version.clone(),
            effect_vocabulary_version: value.effect_vocabulary_version.clone(),
            protocol_id: crate::PROTOCOL_ID.to_string(),
            protocol_version: crate::PROTOCOL_VERSION,
            host: HostCapabilityProfileDto::from(&value.host),
            providers: value
                .providers
                .iter()
                .map(RulesetProviderDto::from)
                .collect(),
            rulesets: value
                .rulesets
                .iter()
                .map(CapabilityIdentityDto::from)
                .collect(),
            packages: value
                .packages
                .iter()
                .map(CapabilityIdentityDto::from)
                .collect(),
            scenarios: value
                .scenarios
                .iter()
                .map(CapabilityIdentityDto::from)
                .collect(),
            capabilities: value
                .capabilities
                .iter()
                .map(CapabilityEntryDto::from)
                .collect(),
        }
    }
}

impl From<&RulesetProviderManifestEntry> for RulesetProviderDto {
    fn from(value: &RulesetProviderManifestEntry) -> Self {
        Self {
            provider: CapabilityIdentityDto::from(&value.provider),
            ruleset: CapabilityIdentityDto::from(&value.ruleset),
            operation_vocabulary_version: value.operation_vocabulary_version.clone(),
            effect_operation_vocabulary_version: value.effect_operation_vocabulary_version.clone(),
            capabilities: value
                .capabilities
                .iter()
                .map(CapabilityIdentityDto::from)
                .collect(),
        }
    }
}

impl From<&CapabilityIdentity> for CapabilityIdentityDto {
    fn from(value: &CapabilityIdentity) -> Self {
        Self {
            id: value.id.clone(),
            version: value.version.clone(),
        }
    }
}

impl From<&CapabilityEntry> for CapabilityEntryDto {
    fn from(value: &CapabilityEntry) -> Self {
        Self {
            id: value.id.clone(),
            kind: value.kind.code().to_string(),
            version: value.version.clone(),
            support: CapabilitySupportDto {
                declared: value.support.declared,
                validation_supported: value.support.validation_supported,
                runtime_executable: value.support.runtime_executable,
                protocol_exposed: value.support.protocol_exposed,
                live_host_exposed: value.support.live_host_exposed,
                ui_exposed: value.support.ui_exposed,
                regression_covered: value.support.regression_covered,
                durable_across_restart: value.support.durable_across_restart,
            },
            evidence: value.evidence.clone(),
        }
    }
}

impl From<&HostCapabilityProfile> for HostCapabilityProfileDto {
    fn from(value: &HostCapabilityProfile) -> Self {
        Self {
            adapter_id: value.adapter_id.clone(),
            storage_mode: value.storage_mode.clone(),
            content_storage_adapter: value.content_storage_adapter.clone(),
            replay_storage_adapter: value.replay_storage_adapter.clone(),
            replay_recovery_mode: value.replay_recovery_mode.clone(),
            session_recovery_mode: value.session_recovery_mode.clone(),
            authority_viewer_mode: value.authority_viewer_mode.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rulebench_rules::{
        assemble_capability_manifest, CapabilityRegistryInput, HostCapabilityProfile,
    };

    use super::*;

    #[test]
    fn mapping_preserves_support_levels_and_protocol_identity() {
        let manifest = assemble_capability_manifest(
            CapabilityRegistryInput {
                providers: Vec::new(),
                rulesets: Vec::new(),
                packages: Vec::new(),
                scenarios: Vec::new(),
                regression_capability_ids: Vec::new(),
            },
            HostCapabilityProfile {
                adapter_id: "test".to_string(),
                storage_mode: "memory".to_string(),
                content_storage_adapter: "none".to_string(),
                replay_storage_adapter: "memory".to_string(),
                replay_recovery_mode: "finalizedArchive".to_string(),
                session_recovery_mode: "none".to_string(),
                authority_viewer_mode: "liveAuthorityReadback".to_string(),
                authored_content_enabled: false,
                exposes_capabilities_through_protocol: true,
                exposes_capabilities_through_live_host: true,
                exposes_capabilities_in_ui: true,
                durable_content: false,
                durable_finalized_replays: false,
                durable_active_sessions: false,
            },
        )
        .expect("manifest is valid");

        let dto = RulebenchCapabilityManifestDto::from(&manifest);
        assert_eq!(dto.protocol_id, crate::PROTOCOL_ID);
        assert_eq!(dto.protocol_version, crate::PROTOCOL_VERSION);
        assert!(dto
            .capabilities
            .iter()
            .any(|capability| capability.id == "operation.damage"));
    }
}
