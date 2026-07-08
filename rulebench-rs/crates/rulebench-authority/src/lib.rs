//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! rejections fail closed, accepted facts are represented as DomainEvent-shaped
//! records, and trace/readout values explain what happened. It does not claim to
//! be upstream ASHA or a complete combat resolver.

#![forbid(unsafe_code)]

mod audit;
mod catalog;
mod content;
mod fixtures;
mod model;
mod modifiers;
mod resolver;
mod runtime;
mod session;
mod state;

pub use audit::{
    fingerprint_projected_state, fingerprint_projection, PROJECTION_FINGERPRINT_ALGORITHM,
    STATE_FINGERPRINT_ALGORITHM,
};
pub use catalog::{
    content_validation_readouts, resolve_catalog_scenario, ruleset_catalog_readout,
    scenario_catalog_cases, scenario_catalog_summaries,
};
pub use content::{validate_scenario_content, validate_scenario_content_report};
pub use fixtures::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    rejected_target_fixture_receipt,
};
pub use model::*;
pub use modifiers::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
};
pub use resolver::{resolve_use_action, validate_intent_shape};
pub use runtime::{
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
    CombatSessionState,
};
pub use session::{
    combat_session_automatic_run_readouts, combat_session_control_history_readouts,
    combat_session_script_readouts, combat_session_summaries, combat_session_transcripts,
    resolve_combat_session_step,
};

#[cfg(test)]
mod tests;
