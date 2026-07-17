use super::fixture::hexing_bolt_fixture_scenario;
use rulebench_rpg_adapter::*;

pub fn combat_session_transcripts() -> Vec<CombatSessionTranscript> {
    vec![hexing_bolt_opening_exchange_session()]
}

pub fn combat_session_control_history_readouts() -> Vec<CombatControlHistoryReadout> {
    vec![hexing_bolt_control_history_readout()]
}

pub fn combat_session_script_readouts() -> Vec<CombatSessionScriptReadout> {
    vec![hexing_bolt_mixed_script_readout()]
}

pub fn combat_session_automatic_run_readouts() -> Vec<CombatSessionAutomaticRunReadout> {
    vec![
        hexing_bolt_bounded_automatic_run_readout(
            "hexing-bolt-bounded-automatic-run",
            CombatAutomationPolicySpec::first_accepted_candidate(),
        ),
        hexing_bolt_bounded_automatic_run_readout(
            "hexing-bolt-lowest-vitality-automatic-run",
            CombatAutomationPolicySpec::lowest_vitality_target(),
        ),
    ]
}

pub fn combat_session_automatic_run_replay_readouts() -> Vec<CombatSessionAutomaticRunReplayReadout>
{
    combat_session_automatic_run_readouts()
        .into_iter()
        .enumerate()
        .map(|(index, run)| hexing_bolt_bounded_automatic_run_replay_readout(index, run))
        .collect()
}

fn hexing_bolt_bounded_automatic_run_replay_readout(
    index: usize,
    run_readout: CombatSessionAutomaticRunReadout,
) -> CombatSessionAutomaticRunReplayReadout {
    let replay_id = if index == 0 {
        "hexing-bolt-bounded-automatic-run-replay".to_string()
    } else {
        format!("{}-replay", run_readout.id)
    };
    let run_spec = CombatSessionAutomaticRunSpec::new(
        run_readout.id.clone(),
        run_readout.title.clone(),
        run_readout.summary.clone(),
        run_readout.max_steps,
        vec![17, 5],
    )
    .with_policy(run_readout.policy.clone());

    verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
        replay_id,
        "Hexing Bolt Bounded Automatic Run Replay",
        "A generated Rust replay verification readout for the bounded automatic run fixture.",
        "hexing-bolt-bounded-automatic-run-replay-session",
        hexing_bolt_fixture_scenario(),
        run_spec,
        run_readout.final_snapshot.current_state_fingerprint,
        run_readout.final_snapshot.finalization,
        run_readout.decision_kind,
        run_readout.executed_step_count,
        run_readout.policy_decisions,
        run_readout.final_snapshot.action_resource_transition_log,
        run_readout.final_snapshot.equipment_ledger,
        run_readout.final_snapshot.class_build_ledger,
        run_readout.final_snapshot.equipment_transition_log,
        run_readout.final_snapshot.reaction_window_lifecycle_log,
        run_readout.final_snapshot.reaction_audit_log,
        run_readout.final_snapshot.modifier_duration_expiration_log,
    ))
}

fn hexing_bolt_bounded_automatic_run_readout(
    session_id: &str,
    policy: CombatAutomationPolicySpec,
) -> CombatSessionAutomaticRunReadout {
    let title = "Hexing Bolt Bounded Automatic Run";
    let summary = "A generated Rust automatic run readout that drives the fixture to ended combat within a max-step guard.";

    let mut session_state = CombatSessionState::new(session_id, hexing_bolt_fixture_scenario());

    session_state.run_automatic_combat(
        CombatSessionAutomaticRunSpec::new(session_id, title, summary, 8, vec![17, 5])
            .with_policy(policy),
    )
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

    let mut session_state = CombatSessionState::new(session_id, hexing_bolt_fixture_scenario());

    session_state.run_script(hexing_bolt_mixed_script_spec())
}

pub(super) fn hexing_bolt_mixed_script_spec() -> CombatSessionScriptSpec {
    CombatSessionScriptSpec::new(
        "hexing-bolt-mixed-control-script",
        "Hexing Bolt Mixed Control Script",
        "A generated Rust script readout that mixes lifecycle control, selected-candidate execution, and rejected selected-candidate planning.",
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
                "script-missing-damage-intent-step",
                "Adept has incomplete roll evidence",
                "Raw intent command rejects because Rust needs damage roll evidence after the attack roll hits.",
                CombatSessionIntentCommandSpec::new(
                    "script-missing-damage-intent",
                    "Adept missing damage roll",
                    "Adept attempts Hexing Bolt with no damage roll supplied.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17],
                ),
            ),
            CombatSessionScriptStepSpec::selected_candidate(
                "script-selected-hit-step",
                "Adept selects Hexing Bolt",
                "The current actor selects the Rust-visible Hexing Bolt candidate and the resolver accepts the hit.",
                CombatSessionCandidateSelectionSpec::new(
                    "script-selected-runtime-hit",
                    "Adept hits Raider",
                    "Hexing Bolt resolves as an accepted hit inside the selected-candidate script.",
                    "hexing_bolt",
                    "entity-raider",
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-advance-turn",
                "Advances to Raider",
                "Control command advances the active turn after the accepted intent.",
                CombatControlCommandSpec::advance_turn(),
            ),
            CombatSessionScriptStepSpec::selected_candidate(
                "script-selected-unavailable-step",
                "Raider has no candidate",
                "Selected-candidate planning rejects because Raider has no matching action in this fixture.",
                CombatSessionCandidateSelectionSpec::new(
                    "script-selected-unavailable",
                    "Raider unavailable selected candidate",
                    "Raider has no command candidates in this fixture.",
                    "hexing_bolt",
                    "entity-raider",
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::intent(
                "script-wrong-turn-intent-step",
                "Adept acts during Raider turn",
                "Raw intent command rejects because Adept is no longer the current actor.",
                CombatSessionIntentCommandSpec::new(
                    "script-wrong-turn-intent",
                    "Adept wrong-turn intent",
                    "Adept attempts Hexing Bolt during Raider's turn and Rust preflight rejects it.",
                    UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
                    vec![17, 5],
                ),
            ),
            CombatSessionScriptStepSpec::control(
                "script-advance-turn-wrap",
                "Advances back to Adept",
                "Control command advances from Raider back to Adept and expires Raider's temporary rattled modifier.",
                CombatControlCommandSpec::advance_turn(),
            ),
            CombatSessionScriptStepSpec::control(
                "script-end",
                "Explicitly ends combat",
                "Control command ends the combat lifecycle after mixed command processing.",
                CombatControlCommandSpec::explicit_end(),
            ),
        ],
    )
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
