//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! rejections fail closed, accepted facts are represented as DomainEvent-shaped
//! records, and trace/readout values explain what happened. The reusable rule
//! substrate lives in `rulebench-rules`; this crate keeps Rulebench-local
//! fixtures, transcript catalogs, emitters, and compatibility exports together.

#![forbid(unsafe_code)]

mod catalog;
mod catalog_types;
mod fixtures;
mod scenarios;
mod session;

pub mod audit {
    pub use rulebench_rules::{
        fingerprint_projected_state, fingerprint_projection, PROJECTION_FINGERPRINT_ALGORITHM,
        STATE_FINGERPRINT_ALGORITHM,
    };
}

pub mod content {
    pub use rulebench_rules::{validate_scenario_content, validate_scenario_content_report};
}

pub mod model {
    pub use crate::catalog_types::*;
    pub use rulebench_rules::*;
}

pub mod modifiers {
    pub use rulebench_rules::{
        active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    };
}

pub mod resolver {
    pub use rulebench_rules::{resolve_use_action, validate_intent_shape};
}

pub mod runtime {
    pub use rulebench_rules::{
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
        CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind,
        CombatSessionScriptReadout, CombatSessionScriptSpec, CombatSessionScriptStepReadout,
        CombatSessionScriptStepSpec, CombatSessionState,
    };
}

pub mod state {
    pub use rulebench_rules::CombatState;
}

pub use catalog::{
    content_validation_readouts, resolve_catalog_scenario, ruleset_catalog_readout,
    scenario_catalog_cases, scenario_catalog_summaries,
};
pub use catalog_types::*;
pub use fixtures::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    rejected_target_fixture_receipt, turn_control_fixture_scenario,
};
pub use rulebench_rules::*;
pub use rulebench_rules::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    fingerprint_projected_state, fingerprint_projection, resolve_use_action, validate_intent_shape,
    validate_scenario_content, validate_scenario_content_report, verify_automatic_run_replay,
    CombatSessionAutoCandidateCommandSpec, CombatSessionAutoCandidateDecisionKind,
    CombatSessionAutoCandidateExecutionReadout, CombatSessionAutoCandidatePlanReadout,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunReplayDecisionKind, CombatSessionAutomaticRunReplayReadout,
    CombatSessionAutomaticRunReplaySpec, CombatSessionAutomaticRunSpec,
    CombatSessionAutomaticStepDecisionKind, CombatSessionAutomaticStepExecutionReadout,
    CombatSessionAutomaticStepOperationKind, CombatSessionAutomaticStepPlanReadout,
    CombatSessionAutomaticStepSpec, CombatSessionCandidateExecutionReadout,
    CombatSessionCandidateSelectionDecisionKind, CombatSessionCandidateSelectionReadout,
    CombatSessionCandidateSelectionSpec, CombatSessionCommandSpec, CombatSessionIntentCommandSpec,
    CombatSessionScriptCommandKind, CombatSessionScriptCommandSpec,
    CombatSessionScriptDecisionKind, CombatSessionScriptReadout, CombatSessionScriptSpec,
    CombatSessionScriptStepReadout, CombatSessionScriptStepSpec, CombatSessionState,
    PROJECTION_FINGERPRINT_ALGORITHM, STATE_FINGERPRINT_ALGORITHM,
};
pub use session::{
    combat_session_automatic_run_readouts, combat_session_automatic_run_replay_readouts,
    combat_session_control_history_readouts, combat_session_script_readouts,
    combat_session_summaries, combat_session_transcripts, resolve_combat_session_step,
};

#[cfg(test)]
mod tests;
