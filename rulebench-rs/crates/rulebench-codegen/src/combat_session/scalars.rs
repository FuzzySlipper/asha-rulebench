use crate::ts_emit::ts_string;

use rpg_ir::{ActionResourceKind, CombatEndPolicy, ModifierTenure, ReactionWindow};
use rulebench_combat::{
    ActionResourceTransitionKind, CombatAutomationNoCandidateBehavior,
    CombatAutomationPolicyValidationCode, CombatControlCommandKind, CombatControlDecisionKind,
    CombatEndConditionKind, CombatLifecyclePhase, CombatOutcomeKind,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepOperationKind, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionScriptCommandKind, CombatSessionScriptDecisionKind, CommandDecisionKind,
    CommandOutcomeClass, CommandPreflightDecisionKind, LifecycleTransitionTrigger,
    ReactionDecisionKind, ReactionResponseKind, ReactionWindowLifecycleKind, ReactionWindowStatus,
    StateFingerprint, TracePhase, TraceStatus,
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
        .map(|kind| ts_string(automatic_step_operation_kind(kind)))
        .unwrap_or_else(|| "null".to_string())
}

pub(crate) fn automatic_no_candidate_behavior(
    behavior: CombatAutomationNoCandidateBehavior,
) -> &'static str {
    match behavior {
        CombatAutomationNoCandidateBehavior::AdvanceTurn => "advanceTurn",
        CombatAutomationNoCandidateBehavior::StopRun => "stopRun",
    }
}

pub(crate) fn automation_policy_validation_code(
    code: CombatAutomationPolicyValidationCode,
) -> &'static str {
    match code {
        CombatAutomationPolicyValidationCode::Accepted => "accepted",
        CombatAutomationPolicyValidationCode::UnsupportedPolicyId => "unsupportedPolicyId",
        CombatAutomationPolicyValidationCode::UnsupportedPolicyVersion => {
            "unsupportedPolicyVersion"
        }
        CombatAutomationPolicyValidationCode::IncompatibleRulesetCapability => {
            "incompatibleRulesetCapability"
        }
    }
}

pub(crate) fn automatic_step_operation_kind(
    kind: CombatSessionAutomaticStepOperationKind,
) -> &'static str {
    match kind {
        CombatSessionAutomaticStepOperationKind::ConditionalEnd => "conditionalEnd",
        CombatSessionAutomaticStepOperationKind::SubmitCandidate => "submitCandidate",
        CombatSessionAutomaticStepOperationKind::AdvanceTurn => "advanceTurn",
    }
}

pub(crate) fn automatic_step_decision_kind(
    kind: CombatSessionAutomaticStepDecisionKind,
) -> &'static str {
    match kind {
        CombatSessionAutomaticStepDecisionKind::ConditionalEnd => "conditionalEnd",
        CombatSessionAutomaticStepDecisionKind::SubmitCandidate => "submitCandidate",
        CombatSessionAutomaticStepDecisionKind::AdvanceTurn => "advanceTurn",
        CombatSessionAutomaticStepDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
        CombatSessionAutomaticStepDecisionKind::RejectedByPolicy => "rejectedByPolicy",
        CombatSessionAutomaticStepDecisionKind::StoppedNoCandidate => "stoppedNoCandidate",
        CombatSessionAutomaticStepDecisionKind::StoppedReactionWindow => "stoppedReactionWindow",
    }
}

pub(crate) fn automatic_run_decision_kind(
    kind: CombatSessionAutomaticRunDecisionKind,
) -> &'static str {
    match kind {
        CombatSessionAutomaticRunDecisionKind::CompletedCombatEnded => "completedCombatEnded",
        CombatSessionAutomaticRunDecisionKind::StoppedAtMaxSteps => "stoppedAtMaxSteps",
        CombatSessionAutomaticRunDecisionKind::RejectedByLifecycle => "rejectedByLifecycle",
        CombatSessionAutomaticRunDecisionKind::RejectedByStepLimit => "rejectedByStepLimit",
        CombatSessionAutomaticRunDecisionKind::RejectedByPolicy => "rejectedByPolicy",
        CombatSessionAutomaticRunDecisionKind::StoppedNoCandidate => "stoppedNoCandidate",
        CombatSessionAutomaticRunDecisionKind::StoppedReactionWindow => "stoppedReactionWindow",
    }
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
        CombatControlDecisionKind::RejectedByReactionWindow => "rejectedByReactionWindow",
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
    kind: CombatSessionCandidateSelectionDecisionKind,
) -> &'static str {
    match kind {
        CombatSessionCandidateSelectionDecisionKind::Accepted => "accepted",
        CombatSessionCandidateSelectionDecisionKind::RejectedByUnavailableCandidates => {
            "rejectedByUnavailableCandidates"
        }
        CombatSessionCandidateSelectionDecisionKind::RejectedByMissingCandidate => {
            "rejectedByMissingCandidate"
        }
        CombatSessionCandidateSelectionDecisionKind::RejectedByPreflight => "rejectedByPreflight",
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

pub(crate) fn reaction_window_timing(timing: ReactionWindow) -> &'static str {
    match timing {
        ReactionWindow::BeforeEffect => "beforeEffect",
        ReactionWindow::AfterEffect => "afterEffect",
    }
}

pub(crate) fn reaction_window_status(status: ReactionWindowStatus) -> &'static str {
    match status {
        ReactionWindowStatus::Open => "open",
        ReactionWindowStatus::Resolved => "resolved",
    }
}

pub(crate) fn reaction_response_kind(kind: ReactionResponseKind) -> &'static str {
    match kind {
        ReactionResponseKind::Pass => "pass",
        ReactionResponseKind::Accept => "accept",
    }
}

pub(crate) fn reaction_decision_kind(kind: ReactionDecisionKind) -> &'static str {
    match kind {
        ReactionDecisionKind::Accepted => "accepted",
        ReactionDecisionKind::RejectedNoOpenWindow => "rejectedNoOpenWindow",
        ReactionDecisionKind::RejectedStaleWindow => "rejectedStaleWindow",
        ReactionDecisionKind::RejectedOutOfOrder => "rejectedOutOfOrder",
        ReactionDecisionKind::RejectedInvalidOption => "rejectedInvalidOption",
        ReactionDecisionKind::RejectedNestedLimit => "rejectedNestedLimit",
    }
}

pub(crate) fn reaction_window_lifecycle_kind(kind: ReactionWindowLifecycleKind) -> &'static str {
    match kind {
        ReactionWindowLifecycleKind::Opened => "opened",
        ReactionWindowLifecycleKind::NestedOpened => "nestedOpened",
        ReactionWindowLifecycleKind::ResponseAccepted => "responseAccepted",
        ReactionWindowLifecycleKind::Resolved => "resolved",
        ReactionWindowLifecycleKind::ResolutionResumed => "resolutionResumed",
    }
}

pub(crate) fn trace_phase(phase: TracePhase) -> &'static str {
    match phase {
        TracePhase::Proposal => "proposal",
        TracePhase::Validation => "validation",
        TracePhase::Resolution => "resolution",
        TracePhase::Commit => "commit",
    }
}

pub(crate) fn trace_status(status: TraceStatus) -> &'static str {
    match status {
        TraceStatus::Accepted => "accepted",
        TraceStatus::Rejected => "rejected",
        TraceStatus::Info => "info",
    }
}

pub(crate) fn combat_end_condition_kind(kind: CombatEndConditionKind) -> &'static str {
    match kind {
        CombatEndConditionKind::Ongoing => "ongoing",
        CombatEndConditionKind::NoActiveEnemies => "noActiveEnemies",
        CombatEndConditionKind::NoActiveAllies => "noActiveAllies",
        CombatEndConditionKind::NoActiveCombatants => "noActiveCombatants",
        CombatEndConditionKind::ExplicitOnly => "explicitOnly",
        CombatEndConditionKind::ExplicitEnd => "explicitEnd",
        CombatEndConditionKind::LastSideStanding => "lastSideStanding",
        CombatEndConditionKind::ObjectiveSideVictory => "objectiveSideVictory",
        CombatEndConditionKind::ObjectiveSideDefeated => "objectiveSideDefeated",
    }
}

pub(crate) fn combat_end_policy_kind(policy: &CombatEndPolicy) -> &'static str {
    match policy {
        CombatEndPolicy::LastSideStanding => "lastSideStanding",
        CombatEndPolicy::ObjectiveSideVictory { .. } => "objectiveSideVictory",
        CombatEndPolicy::ExplicitOnly => "explicitOnly",
    }
}

pub(crate) fn combat_outcome_kind(kind: CombatOutcomeKind) -> &'static str {
    match kind {
        CombatOutcomeKind::Ongoing => "ongoing",
        CombatOutcomeKind::Victory => "victory",
        CombatOutcomeKind::Defeat => "defeat",
        CombatOutcomeKind::Draw => "draw",
        CombatOutcomeKind::ExplicitEnd => "explicitEnd",
    }
}

pub(crate) fn action_resource_kind(kind: ActionResourceKind) -> &'static str {
    match kind {
        ActionResourceKind::StandardAction => "standardAction",
        ActionResourceKind::SpellSlot => "spellSlot",
        ActionResourceKind::Charge => "charge",
        ActionResourceKind::Cooldown => "cooldown",
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
