use std::collections::HashSet;

use rulebench_rules::{validate_scenario_content, CombatSessionScriptSpec, RulebenchScenario};

use crate::FixtureGoldenManifest;

/// Data-only Rulebench scenario package.
///
/// A package selects and configures Rust-owned behavior. It never carries
/// callbacks or another rule interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackage {
    pub identity: ScenarioPackageIdentity,
    pub display: ScenarioPackageDisplayMetadata,
    pub ruleset: ScenarioPackageRulesetReference,
    pub content_references: Vec<ScenarioPackageContentReference>,
    pub initial_state: ScenarioPackageInitialState,
    pub scripts: Vec<ScenarioPackageScript>,
    pub expected_evidence: Vec<ScenarioPackageEvidenceExpectation>,
    pub golden_manifest: FixtureGoldenManifest,
}

impl ScenarioPackage {
    pub fn validate(&self) -> Result<(), Vec<ScenarioPackageValidationError>> {
        let mut errors = Vec::new();

        validate_identity(&self.identity, &mut errors);
        validate_display(&self.display, &mut errors);
        validate_ruleset_reference(self, &mut errors);
        validate_content_references(&self.content_references, &mut errors);
        validate_initial_state(&self.initial_state, &mut errors);
        validate_scripts(&self.scripts, &mut errors);
        validate_expected_evidence(&self.expected_evidence, &mut errors);
        validate_golden_manifest(self, &mut errors);

        if !validate_scenario_content(&self.initial_state.scenario).is_empty() {
            errors.push(ScenarioPackageValidationError::InvalidInitialScenarioContent);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageIdentity {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageDisplayMetadata {
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageRulesetReference {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageContentReference {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageInitialState {
    pub scenario: RulebenchScenario,
    pub participant_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageScript {
    pub session_id: String,
    pub script: CombatSessionScriptSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPackageEvidenceExpectation {
    pub id: String,
    pub kind: ScenarioPackageEvidenceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioPackageEvidenceKind {
    CatalogCase,
    SessionTranscript,
    ControlHistory,
    Script,
    AutomaticRun,
    ReplayVerification,
    Receipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioPackageValidationError {
    EmptyPackageId,
    EmptyPackageVersion,
    EmptyDisplayTitle,
    EmptyDisplaySummary,
    EmptyRulesetId,
    EmptyRulesetVersion,
    ReferencedRulesetMissing { ruleset_id: String },
    ReferencedRulesetVersionMismatch { ruleset_id: String },
    SelectedRulesetMismatch { ruleset_id: String },
    EmptyContentReferenceId,
    EmptyContentReferenceVersion { content_id: String },
    DuplicateContentReferenceId { content_id: String },
    EmptyParticipantId,
    DuplicateParticipantId { participant_id: String },
    ReferencedParticipantMissing { participant_id: String },
    MissingScenarioParticipant { participant_id: String },
    EmptyScriptSessionId,
    EmptyScriptId { session_id: String },
    DuplicateScriptId { script_id: String },
    EmptyEvidenceId,
    DuplicateEvidenceId { evidence_id: String },
    GoldenManifestPackageMismatch { package_id: String },
    GoldenArtifactMissingExpectedEvidence { artifact_id: String },
    ExpectedEvidenceMissingGoldenArtifact { evidence_id: String },
    DuplicateGoldenArtifactId { artifact_id: String },
    InvalidInitialScenarioContent,
}

impl ScenarioPackageValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            ScenarioPackageValidationError::EmptyPackageId => "emptyPackageId",
            ScenarioPackageValidationError::EmptyPackageVersion => "emptyPackageVersion",
            ScenarioPackageValidationError::EmptyDisplayTitle => "emptyDisplayTitle",
            ScenarioPackageValidationError::EmptyDisplaySummary => "emptyDisplaySummary",
            ScenarioPackageValidationError::EmptyRulesetId => "emptyRulesetId",
            ScenarioPackageValidationError::EmptyRulesetVersion => "emptyRulesetVersion",
            ScenarioPackageValidationError::ReferencedRulesetMissing { .. } => {
                "referencedRulesetMissing"
            }
            ScenarioPackageValidationError::ReferencedRulesetVersionMismatch { .. } => {
                "referencedRulesetVersionMismatch"
            }
            ScenarioPackageValidationError::SelectedRulesetMismatch { .. } => {
                "selectedRulesetMismatch"
            }
            ScenarioPackageValidationError::EmptyContentReferenceId => "emptyContentReferenceId",
            ScenarioPackageValidationError::EmptyContentReferenceVersion { .. } => {
                "emptyContentReferenceVersion"
            }
            ScenarioPackageValidationError::DuplicateContentReferenceId { .. } => {
                "duplicateContentReferenceId"
            }
            ScenarioPackageValidationError::EmptyParticipantId => "emptyParticipantId",
            ScenarioPackageValidationError::DuplicateParticipantId { .. } => {
                "duplicateParticipantId"
            }
            ScenarioPackageValidationError::ReferencedParticipantMissing { .. } => {
                "referencedParticipantMissing"
            }
            ScenarioPackageValidationError::MissingScenarioParticipant { .. } => {
                "missingScenarioParticipant"
            }
            ScenarioPackageValidationError::EmptyScriptSessionId => "emptyScriptSessionId",
            ScenarioPackageValidationError::EmptyScriptId { .. } => "emptyScriptId",
            ScenarioPackageValidationError::DuplicateScriptId { .. } => "duplicateScriptId",
            ScenarioPackageValidationError::EmptyEvidenceId => "emptyEvidenceId",
            ScenarioPackageValidationError::DuplicateEvidenceId { .. } => "duplicateEvidenceId",
            ScenarioPackageValidationError::GoldenManifestPackageMismatch { .. } => {
                "goldenManifestPackageMismatch"
            }
            ScenarioPackageValidationError::GoldenArtifactMissingExpectedEvidence { .. } => {
                "goldenArtifactMissingExpectedEvidence"
            }
            ScenarioPackageValidationError::ExpectedEvidenceMissingGoldenArtifact { .. } => {
                "expectedEvidenceMissingGoldenArtifact"
            }
            ScenarioPackageValidationError::DuplicateGoldenArtifactId { .. } => {
                "duplicateGoldenArtifactId"
            }
            ScenarioPackageValidationError::InvalidInitialScenarioContent => {
                "invalidInitialScenarioContent"
            }
        }
    }
}

fn validate_identity(
    identity: &ScenarioPackageIdentity,
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    if identity.id.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyPackageId);
    }
    if identity.version.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyPackageVersion);
    }
}

fn validate_display(
    display: &ScenarioPackageDisplayMetadata,
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    if display.title.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyDisplayTitle);
    }
    if display.summary.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyDisplaySummary);
    }
}

fn validate_ruleset_reference(
    package: &ScenarioPackage,
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    if package.ruleset.id.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyRulesetId);
        return;
    }
    if package.ruleset.version.is_empty() {
        errors.push(ScenarioPackageValidationError::EmptyRulesetVersion);
    }

    let Some(ruleset) = package
        .initial_state
        .scenario
        .ruleset_by_id(&package.ruleset.id)
    else {
        errors.push(ScenarioPackageValidationError::ReferencedRulesetMissing {
            ruleset_id: package.ruleset.id.clone(),
        });
        return;
    };

    if ruleset.version != package.ruleset.version {
        errors.push(
            ScenarioPackageValidationError::ReferencedRulesetVersionMismatch {
                ruleset_id: package.ruleset.id.clone(),
            },
        );
    }
    if package.initial_state.scenario.selected_ruleset_id != package.ruleset.id {
        errors.push(ScenarioPackageValidationError::SelectedRulesetMismatch {
            ruleset_id: package.ruleset.id.clone(),
        });
    }
}

fn validate_content_references(
    references: &[ScenarioPackageContentReference],
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    let mut seen_ids = HashSet::new();
    for reference in references {
        if reference.id.is_empty() {
            errors.push(ScenarioPackageValidationError::EmptyContentReferenceId);
            continue;
        }
        if !seen_ids.insert(reference.id.clone()) {
            errors.push(
                ScenarioPackageValidationError::DuplicateContentReferenceId {
                    content_id: reference.id.clone(),
                },
            );
        }
        if reference.version.is_empty() {
            errors.push(
                ScenarioPackageValidationError::EmptyContentReferenceVersion {
                    content_id: reference.id.clone(),
                },
            );
        }
    }
}

fn validate_initial_state(
    initial_state: &ScenarioPackageInitialState,
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    let scenario_participant_ids = initial_state
        .scenario
        .combatants
        .iter()
        .map(|combatant| combatant.id.as_str())
        .collect::<HashSet<_>>();
    let mut declared_participant_ids = HashSet::new();

    for participant_id in &initial_state.participant_ids {
        if participant_id.is_empty() {
            errors.push(ScenarioPackageValidationError::EmptyParticipantId);
            continue;
        }
        if !declared_participant_ids.insert(participant_id.as_str()) {
            errors.push(ScenarioPackageValidationError::DuplicateParticipantId {
                participant_id: participant_id.clone(),
            });
        }
        if !scenario_participant_ids.contains(participant_id.as_str()) {
            errors.push(
                ScenarioPackageValidationError::ReferencedParticipantMissing {
                    participant_id: participant_id.clone(),
                },
            );
        }
    }

    for participant_id in scenario_participant_ids {
        if !declared_participant_ids.contains(participant_id) {
            errors.push(ScenarioPackageValidationError::MissingScenarioParticipant {
                participant_id: participant_id.to_string(),
            });
        }
    }
}

fn validate_scripts(
    scripts: &[ScenarioPackageScript],
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    let mut seen_script_ids = HashSet::new();
    for script in scripts {
        if script.session_id.is_empty() {
            errors.push(ScenarioPackageValidationError::EmptyScriptSessionId);
        }
        if script.script.id.is_empty() {
            errors.push(ScenarioPackageValidationError::EmptyScriptId {
                session_id: script.session_id.clone(),
            });
            continue;
        }
        if !seen_script_ids.insert(script.script.id.clone()) {
            errors.push(ScenarioPackageValidationError::DuplicateScriptId {
                script_id: script.script.id.clone(),
            });
        }
    }
}

fn validate_expected_evidence(
    expected_evidence: &[ScenarioPackageEvidenceExpectation],
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    let mut seen_ids = HashSet::new();
    for expectation in expected_evidence {
        if expectation.id.is_empty() {
            errors.push(ScenarioPackageValidationError::EmptyEvidenceId);
            continue;
        }
        if !seen_ids.insert(expectation.id.clone()) {
            errors.push(ScenarioPackageValidationError::DuplicateEvidenceId {
                evidence_id: expectation.id.clone(),
            });
        }
    }
}

fn validate_golden_manifest(
    package: &ScenarioPackage,
    errors: &mut Vec<ScenarioPackageValidationError>,
) {
    if package.golden_manifest.package_id != package.identity.id {
        errors.push(
            ScenarioPackageValidationError::GoldenManifestPackageMismatch {
                package_id: package.golden_manifest.package_id.clone(),
            },
        );
    }

    let expected_ids = package
        .expected_evidence
        .iter()
        .map(|expectation| expectation.id.as_str())
        .collect::<HashSet<_>>();
    let mut golden_ids = HashSet::new();
    for artifact in &package.golden_manifest.artifacts {
        if !golden_ids.insert(artifact.id.as_str()) {
            errors.push(ScenarioPackageValidationError::DuplicateGoldenArtifactId {
                artifact_id: artifact.id.clone(),
            });
        }
        if !expected_ids.contains(artifact.id.as_str()) {
            errors.push(
                ScenarioPackageValidationError::GoldenArtifactMissingExpectedEvidence {
                    artifact_id: artifact.id.clone(),
                },
            );
        }
    }

    for evidence_id in expected_ids {
        if !golden_ids.contains(evidence_id) {
            errors.push(
                ScenarioPackageValidationError::ExpectedEvidenceMissingGoldenArtifact {
                    evidence_id: evidence_id.to_string(),
                },
            );
        }
    }
}
