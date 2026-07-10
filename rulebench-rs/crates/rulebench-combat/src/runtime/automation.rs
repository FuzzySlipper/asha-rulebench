//! Automatic command selection and execution.

use super::*;

mod policy;
pub use policy::*;

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
    pub policy: CombatAutomationPolicySpec,
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
            policy: CombatAutomationPolicySpec::first_accepted_candidate(),
        }
    }

    pub fn with_policy(mut self, policy: CombatAutomationPolicySpec) -> Self {
        self.policy = policy;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutoCandidateDecisionKind {
    Accepted,
    RejectedByUnavailableCandidates,
    RejectedByNoAcceptedCandidate,
    RejectedByPolicy,
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
            CombatSessionAutoCandidateDecisionKind::RejectedByPolicy => "rejectedByPolicy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutoCandidatePlanReadout {
    pub policy: CombatAutomationPolicySpec,
    pub policy_validation: CombatAutomationPolicyValidationReadout,
    pub accepted: bool,
    pub decision_kind: CombatSessionAutoCandidateDecisionKind,
    pub current_actor_id: Option<String>,
    pub candidate_count: usize,
    pub accepted_candidate_count: usize,
    pub selected_action_id: Option<String>,
    pub selected_target_id: Option<String>,
    pub selected_candidate_index: Option<usize>,
    pub candidate_order: Vec<CombatAutomationCandidateEvidence>,
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
    pub policy: CombatAutomationPolicySpec,
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
            policy: CombatAutomationPolicySpec::first_accepted_candidate(),
        }
    }

    pub fn with_policy(mut self, policy: CombatAutomationPolicySpec) -> Self {
        self.policy = policy;
        self
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
    RejectedByPolicy,
    StoppedNoCandidate,
}

impl CombatSessionAutomaticStepDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticStepDecisionKind::ConditionalEnd => "conditionalEnd",
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate => "submitCandidate",
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn => "advanceTurn",
            CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CombatSessionAutomaticStepDecisionKind::RejectedByPolicy => "rejectedByPolicy",
            CombatSessionAutomaticStepDecisionKind::StoppedNoCandidate => "stoppedNoCandidate",
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
    pub policy_validation: CombatAutomationPolicyValidationReadout,
    pub policy_decision: CombatAutomationPolicyDecisionEvidence,
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
    pub policy: CombatAutomationPolicySpec,
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
            policy: CombatAutomationPolicySpec::first_accepted_candidate(),
        }
    }

    pub fn with_policy(mut self, policy: CombatAutomationPolicySpec) -> Self {
        self.policy = policy;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticRunDecisionKind {
    CompletedCombatEnded,
    StoppedAtMaxSteps,
    RejectedByLifecycle,
    RejectedByStepLimit,
    RejectedByPolicy,
    StoppedNoCandidate,
}

impl CombatSessionAutomaticRunDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded => "completedCombatEnded",
            CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps => "stoppedAtMaxSteps",
            CombatSessionAutomaticRunDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
            CombatSessionAutomaticRunDecisionKind::RejectedByStepLimit => "rejectedByStepLimit",
            CombatSessionAutomaticRunDecisionKind::RejectedByPolicy => "rejectedByPolicy",
            CombatSessionAutomaticRunDecisionKind::StoppedNoCandidate => "stoppedNoCandidate",
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
    pub policy: CombatAutomationPolicySpec,
    pub executed_step_count: u32,
    pub steps: Vec<CombatSessionAutomaticStepExecutionReadout>,
    pub policy_decisions: Vec<CombatAutomationPolicyDecisionEvidence>,
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
    policy: CombatAutomationPolicySpec,
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
        policy,
        executed_step_count: steps.len() as u32,
        policy_decisions: steps
            .iter()
            .map(|step| step.plan.policy_decision.clone())
            .collect(),
        steps,
        final_snapshot,
        reason: reason.into(),
    }
}

pub(super) fn plan_automatic_step(
    policy: CombatAutomationPolicySpec,
    state_before_fingerprint: StateFingerprint,
    lifecycle_phase: CombatLifecyclePhase,
    current_actor_id: Option<String>,
    combat_end_condition: CombatEndConditionReadout,
    auto_candidate_plan: impl FnOnce() -> CombatSessionAutoCandidatePlanReadout,
) -> CombatSessionAutomaticStepPlanReadout {
    let policy_validation = validate_combat_automation_policy(&policy);
    if !policy_validation.accepted {
        let reason = policy_validation.reason.clone();
        return automatic_step_plan_readout(
            policy,
            state_before_fingerprint,
            false,
            CombatSessionAutomaticStepDecisionKind::RejectedByPolicy,
            None,
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            None,
            policy_validation,
            reason,
        );
    }
    if lifecycle_phase == CombatLifecyclePhase::Ended {
        return automatic_step_plan_readout(
            policy,
            state_before_fingerprint,
            false,
            CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle,
            None,
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            None,
            policy_validation,
            "Automatic combat step rejected because combat is already ended.",
        );
    }

    if combat_end_condition.combat_should_end {
        return automatic_step_plan_readout(
            policy,
            state_before_fingerprint,
            true,
            CombatSessionAutomaticStepDecisionKind::ConditionalEnd,
            Some(CombatSessionAutomaticStepOperationKind::ConditionalEnd),
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            None,
            policy_validation,
            "Automatic combat step planned conditional combat end.",
        );
    }

    let candidate_plan = auto_candidate_plan();
    if candidate_plan.accepted {
        return automatic_step_plan_readout(
            policy,
            state_before_fingerprint,
            true,
            CombatSessionAutomaticStepDecisionKind::SubmitCandidate,
            Some(CombatSessionAutomaticStepOperationKind::SubmitCandidate),
            lifecycle_phase,
            current_actor_id,
            combat_end_condition,
            Some(candidate_plan),
            policy_validation,
            "Automatic combat step planned first accepted command candidate.",
        );
    }

    let stop = policy.no_candidate_behavior == CombatAutomationNoCandidateBehavior::StopRun;
    automatic_step_plan_readout(
        policy,
        state_before_fingerprint,
        true,
        if stop {
            CombatSessionAutomaticStepDecisionKind::StoppedNoCandidate
        } else {
            CombatSessionAutomaticStepDecisionKind::AdvanceTurn
        },
        if stop {
            None
        } else {
            Some(CombatSessionAutomaticStepOperationKind::AdvanceTurn)
        },
        lifecycle_phase,
        current_actor_id,
        combat_end_condition,
        Some(candidate_plan),
        policy_validation,
        if stop {
            "Automatic combat step stopped because policy found no accepted command candidate."
        } else {
            "Automatic combat step planned turn advancement because no accepted command candidate is available."
        },
    )
}

fn automatic_step_plan_readout(
    policy: CombatAutomationPolicySpec,
    state_before_fingerprint: StateFingerprint,
    accepted: bool,
    decision_kind: CombatSessionAutomaticStepDecisionKind,
    operation_kind: Option<CombatSessionAutomaticStepOperationKind>,
    lifecycle_phase: CombatLifecyclePhase,
    current_actor_id: Option<String>,
    combat_end_condition: CombatEndConditionReadout,
    auto_candidate_plan: Option<CombatSessionAutoCandidatePlanReadout>,
    policy_validation: CombatAutomationPolicyValidationReadout,
    reason: impl Into<String>,
) -> CombatSessionAutomaticStepPlanReadout {
    let reason = reason.into();
    let (
        candidate_count,
        accepted_candidate_count,
        selected_action_id,
        selected_target_id,
        selected_candidate_index,
        candidates,
    ) = auto_candidate_plan
        .as_ref()
        .map_or((0, 0, None, None, None, Vec::new()), |plan| {
            (
                plan.candidate_count,
                plan.accepted_candidate_count,
                plan.selected_action_id.clone(),
                plan.selected_target_id.clone(),
                plan.selected_candidate_index,
                plan.candidate_order.clone(),
            )
        });
    CombatSessionAutomaticStepPlanReadout {
        accepted,
        decision_kind,
        operation_kind,
        lifecycle_phase,
        current_actor_id,
        combat_end_condition,
        auto_candidate_plan,
        policy_validation,
        policy_decision: CombatAutomationPolicyDecisionEvidence {
            policy,
            state_before_fingerprint,
            operation_kind,
            selected_action_id,
            selected_target_id,
            selected_candidate_index,
            candidate_count,
            accepted_candidate_count,
            candidates,
            reason: reason.clone(),
        },
        reason,
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
    let policy = spec.policy.clone();
    let policy_validation = validate_combat_automation_policy(&policy);
    let candidate_count = candidates.candidates.len();
    let accepted_candidate_count = candidates
        .candidates
        .iter()
        .filter(|candidate| candidate.accepted)
        .count();
    let candidate_order = candidates
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| CombatAutomationCandidateEvidence {
            index,
            action_id: candidate.action_id.clone(),
            target_id: candidate.target_id.clone(),
            accepted: candidate.accepted,
            decision_kind: candidate.decision_kind,
        })
        .collect::<Vec<_>>();

    if !policy_validation.accepted {
        return CombatSessionAutoCandidatePlanReadout {
            policy,
            policy_validation: policy_validation.clone(),
            accepted: false,
            decision_kind: CombatSessionAutoCandidateDecisionKind::RejectedByPolicy,
            current_actor_id: candidates.current_actor_id,
            candidate_count,
            accepted_candidate_count,
            selected_action_id: None,
            selected_target_id: None,
            selected_candidate_index: None,
            candidate_order,
            unavailable_reason: None,
            reason: policy_validation.reason,
            selection: None,
        };
    }

    if !candidates.available {
        return CombatSessionAutoCandidatePlanReadout {
            policy,
            policy_validation,
            accepted: false,
            decision_kind: CombatSessionAutoCandidateDecisionKind::RejectedByUnavailableCandidates,
            current_actor_id: candidates.current_actor_id,
            candidate_count,
            accepted_candidate_count,
            selected_action_id: None,
            selected_target_id: None,
            selected_candidate_index: None,
            candidate_order,
            unavailable_reason: candidates.unavailable_reason,
            reason: candidate_selection_unavailable_reason(candidates.unavailable_reason),
            selection: None,
        };
    }

    let Some((selected_candidate_index, candidate)) = candidates
        .candidates
        .iter()
        .enumerate()
        .find(|(_, candidate)| candidate.accepted)
        .map(|(index, candidate)| (index, candidate.clone()))
    else {
        return CombatSessionAutoCandidatePlanReadout {
            policy,
            policy_validation,
            accepted: false,
            decision_kind: CombatSessionAutoCandidateDecisionKind::RejectedByNoAcceptedCandidate,
            current_actor_id: candidates.current_actor_id,
            candidate_count,
            accepted_candidate_count,
            selected_action_id: None,
            selected_target_id: None,
            selected_candidate_index: None,
            candidate_order,
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
        policy,
        policy_validation,
        accepted: selection.accepted,
        decision_kind: CombatSessionAutoCandidateDecisionKind::Accepted,
        current_actor_id: selection.current_actor_id.clone(),
        candidate_count,
        accepted_candidate_count,
        selected_action_id: Some(selected_action_id),
        selected_target_id: Some(selected_target_id),
        selected_candidate_index: Some(selected_candidate_index),
        candidate_order,
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
        Some(CurrentActorOptionsUnavailableReason::ReactionWindowOpen) => {
            "No command candidates are available until the open reaction window resolves."
        }
        Some(CurrentActorOptionsUnavailableReason::NoVisibleActiveTargets) => {
            "No command candidates are available because there are no visible active targets."
        }
        None => "No command candidates are available.",
    }
    .to_string()
}
