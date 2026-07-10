use rulebench_rules::{
    inspect_replay_package, ReplayArchiveError, ReplayArchiveMetadata, ReplayCommandInspection,
    ReplayComparisonDifference, ReplayComparisonReadout, ReplayMismatch, ReplayPackage,
    ReplayStepEvidence, ReplayVerificationReadout, StateFingerprint,
};
use serde::{Deserialize, Serialize};

use crate::{
    LiveAuditEntryDto, LiveDomainEventDto, LiveRollEvidenceDto, LiveSessionSnapshotDto,
    LiveTraceEntryDto,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayStateFingerprintDto {
    pub algorithm: String,
    pub value: String,
}

impl From<&StateFingerprint> for ReplayStateFingerprintDto {
    fn from(value: &StateFingerprint) -> Self {
        Self {
            algorithm: value.algorithm.clone(),
            value: value.value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayArchiveMetadataDto {
    pub package_id: String,
    pub session_id: String,
    pub scenario_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub completed_at: String,
}

impl From<&ReplayArchiveMetadata> for ReplayArchiveMetadataDto {
    fn from(value: &ReplayArchiveMetadata) -> Self {
        Self {
            package_id: value.package_id.clone(),
            session_id: value.session_id.clone(),
            scenario_id: value.scenario_id.clone(),
            ruleset_id: value.ruleset_id.clone(),
            ruleset_version: value.ruleset_version.clone(),
            completed_at: value.completed_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayPackageReviewDto {
    pub package_version: String,
    pub package_id: String,
    pub session_id: String,
    pub scenario_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub command_count: u32,
    pub final_state_fingerprint: ReplayStateFingerprintDto,
    pub fingerprint_kind: String,
    pub narration_title: Option<String>,
    pub narration_summary: Option<String>,
    pub commands: Vec<ReplayCommandReviewDto>,
}

impl From<&ReplayPackage> for ReplayPackageReviewDto {
    fn from(value: &ReplayPackage) -> Self {
        Self {
            package_version: value.package_version.clone(),
            package_id: value.id.clone(),
            session_id: value.initial_session.session.id.clone(),
            scenario_id: value.initial_session.scenario.metadata.id.clone(),
            ruleset_id: value.ruleset.ruleset_id.clone(),
            ruleset_version: value.ruleset.ruleset_version.clone(),
            command_count: value.commands.len() as u32,
            final_state_fingerprint: (&value.final_state_fingerprint).into(),
            fingerprint_kind: value.fingerprint_kind.clone(),
            narration_title: value.narration.as_ref().map(|item| item.title.clone()),
            narration_summary: value.narration.as_ref().map(|item| item.summary.clone()),
            commands: inspect_replay_package(value)
                .commands
                .iter()
                .map(ReplayCommandReviewDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayCommandReviewDto {
    pub sequence: u32,
    pub id: String,
    pub command_kind: String,
    pub supplied_roll_stream: Vec<i32>,
    pub narration_summary: Option<String>,
    pub expected: ReplayStepEvidenceDto,
    pub actual: ReplayStepEvidenceDto,
    pub snapshot: LiveSessionSnapshotDto,
}

impl From<&ReplayCommandInspection> for ReplayCommandReviewDto {
    fn from(value: &ReplayCommandInspection) -> Self {
        Self {
            sequence: value.sequence,
            id: value.id.clone(),
            command_kind: value.command_kind.clone(),
            supplied_roll_stream: value.supplied_roll_stream.clone(),
            narration_summary: value.narration_summary.clone(),
            expected: ReplayStepEvidenceDto::from(&value.expected),
            actual: ReplayStepEvidenceDto::from(&value.actual),
            snapshot: LiveSessionSnapshotDto::from(&value.snapshot),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayStepEvidenceDto {
    pub accepted: bool,
    pub decision_code: String,
    pub state_before_fingerprint: ReplayStateFingerprintDto,
    pub state_after_fingerprint: ReplayStateFingerprintDto,
    pub accepted_events: Vec<LiveDomainEventDto>,
    pub command_audit: Vec<LiveAuditEntryDto>,
    pub rolls: Vec<LiveRollEvidenceDto>,
    pub trace: Vec<LiveTraceEntryDto>,
}

impl From<&ReplayStepEvidence> for ReplayStepEvidenceDto {
    fn from(value: &ReplayStepEvidence) -> Self {
        Self {
            accepted: value.accepted,
            decision_code: value.decision_code.clone(),
            state_before_fingerprint: ReplayStateFingerprintDto::from(
                &value.state_before_fingerprint,
            ),
            state_after_fingerprint: ReplayStateFingerprintDto::from(
                &value.state_after_fingerprint,
            ),
            accepted_events: value
                .accepted_events
                .iter()
                .map(LiveDomainEventDto::from)
                .collect(),
            command_audit: value
                .command_audit
                .iter()
                .map(LiveAuditEntryDto::from)
                .collect(),
            rolls: value.rolls.iter().map(LiveRollEvidenceDto::from).collect(),
            trace: value.trace.iter().map(LiveTraceEntryDto::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayMismatchDto {
    pub command_sequence: Option<u32>,
    pub command_id: Option<String>,
    pub dimension: String,
    pub reason: String,
}

impl From<&ReplayMismatch> for ReplayMismatchDto {
    fn from(value: &ReplayMismatch) -> Self {
        Self {
            command_sequence: value.command_sequence,
            command_id: value.command_id.clone(),
            dimension: value.dimension.code().to_string(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayVerificationReadoutDto {
    pub accepted: bool,
    pub decision_kind: String,
    pub verified_step_count: u32,
    pub mismatch: Option<ReplayMismatchDto>,
    pub final_state_fingerprint: Option<ReplayStateFingerprintDto>,
    pub finalized: bool,
}

impl From<&ReplayVerificationReadout> for ReplayVerificationReadoutDto {
    fn from(value: &ReplayVerificationReadout) -> Self {
        Self {
            accepted: value.accepted,
            decision_kind: value.decision_kind.code().to_string(),
            verified_step_count: value.verified_step_count,
            mismatch: value.mismatch.as_ref().map(ReplayMismatchDto::from),
            final_state_fingerprint: value
                .final_state_fingerprint
                .as_ref()
                .map(ReplayStateFingerprintDto::from),
            finalized: value.finalized,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayComparisonDifferenceDto {
    pub code: String,
    pub path: String,
    pub command_sequence: Option<u32>,
    pub command_id: Option<String>,
    pub expected_summary: String,
    pub actual_summary: String,
}

impl From<&ReplayComparisonDifference> for ReplayComparisonDifferenceDto {
    fn from(value: &ReplayComparisonDifference) -> Self {
        Self {
            code: value.code.code().to_string(),
            path: value.path.clone(),
            command_sequence: value.command_sequence,
            command_id: value.command_id.clone(),
            expected_summary: value.expected_summary.clone(),
            actual_summary: value.actual_summary.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayComparisonReadoutDto {
    pub matches: bool,
    pub expected_package_id: String,
    pub actual_package_id: String,
    pub compared_command_count: u32,
    pub first_difference: Option<ReplayComparisonDifferenceDto>,
    pub differences: Vec<ReplayComparisonDifferenceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayComparisonRequestDto {
    pub expected_package_id: String,
    pub actual_package_id: String,
}

impl From<&ReplayComparisonReadout> for ReplayComparisonReadoutDto {
    fn from(value: &ReplayComparisonReadout) -> Self {
        Self {
            matches: value.matches,
            expected_package_id: value.expected_package_id.clone(),
            actual_package_id: value.actual_package_id.clone(),
            compared_command_count: value.compared_command_count,
            first_difference: value
                .first_difference
                .as_ref()
                .map(ReplayComparisonDifferenceDto::from),
            differences: value
                .differences
                .iter()
                .map(ReplayComparisonDifferenceDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayArchiveErrorDto {
    pub kind: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl From<&ReplayArchiveError> for ReplayArchiveErrorDto {
    fn from(value: &ReplayArchiveError) -> Self {
        let kind = match value {
            ReplayArchiveError::InvalidPackage | ReplayArchiveError::CombatNotFinalized => {
                "invalidPackage"
            }
            ReplayArchiveError::Storage(_) => "storage",
            ReplayArchiveError::UnknownPackage { .. } => "notFound",
            ReplayArchiveError::CorruptPackage { .. } => "corrupt",
            ReplayArchiveError::UnsupportedPackageVersion { .. } => "unsupportedVersion",
        };
        Self {
            kind: kind.to_string(),
            code: value.code().to_string(),
            message: format!("{value:?}"),
            retryable: matches!(value, ReplayArchiveError::Storage(_)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_rules::{
        ReplayComparisonDifferenceCode, ReplayMismatchDimension, ReplayVerificationDecisionKind,
    };

    #[test]
    fn comparison_mapping_preserves_machine_codes_and_paths() {
        let difference = ReplayComparisonDifference {
            code: ReplayComparisonDifferenceCode::Rolls,
            path: "commands[2].expected.rolls".to_string(),
            command_sequence: Some(2),
            command_id: Some("third".to_string()),
            expected_summary: "[10]".to_string(),
            actual_summary: "[11]".to_string(),
        };
        let readout = ReplayComparisonReadout {
            matches: false,
            expected_package_id: "expected".to_string(),
            actual_package_id: "actual".to_string(),
            compared_command_count: 3,
            first_difference: Some(difference.clone()),
            differences: vec![difference],
        };

        let dto = ReplayComparisonReadoutDto::from(&readout);

        assert_eq!(dto.differences[0].code, "replayRollsMismatch");
        assert_eq!(dto.differences[0].path, "commands[2].expected.rolls");
    }

    #[test]
    fn verification_mapping_preserves_authority_mismatch() {
        let readout = ReplayVerificationReadout {
            accepted: false,
            decision_kind: ReplayVerificationDecisionKind::MismatchedEvidence,
            package_validation: rulebench_rules::ReplayPackageValidationReport {
                accepted: true,
                diagnostics: Vec::new(),
            },
            verified_step_count: 2,
            mismatch: Some(ReplayMismatch {
                command_sequence: Some(2),
                command_id: Some("third".to_string()),
                dimension: ReplayMismatchDimension::Rolls,
                reason: "Rolls differed.".to_string(),
            }),
            final_state_fingerprint: None,
            finalized: false,
        };

        let dto = ReplayVerificationReadoutDto::from(&readout);

        assert_eq!(dto.decision_kind, "mismatchedEvidence");
        assert_eq!(dto.mismatch.expect("mismatch").dimension, "rolls");
    }

    #[test]
    fn compatibility_and_storage_failures_remain_classified() {
        let version = ReplayArchiveErrorDto::from(&ReplayArchiveError::UnsupportedPackageVersion {
            version: "2.0.0".to_string(),
        });
        let storage = ReplayArchiveErrorDto::from(&ReplayArchiveError::Storage(
            rulebench_rules::ReplayArchiveStorageError::WriteFailed {
                package_id: "replay".to_string(),
            },
        ));

        assert_eq!(version.kind, "unsupportedVersion");
        assert!(!version.retryable);
        assert_eq!(storage.kind, "storage");
        assert!(storage.retryable);
    }
}
