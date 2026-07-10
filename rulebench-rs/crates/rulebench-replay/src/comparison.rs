use crate::{ReplayArchiveEntry, ReplayPackage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayComparisonDifferenceCode {
    PackageVersion,
    Content,
    Ruleset,
    CommandCount,
    Command,
    Decision,
    AcceptedEvents,
    CommandAudit,
    Rolls,
    Trace,
    StateBeforeFingerprint,
    StateAfterFingerprint,
    FinalStateFingerprint,
}

impl ReplayComparisonDifferenceCode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PackageVersion => "replayPackageVersionMismatch",
            Self::Content => "replayContentMismatch",
            Self::Ruleset => "replayRulesetMismatch",
            Self::CommandCount => "replayCommandCountMismatch",
            Self::Command => "replayCommandMismatch",
            Self::Decision => "replayDecisionMismatch",
            Self::AcceptedEvents => "replayAcceptedEventsMismatch",
            Self::CommandAudit => "replayCommandAuditMismatch",
            Self::Rolls => "replayRollsMismatch",
            Self::Trace => "replayTraceMismatch",
            Self::StateBeforeFingerprint => "replayStateBeforeFingerprintMismatch",
            Self::StateAfterFingerprint => "replayStateAfterFingerprintMismatch",
            Self::FinalStateFingerprint => "replayFinalStateFingerprintMismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayComparisonDifference {
    pub code: ReplayComparisonDifferenceCode,
    pub path: String,
    pub command_sequence: Option<u32>,
    pub command_id: Option<String>,
    pub expected_summary: String,
    pub actual_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayComparisonReadout {
    pub matches: bool,
    pub expected_package_id: String,
    pub actual_package_id: String,
    pub compared_command_count: u32,
    pub first_difference: Option<ReplayComparisonDifference>,
    pub differences: Vec<ReplayComparisonDifference>,
}

pub fn compare_replay_archive_entries(
    expected: &ReplayArchiveEntry,
    actual: &ReplayArchiveEntry,
) -> ReplayComparisonReadout {
    compare_replay_packages(&expected.package, &actual.package)
}

pub fn compare_replay_packages(
    expected: &ReplayPackage,
    actual: &ReplayPackage,
) -> ReplayComparisonReadout {
    let mut differences = Vec::new();
    compare(
        &mut differences,
        ReplayComparisonDifferenceCode::PackageVersion,
        "packageVersion",
        None,
        None,
        &expected.package_version,
        &actual.package_version,
    );
    compare(
        &mut differences,
        ReplayComparisonDifferenceCode::Content,
        "initialSession.scenario.contentPackSet",
        None,
        None,
        &expected.initial_session.scenario.content_pack_set,
        &actual.initial_session.scenario.content_pack_set,
    );
    compare(
        &mut differences,
        ReplayComparisonDifferenceCode::Ruleset,
        "ruleset",
        None,
        None,
        &expected.ruleset,
        &actual.ruleset,
    );
    compare(
        &mut differences,
        ReplayComparisonDifferenceCode::CommandCount,
        "commands.length",
        None,
        None,
        &expected.commands.len(),
        &actual.commands.len(),
    );

    for (index, (expected_command, actual_command)) in
        expected.commands.iter().zip(&actual.commands).enumerate()
    {
        let sequence = Some(index as u32);
        let command_id = Some(expected_command.id.clone());
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::Command,
            &format!("commands[{index}].command"),
            sequence,
            command_id.clone(),
            &(expected_command.id.clone(), &expected_command.command),
            &(actual_command.id.clone(), &actual_command.command),
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::Decision,
            &format!("commands[{index}].expected.decision"),
            sequence,
            command_id.clone(),
            &(
                expected_command.expected.accepted,
                &expected_command.expected.decision_code,
            ),
            &(
                actual_command.expected.accepted,
                &actual_command.expected.decision_code,
            ),
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::AcceptedEvents,
            &format!("commands[{index}].expected.acceptedEvents"),
            sequence,
            command_id.clone(),
            &expected_command.expected.accepted_events,
            &actual_command.expected.accepted_events,
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::CommandAudit,
            &format!("commands[{index}].expected.commandAudit"),
            sequence,
            command_id.clone(),
            &expected_command.expected.command_audit,
            &actual_command.expected.command_audit,
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::Rolls,
            &format!("commands[{index}].expected.rolls"),
            sequence,
            command_id.clone(),
            &expected_command.expected.rolls,
            &actual_command.expected.rolls,
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::Trace,
            &format!("commands[{index}].expected.trace"),
            sequence,
            command_id.clone(),
            &expected_command.expected.trace,
            &actual_command.expected.trace,
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::StateBeforeFingerprint,
            &format!("commands[{index}].expected.stateBeforeFingerprint"),
            sequence,
            command_id.clone(),
            &expected_command.expected.state_before_fingerprint,
            &actual_command.expected.state_before_fingerprint,
        );
        compare(
            &mut differences,
            ReplayComparisonDifferenceCode::StateAfterFingerprint,
            &format!("commands[{index}].expected.stateAfterFingerprint"),
            sequence,
            command_id,
            &expected_command.expected.state_after_fingerprint,
            &actual_command.expected.state_after_fingerprint,
        );
    }

    compare(
        &mut differences,
        ReplayComparisonDifferenceCode::FinalStateFingerprint,
        "finalStateFingerprint",
        None,
        None,
        &expected.final_state_fingerprint,
        &actual.final_state_fingerprint,
    );

    ReplayComparisonReadout {
        matches: differences.is_empty(),
        expected_package_id: expected.id.clone(),
        actual_package_id: actual.id.clone(),
        compared_command_count: expected.commands.len().min(actual.commands.len()) as u32,
        first_difference: differences.first().cloned(),
        differences,
    }
}

fn compare<T: std::fmt::Debug + PartialEq>(
    differences: &mut Vec<ReplayComparisonDifference>,
    code: ReplayComparisonDifferenceCode,
    path: &str,
    command_sequence: Option<u32>,
    command_id: Option<String>,
    expected: &T,
    actual: &T,
) {
    if expected != actual {
        differences.push(ReplayComparisonDifference {
            code,
            path: path.to_string(),
            command_sequence,
            command_id,
            expected_summary: format!("{expected:?}"),
            actual_summary: format!("{actual:?}"),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::tests::recorded_control_package;
    use rulebench_combat::{
        CombatControlCommandSpec, DomainEvent, RollConsumptionEntry, RollRequestKind, TraceEntry,
        TracePhase, TraceStatus,
    };

    #[test]
    fn identical_packages_have_no_differences() {
        let package = recorded_control_package();
        assert!(compare_replay_packages(&package, &package).matches);
    }

    #[test]
    fn every_review_dimension_has_a_machine_readable_code_and_path() {
        let expected = recorded_control_package();
        let cases = vec![
            changed(&expected, |package| {
                package.commands[0].command =
                    crate::ReplayCommand::Control(CombatControlCommandSpec::explicit_end())
            }),
            changed(&expected, |package| {
                package.commands[0].expected.decision_code = "changed".to_string()
            }),
            changed(&expected, |package| {
                package.commands[0]
                    .expected
                    .accepted_events
                    .push(DomainEvent::ActionUsed {
                        actor_id: "a".to_string(),
                        action_id: "x".to_string(),
                        target_id: "b".to_string(),
                    })
            }),
            changed(&expected, |package| {
                package.commands[0]
                    .expected
                    .rolls
                    .push(RollConsumptionEntry {
                        sequence: 1,
                        request_kind: RollRequestKind::AttackRoll,
                        supplied_value: Some(10),
                        consumed: true,
                        reason: "test".to_string(),
                    })
            }),
            changed(&expected, |package| {
                package.commands[0].expected.trace.push(TraceEntry::new(
                    1,
                    TracePhase::Validation,
                    TraceStatus::Info,
                    "test",
                    "test",
                ))
            }),
            changed(&expected, |package| {
                package.commands[0].expected.state_after_fingerprint.value = "changed".to_string()
            }),
            changed(&expected, |package| {
                package.final_state_fingerprint.value = "changed".to_string()
            }),
        ];
        let expected_codes = [
            ReplayComparisonDifferenceCode::Command,
            ReplayComparisonDifferenceCode::Decision,
            ReplayComparisonDifferenceCode::AcceptedEvents,
            ReplayComparisonDifferenceCode::Rolls,
            ReplayComparisonDifferenceCode::Trace,
            ReplayComparisonDifferenceCode::StateAfterFingerprint,
            ReplayComparisonDifferenceCode::FinalStateFingerprint,
        ];

        for (actual, expected_code) in cases.into_iter().zip(expected_codes) {
            let readout = compare_replay_packages(&expected, &actual);
            let difference = readout.first_difference.expect("case differs");
            assert_eq!(difference.code, expected_code);
            assert!(!difference.code.code().is_empty());
            assert!(!difference.path.is_empty());
        }
    }

    #[test]
    fn comparison_preserves_first_and_later_differences() {
        let expected = recorded_control_package();
        let mut actual = expected.clone();
        actual.commands[1].expected.decision_code = "changed".to_string();
        actual.final_state_fingerprint.value = "changed".to_string();

        let readout = compare_replay_packages(&expected, &actual);

        assert_eq!(readout.differences.len(), 2);
        assert_eq!(
            readout.first_difference.map(|value| value.path),
            Some("commands[1].expected.decision".to_string())
        );
    }

    fn changed(package: &ReplayPackage, change: impl FnOnce(&mut ReplayPackage)) -> ReplayPackage {
        let mut changed = package.clone();
        change(&mut changed);
        changed
    }
}
