use rulebench_rules::{
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticStepExecutionReadout,
    CombatSessionSnapshot,
};
use serde::{Deserialize, Serialize};

use super::{LiveCommandStepDto, LiveSessionSnapshotDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveAutomaticStepDto {
    pub accepted: bool,
    pub decision_kind: String,
    pub operation_kind: Option<String>,
    pub lifecycle_phase: String,
    pub current_actor_id: Option<String>,
    pub policy_id: String,
    pub policy_version: u32,
    pub selected_action_id: Option<String>,
    pub selected_target_id: Option<String>,
    pub candidate_count: usize,
    pub accepted_candidate_count: usize,
    pub submitted_step: Option<LiveCommandStepDto>,
    pub reason: String,
    pub snapshot: Option<LiveSessionSnapshotDto>,
}

impl LiveAutomaticStepDto {
    pub fn new(
        readout: &CombatSessionAutomaticStepExecutionReadout,
        snapshot: &CombatSessionSnapshot,
    ) -> Self {
        Self::from_readout(readout, Some(snapshot))
    }

    fn from_run(readout: &CombatSessionAutomaticStepExecutionReadout) -> Self {
        Self::from_readout(readout, None)
    }

    fn from_readout(
        readout: &CombatSessionAutomaticStepExecutionReadout,
        snapshot: Option<&CombatSessionSnapshot>,
    ) -> Self {
        Self {
            accepted: readout.plan.accepted,
            decision_kind: readout.plan.decision_kind.code().to_string(),
            operation_kind: readout
                .plan
                .operation_kind
                .map(|kind| kind.code().to_string()),
            lifecycle_phase: readout.plan.lifecycle_phase.code().to_string(),
            current_actor_id: readout.plan.current_actor_id.clone(),
            policy_id: readout.plan.policy_decision.policy.id.clone(),
            policy_version: readout.plan.policy_decision.policy.version,
            selected_action_id: readout.plan.policy_decision.selected_action_id.clone(),
            selected_target_id: readout.plan.policy_decision.selected_target_id.clone(),
            candidate_count: readout.plan.policy_decision.candidate_count,
            accepted_candidate_count: readout.plan.policy_decision.accepted_candidate_count,
            submitted_step: readout
                .auto_candidate
                .as_ref()
                .and_then(|execution| execution.submitted_step.as_ref())
                .map(LiveCommandStepDto::from),
            reason: readout.plan.reason.clone(),
            snapshot: snapshot.map(LiveSessionSnapshotDto::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveAutomaticRunDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub accepted: bool,
    pub decision_kind: String,
    pub max_steps: u32,
    pub executed_step_count: u32,
    pub policy_id: String,
    pub policy_version: u32,
    pub steps: Vec<LiveAutomaticStepDto>,
    pub final_snapshot: LiveSessionSnapshotDto,
    pub reason: String,
}

impl From<&CombatSessionAutomaticRunReadout> for LiveAutomaticRunDto {
    fn from(value: &CombatSessionAutomaticRunReadout) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            summary: value.summary.clone(),
            accepted: value.accepted,
            decision_kind: value.decision_kind.code().to_string(),
            max_steps: value.max_steps,
            executed_step_count: value.executed_step_count,
            policy_id: value.policy.id.clone(),
            policy_version: value.policy.version,
            steps: value
                .steps
                .iter()
                .map(LiveAutomaticStepDto::from_run)
                .collect(),
            final_snapshot: LiveSessionSnapshotDto::from(&value.final_snapshot),
            reason: value.reason.clone(),
        }
    }
}
