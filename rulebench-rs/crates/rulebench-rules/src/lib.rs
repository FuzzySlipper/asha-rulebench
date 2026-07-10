//! Reusable rule model, state, resolution, and runtime substrate for ASHA RuleBench.
//!
//! This crate is intentionally free of RuleBench artifact emitters, canned
//! transcript catalogs, and UI-facing fixture generation. It owns the portable
//! Rust rule authority that a game or test bench can both call through.

#![forbid(unsafe_code)]

pub mod audit;
pub mod content;
pub mod model;
pub mod modifiers;
pub mod resolver;
pub mod runtime;
pub mod state;

pub use audit::{
    fingerprint_projected_state, fingerprint_projection, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use rulebench_combat::{
    CombatSessionApi, CombatSessionApiError, CombatSessionArchive, CombatSessionCreateReadout,
    CombatSessionCreateRequest, CombatSessionHandle,
};
pub use content::{validate_scenario_content, validate_scenario_content_report};
pub use model::*;
pub use modifiers::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
};
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{
    verify_automatic_run_replay, CombatSessionAutoCandidateCommandSpec,
    CombatSessionAutoCandidateDecisionKind, CombatSessionAutoCandidateExecutionReadout,
    CombatSessionAutoCandidatePlanReadout, CombatSessionAutomaticRunDecisionKind,
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticRunReplayDecisionKind,
    CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepOperationKind,
    CombatSessionAutomaticStepPlanReadout, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
    CombatSessionCommandSpec, CombatSessionIntentCommandSpec, CombatSessionScriptCommandKind,
    CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
    CombatSessionState,
};
