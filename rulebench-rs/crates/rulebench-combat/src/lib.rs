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
    fingerprint_action_resource_ledger, fingerprint_projected_state, fingerprint_projection,
    ACTION_RESOURCE_FINGERPRINT_ALGORITHM, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use modifiers::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    evaluate_effective_stats_for_combatant, EffectiveStatEvaluationError,
};
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{
    validate_combat_automation_policy, validate_combat_automation_policy_for_context,
    CombatAutomationCandidateEvidence, CombatAutomationNoCandidateBehavior,
    CombatAutomationPolicyContext, CombatAutomationPolicyDecisionEvidence,
    CombatAutomationPolicyRegistration, CombatAutomationPolicyRequirement,
    CombatAutomationPolicySelector, CombatAutomationPolicySpec,
    CombatAutomationPolicyValidationCode, CombatAutomationPolicyValidationReadout,
    CombatSessionAutoCandidateCommandSpec, CombatSessionAutoCandidateDecisionKind,
    CombatSessionAutoCandidateExecutionReadout, CombatSessionAutoCandidatePlanReadout,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepOperationKind,
    CombatSessionAutomaticStepPlanReadout, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
    CombatSessionCommandSpec, CombatSessionIntentCommandSpec, CombatSessionScriptCommandKind,
    CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
    CombatSessionState, COMBAT_AUTOMATION_POLICY_REGISTRY, FIRST_ACCEPTED_CANDIDATE_POLICY_ID,
    FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION, LOWEST_VITALITY_TARGET_POLICY_ID,
    LOWEST_VITALITY_TARGET_POLICY_VERSION, OBJECTIVE_SIDE_PRESSURE_POLICY_ID,
    OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION,
};
pub use state::CombatState;

pub use rpg_runtime::GOVERNED_ASHA_REVISION;

pub const RUNTIME_EFFECT_OPERATION_REGISTRY: &[model::EffectOperationId] = &[
    model::EffectOperationId::Damage,
    model::EffectOperationId::Heal,
    model::EffectOperationId::GrantTemporaryVitality,
    model::EffectOperationId::ApplyModifier,
    model::EffectOperationId::Move,
    model::EffectOperationId::ChangeResource,
    model::EffectOperationId::OpenReactionWindow,
];

pub const RUNTIME_TARGETING_OPERATION_REGISTRY: &[model::TargetingOperationId] = &[
    model::TargetingOperationId::SingleCombatant,
    model::TargetingOperationId::MultipleCombatants,
    model::TargetingOperationId::ManhattanBurstArea,
    model::TargetingOperationId::CellMovement,
];
