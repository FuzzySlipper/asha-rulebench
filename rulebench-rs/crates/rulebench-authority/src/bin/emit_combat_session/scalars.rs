use crate::ts_emit::ts_string;

use rulebench_authority::{
    ActionResourceKind, ActionResourceTransitionKind, CombatControlCommandKind,
    CombatControlDecisionKind, CombatEndConditionKind, CombatLifecyclePhase,
    CombatSessionAutomaticStepOperationKind, CombatSessionScriptCommandKind,
    CombatSessionScriptDecisionKind, CommandDecisionKind, CommandOutcomeClass,
    CommandPreflightDecisionKind, LifecycleTransitionTrigger, ModifierTenure, StateFingerprint,
};

pub(crate) fn render_fingerprint(fingerprint: &StateFingerprint, _indent: &str) -> String {
    format!(
        "{{ algorithm: {}, value: {} }}",
        ts_string(&fingerprint.algorithm),
        ts_string(&fingerprint.value)
    )
}

pub(crate) fn render_optional_string(value: &Option<String>) -> String {
    value
        .as_ref()
        .map(|inner| ts_string(inner))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_optional_u32(value: Option<u32>) -> String {
    value
        .map(|inner| inner.to_string())
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_optional_i32(value: Option<i32>) -> String {
    value
        .map(|inner| inner.to_string())
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn render_optional_automatic_step_operation_kind(
    value: Option<CombatSessionAutomaticStepOperationKind>,
) -> String {
    value
        .map(|kind| ts_string(kind.code()))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn control_command_kind(kind: CombatControlCommandKind) -> &'static str {
    match kind {
        CombatControlCommandKind::ExplicitStart => "explicitStart",
        CombatControlCommandKind::ExplicitEnd => "explicitEnd",
        CombatControlCommandKind::AdvanceTurn => "advanceTurn",
        CombatControlCommandKind::EndIfConditionMet => "endIfConditionMet",
    }
}

pub(crate) fn control_decision_kind(kind: CombatControlDecisionKind) -> &'static str {
    match kind {
        CombatControlDecisionKind::Accepted => "accepted",
        CombatControlDecisionKind::RejectedNoop => "rejectedNoop",
        CombatControlDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
        CombatControlDecisionKind::RejectedByEmptyTurnOrder => "rejectedByEmptyTurnOrder",
        CombatControlDecisionKind::RejectedByEndCondition => "rejectedByEndCondition",
    }
}

pub(crate) fn script_command_kind(kind: CombatSessionScriptCommandKind) -> &'static str {
    match kind {
        CombatSessionScriptCommandKind::Intent => "intent",
        CombatSessionScriptCommandKind::Control => "control",
        CombatSessionScriptCommandKind::SelectedCandidate => "selectedCandidate",
    }
}

pub(crate) fn script_decision_kind(kind: CombatSessionScriptDecisionKind) -> &'static str {
    match kind {
        CombatSessionScriptDecisionKind::Intent(decision_kind) => {
            command_decision_kind(decision_kind)
        }
        CombatSessionScriptDecisionKind::Control(decision_kind) => {
            control_decision_kind(decision_kind)
        }
        CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(decision_kind) => {
            command_decision_kind(decision_kind)
        }
        CombatSessionScriptDecisionKind::SelectedCandidateSelection(decision_kind) => {
            candidate_selection_decision_kind(decision_kind)
        }
    }
}

pub(crate) fn candidate_selection_decision_kind(
    kind: rulebench_authority::CombatSessionCandidateSelectionDecisionKind,
) -> &'static str {
    match kind {
        rulebench_authority::CombatSessionCandidateSelectionDecisionKind::Accepted => "accepted",
        rulebench_authority::CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates => {
            "rejectedByUnavailableCandidates"
        }
        rulebench_authority::CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate => {
            "rejectedByMissingCandidate"
        }
        rulebench_authority::CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight => {
            "rejectedByPreflight"
        }
    }
}

pub(crate) fn command_decision_kind(kind: CommandDecisionKind) -> &'static str {
    match kind {
        CommandDecisionKind::AcceptedByResolver => "acceptedByResolver",
        CommandDecisionKind::RejectedByResolver => "rejectedByResolver",
        CommandDecisionKind::RejectedByPreflight => "rejectedByPreflight",
        CommandDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
        CommandDecisionKind::RejectedByTurnOrder => "rejectedByTurnOrder",
    }
}

pub(crate) fn preflight_decision_kind(kind: CommandPreflightDecisionKind) -> &'static str {
    kind.code()
}

pub(crate) fn lifecycle_phase(phase: CombatLifecyclePhase) -> &'static str {
    match phase {
        CombatLifecyclePhase::Ready => "ready",
        CombatLifecyclePhase::InProgress => "inProgress",
        CombatLifecyclePhase::Ended => "ended",
    }
}

pub(crate) fn lifecycle_transition_trigger(trigger: LifecycleTransitionTrigger) -> &'static str {
    trigger.code()
}

pub(crate) fn modifier_tenure(tenure: ModifierTenure) -> &'static str {
    tenure.code()
}

pub(crate) fn combat_end_condition_kind(kind: CombatEndConditionKind) -> &'static str {
    kind.code()
}

pub(crate) fn action_resource_kind(kind: ActionResourceKind) -> &'static str {
    match kind {
        ActionResourceKind::StandardAction => "standardAction",
    }
}

pub(crate) fn action_resource_transition_kind(kind: ActionResourceTransitionKind) -> &'static str {
    kind.code()
}

pub(crate) fn outcome_class(outcome_class: CommandOutcomeClass) -> &'static str {
    match outcome_class {
        CommandOutcomeClass::AcceptedHit => "acceptedHit",
        CommandOutcomeClass::AcceptedMiss => "acceptedMiss",
        CommandOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        CommandOutcomeClass::RejectedInvalidCommand => "rejectedInvalidCommand",
    }
}
