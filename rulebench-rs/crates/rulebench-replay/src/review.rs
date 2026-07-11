use rulebench_combat::{
    CombatSessionCreateRequest, CombatSessionSnapshot, CombatSessionState, CommandRollMode,
    RulesetArtifactProvenance,
};

use crate::{
    verification::execute_command, ReplayAcceptedEvents, ReplayCommand,
    ReplayCommandRandomnessProvenance, ReplayCommandRecord, ReplayEvidence,
    ReplayGeneratedRollRequest, ReplayPackage, ReplayRandomnessSource, ReplayRollEvidence,
    ReplayStepEvidence, ReplayTraceEvidence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommandRecordingSpec {
    pub id: String,
    pub command: ReplayCommand,
}

impl ReplayCommandRecordingSpec {
    pub fn new(id: impl Into<String>, command: ReplayCommand) -> Self {
        Self {
            id: id.into(),
            command,
        }
    }
}

pub fn record_replay_package(
    package_id: impl Into<String>,
    initial_session: CombatSessionCreateRequest,
    ruleset: RulesetArtifactProvenance,
    commands: Vec<ReplayCommandRecordingSpec>,
) -> ReplayPackage {
    let mut session = CombatSessionState::new(
        initial_session.session.id.clone(),
        initial_session.scenario.clone(),
    );
    let mut evidence = ReplayEvidence::default();
    let commands = commands
        .into_iter()
        .enumerate()
        .map(|(index, command)| {
            let sequence = index as u32;
            let expected = execute_command(&mut session, &command.command);
            let generated_seed = match &command.command {
                ReplayCommand::Intent(spec) => match spec.roll_mode {
                    CommandRollMode::AuthorityGenerated { seed }
                    | CommandRollMode::RecordedGenerated { seed } => Some(seed),
                    CommandRollMode::Supplied => None,
                },
                _ => None,
            };
            let mut recorded_command = command.command.clone();
            if let (Some(seed), ReplayCommand::Intent(spec)) =
                (generated_seed, &mut recorded_command)
            {
                spec.roll_stream = expected
                    .rolls
                    .iter()
                    .filter_map(|roll| roll.supplied_value)
                    .collect();
                spec.roll_mode = CommandRollMode::RecordedGenerated { seed };
            }
            let supplied_roll_stream = recorded_command.supplied_roll_stream().to_vec();
            evidence.accepted_events.push(ReplayAcceptedEvents {
                command_sequence: sequence,
                events: expected.accepted_events.clone(),
            });
            evidence
                .command_audit
                .extend(expected.command_audit.clone());
            evidence.rolls.push(ReplayRollEvidence {
                command_sequence: sequence,
                consumption: expected.rolls.clone(),
            });
            evidence.trace.push(ReplayTraceEvidence {
                command_sequence: sequence,
                entries: expected.trace.clone(),
            });
            if let Some(seed) = generated_seed {
                let generated_requests = expected
                    .rolls
                    .iter()
                    .filter_map(|roll| {
                        roll.supplied_value.map(|value| {
                            let maximum = if roll.request_kind.code() == "damageRoll" {
                                8
                            } else {
                                20
                            };
                            ReplayGeneratedRollRequest {
                                sequence: roll.sequence,
                                request_id: format!("{}:{}", command.id, roll.request_kind.code()),
                                request_kind: roll.request_kind,
                                minimum: 1,
                                maximum,
                                value,
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                evidence.randomness.push(ReplayCommandRandomnessProvenance {
                    command_sequence: sequence,
                    source_id: format!("command:{}", command.id),
                    source: ReplayRandomnessSource::Generated {
                        seed,
                        algorithm_version: crate::REPLAY_RANDOMNESS_ALGORITHM_VERSION.to_string(),
                    },
                    supplied_values: supplied_roll_stream.clone(),
                    generated_requests,
                    consumption: expected.rolls.clone(),
                    unused_values: Vec::new(),
                });
            } else if !supplied_roll_stream.is_empty() {
                evidence
                    .randomness
                    .push(ReplayCommandRandomnessProvenance::supplied(
                        sequence,
                        format!("command:{}", command.id),
                        supplied_roll_stream,
                        expected.rolls.clone(),
                    ));
            }
            ReplayCommandRecord {
                sequence,
                id: command.id,
                command: recorded_command,
                expected,
            }
        })
        .collect();
    ReplayPackage::new(
        package_id,
        initial_session,
        ruleset,
        commands,
        evidence,
        session.snapshot().current_state_fingerprint,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPackageInspection {
    pub commands: Vec<ReplayCommandInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommandInspection {
    pub sequence: u32,
    pub id: String,
    pub command_kind: String,
    pub supplied_roll_stream: Vec<i32>,
    pub narration_summary: Option<String>,
    pub expected: ReplayStepEvidence,
    pub actual: ReplayStepEvidence,
    pub snapshot: CombatSessionSnapshot,
}

pub fn inspect_replay_package(package: &ReplayPackage) -> ReplayPackageInspection {
    let mut session = CombatSessionState::new(
        package.initial_session.session.id.clone(),
        package.initial_session.scenario.clone(),
    );
    let commands = package
        .commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let actual = execute_command(&mut session, &command.command);
            ReplayCommandInspection {
                sequence: command.sequence,
                id: command.id.clone(),
                command_kind: command.command.code().to_string(),
                supplied_roll_stream: command.command.supplied_roll_stream().to_vec(),
                narration_summary: package
                    .narration
                    .as_ref()
                    .and_then(|narration| narration.command_summaries.get(index).cloned()),
                expected: command.expected.clone(),
                actual,
                snapshot: session.snapshot(),
            }
        })
        .collect();
    ReplayPackageInspection { commands }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::tests::recorded_control_package;

    #[test]
    fn inspection_keeps_expected_actual_and_snapshot_evidence_together() {
        let package = recorded_control_package();

        let inspection = inspect_replay_package(&package);

        assert_eq!(inspection.commands.len(), 3);
        assert_eq!(inspection.commands[0].command_kind, "control");
        assert_eq!(
            inspection.commands[0].expected,
            inspection.commands[0].actual
        );
        assert_eq!(
            inspection.commands[2].snapshot.lifecycle.phase.code(),
            "ended"
        );
        assert!(inspection.commands[2].snapshot.finalization.is_some());
    }

    #[test]
    fn recording_builds_a_verifiable_package_from_authority_execution() {
        let source = recorded_control_package();
        let recorded = record_replay_package(
            "recorded-again",
            source.initial_session.clone(),
            source.ruleset.clone(),
            source
                .commands
                .iter()
                .map(|command| {
                    ReplayCommandRecordingSpec::new(&command.id, command.command.clone())
                })
                .collect(),
        );

        assert!(crate::verify_replay_package(&recorded).accepted);
        assert_eq!(recorded.evidence.accepted_events.len(), 3);
        assert_eq!(recorded.evidence.rolls.len(), 3);
        assert_eq!(recorded.evidence.trace.len(), 3);
    }
}
