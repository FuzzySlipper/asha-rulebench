use crate::model::{
    CombatControlHistoryReadout, CombatSessionError, CombatSessionStepReadout,
    CombatSessionSummary, CombatSessionTranscript,
};
use crate::{
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticRunReplayReadout,
    CombatSessionScriptReadout,
};

pub fn combat_session_summaries() -> Vec<CombatSessionSummary> {
    combat_session_transcripts()
        .into_iter()
        .map(|transcript| transcript.summary)
        .collect()
}

pub fn resolve_combat_session_step(
    session_id: &str,
    step_id: &str,
) -> Result<CombatSessionStepReadout, CombatSessionError> {
    let Some(transcript) = combat_session_transcripts()
        .into_iter()
        .find(|transcript| transcript.summary.id == session_id)
    else {
        return Err(CombatSessionError::UnknownSessionId);
    };

    transcript
        .steps
        .into_iter()
        .find(|step| step.step.id == step_id)
        .ok_or(CombatSessionError::UnknownStepId)
}

pub fn combat_session_transcripts() -> Vec<CombatSessionTranscript> {
    rulebench_fixtures::combat_session_transcripts()
}

pub fn combat_session_control_history_readouts() -> Vec<CombatControlHistoryReadout> {
    rulebench_fixtures::combat_session_control_history_readouts()
}

pub fn combat_session_script_readouts() -> Vec<CombatSessionScriptReadout> {
    rulebench_fixtures::combat_session_script_readouts()
}

pub fn combat_session_automatic_run_readouts() -> Vec<CombatSessionAutomaticRunReadout> {
    rulebench_fixtures::combat_session_automatic_run_readouts()
}

pub fn combat_session_automatic_run_replay_readouts() -> Vec<CombatSessionAutomaticRunReplayReadout>
{
    rulebench_fixtures::combat_session_automatic_run_replay_readouts()
}
