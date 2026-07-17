//! Rulebench-local command entrypoints and integration test harness.
//!
//! Authority behavior lives in `rulebench-rpg-adapter`; authored scenario content and
//! regression evidence live in `rulebench-fixtures`; checked TypeScript artifact
//! rendering lives in `rulebench-codegen`. This crate intentionally provides no
//! library compatibility API: its two stable binary entrypoints preserve the
//! existing catalog and combat-session generation commands while repository
//! callers use those owning crate APIs directly.
//!
//! Non-claims: this is not the portable authority facade, a rules owner, or a
//! home for scenario-specific behavior.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_support {
    pub(crate) use rulebench_fixtures::*;

    pub(crate) fn scenario_catalog_summaries() -> Vec<ScenarioCatalogSummary> {
        aggregated_scenario_catalog_cases()
            .into_iter()
            .map(|case| case.summary)
            .collect()
    }

    pub(crate) fn combat_session_summaries() -> Vec<CombatSessionSummary> {
        aggregated_combat_session_transcripts()
            .into_iter()
            .map(|transcript| transcript.summary)
            .collect()
    }

    pub(crate) fn resolve_combat_session_step(
        session_id: &str,
        step_id: &str,
    ) -> Result<CombatSessionStepReadout, CombatSessionError> {
        let transcript = aggregated_combat_session_transcripts()
            .into_iter()
            .find(|transcript| transcript.summary.id == session_id)
            .ok_or(CombatSessionError::UnknownSessionId)?;

        transcript
            .steps
            .into_iter()
            .find(|step| step.step.id == step_id)
            .ok_or(CombatSessionError::UnknownStepId)
    }
}
