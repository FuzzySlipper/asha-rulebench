//! Supported portable convenience facade for ASHA Rulebench rule authority.
//!
//! This crate re-exports the stable public API from the portable authority
//! layers: core values, ruleset declarations, canonical content, combat
//! execution, and replay verification. New portable consumers may depend on
//! the focused owner crates when they need a narrower surface; this facade is
//! the supported one-crate entry point for consumers that need the complete
//! authority contract.
//!
//! Stability: public items re-exported here are the local `v0` portable
//! contract. Rulebench-only fixtures, generated artifacts, bridge adapters,
//! and UI concerns are intentionally absent.
//!
//! Non-claims: this is not a generic rules-engine API, a host runtime, or a
//! promise that every currently public owner-crate item has cross-project
//! compatibility beyond the documented Rulebench portable contract.

#![forbid(unsafe_code)]

pub use rulebench_combat::model::*;
pub use rulebench_combat::{
    active_modifier_stat_adjustments_for_combatant, effective_stats_for_combatant,
    fingerprint_projected_state, fingerprint_projection, resolve_use_action, validate_intent_shape,
    CombatSessionApi, CombatSessionApiError, CombatSessionArchive,
    CombatSessionAutoCandidateCommandSpec, CombatSessionAutoCandidateDecisionKind,
    CombatSessionAutoCandidateExecutionReadout, CombatSessionAutoCandidatePlanReadout,
    CombatSessionAutomaticRunDecisionKind, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepDecisionKind,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepOperationKind,
    CombatSessionAutomaticStepPlanReadout, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateExecutionReadout, CombatSessionCandidateSelectionDecisionKind,
    CombatSessionCandidateSelectionReadout, CombatSessionCandidateSelectionSpec,
    CombatSessionCommandSpec, CombatSessionCreateReadout, CombatSessionCreateRequest,
    CombatSessionHandle, CombatSessionIntentCommandSpec, CombatSessionScriptCommandKind,
    CombatSessionScriptCommandSpec, CombatSessionScriptDecisionKind, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepReadout, CombatSessionScriptStepSpec,
    CombatSessionState, CombatState, PROJECTION_FINGERPRINT_ALGORITHM, STATE_FINGERPRINT_ALGORITHM,
};
pub use rulebench_content::{validate_scenario_content, validate_scenario_content_report};
pub use rulebench_core::Team;
pub use rulebench_replay::{
    verify_automatic_run_replay, CombatSessionAutomaticRunReplayDecisionKind,
    CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
};

#[cfg(test)]
mod tests {
    use super::{
        verify_automatic_run_replay, CombatSessionApi, CombatSessionAutomaticRunReplayReadout,
        CombatSessionAutomaticRunReplaySpec, CombatSessionHandle, RulesetMetadata,
    };

    #[test]
    fn facade_exposes_the_documented_portable_contract() {
        let api = CombatSessionApi::new();
        let handle = CombatSessionHandle::new("portable-session");
        let ruleset = RulesetMetadata {
            id: "portable.ruleset".to_string(),
            name: "Portable Ruleset".to_string(),
            version: "0.1.0".to_string(),
            summary: "Facade contract smoke.".to_string(),
            modules: Vec::new(),
        };
        let replay_verifier: fn(
            CombatSessionAutomaticRunReplaySpec,
        ) -> CombatSessionAutomaticRunReplayReadout = verify_automatic_run_replay;

        assert!(api.list_active_sessions().is_empty());
        assert_eq!(ruleset.id, "portable.ruleset");
        assert_eq!(handle.id, "portable-session");
        let _ = replay_verifier;
    }
}
