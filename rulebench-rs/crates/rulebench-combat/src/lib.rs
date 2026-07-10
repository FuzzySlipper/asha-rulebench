//! Authoritative combat state, resolution, and control runtime.

#![forbid(unsafe_code)]

pub mod model;

pub use model::*;

mod api;
mod audit;
mod modifiers;
mod resolver;
mod runtime;
mod state;

pub use api::{
    CombatSessionApi, CombatSessionApiError, CombatSessionArchive, CombatSessionCreateReadout,
    CombatSessionCreateRequest, CombatSessionHandle,
};
pub use audit::{
    fingerprint_projected_state, fingerprint_projection, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use modifiers::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    evaluate_effective_stats_for_combatant, EffectiveStatEvaluationError,
};
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{
    validate_combat_automation_policy, CombatAutomationCandidateEvidence,
    CombatAutomationNoCandidateBehavior, CombatAutomationPolicyDecisionEvidence,
    CombatAutomationPolicySpec, CombatAutomationPolicyValidationCode,
    CombatAutomationPolicyValidationReadout, CombatSessionAutoCandidateCommandSpec,
    CombatSessionAutoCandidateDecisionKind, CombatSessionAutoCandidateExecutionReadout,
    CombatSessionAutoCandidatePlanReadout, CombatSessionAutomaticRunDecisionKind,
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticRunSpec,
    CombatSessionAutomaticStepDecisionKind, CombatSessionAutomaticStepExecutionReadout,
    CombatSessionAutomaticStepOperationKind, CombatSessionAutomaticStepPlanReadout,
    CombatSessionAutomaticStepSpec, CombatSessionCandidateExecutionReadout,
    CombatSessionCandidateSelectionDecisionKind, CombatSessionCandidateSelectionReadout,
    CombatSessionCandidateSelectionSpec, CombatSessionCommandSpec, CombatSessionIntentCommandSpec,
    CombatSessionScriptCommandKind, CombatSessionScriptCommandSpec,
    CombatSessionScriptDecisionKind, CombatSessionScriptReadout, CombatSessionScriptSpec,
    CombatSessionScriptStepReadout, CombatSessionScriptStepSpec, CombatSessionState,
    FIRST_ACCEPTED_CANDIDATE_POLICY_ID, FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION,
};
pub use state::CombatState;
