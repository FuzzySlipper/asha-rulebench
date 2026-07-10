use crate::{
    validate_replay_package, ReplayCommand, ReplayMismatchDimension::*, ReplayPackage,
    ReplayPackageValidationReport, ReplayStepEvidence,
};
use rulebench_combat::{
    CombatSessionAutomaticStepExecutionReadout, CombatSessionState, DomainEvent,
    RollConsumptionEntry, StateFingerprint, TraceEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayVerificationDecisionKind {
    Verified,
    InvalidPackage,
    MismatchedEvidence,
}

impl ReplayVerificationDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::InvalidPackage => "invalidPackage",
            Self::MismatchedEvidence => "mismatchedEvidence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayMismatchDimension {
    Decision,
    StateBeforeFingerprint,
    AcceptedEvents,
    CommandAudit,
    Rolls,
    Trace,
    StateAfterFingerprint,
    FinalStateFingerprint,
}

impl ReplayMismatchDimension {
    pub const fn code(self) -> &'static str {
        match self {
            Decision => "decision",
            StateBeforeFingerprint => "stateBeforeFingerprint",
            AcceptedEvents => "acceptedEvents",
            CommandAudit => "commandAudit",
            Rolls => "rolls",
            Trace => "trace",
            StateAfterFingerprint => "stateAfterFingerprint",
            FinalStateFingerprint => "finalStateFingerprint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMismatch {
    pub command_sequence: Option<u32>,
    pub command_id: Option<String>,
    pub dimension: ReplayMismatchDimension,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayVerificationReadout {
    pub accepted: bool,
    pub decision_kind: ReplayVerificationDecisionKind,
    pub package_validation: ReplayPackageValidationReport,
    pub verified_step_count: u32,
    pub mismatch: Option<ReplayMismatch>,
    pub final_state_fingerprint: Option<StateFingerprint>,
    pub finalized: bool,
}

pub fn verify_replay_package(package: &ReplayPackage) -> ReplayVerificationReadout {
    let package_validation = validate_replay_package(package);
    if !package_validation.accepted {
        return ReplayVerificationReadout {
            accepted: false,
            decision_kind: ReplayVerificationDecisionKind::InvalidPackage,
            package_validation,
            verified_step_count: 0,
            mismatch: None,
            final_state_fingerprint: None,
            finalized: false,
        };
    }

    let mut session = CombatSessionState::new(
        package.initial_session.session.id.clone(),
        package.initial_session.scenario.clone(),
    );

    for command in &package.commands {
        let actual = execute_command(&mut session, &command.command);
        if let Some(dimension) = first_step_mismatch(&command.expected, &actual) {
            return ReplayVerificationReadout {
                accepted: false,
                decision_kind: ReplayVerificationDecisionKind::MismatchedEvidence,
                package_validation,
                verified_step_count: command.sequence,
                mismatch: Some(ReplayMismatch {
                    command_sequence: Some(command.sequence),
                    command_id: Some(command.id.clone()),
                    dimension,
                    reason: format!(
                        "Replay command {} first differed in {}.",
                        command.id,
                        dimension.code()
                    ),
                }),
                final_state_fingerprint: Some(actual.state_after_fingerprint),
                finalized: false,
            };
        }
    }

    let final_state_fingerprint = session.snapshot().current_state_fingerprint;
    let finalized = session.finalization().is_some();
    if final_state_fingerprint != package.final_state_fingerprint {
        return ReplayVerificationReadout {
            accepted: false,
            decision_kind: ReplayVerificationDecisionKind::MismatchedEvidence,
            package_validation,
            verified_step_count: package.commands.len() as u32,
            mismatch: Some(ReplayMismatch {
                command_sequence: None,
                command_id: None,
                dimension: FinalStateFingerprint,
                reason: "Replay final state fingerprint differed after all commands.".to_string(),
            }),
            final_state_fingerprint: Some(final_state_fingerprint),
            finalized,
        };
    }

    ReplayVerificationReadout {
        accepted: true,
        decision_kind: ReplayVerificationDecisionKind::Verified,
        package_validation,
        verified_step_count: package.commands.len() as u32,
        mismatch: None,
        final_state_fingerprint: Some(final_state_fingerprint),
        finalized,
    }
}

pub(crate) fn execute_command(
    session: &mut CombatSessionState,
    command: &ReplayCommand,
) -> ReplayStepEvidence {
    let before = session.snapshot();
    let audit_start = before.audit_log.len();
    let (accepted, decision_code, events, rolls, trace) = match command {
        ReplayCommand::Intent(spec) => {
            let readout = session.submit_intent_command(spec.clone());
            (
                readout.receipt.accepted,
                readout.audit_entry.decision_kind.code().to_string(),
                readout.receipt.events,
                readout.receipt.roll_consumption,
                readout.receipt.trace,
            )
        }
        ReplayCommand::Control(spec) => {
            let readout = session.submit_control_command(spec.clone());
            (
                readout.accepted,
                readout.decision_kind.code().to_string(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        }
        ReplayCommand::SelectedCandidate(spec) => {
            let readout = session.submit_candidate_command(spec.clone());
            let (events, rolls, trace) = readout
                .submitted_step
                .map(|step| {
                    (
                        step.receipt.events,
                        step.receipt.roll_consumption,
                        step.receipt.trace,
                    )
                })
                .unwrap_or_default();
            (
                readout.selection.accepted,
                readout.selection.decision_kind.code().to_string(),
                events,
                rolls,
                trace,
            )
        }
        ReplayCommand::AutomaticStep(spec) => {
            let readout = session.submit_automatic_step(spec.clone());
            let (events, rolls, trace) = automatic_step_evidence(&readout);
            (
                readout.plan.accepted,
                readout.plan.decision_kind.code().to_string(),
                events,
                rolls,
                trace,
            )
        }
        ReplayCommand::AutomaticRun(spec) => {
            let readout = session.run_automatic_combat(spec.clone());
            let mut events = Vec::new();
            let mut rolls = Vec::new();
            let mut trace = Vec::new();
            for step in &readout.steps {
                let (step_events, step_rolls, step_trace) = automatic_step_evidence(step);
                events.extend(step_events);
                rolls.extend(step_rolls);
                trace.extend(step_trace);
            }
            (
                readout.accepted,
                readout.decision_kind.code().to_string(),
                events,
                rolls,
                trace,
            )
        }
        ReplayCommand::Equipment(spec) => {
            let readout = session.submit_equipment_command(spec.clone());
            (
                readout.accepted,
                readout.decision_kind.code().to_string(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        }
        ReplayCommand::Reaction(spec) => {
            let readout = session.submit_reaction_command(spec.clone());
            (
                readout.accepted,
                readout.decision_kind.code().to_string(),
                Vec::new(),
                Vec::new(),
                readout.trace,
            )
        }
    };
    let after = session.snapshot();

    ReplayStepEvidence {
        accepted,
        decision_code,
        state_before_fingerprint: before.current_state_fingerprint,
        state_after_fingerprint: after.current_state_fingerprint,
        accepted_events: events,
        command_audit: after.audit_log[audit_start..].to_vec(),
        rolls,
        trace,
    }
}

fn automatic_step_evidence(
    readout: &CombatSessionAutomaticStepExecutionReadout,
) -> (Vec<DomainEvent>, Vec<RollConsumptionEntry>, Vec<TraceEntry>) {
    readout
        .auto_candidate
        .as_ref()
        .and_then(|candidate| candidate.submitted_step.as_ref())
        .map(|step| {
            (
                step.receipt.events.clone(),
                step.receipt.roll_consumption.clone(),
                step.receipt.trace.clone(),
            )
        })
        .unwrap_or_default()
}

fn first_step_mismatch(
    expected: &ReplayStepEvidence,
    actual: &ReplayStepEvidence,
) -> Option<ReplayMismatchDimension> {
    if expected.accepted != actual.accepted || expected.decision_code != actual.decision_code {
        Some(Decision)
    } else if expected.state_before_fingerprint != actual.state_before_fingerprint {
        Some(StateBeforeFingerprint)
    } else if expected.accepted_events != actual.accepted_events {
        Some(AcceptedEvents)
    } else if expected.command_audit != actual.command_audit {
        Some(CommandAudit)
    } else if expected.rolls != actual.rolls {
        Some(Rolls)
    } else if expected.trace != actual.trace {
        Some(Trace)
    } else if expected.state_after_fingerprint != actual.state_after_fingerprint {
        Some(StateAfterFingerprint)
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::package_validation::tests::valid_package;
    use crate::{ReplayCommand, ReplayCommandRecord};
    use rulebench_combat::{CombatControlCommandSpec, RollConsumptionEntry, RollRequestKind};

    #[test]
    fn replay_verifies_accepted_and_rejected_no_op_commands() {
        let package = recorded_control_package();

        let readout = verify_replay_package(&package);

        assert!(readout.accepted);
        assert_eq!(readout.verified_step_count, 3);
        assert!(readout.finalized);
        assert_eq!(readout.mismatch, None);
    }

    #[test]
    fn replay_reports_the_first_step_and_dimension() {
        let mut package = recorded_control_package();
        package.commands[1].expected.decision_code = "accepted".to_string();

        let readout = verify_replay_package(&package);

        assert!(!readout.accepted);
        assert_eq!(readout.verified_step_count, 1);
        assert_eq!(
            readout.mismatch,
            Some(ReplayMismatch {
                command_sequence: Some(1),
                command_id: Some("start-again".to_string()),
                dimension: ReplayMismatchDimension::Decision,
                reason: "Replay command start-again first differed in decision.".to_string(),
            })
        );
    }

    #[test]
    fn replay_does_not_substitute_missing_roll_evidence() {
        let mut package = recorded_control_package();
        package.commands[0]
            .expected
            .rolls
            .push(RollConsumptionEntry {
                sequence: 1,
                request_kind: RollRequestKind::AttackRoll,
                supplied_value: Some(20),
                consumed: true,
                reason: "Recorded roll that the command did not supply.".to_string(),
            });

        let readout = verify_replay_package(&package);

        assert_eq!(
            readout.mismatch.as_ref().map(|value| value.dimension),
            Some(ReplayMismatchDimension::Rolls)
        );
    }

    pub(crate) fn recorded_control_package() -> ReplayPackage {
        let mut package = valid_package();
        package.commands = vec![
            ReplayCommandRecord {
                sequence: 0,
                id: "start".to_string(),
                command: ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
                expected: package.commands[0].expected.clone(),
            },
            ReplayCommandRecord {
                sequence: 1,
                id: "start-again".to_string(),
                command: ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
                expected: package.commands[0].expected.clone(),
            },
            ReplayCommandRecord {
                sequence: 2,
                id: "end".to_string(),
                command: ReplayCommand::Control(CombatControlCommandSpec::explicit_end()),
                expected: package.commands[0].expected.clone(),
            },
        ];

        let mut session = CombatSessionState::new(
            package.initial_session.session.id.clone(),
            package.initial_session.scenario.clone(),
        );
        for command in &mut package.commands {
            command.expected = execute_command(&mut session, &command.command);
        }
        package.final_state_fingerprint = session.snapshot().current_state_fingerprint;
        package
    }
}
