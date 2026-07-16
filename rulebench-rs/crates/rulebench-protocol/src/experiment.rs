use serde::{Deserialize, Serialize};

use crate::CombatAutomationPolicyDto;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyRulesetCompatibilityDto {
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub compatible: bool,
    pub code: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AutomationPolicyCatalogEntryDto {
    pub id: String,
    pub version: u32,
    pub title: String,
    pub summary: String,
    pub selector: String,
    pub requirement: String,
    pub compatibility: Vec<PolicyRulesetCompatibilityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentMatrixRequestDto {
    pub id: String,
    pub scenario_ids: Vec<String>,
    pub policies: Vec<CombatAutomationPolicyDto>,
    pub seeds: Vec<u32>,
    pub max_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentDecisionEvidenceDto {
    pub index: usize,
    pub state_before_fingerprint: String,
    pub operation_kind: Option<String>,
    pub selected_action_id: Option<String>,
    pub selected_target_id: Option<String>,
    pub selected_candidate_index: Option<usize>,
    pub candidate_count: usize,
    pub accepted_candidate_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentMetricsDto {
    pub executed_step_count: u32,
    pub accepted_command_count: usize,
    pub initial_total_hit_points: i32,
    pub final_total_hit_points: i32,
    pub observed_hit_point_delta: i32,
    pub audit_entry_count: usize,
    pub combat_log_entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentTrialReadoutDto {
    pub id: String,
    pub scenario_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub content_pack_id: Option<String>,
    pub content_pack_version: Option<String>,
    pub policy_id: String,
    pub policy_version: u32,
    pub policy_no_candidate_behavior: String,
    pub seed: u32,
    pub max_steps: u32,
    pub accepted: bool,
    pub stop_reason: String,
    pub finalization_outcome: Option<String>,
    pub initial_state_fingerprint: String,
    pub final_state_fingerprint: String,
    pub materialized_rolls: Vec<i32>,
    pub decisions: Vec<ExperimentDecisionEvidenceDto>,
    pub metrics: ExperimentMetricsDto,
    pub replay_package_id: String,
    pub replay_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentReadoutDto {
    pub id: String,
    pub status: String,
    pub planned_trial_count: usize,
    pub completed_trial_count: usize,
    pub max_steps_per_trial: u32,
    pub trials: Vec<ExperimentTrialReadoutDto>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentComparisonRequestDto {
    pub expected_experiment_id: String,
    pub expected_trial_id: String,
    pub actual_experiment_id: String,
    pub actual_trial_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentComparisonReadoutDto {
    pub identical: bool,
    pub first_divergence_index: Option<usize>,
    pub expected_trial_id: String,
    pub actual_trial_id: String,
    pub expected_evidence: Option<ExperimentDecisionEvidenceDto>,
    pub actual_evidence: Option<ExperimentDecisionEvidenceDto>,
    pub reason: String,
}
