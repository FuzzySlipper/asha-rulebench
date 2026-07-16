use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionRecoveryEntryDto {
    pub session_id: String,
    pub origin: String,
    pub state: String,
    pub generation: u64,
    pub last_verified_frame_id: String,
    pub pending_reaction_window_id: Option<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionRecoveryIssueDto {
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionRecoveryCatalogDto {
    pub sessions: Vec<SessionRecoveryEntryDto>,
    pub issues: Vec<SessionRecoveryIssueDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionRecoveryForkRequestDto {
    pub new_session_id: String,
}
