//! Automatic command selection and execution.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateSelectionSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub action_id: String,
    pub target_id: String,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionCandidateSelectionSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        action_id: impl Into<String>,
        target_id: impl Into<String>,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            action_id: action_id.into(),
            target_id: target_id.into(),
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionCandidateSelectionDecisionKind {
    Accepted,
    RejectedByUnavailableCandidates,
    RejectedByMissingCandidate,
    RejectedByPreflight,
}

impl CombatSessionCandidateSelectionDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionCandidateSelectionDecisionKind::Accepted => "accepted",
            CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates => {
                "rejectedByUnavailableCandidates"
            }
            CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate => {
                "rejectedByMissingCandidate"
            }
            CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight => {
                "rejectedByPreflight"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateSelectionReadout {
    pub action_id: String,
    pub target_id: String,
    pub accepted: bool,
    pub decision_kind: CombatSessionCandidateSelectionDecisionKind,
    pub current_actor_id: Option<String>,
    pub unavailable_reason: Option<CurrentActorOptionsUnavailableReason>,
    pub preflight_decision_kind: Option<CommandPreflightDecisionKind>,
    pub rejection: Option<RulebenchRejection>,
    pub reason: String,
    pub command: Option<CombatSessionIntentCommandSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCandidateExecutionReadout {
    pub selection: CombatSessionCandidateSelectionReadout,
    pub submitted_step: Option<CombatSessionStepReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutoCandidateCommandSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionAutoCandidateCommandSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutoCandidateDecisionKind {
    Accepted,
    RejectedByUnavailableCandidates,
    RejectedByNoAcceptedCandidate,
}

impl CombatSessionAutoCandidateDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutoCandidateDecisionKind::Accepted => "accepted",
            CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates => {
                "rejectedByUnavailableCandidates"
            }
            CombatSessionAutoCandidateDecisionKind::RejectedByNoAcceptedCandidate => {
                "rejectedByNoAcceptedCandidate"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutoCandidatePlanReadout {
    pub accepted: bool,
    pub decision_kind: CombatSessionAutoCandidateDecisionKind,
    pub current_actor_id: Option<String>,
    pub candidate_count: usize,
    pub accepted_candidate_count: usize,
    pub selected_action_id: Option<String>,
    pub selected_target_id: Option<String>,
    pub unavailable_reason: Option<CurrentActorOptionsUnavailableReason>,
    pub reason: String,
    pub selection: Option<CombatSessionCandidateSelectionReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutoCandidateExecutionReadout {
    pub plan: CombatSessionAutoCandidatePlanReadout,
    pub submitted_step: Option<CombatSessionStepReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticStepSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionAutomaticStepSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticStepOperationKind {
    ConditionalEnd,
    SubmitCandidate,
    AdvanceTurn,
}

impl CombatSessionAutomaticStepOperationKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticStepOperationKind::ConditionalEnd => "conditionalEnd",
            CombatSessionAutomaticStepOperationKind::SubmitCandidate => "submitCandidate",
            CombatSessionAutomaticStepOperationKind::AdvanceTurn => "advanceTurn",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticStepDecisionKind {
    ConditionalEnd,
    SubmitCandidate,
    AdvanceTurn,
    RejectedByLifecycle,
}

impl CombatSessionAutomaticStepDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticStepDecisionKind::ConditionalEnd => "conditionalEnd",
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate => "submitCandidate",
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn => "advanceTurn",
            CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticStepPlanReadout {
    pub accepted: bool,
    pub decision_kind: CombatSessionAutomaticStepDecisionKind,
    pub operation_kind: Option<CombatSessionAutomaticStepOperationKind>,
    pub lifecycle_phase: CombatLifecyclePhase,
    pub current_actor_id: Option<String>,
    pub combat_end_condition: CombatEndConditionReadout,
    pub auto_candidate_plan: Option<CombatSessionAutoCandidatePlanReadout>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticStepExecutionReadout {
    pub plan: CombatSessionAutomaticStepPlanReadout,
    pub control: Option<CombatControlReadout>,
    pub auto_candidate: Option<CombatSessionAutoCandidateExecutionReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub max_steps: u32,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionAutomaticRunSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        max_steps: u32,
        roll_stream: Vec<i32>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            max_steps,
            roll_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticRunDecisionKind {
    CompletedCombatEnded,
    StoppedAtMaxSteps,
    RejectedByLifecycle,
    RejectedByStepLimit,
}

impl CombatSessionAutomaticRunDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded => "completedCombatEnded",
            CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps => "stoppedAtMaxSteps",
            CombatSessionAutomaticRunDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CombatSessionAutomaticRunDecisionKind::RejectedByStepLimit => "rejectedByStepLimit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunReadout {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub accepted: bool,
    pub decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub max_steps: u32,
    pub executed_step_count: u32,
    pub steps: Vec<CombatSessionAutomaticStepExecutionReadout>,
    pub final_snapshot: CombatSessionSnapshot,
    pub reason: String,
}

pub(super) fn combat_session_automatic_run_readout(
    id: String,
    title: String,
    summary: String,
    accepted: bool,
    decision_kind: CombatSessionAutomaticRunDecisionKind,
    max_steps: u32,
    steps: Vec<CombatSessionAutomaticStepExecutionReadout>,
    final_snapshot: CombatSessionSnapshot,
    reason: impl Into<String>,
) -> CombatSessionAutomaticRunReadout {
    CombatSessionAutomaticRunReadout {
        id,
        title,
        summary,
        accepted,
        decision_kind,
        max_steps,
        executed_step_count: steps.len() as u32,
        steps,
        final_snapshot,
        reason: reason.into(),
    }
}

pub(super) fn plan_automatic_step(
    lifecycle_phase: CombatLifecyclePhase,
    current_actor_id: Option<String>,
    combat_end_condition: CombatEndConditionReadout,
    auto_candidate_plan: impl FnOnce() -> CombatSessionAutoCandidatePlanReadout,
) -> CombatSessionAutomaticStepPlanReadout {
    if lifecycle_phase == CombatLifecyclePhase::Ended {
        return CombatSessionAutomaticStepPlanReadout {
            accepted: false,
            decision_kind: CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle,
            operation_kind: None,
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            auto_candidate_plan: None,
            reason: "Automatic combat step rejected because combat is already ended.".to_string(),
        };
    }

    if combat_end_condition.combat_should_end {
        return CombatSessionAutomaticStepPlanReadout {
            accepted: true,
            decision_kind: CombatSessionAutomaticStepDecisionKind::ConditionalEnd,
            operation_kind: Some(CombatSessionAutomaticStepOperationKind::ConditionalEnd),
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            auto_candidate_plan: None,
            reason: "Automatic combat step planned conditional combat end.".to_string(),
        };
    }

    let candidate_plan = auto_candidate_plan();
    if candidate_plan.accepted {
        return CombatSessionAutomaticStepPlanReadout {
            accepted: true,
            decision_kind: CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
            operation_kind: Some(CombatSessionAutomaticStepOperationKind::SubmitCandidate),
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            auto_candidate_plan: Some(candidate_plan),
            reason: "Automatic combat step planned first accepted command candidate.".to_string(),
        };
    }

    CombatSessionAutomaticStepPlanReadout {
        accepted: true,
        decision_kind: CombatSessionAutomaticStepDecisionKind::AdvanceTurn,
        operation_kind: Some(CombatSessionAutomaticStepOperationKind::AdvanceTurn),
        lifecycle_phase,
        current_actor_id,
        combat_end_condition,
        auto_candidate_plan: Some(candidate_plan),
        reason: "Automatic combat step planned turn advancement because no accepted command candidate is available."
            .to_string(),
    }
}

pub(super) fn plan_candidate_command(
    spec: CombatSessionCandidateSelectionSpec,
    candidates: CommandCandidateSummary,
) -> CombatSessionCandidateSelectionReadout {
    if !candidates.available {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind:
                CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: candidates.unavailable_reason,
            preflight_decision_kind: None,
            rejection: None,
            reason: candidate_selection_unavailable_reason(candidates.unavailable_reason),
            command: None,
        };
    }

    let Some(candidate) = candidates.candidates.iter().find(|candidate| {
        candidate.action_id == spec.action_id && candidate.target_id == spec.target_id
    }) else {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind: CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: None,
            preflight_decision_kind: None,
            rejection: None,
            reason: "Selected command candidate is not available for the current actor."
                .to_string(),
            command: None,
        };
    };

    if !candidate.accepted {
        return CombatSessionCandidateSelectionReadout {
            action_id: spec.action_id,
            target_id: spec.target_id,
            accepted: false,
            decision_kind: CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight,
            current_actor_id: candidates.current_actor_id,
            unavailable_reason: None,
            preflight_decision_kind: Some(candidate.decision_kind),
            rejection: candidate.rejection,
            reason: candidate.reason.clone(),
            command: None,
        };
    }

    let command = CombatSessionIntentCommandSpec::new(
        spec.id,
        spec.title,
        spec.summary,
        candidate.intent.clone(),
        spec.roll_stream,
    );

    CombatSessionCandidateSelectionReadout {
        action_id: spec.action_id,
        target_id: spec.target_id,
        accepted: true,
        decision_kind: CombatSessionCandidateSelectionDecisionKind::Accepted,
        current_actor_id: candidates.current_actor_id,
        unavailable_reason: None,
        preflight_decision_kind: Some(candidate.decision_kind),
        rejection: None,
        reason: "Selected command candidate planned for deterministic submission.".to_string(),
        command: Some(command),
    }
}

pub(super) fn plan_auto_candidate_command(
    spec: CombatSessionAutoCandidateCommandSpec,
    candidates: CommandCandidateSummary,
) -> CombatSessionAutoCandidatePlanReadout {
    let candidate_count = candidates.candidates.len();
    let accepted_candidate_count = candidates
        .candidates
        .iter()
        .filter(|candidate| candidate.accepted)
        .count();

    if !candidates.available {
        return CombatSessionAutoCandidatePlanReadout {
            accepted: false,
            decision_kind: CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates,
            current_actor_id: candidates.current_actor_id,
            candidate_count,
            accepted_candidate_count,
            selected_action_id: None,
            selected_target_id: None,
            unavailable_reason: candidates.unavailable_reason,
            reason: candidate_selection_unavailable_reason(candidates.unavailable_reason),
            selection: None,
        };
    }

    let Some(candidate) = candidates
        .candidates
        .iter()
        .find(|candidate| candidate.accepted)
        .cloned()
    else {
        return CombatSessionAutoCandidatePlanReadout {
            accepted: false,
            decision_kind: CombatSessionAutoCandidateDecisionKind::RejectedByNoAcceptedCandidate,
            current_actor_id: candidates.current_actor_id,
            candidate_count,
            accepted_candidate_count,
            selected_action_id: None,
            selected_target_id: None,
            unavailable_reason: None,
            reason:
                "No accepted command candidates are available for deterministic auto submission."
                    .to_string(),
            selection: None,
        };
    };

    let selected_action_id = candidate.action_id.clone();
    let selected_target_id = candidate.target_id.clone();
    let selection = plan_candidate_command(
        CombatSessionCandidateSelectionSpec::new(
            spec.id,
            spec.title,
            spec.summary,
            candidate.action_id,
            candidate.target_id,
            spec.roll_stream,
        ),
        candidates,
    );

    CombatSessionAutoCandidatePlanReadout {
        accepted: selection.accepted,
        decision_kind: CombatSessionAutoCandidateDecisionKind::Accepted,
        current_actor_id: selection.current_actor_id.clone(),
        candidate_count,
        accepted_candidate_count,
        selected_action_id: Some(selected_action_id),
        selected_target_id: Some(selected_target_id),
        unavailable_reason: None,
        reason: "First accepted command candidate planned for deterministic auto submission."
            .to_string(),
        selection: Some(selection),
    }
}

fn candidate_selection_unavailable_reason(
    reason: Option<CurrentActorOptionsUnavailableReason>,
) -> String {
    match reason {
        Some(CurrentActorOptionsUnavailableReason::CombatEnded) => {
            "No command candidates are available because combat is ended."
        }
        Some(CurrentActorOptionsUnavailableReason::NoCurrentActor) => {
            "No command candidates are available because there is no current actor."
        }
        Some(CurrentActorOptionsUnavailableReason::CurrentActorDefeated) => {
            "No command candidates are available because the current actor is defeated."
        }
        Some(CurrentActorOptionsUnavailableReason::NoMatchingActions) => {
            "No command candidates are available because the current actor has no matching actions."
        }
        Some(CurrentActorOptionsUnavailableReason::NoAvailableResources) => {
            "No command candidates are available because the current actor cannot cover any action resource costs."
        }
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets) => {
            "No command candidates are available because there are no visible active targets."
        }
        None => "No command candidates are available.",
    }
    .to_string()
}
