use std::collections::BTreeSet;

use crate::{ReplayPackage, REPLAY_PACKAGE_FINGERPRINT_KIND, REPLAY_PACKAGE_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayPackageDiagnosticCode {
    UnsupportedPackageVersion,
    EmptyPackageId,
    EmptySessionId,
    MissingContentPackSet,
    InvalidContentPackSet,
    MissingSelectedRuleset,
    IncompatibleRulesetProvenance,
    EmptyCommands,
    InvalidCommandSequence,
    EmptyCommandId,
    EmptyFinalStateFingerprint,
    InvalidFingerprintKind,
    UnknownEvidenceCommand,
    InvalidRandomnessProvenance,
}

impl ReplayPackageDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedPackageVersion => "unsupportedReplayPackageVersion",
            Self::EmptyPackageId => "emptyReplayPackageId",
            Self::EmptySessionId => "emptyReplaySessionId",
            Self::MissingContentPackSet => "missingReplayContentPackSet",
            Self::InvalidContentPackSet => "invalidReplayContentPackSet",
            Self::MissingSelectedRuleset => "missingReplaySelectedRuleset",
            Self::IncompatibleRulesetProvenance => "incompatibleReplayRulesetProvenance",
            Self::EmptyCommands => "emptyReplayCommands",
            Self::InvalidCommandSequence => "invalidReplayCommandSequence",
            Self::EmptyCommandId => "emptyReplayCommandId",
            Self::EmptyFinalStateFingerprint => "emptyReplayFinalStateFingerprint",
            Self::InvalidFingerprintKind => "invalidReplayFingerprintKind",
            Self::UnknownEvidenceCommand => "unknownReplayEvidenceCommand",
            Self::InvalidRandomnessProvenance => "invalidReplayRandomnessProvenance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPackageDiagnostic {
    pub code: ReplayPackageDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPackageValidationReport {
    pub accepted: bool,
    pub diagnostics: Vec<ReplayPackageDiagnostic>,
}

pub fn validate_replay_package(package: &ReplayPackage) -> ReplayPackageValidationReport {
    let mut diagnostics = Vec::new();

    if package.package_version != REPLAY_PACKAGE_VERSION {
        push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::UnsupportedPackageVersion,
            "packageVersion",
            format!(
                "Replay package version {} is unsupported; expected {REPLAY_PACKAGE_VERSION}.",
                package.package_version
            ),
        );
    }
    required(
        &mut diagnostics,
        &package.id,
        ReplayPackageDiagnosticCode::EmptyPackageId,
        "id",
        "Replay package id is required.",
    );
    required(
        &mut diagnostics,
        &package.initial_session.session.id,
        ReplayPackageDiagnosticCode::EmptySessionId,
        "initialSession.session.id",
        "Replay session id is required.",
    );

    match &package.initial_session.scenario.content_pack_set {
        None => push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::MissingContentPackSet,
            "initialSession.scenario.contentPackSet",
            "Replay requires an exact canonical content pack set reference.",
        ),
        Some(reference) if !reference.is_self_consistent() => push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::InvalidContentPackSet,
            "initialSession.scenario.contentPackSet",
            "Replay content pack set reference is not self-consistent.",
        ),
        Some(_) => {}
    }

    match package.initial_session.scenario.selected_ruleset() {
        None => push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::MissingSelectedRuleset,
            "initialSession.scenario.selectedRulesetId",
            "Replay scenario must contain its selected ruleset.",
        ),
        Some(ruleset)
            if ruleset
                .validate_artifact_provenance(&package.ruleset)
                .is_err() =>
        {
            push(
                &mut diagnostics,
                ReplayPackageDiagnosticCode::IncompatibleRulesetProvenance,
                "ruleset",
                "Replay ruleset provenance is incompatible with the selected ruleset.",
            )
        }
        Some(_) => {}
    }

    if package.commands.is_empty() {
        push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::EmptyCommands,
            "commands",
            "Replay package must contain at least one command.",
        );
    }
    let mut command_sequences = BTreeSet::new();
    for (index, command) in package.commands.iter().enumerate() {
        let expected = index as u32;
        if command.sequence != expected || !command_sequences.insert(command.sequence) {
            push(
                &mut diagnostics,
                ReplayPackageDiagnosticCode::InvalidCommandSequence,
                &format!("commands[{index}].sequence"),
                format!(
                    "Replay command sequence must be contiguous from zero; expected {expected}."
                ),
            );
        }
        required(
            &mut diagnostics,
            &command.id,
            ReplayPackageDiagnosticCode::EmptyCommandId,
            &format!("commands[{index}].id"),
            "Replay command id is required.",
        );
    }

    required(
        &mut diagnostics,
        &package.final_state_fingerprint.value,
        ReplayPackageDiagnosticCode::EmptyFinalStateFingerprint,
        "finalStateFingerprint.value",
        "Replay final state fingerprint is required.",
    );
    if package.fingerprint_kind != REPLAY_PACKAGE_FINGERPRINT_KIND {
        push(&mut diagnostics, ReplayPackageDiagnosticCode::InvalidFingerprintKind, "fingerprintKind", "Replay fingerprints must be identified as deterministic, non-cryptographic comparison keys.");
    }

    for (path, sequence) in evidence_sequences(package) {
        if !command_sequences.contains(&sequence) {
            push(
                &mut diagnostics,
                ReplayPackageDiagnosticCode::UnknownEvidenceCommand,
                &path,
                format!("Replay evidence references unknown command sequence {sequence}."),
            );
        }
    }

    for randomness_diagnostic in crate::validate_replay_randomness(package).diagnostics {
        push(
            &mut diagnostics,
            ReplayPackageDiagnosticCode::InvalidRandomnessProvenance,
            &randomness_diagnostic.path,
            format!(
                "{}: {}",
                randomness_diagnostic.code.code(),
                randomness_diagnostic.message
            ),
        );
    }

    ReplayPackageValidationReport {
        accepted: diagnostics.is_empty(),
        diagnostics,
    }
}

fn evidence_sequences(package: &ReplayPackage) -> Vec<(String, u32)> {
    let mut values = Vec::new();
    values.extend(
        package
            .evidence
            .accepted_events
            .iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    format!("evidence.acceptedEvents[{index}].commandSequence"),
                    value.command_sequence,
                )
            }),
    );
    values.extend(
        package
            .evidence
            .rolls
            .iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    format!("evidence.rolls[{index}].commandSequence"),
                    value.command_sequence,
                )
            }),
    );
    values.extend(
        package
            .evidence
            .trace
            .iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    format!("evidence.trace[{index}].commandSequence"),
                    value.command_sequence,
                )
            }),
    );
    values
}

fn required(
    diagnostics: &mut Vec<ReplayPackageDiagnostic>,
    value: &str,
    code: ReplayPackageDiagnosticCode,
    path: &str,
    message: &str,
) {
    if value.trim().is_empty() {
        push(diagnostics, code, path, message);
    }
}

fn push(
    diagnostics: &mut Vec<ReplayPackageDiagnostic>,
    code: ReplayPackageDiagnosticCode,
    path: &str,
    message: impl Into<String>,
) {
    diagnostics.push(ReplayPackageDiagnostic {
        code,
        path: path.to_string(),
        message: message.into(),
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{ReplayCommand, ReplayCommandRecord, ReplayEvidence};
    use rulebench_combat::{
        fingerprint_content_pack_set, ActionDefinition, ActionResourceCost, AttackCheckDeclaration,
        CheckDeclaration, CombatControlCommandSpec, CombatSessionCreateRequest, ContentFingerprint,
        ContentPackReference, ContentPackSetReference, DefenseReference, Grid, HitEffect,
        RulebenchScenario, RulesetMetadata, ScenarioMetadata, StateFingerprint, TargetKind,
        TargetSelection, TargetTeamConstraint, TargetingDeclaration, VisibilityRequirement,
        CONTENT_PACK_FINGERPRINT_ALGORITHM,
    };

    #[test]
    fn complete_package_is_accepted() {
        assert!(validate_replay_package(&valid_package()).accepted);
    }

    #[test]
    fn version_and_completeness_fail_with_stable_diagnostics() {
        let mut package = valid_package();
        package.package_version = "2.0.0".to_string();
        package.commands.clear();
        package.initial_session.scenario.content_pack_set = None;
        package.fingerprint_kind = "sha256".to_string();

        let report = validate_replay_package(&package);
        let codes = report
            .diagnostics
            .iter()
            .map(|value| value.code.code())
            .collect::<Vec<_>>();

        assert!(!report.accepted);
        assert_eq!(
            codes,
            vec![
                "unsupportedReplayPackageVersion",
                "missingReplayContentPackSet",
                "emptyReplayCommands",
                "invalidReplayFingerprintKind"
            ]
        );
    }

    pub(crate) fn valid_package() -> crate::ReplayPackage {
        let ruleset = RulesetMetadata {
            id: "test.rules".to_string(),
            name: "Test rules".to_string(),
            version: "1.0.0".to_string(),
            summary: "Replay validation rules.".to_string(),
            modules: Vec::new(),
        };
        let provenance = ruleset.artifact_provenance();
        crate::ReplayPackage::new(
            "test-package",
            CombatSessionCreateRequest::new("test-session", scenario(ruleset)),
            provenance,
            vec![ReplayCommandRecord {
                sequence: 0,
                id: "start".to_string(),
                command: ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
                expected: crate::ReplayStepEvidence {
                    accepted: true,
                    decision_code: "accepted".to_string(),
                    state_before_fingerprint: fingerprint(),
                    state_after_fingerprint: fingerprint(),
                    accepted_events: Vec::new(),
                    command_audit: Vec::new(),
                    rolls: Vec::new(),
                    trace: Vec::new(),
                },
            }],
            ReplayEvidence::default(),
            fingerprint(),
        )
    }

    fn fingerprint() -> StateFingerprint {
        StateFingerprint {
            algorithm: "fnv1a64.rulebench-state.v0".to_string(),
            value: "0123456789abcdef".to_string(),
        }
    }

    fn scenario(ruleset: RulesetMetadata) -> RulebenchScenario {
        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "replay-test".to_string(),
                title: "Replay test".to_string(),
                summary: "Replay package validation scenario.".to_string(),
                seed_label: "replay-test".to_string(),
            },
            content_pack_set: Some(content_pack_set()),
            selected_ruleset_id: ruleset.id.clone(),
            rulesets: vec![ruleset],
            grid: Grid {
                width: 0,
                height: 0,
                cells: Vec::new(),
            },
            combatants: Vec::new(),
            entities: Vec::new(),
            abilities: Vec::new(),
            selected_ability_id: None,
            classes: Vec::new(),
            selected_class_id: None,
            stat_definitions: Vec::new(),
            modifiers: Vec::new(),
            items: Vec::new(),
            selected_item_id: None,
            actions: Vec::new(),
            selected_action: placeholder_action(),
        }
    }

    fn placeholder_action() -> ActionDefinition {
        ActionDefinition {
            id: "placeholder".to_string(),
            ruleset_id: "test.rules".to_string(),
            ability_id: "placeholder-ability".to_string(),
            name: "Placeholder".to_string(),
            actor_id: "placeholder-actor".to_string(),
            targeting: TargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 0,
                visibility_requirement: VisibilityRequirement::Ignored,
                target_ids: Vec::new(),
                visible_target_ids: Vec::new(),
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 0,
                modifier_stat_id: "placeholder-stat".to_string(),
                defense: DefenseReference {
                    id: "placeholder-defense".to_string(),
                    label: "Placeholder defense".to_string(),
                },
            }),
            hit: HitEffect {
                damage_bonus: 0,
                damage_type: "placeholder".to_string(),
                modifier_id: "placeholder-modifier".to_string(),
                modifier_label: "Placeholder modifier".to_string(),
                modifier_duration: "placeholder".to_string(),
                operations: Vec::new(),
            },
            resource_costs: vec![ActionResourceCost::standard_action()],
            action_text: "Placeholder action.".to_string(),
            effect_text: "Placeholder effect.".to_string(),
        }
    }

    fn content_pack_set() -> ContentPackSetReference {
        let root = ContentPackReference {
            id: "replay.content".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: ContentFingerprint {
                algorithm: CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
                value: "0123456789abcdef".to_string(),
            },
        };
        let packs = vec![root.clone()];
        ContentPackSetReference {
            fingerprint: fingerprint_content_pack_set(&root, &packs),
            root,
            packs,
        }
    }
}
