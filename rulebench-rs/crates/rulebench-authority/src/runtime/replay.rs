use super::{
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionState,
};
use crate::model::{RulebenchScenario, StateFingerprint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunReplaySpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub initial_session_id: String,
    pub initial_scenario: RulebenchScenario,
    pub run: CombatSessionAutomaticRunSpec,
    pub expected_final_state_fingerprint: StateFingerprint,
    pub expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub expected_executed_step_count: u32,
}

impl CombatSessionAutomaticRunReplaySpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        initial_session_id: impl Into<String>,
        initial_scenario: RulebenchScenario,
        run: CombatSessionAutomaticRunSpec,
        expected_final_state_fingerprint: StateFingerprint,
        expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
        expected_executed_step_count: u32,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            initial_session_id: initial_session_id.into(),
            initial_scenario,
            run,
            expected_final_state_fingerprint,
            expected_run_decision_kind,
            expected_executed_step_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticRunReplayDecisionKind {
    Verified,
    MismatchedEvidence,
}

impl CombatSessionAutomaticRunReplayDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticRunReplayDecisionKind::Verified => "verified",
            CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence => "mismatchedEvidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunReplayReadout {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub accepted: bool,
    pub decision_kind: CombatSessionAutomaticRunReplayDecisionKind,
    pub expected_final_state_fingerprint: StateFingerprint,
    pub actual_final_state_fingerprint: StateFingerprint,
    pub final_state_fingerprint_matches: bool,
    pub expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub actual_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub run_decision_kind_matches: bool,
    pub expected_executed_step_count: u32,
    pub actual_executed_step_count: u32,
    pub executed_step_count_matches: bool,
    pub replayed_run: CombatSessionAutomaticRunReadout,
    pub reason: String,
}

pub fn verify_automatic_run_replay(
    spec: CombatSessionAutomaticRunReplaySpec,
) -> CombatSessionAutomaticRunReplayReadout {
    let mut replay_session =
        CombatSessionState::new(spec.initial_session_id.clone(), spec.initial_scenario);
    let replayed_run = replay_session.run_automatic_combat(spec.run);
    let actual_final_state_fingerprint = replayed_run
        .final_snapshot
        .current_state_fingerprint
        .clone();
    let actual_run_decision_kind = replayed_run.decision_kind;
    let actual_executed_step_count = replayed_run.executed_step_count;

    let final_state_fingerprint_matches =
        actual_final_state_fingerprint == spec.expected_final_state_fingerprint;
    let run_decision_kind_matches = actual_run_decision_kind == spec.expected_run_decision_kind;
    let executed_step_count_matches =
        actual_executed_step_count == spec.expected_executed_step_count;
    let accepted =
        final_state_fingerprint_matches && run_decision_kind_matches && executed_step_count_matches;
    let decision_kind = if accepted {
        CombatSessionAutomaticRunReplayDecisionKind::Verified
    } else {
        CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence
    };
    let reason = if accepted {
        "Automatic run replay verified expected final evidence.".to_string()
    } else {
        "Automatic run replay produced evidence that does not match expected final evidence."
            .to_string()
    };

    CombatSessionAutomaticRunReplayReadout {
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        accepted,
        decision_kind,
        expected_final_state_fingerprint: spec.expected_final_state_fingerprint,
        actual_final_state_fingerprint,
        final_state_fingerprint_matches,
        expected_run_decision_kind: spec.expected_run_decision_kind,
        actual_run_decision_kind,
        run_decision_kind_matches,
        expected_executed_step_count: spec.expected_executed_step_count,
        actual_executed_step_count,
        executed_step_count_matches,
        replayed_run,
        reason,
    }
}
