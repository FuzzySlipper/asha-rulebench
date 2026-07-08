use crate::fixtures::hexing_bolt_fixture_scenario;
use crate::model::*;
use crate::resolver::resolve_use_action;
use crate::state::CombatState;

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

fn hexing_bolt_opening_exchange_session() -> CombatSessionTranscript {
    let session_id = "hexing-bolt-opening-exchange";
    let session_title = "Hexing Bolt Opening Exchange";
    let session_summary =
        "A deterministic three-step transcript for accepted hit, accepted miss, and target-legality rejection.";
    let session_seed_label = "roll-streams:17,5|2,5|17,5";

    let step_specs = vec![
        session_step_spec(
            "adept-hexing-bolt-hit",
            0,
            "Adept hits Raider",
            "Hexing Bolt hits Raider, applying damage and rattled.",
            CommandOutcomeClass::AcceptedHit,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![17, 5],
        ),
        session_step_spec(
            "adept-hexing-bolt-miss",
            1,
            "Adept misses Raider",
            "Hexing Bolt misses Raider; the prior state remains authoritative.",
            CommandOutcomeClass::AcceptedMiss,
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            vec![2, 5],
        ),
        session_step_spec(
            "adept-hexing-bolt-self-target-rejected",
            2,
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

    let mut state = CombatState::from_scenario(&scenario);
    let mut steps = Vec::new();
    for spec in step_specs {
        scenario = state.apply_to_scenario(scenario);
        let state_before = state.project("State before command resolution.");
        let receipt = resolve_use_action(&scenario, spec.intent.clone(), &spec.roll_stream);
        let state_after = receipt
            .projection
            .clone()
            .expect("session resolver always produces projection");
        let command = CommandAttempt {
            step_id: spec.id.to_string(),
            step_index: spec.index,
            actor_id: spec.intent.actor_id,
            action_id: spec.intent.action_id,
            target_id: spec.intent.target_id,
            roll_stream: spec.roll_stream,
            outcome_class: spec.outcome_class,
        };
        let step = CombatSessionStepSummary {
            id: spec.id.to_string(),
            index: spec.index,
            title: spec.title.to_string(),
            summary: spec.summary.to_string(),
            outcome_class: spec.outcome_class,
            log_index: spec.index + 1,
        };
        let combat_log = vec![combat_log_entry(&step, &receipt)];

        steps.push(CombatSessionStepReadout {
            session_id: session_id.to_string(),
            step,
            command,
            scenario: scenario.clone(),
            receipt,
            combat_log,
            state_before,
            state_after: state_after.clone(),
        });

        state = CombatState::from_projection(&state_after);
    }

    CombatSessionTranscript {
        summary: CombatSessionSummary {
            id: session_id.to_string(),
            title: session_title.to_string(),
            summary: session_summary.to_string(),
            seed_label: session_seed_label.to_string(),
            steps: steps.iter().map(|readout| readout.step.clone()).collect(),
        },
        steps,
    }
}

struct SessionStepSpec {
    id: &'static str,
    index: u32,
    title: &'static str,
    summary: &'static str,
    outcome_class: CommandOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
}

fn session_step_spec(
    id: &'static str,
    index: u32,
    title: &'static str,
    summary: &'static str,
    outcome_class: CommandOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
) -> SessionStepSpec {
    SessionStepSpec {
        id,
        index,
        title,
        summary,
        outcome_class,
        intent,
        roll_stream,
    }
}

fn combat_log_entry(step: &CombatSessionStepSummary, receipt: &RulebenchReceipt) -> CombatLogEntry {
    CombatLogEntry {
        id: format!("log-{}", step.id),
        step_id: step.id.clone(),
        log_index: step.log_index,
        title: step.title.clone(),
        summary: step.summary.clone(),
        outcome_class: step.outcome_class,
        event_types: receipt.events.iter().map(domain_event_type).collect(),
    }
}

fn domain_event_type(event: &DomainEvent) -> String {
    match event {
        DomainEvent::IntentShapeAccepted { .. } => "IntentShapeAccepted",
        DomainEvent::ActionUsed { .. } => "ActionUsed",
        DomainEvent::AttackRolled { .. } => "AttackRolled",
        DomainEvent::DamageApplied { .. } => "DamageApplied",
        DomainEvent::ModifierApplied { .. } => "ModifierApplied",
    }
    .to_string()
}
