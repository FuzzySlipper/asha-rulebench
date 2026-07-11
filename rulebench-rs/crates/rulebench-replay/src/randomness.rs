use std::collections::BTreeSet;

use crate::ReplayPackage;
use rulebench_combat::{RollConsumptionEntry, RollRequestKind};

pub const REPLAY_RANDOMNESS_ALGORITHM_VERSION: &str = "xorshift64star.rulebench-rolls.v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayRandomnessSource {
    Supplied,
    Generated {
        seed: u64,
        algorithm_version: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRollGenerationSpec {
    pub request_id: String,
    pub request_kind: RollRequestKind,
    pub minimum: i32,
    pub maximum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayGeneratedRollRequest {
    pub sequence: u32,
    pub request_id: String,
    pub request_kind: RollRequestKind,
    pub minimum: i32,
    pub maximum: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommandRandomnessProvenance {
    pub command_sequence: u32,
    pub source_id: String,
    pub source: ReplayRandomnessSource,
    pub supplied_values: Vec<i32>,
    pub generated_requests: Vec<ReplayGeneratedRollRequest>,
    pub consumption: Vec<RollConsumptionEntry>,
    pub unused_values: Vec<i32>,
}

impl ReplayCommandRandomnessProvenance {
    pub fn supplied(
        command_sequence: u32,
        source_id: impl Into<String>,
        values: Vec<i32>,
        consumption: Vec<RollConsumptionEntry>,
    ) -> Self {
        let consumed_count = consumption.iter().filter(|entry| entry.consumed).count();
        let unused_values = values.iter().skip(consumed_count).copied().collect();
        Self {
            command_sequence,
            source_id: source_id.into(),
            source: ReplayRandomnessSource::Supplied,
            supplied_values: values,
            generated_requests: Vec::new(),
            consumption,
            unused_values,
        }
    }
}

pub fn generate_replay_randomness(
    command_sequence: u32,
    source_id: impl Into<String>,
    seed: u64,
    requests: &[ReplayRollGenerationSpec],
) -> ReplayCommandRandomnessProvenance {
    let mut state = seed;
    let generated_requests = requests
        .iter()
        .enumerate()
        .map(|(index, request)| ReplayGeneratedRollRequest {
            sequence: index as u32,
            request_id: request.request_id.clone(),
            request_kind: request.request_kind,
            minimum: request.minimum,
            maximum: request.maximum,
            value: next_bounded_value(&mut state, request.minimum, request.maximum),
        })
        .collect::<Vec<_>>();
    let supplied_values = generated_requests
        .iter()
        .map(|value| value.value)
        .collect::<Vec<_>>();
    let unused_values = supplied_values.clone();
    ReplayCommandRandomnessProvenance {
        command_sequence,
        source_id: source_id.into(),
        source: ReplayRandomnessSource::Generated {
            seed,
            algorithm_version: REPLAY_RANDOMNESS_ALGORITHM_VERSION.to_string(),
        },
        supplied_values,
        generated_requests,
        consumption: Vec::new(),
        unused_values,
    }
}

pub fn reproduce_replay_roll_stream(
    provenance: &ReplayCommandRandomnessProvenance,
) -> Result<Vec<i32>, ReplayRandomnessDiagnosticCode> {
    match provenance.source {
        ReplayRandomnessSource::Supplied => Ok(provenance.supplied_values.clone()),
        ReplayRandomnessSource::Generated {
            seed,
            ref algorithm_version,
        } => {
            if algorithm_version != REPLAY_RANDOMNESS_ALGORITHM_VERSION {
                return Err(ReplayRandomnessDiagnosticCode::UnsupportedGenerator);
            }
            let requests = provenance
                .generated_requests
                .iter()
                .map(|request| ReplayRollGenerationSpec {
                    request_id: request.request_id.clone(),
                    request_kind: request.request_kind,
                    minimum: request.minimum,
                    maximum: request.maximum,
                })
                .collect::<Vec<_>>();
            Ok(generate_replay_randomness(
                provenance.command_sequence,
                provenance.source_id.clone(),
                seed,
                &requests,
            )
            .supplied_values)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayRandomnessDiagnosticCode {
    MissingProvenance,
    UnknownCommand,
    DuplicateCommand,
    EmptySourceId,
    StreamMismatch,
    ConsumptionMismatch,
    UnusedValuesMismatch,
    ReorderedRequest,
    GeneratedValueMismatch,
    InvalidBounds,
    UnsupportedGenerator,
}

impl ReplayRandomnessDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingProvenance => "missingReplayRandomnessProvenance",
            Self::UnknownCommand => "unknownReplayRandomnessCommand",
            Self::DuplicateCommand => "duplicateReplayRandomnessCommand",
            Self::EmptySourceId => "emptyReplayRandomnessSourceId",
            Self::StreamMismatch => "replayRandomnessStreamMismatch",
            Self::ConsumptionMismatch => "replayRandomnessConsumptionMismatch",
            Self::UnusedValuesMismatch => "replayRandomnessUnusedValuesMismatch",
            Self::ReorderedRequest => "reorderedReplayRandomnessRequest",
            Self::GeneratedValueMismatch => "replayGeneratedValueMismatch",
            Self::InvalidBounds => "invalidReplayRandomnessBounds",
            Self::UnsupportedGenerator => "unsupportedReplayRandomnessGenerator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRandomnessDiagnostic {
    pub code: ReplayRandomnessDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRandomnessValidationReport {
    pub accepted: bool,
    pub diagnostics: Vec<ReplayRandomnessDiagnostic>,
}

pub fn validate_replay_randomness(package: &ReplayPackage) -> ReplayRandomnessValidationReport {
    let mut diagnostics = Vec::new();
    let command_sequences = package
        .commands
        .iter()
        .map(|value| value.sequence)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();

    for (index, provenance) in package.evidence.randomness.iter().enumerate() {
        let path = format!("evidence.randomness[{index}]");
        if !command_sequences.contains(&provenance.command_sequence) {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::UnknownCommand,
                &path,
                "Randomness provenance references an unknown command.",
            );
            continue;
        }
        if !seen.insert(provenance.command_sequence) {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::DuplicateCommand,
                &path,
                "A command may have only one randomness provenance record.",
            );
        }
        if provenance.source_id.trim().is_empty() {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::EmptySourceId,
                &format!("{path}.sourceId"),
                "Randomness source id is required.",
            );
        }
        let command = package
            .commands
            .iter()
            .find(|value| value.sequence == provenance.command_sequence)
            .expect("known command was checked");
        match reproduce_replay_roll_stream(provenance) {
            Err(code) => diagnostic(
                &mut diagnostics,
                code,
                &format!("{path}.source"),
                "Randomness generator is not reproducible by this authority version.",
            ),
            Ok(values) if values != command.command.supplied_roll_stream() => diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::StreamMismatch,
                &format!("{path}.suppliedValues"),
                "Reproduced roll stream does not exactly match the recorded command stream.",
            ),
            Ok(values) if values != provenance.supplied_values => diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::GeneratedValueMismatch,
                &format!("{path}.generatedRequests"),
                "Generated request values do not reproduce the recorded supplied values.",
            ),
            Ok(_) => {}
        }
        if provenance.consumption != command.expected.rolls {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::ConsumptionMismatch,
                &format!("{path}.consumption"),
                "Randomness consumption must exactly match ordered replay roll evidence.",
            );
        }
        let consumed_count = provenance
            .consumption
            .iter()
            .filter(|value| value.consumed)
            .count();
        let expected_unused = provenance
            .supplied_values
            .iter()
            .skip(consumed_count)
            .copied()
            .collect::<Vec<_>>();
        if provenance.unused_values != expected_unused {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::UnusedValuesMismatch,
                &format!("{path}.unusedValues"),
                "Unused roll values do not match the unconsumed stream suffix.",
            );
        }
        for (request_index, request) in provenance.generated_requests.iter().enumerate() {
            if request.sequence != request_index as u32 {
                diagnostic(
                    &mut diagnostics,
                    ReplayRandomnessDiagnosticCode::ReorderedRequest,
                    &format!("{path}.generatedRequests[{request_index}].sequence"),
                    "Generated roll requests must remain in contiguous authority order.",
                );
            }
            if request.minimum > request.maximum {
                diagnostic(
                    &mut diagnostics,
                    ReplayRandomnessDiagnosticCode::InvalidBounds,
                    &format!("{path}.generatedRequests[{request_index}]"),
                    "Generated roll bounds are invalid.",
                );
            }
        }
    }

    for command in &package.commands {
        if !command.command.supplied_roll_stream().is_empty() && !seen.contains(&command.sequence) {
            diagnostic(
                &mut diagnostics,
                ReplayRandomnessDiagnosticCode::MissingProvenance,
                &format!("commands[{}]", command.sequence),
                "Commands with roll inputs require explicit randomness provenance.",
            );
        }
    }

    ReplayRandomnessValidationReport {
        accepted: diagnostics.is_empty(),
        diagnostics,
    }
}

fn next_bounded_value(state: &mut u64, minimum: i32, maximum: i32) -> i32 {
    if minimum > maximum {
        return minimum;
    }
    if *state == 0 {
        *state = 0x9e37_79b9_7f4a_7c15;
    }
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    let value = state.wrapping_mul(0x2545f4914f6cdd1d);
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    minimum + (value % width as u64) as i32
}

fn diagnostic(
    diagnostics: &mut Vec<ReplayRandomnessDiagnostic>,
    code: ReplayRandomnessDiagnosticCode,
    path: &str,
    message: &str,
) {
    diagnostics.push(ReplayRandomnessDiagnostic {
        code,
        path: path.to_string(),
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::tests::recorded_control_package;

    #[test]
    fn generated_provenance_reproduces_the_same_stream() {
        let requests = vec![
            ReplayRollGenerationSpec {
                request_id: "attack".to_string(),
                request_kind: RollRequestKind::AttackRoll,
                minimum: 1,
                maximum: 20,
            },
            ReplayRollGenerationSpec {
                request_id: "damage".to_string(),
                request_kind: RollRequestKind::DamageRoll,
                minimum: 1,
                maximum: 8,
            },
        ];
        let provenance = generate_replay_randomness(0, "seeded.test", 42, &requests);
        assert_eq!(
            reproduce_replay_roll_stream(&provenance),
            Ok(provenance.supplied_values)
        );
    }

    #[test]
    fn missing_extra_and_reordered_evidence_have_precise_codes() {
        let mut package = recorded_control_package();
        package.commands[0].command = crate::ReplayCommand::AutomaticRun(
            rulebench_combat::CombatSessionAutomaticRunSpec::new("run", "Run", "Run", 1, vec![10]),
        );
        assert_eq!(
            validate_replay_randomness(&package).diagnostics[0].code,
            ReplayRandomnessDiagnosticCode::MissingProvenance
        );

        let mut provenance = generate_replay_randomness(
            0,
            "seeded.test",
            42,
            &[ReplayRollGenerationSpec {
                request_id: "attack".to_string(),
                request_kind: RollRequestKind::AttackRoll,
                minimum: 1,
                maximum: 20,
            }],
        );
        provenance.generated_requests[0].sequence = 1;
        provenance.supplied_values = vec![10];
        package.evidence.randomness = vec![provenance.clone(), provenance];
        let codes = validate_replay_randomness(&package)
            .diagnostics
            .into_iter()
            .map(|value| value.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&ReplayRandomnessDiagnosticCode::DuplicateCommand));
        assert!(codes.contains(&ReplayRandomnessDiagnosticCode::ReorderedRequest));
    }
}
