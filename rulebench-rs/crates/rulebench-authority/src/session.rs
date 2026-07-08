use crate::fixtures::hexing_bolt_fixture_scenario;
use crate::model::*;
use crate::runtime::{
    CombatSessionCommandSpec, CombatSessionIntentCommandSpec, CombatSessionScriptReadout,
    CombatSessionScriptSpec, CombatSessionScriptStepSpec, CombatSessionState,
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
    vec![hexing_bolt_opening_exchange_session()]
}

pub fn combat_session_control_history_readouts() -> Vec<CombatControlHistoryReadout> {
    vec![hexing_bolt_control_history_readout()]
}

pub fn combat_session_script_readouts() -> Vec<CombatSessionScriptReadout> {
    vec![hexing_bolt_mixed_script_readout()]
}

fn hexing_bolt_opening_exchange_session() -> CombatSessionTranscript {
    let session_id = "hexing-bolt-opening-exchange";
    let session_title = "Hexing Bolt Opening Exchange";
    let session_summary =
        "A deterministic three-step transcript for accepted hit, accepted miss, and target-legality rejection.";
    let session_seed_label = "roll-streams:17,5|2,5|17,5";

    let step_specs = vec![
        session_step_spec(
            "adept-hexing-bolt-hit",
            "Adept hits Raider",
            "Hexing Bolt hits Raider, applying damage and rattled.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ),
        session_step_spec(
            "adept-hexing-bolt-miss",
            "Adept misses Raider",
            "Hexing Bolt misses Raider; the prior state remains authoritative.",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ),
        session_step_spec(
            "adept-hexing-bolt-self-target-rejected",
            "Adept targets themself",
            "Target legality rejects a non-hostile self target; no events are accepted.",
            CommandOutcomeClass::RejectedTargetLegality,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
            vec![17, 5],
        ),
    ];

    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: session_id.to_string(),
        title: session_title.to_string(),
        summary: session_summary.to_string(),
        seed_label: session_seed_label.to_string(),
    };

    let mut session_state = CombatSessionState::new(session_id, scenario);
    let mut readouts = Vec::new();
    for spec in step_specs {
        readouts.push(session_state.submit_command(CombatSessionCommandSpec::new(
            spec.id,
            spec.title,
            spec.summary,
            spec.outcome_class,
            spec.intent,
            spec.roll_stream,
        )));
    }

    CombatSessionTranscript {
        summary: CombatSessionSummary {
            id: session_id.to_string(),
            title: session_title.to_string(),
            summary: session_summary.to_string(),
            seed_label: session_seed_label.to_string(),
            steps: readouts
                .iter()
                .map(|readout| readout.step.clone())
                .collect(),
        },
        steps: readouts,
    }
}

fn hexing_bolt_mixed_script_readout() -> CombatSessionScriptReadout {
    let session_id = "hexing-bolt-mixed-control-script";
    let script_id = "hexing-bolt-mixed-control-script";
    let title = "Hexing Bolt Mixed Control Script";
    let summary = "A generated Rust script readout that mixes lifecycle control, accepted intent, and rejected wrong-actor intent.";

    let mut session_state = CombatSessionState::new(session_id, hexing_bolt_fixture_scenario());

    session_state.run_script(CombatSessionScriptSpec::new(
        script_id,
        title,
        summary,
        vec![
            CombatSessionScriptStepSpec::control(
                "script-start",
                "Explicitly starts combat",
                "Control command starts the combat lifecycle before the first intent.",
                CombatControlCommandSpec::explicit_start(),
            ),
            CombatSessionScriptStepSpec::control(
                "script-repeat-start",
                "Repeats combat start",
                "A repeated explicit start is rejected as a no-op without changing state.",
                CombatControlCommandSpec::explicit_start(),
            ),
            CombatSessionScriptStepSpec::intent(
                "script-hit-step",
                "Adept hits Raider",
                "The current actor uses Hexing Bolt and the resolver accepts the hit.",
                CombatSessionIntentCommandSpec::new(
                    "script-runtime-hit",
                    "Adept hits Raider",
                    "Hexing Bolt resolves as an accepted hit inside the mixed script.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-advance-turn",
                "Advances to Raider",
                "Control command advances the active turn after the accepted intent.",
                CombatControlCommandSpec::advance_turn(),
            ),
            CombatSessionScriptStepSpec::intent(
                "script-wrong-actor-step",
                "Adept acts out of turn",
                "The preflight layer rejects an Adept intent while Raider is the current actor.",
                CombatSessionIntentCommandSpec::new(
                    "script-runtime-wrong-actor",
                    "Adept acts out of turn",
                    "Hexing Bolt is rejected before resolver mutation because the actor is not current.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-end",
                "Explicitly ends combat",
                "Control command ends the combat lifecycle after mixed command processing.",
                CombatControlCommandSpec::explicit_end(),
            ),
        ],
    ))
}

fn hexing_bolt_control_history_readout() -> CombatControlHistoryReadout {
    let session_id = "hexing-bolt-control-sequence";
    let title = "Hexing Bolt Control Sequence";
    let summary = "A generated Rust control-history fixture for explicit start, turn advance, explicit end, and rejected post-end turn advance.";

    let mut session_state = CombatSessionState::new(session_id, hexing_bolt_fixture_scenario());
    session_state.submit_control_command(CombatControlCommandSpec::explicit_start());
    session_state.submit_control_command(CombatControlCommandSpec::advance_turn());
    session_state.submit_control_command(CombatControlCommandSpec::explicit_end());
    session_state.submit_control_command(CombatControlCommandSpec::advance_turn());

    CombatControlHistoryReadout {
        session_id: session_id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        history: session_state.control_history().to_vec(),
    }
}

struct SessionStepSpec {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    outcome_class: CommandOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
}

fn session_step_spec(
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    outcome_class: CommandOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
) -> SessionStepSpec {
    SessionStepSpec {
        id,
        title,
        summary,
        outcome_class,
        intent,
        roll_stream,
    }
}
