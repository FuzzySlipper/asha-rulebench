use rulebench_content::{
    AuthoredActionAbilityGrantReceipt, AuthoredActionBindingReceipt, AuthoredActionBindingRequest,
};
use serde::{Deserialize, Serialize};

use crate::{ContentFingerprintDto, ContentPackReferenceDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionBindingRequestDto {
    pub content_pack: ContentPackReferenceDto,
    pub action_id: String,
    pub actor_id: String,
}

impl AuthoredActionBindingRequestDto {
    pub fn to_authority(&self) -> AuthoredActionBindingRequest {
        AuthoredActionBindingRequest {
            content_pack: self.content_pack.to_authority(),
            action_id: self.action_id.clone(),
            actor_id: self.actor_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionAbilityGrantReceiptDto {
    pub grant_kind: String,
    pub actor_id: String,
    pub ability_id: String,
}

impl From<&AuthoredActionAbilityGrantReceipt> for AuthoredActionAbilityGrantReceiptDto {
    fn from(value: &AuthoredActionAbilityGrantReceipt) -> Self {
        Self {
            grant_kind: "sessionLocalBaseAbility".to_string(),
            actor_id: value.actor_id.clone(),
            ability_id: value.ability_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionBindingReceiptDto {
    pub binding_version: String,
    pub content_pack_root: ContentPackReferenceDto,
    pub content_pack_references: Vec<ContentPackReferenceDto>,
    pub content_pack_set_fingerprint: ContentFingerprintDto,
    pub action_id: String,
    pub action_definition_fingerprint: ContentFingerprintDto,
    pub ability_id: String,
    pub scenario_id: String,
    pub actor_id: String,
    pub grant: AuthoredActionAbilityGrantReceiptDto,
    pub targeting_operation_vocabulary_version: String,
    pub check_vocabulary_version: String,
    pub effect_operation_vocabulary_version: String,
}

impl From<&AuthoredActionBindingReceipt> for AuthoredActionBindingReceiptDto {
    fn from(value: &AuthoredActionBindingReceipt) -> Self {
        Self {
            binding_version: value.binding_version.clone(),
            content_pack_root: ContentPackReferenceDto::from(&value.content_pack_set.root),
            content_pack_references: value
                .content_pack_set
                .packs
                .iter()
                .map(ContentPackReferenceDto::from)
                .collect(),
            content_pack_set_fingerprint: ContentFingerprintDto {
                algorithm: value.content_pack_set.fingerprint.algorithm.clone(),
                value: value.content_pack_set.fingerprint.value.clone(),
            },
            action_id: value.action_id.clone(),
            action_definition_fingerprint: ContentFingerprintDto {
                algorithm: value.action_definition_fingerprint.algorithm.clone(),
                value: value.action_definition_fingerprint.value.clone(),
            },
            ability_id: value.ability_id.clone(),
            scenario_id: value.scenario_id.clone(),
            actor_id: value.actor_id.clone(),
            grant: AuthoredActionAbilityGrantReceiptDto::from(&value.grant),
            targeting_operation_vocabulary_version: value
                .targeting_operation_vocabulary_version
                .clone(),
            check_vocabulary_version: value.check_vocabulary_version.clone(),
            effect_operation_vocabulary_version: value.effect_operation_vocabulary_version.clone(),
        }
    }
}
