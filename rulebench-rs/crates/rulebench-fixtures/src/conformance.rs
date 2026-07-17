use std::collections::{BTreeMap, BTreeSet};

use rpg_ir::{
    EffectOperationId, HitEffectOperation, TargetKind, TargetSelection, TargetingOperationId,
};
use rulebench_combat::{
    fingerprint_projected_state, resolve_use_action, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticRunSpec, CombatSessionCreateRequest, CombatSessionIntentCommandSpec,
    CombatSessionState, DomainEvent, RulebenchReceipt, RulebenchRejection,
};
use rulebench_content::UseActionIntent;
use rulebench_protocol::{executable_conformance_capabilities, CapabilityIdentity};
use rulebench_replay::{
    record_replay_package, verify_automatic_run_replay, verify_replay_package,
    CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec, ReplayCommand,
    ReplayCommandRecordingSpec,
};

use crate::{
    ContentImportExampleOutcome, ScenarioCatalogCase, ScenarioOutcomeClass, ScenarioPackageRegistry,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityConformanceFilter {
    pub capability_id: Option<String>,
    pub package_id: Option<String>,
    pub ruleset_id: Option<String>,
    pub scenario_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityConformanceFailureKind {
    CaseEvidenceMismatch,
    MissingExecutableCapability,
    UnknownCapabilityReference,
    VersionMismatch,
}

impl CapabilityConformanceFailureKind {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CaseEvidenceMismatch => "caseEvidenceMismatch",
            Self::MissingExecutableCapability => "missingExecutableCapability",
            Self::UnknownCapabilityReference => "unknownCapabilityReference",
            Self::VersionMismatch => "versionMismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityConformanceFailure {
    pub kind: CapabilityConformanceFailureKind,
    pub case_id: Option<String>,
    pub capability: CapabilityIdentity,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityConformanceCaseReadout {
    pub case_id: String,
    pub package_id: String,
    pub package_version: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub scenario_id: String,
    pub capabilities: Vec<CapabilityIdentity>,
    pub accepted: bool,
    pub event_count: usize,
    pub trace_count: usize,
    pub final_state_fingerprint: String,
    pub replay_verified: bool,
    pub replay_mismatch_classified: bool,
    pub rejection_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityConformanceReport {
    pub accepted: bool,
    pub cases: Vec<CapabilityConformanceCaseReadout>,
    pub covered_capabilities: Vec<CapabilityIdentity>,
    pub failures: Vec<CapabilityConformanceFailure>,
}

pub fn run_capability_conformance(
    registry: &ScenarioPackageRegistry,
    filter: &CapabilityConformanceFilter,
) -> CapabilityConformanceReport {
    let executable = executable_conformance_capabilities();
    let executable_by_id = executable
        .iter()
        .map(|identity| (identity.id.as_str(), identity))
        .collect::<BTreeMap<_, _>>();
    let mut cases = Vec::new();
    let mut failures = Vec::new();

    for registration in registry.registrations() {
        let package = &registration.package;
        if !matches_filter(filter.package_id.as_deref(), &package.identity.id)
            || !matches_filter(filter.ruleset_id.as_deref(), &package.ruleset.id)
        {
            continue;
        }
        for case in registration.scenario_catalog_cases() {
            if !matches_filter(filter.scenario_id.as_deref(), &case.summary.id) {
                continue;
            }
            // Rejection and miss catalog cases remain valuable scenario
            // regressions, but cannot establish positive executable coverage.
            if case.summary.outcome_class != ScenarioOutcomeClass::AcceptedHit {
                continue;
            }
            let Some(action) = case
                .scenario
                .actions
                .iter()
                .find(|action| action.id == case.intent.action_id)
            else {
                continue;
            };
            let capabilities = action_capabilities(action);
            if !capabilities
                .iter()
                .any(|identity| matches_filter(filter.capability_id.as_deref(), &identity.id))
            {
                continue;
            }
            let readout = execute_scenario_case(
                &case,
                &package.identity.id,
                &package.identity.version,
                &package.ruleset.id,
                &package.ruleset.version,
                capabilities,
            );
            validate_case_references(&readout, &executable_by_id, &mut failures);
            if !readout.accepted {
                for capability in &readout.capabilities {
                    failures.push(CapabilityConformanceFailure {
                        kind: CapabilityConformanceFailureKind::CaseEvidenceMismatch,
                        case_id: Some(readout.case_id.clone()),
                        capability: capability.clone(),
                        detail: "authority evidence, rejection probes, or replay evidence did not satisfy the conformance contract".to_string(),
                    });
                }
            }
            cases.push(readout);
        }
    }

    append_movement_cases(
        registry,
        filter,
        &executable_by_id,
        &mut cases,
        &mut failures,
    );
    append_policy_cases(
        registry,
        filter,
        &executable_by_id,
        &mut cases,
        &mut failures,
    );

    let covered_capabilities = cases
        .iter()
        .filter(|case| case.accepted)
        .flat_map(|case| case.capabilities.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let covered = covered_capabilities.iter().collect::<BTreeSet<_>>();
    for capability in executable.iter().filter(|identity| {
        matches_filter(filter.capability_id.as_deref(), &identity.id)
            && filter.package_id.is_none()
            && filter.ruleset_id.is_none()
            && filter.scenario_id.is_none()
    }) {
        if !covered.contains(capability) {
            failures.push(CapabilityConformanceFailure {
                kind: CapabilityConformanceFailureKind::MissingExecutableCapability,
                case_id: None,
                capability: capability.clone(),
                detail: "owner registry reports executable but no executed conformance case supplied complete evidence".to_string(),
            });
        }
    }

    CapabilityConformanceReport {
        accepted: failures.is_empty() && !cases.is_empty(),
        cases,
        covered_capabilities,
        failures,
    }
}

fn append_movement_cases(
    registry: &ScenarioPackageRegistry,
    filter: &CapabilityConformanceFilter,
    executable_by_id: &BTreeMap<&str, &CapabilityIdentity>,
    cases: &mut Vec<CapabilityConformanceCaseReadout>,
    failures: &mut Vec<CapabilityConformanceFailure>,
) {
    let capability = CapabilityIdentity {
        id: "targeting.cellMovement".to_string(),
        version: rpg_ir::OperationPipelineV2::VOCABULARY_VERSION.to_string(),
    };
    if !matches_filter(filter.capability_id.as_deref(), &capability.id) {
        return;
    }
    for registration in registry.registrations() {
        let package = &registration.package;
        if !matches_filter(filter.package_id.as_deref(), &package.identity.id)
            || !matches_filter(filter.ruleset_id.as_deref(), &package.ruleset.id)
        {
            continue;
        }
        let scenario = package.initial_state.scenario.clone();
        let Some(action) = scenario
            .actions
            .iter()
            .find(|action| action.movement.is_some())
        else {
            continue;
        };
        let Some(actor) = scenario
            .combatants
            .iter()
            .find(|combatant| combatant.id == action.actor_id)
        else {
            continue;
        };
        let movement = action
            .movement
            .as_ref()
            .expect("movement action is selected");
        let Some(destination) = scenario.grid.cells.iter().find_map(|cell| {
            let occupied = scenario
                .combatants
                .iter()
                .any(|combatant| combatant.position == cell.position);
            let blocked = cell.terrain_tags.iter().any(|tag| {
                movement
                    .blocking_terrain_tags
                    .iter()
                    .any(|blocked_tag| blocked_tag == tag)
            });
            let distance = actor.position.x.abs_diff(cell.position.x)
                + actor.position.y.abs_diff(cell.position.y);
            (!occupied && !blocked && distance > 0 && distance <= movement.allowance)
                .then_some(cell.position)
        }) else {
            continue;
        };
        let scenario_id = format!("{}-cell-movement", scenario.metadata.id);
        if !matches_filter(filter.scenario_id.as_deref(), &scenario_id) {
            continue;
        }
        let actor_id = action.actor_id.clone();
        let action_id = action.id.clone();
        let case = ScenarioCatalogCase {
            summary: crate::ScenarioCatalogSummary {
                id: scenario_id,
                title: format!("{} Cell Movement Conformance", scenario.metadata.title),
                summary: "Executes a registered typed cell destination through the session authority owner.".to_string(),
                seed_label: "no-roll-movement".to_string(),
                outcome_class: ScenarioOutcomeClass::AcceptedHit,
            },
            scenario,
            intent: UseActionIntent::for_cell(actor_id, action_id, destination),
            roll_stream: Vec::new(),
        };
        let readout = execute_scenario_case(
            &case,
            &package.identity.id,
            &package.identity.version,
            &package.ruleset.id,
            &package.ruleset.version,
            vec![capability.clone()],
        );
        validate_case_references(&readout, executable_by_id, failures);
        if !readout.accepted {
            failures.push(CapabilityConformanceFailure {
                kind: CapabilityConformanceFailureKind::CaseEvidenceMismatch,
                case_id: Some(readout.case_id.clone()),
                capability: capability.clone(),
                detail: "movement authority evidence or replay proof did not satisfy the conformance contract".to_string(),
            });
        }
        cases.push(readout);
    }
}

fn execute_scenario_case(
    case: &ScenarioCatalogCase,
    package_id: &str,
    package_version: &str,
    ruleset_id: &str,
    ruleset_version: &str,
    capabilities: Vec<CapabilityIdentity>,
) -> CapabilityConformanceCaseReadout {
    let first = execute_case_receipt(case);
    let second = execute_case_receipt(case);
    let deterministic = first == second;
    let authority_evidence = first.accepted
        && first.projection.is_some()
        && !first.events.is_empty()
        && !first.trace.is_empty()
        && capabilities.iter().all(|identity| {
            receipt_has_capability_evidence(&case.scenario, &case.intent, &first, identity)
        });
    let initial_state_fingerprint = fingerprint_projected_state(
        &rulebench_combat::CombatState::from_scenario(&case.scenario)
            .project("conformance-initial"),
    )
    .value;
    let final_state_fingerprint = first
        .projection
        .as_ref()
        .map(fingerprint_projected_state)
        .map(|fingerprint| fingerprint.value)
        .unwrap_or_default();
    let (rejection_codes, rejection_probes_passed) = execute_rejection_probes(case, &capabilities);
    let (replay_verified, replay_mismatch_classified) = execute_replay_proof(case);

    CapabilityConformanceCaseReadout {
        case_id: format!("{}@1", case.summary.id),
        package_id: package_id.to_string(),
        package_version: package_version.to_string(),
        ruleset_id: ruleset_id.to_string(),
        ruleset_version: ruleset_version.to_string(),
        scenario_id: case.summary.id.clone(),
        capabilities,
        accepted: deterministic
            && authority_evidence
            && !final_state_fingerprint.is_empty()
            && final_state_fingerprint != initial_state_fingerprint
            && rejection_probes_passed
            && replay_verified
            && replay_mismatch_classified,
        event_count: first.events.len(),
        trace_count: first.trace.len(),
        final_state_fingerprint,
        replay_verified,
        replay_mismatch_classified,
        rejection_codes,
    }
}

fn execute_case_receipt(case: &ScenarioCatalogCase) -> RulebenchReceipt {
    let is_movement = case
        .scenario
        .actions
        .iter()
        .find(|action| action.id == case.intent.action_id)
        .is_some_and(|action| action.movement.is_some());
    if !is_movement {
        return resolve_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
    }
    let mut session = CombatSessionState::new(
        format!("conformance-{}", case.summary.id),
        case.scenario.clone(),
    );
    session
        .submit_intent_command(CombatSessionIntentCommandSpec::new(
            case.summary.id.clone(),
            case.summary.title.clone(),
            "Execute registered movement conformance through the session owner.",
            case.intent.clone(),
            case.roll_stream.clone(),
        ))
        .receipt
}

fn action_capabilities(action: &rpg_ir::ActionDefinition) -> Vec<CapabilityIdentity> {
    let targeting = if action.movement.is_some() {
        TargetingOperationId::CellMovement
    } else {
        match (action.targeting.target_kind, action.targeting.selection) {
            (TargetKind::Combatant, TargetSelection::Single) => {
                TargetingOperationId::SingleCombatant
            }
            (TargetKind::Combatant, TargetSelection::Multiple) => {
                TargetingOperationId::MultipleCombatants
            }
            (TargetKind::Area, _) => TargetingOperationId::ManhattanBurstArea,
        }
    };
    let mut capabilities = vec![CapabilityIdentity {
        id: format!("targeting.{}", targeting.code()),
        version: rpg_ir::OperationPipelineV2::VOCABULARY_VERSION.to_string(),
    }];
    if action.movement.is_none() {
        capabilities.extend(
            action
                .hit
                .operations
                .iter()
                .map(|operation| CapabilityIdentity {
                    id: format!("operation.{}", operation.id().code()),
                    version: EffectOperationId::VOCABULARY_VERSION.to_string(),
                }),
        );
    }
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

fn receipt_has_capability_evidence(
    scenario: &rulebench_content::RulebenchScenario,
    intent: &UseActionIntent,
    receipt: &RulebenchReceipt,
    capability: &CapabilityIdentity,
) -> bool {
    match capability.id.as_str() {
        "operation.damage" => {
            receipt
                .target_results
                .iter()
                .any(|result| result.damage.is_some())
                || receipt.damage.is_some()
        }
        "operation.heal" => {
            let outcome = receipt.healing.as_ref().or_else(|| {
                receipt
                    .target_results
                    .iter()
                    .find_map(|result| result.healing.as_ref())
            });
            outcome.is_some_and(|healing| {
                healing.after.current <= healing.after.max
                    && healing.amount == healing.after.current - healing.before.current
                    && receipt.events.iter().any(|event| {
                        matches!(
                            event,
                            DomainEvent::HealingApplied {
                                target_id,
                                amount,
                                healing_type,
                            } if target_id == &healing.target_id
                                && amount == &healing.amount
                                && healing_type == &healing.healing_type
                        )
                    })
            })
        }
        "operation.grantTemporaryVitality" => {
            let outcome = receipt.temporary_vitality.as_ref().or_else(|| {
                receipt
                    .target_results
                    .iter()
                    .find_map(|result| result.temporary_vitality.as_ref())
            });
            outcome.is_some_and(|vitality| {
                vitality.after == vitality.before.max(vitality.requested_amount)
                    && receipt.events.iter().any(|event| {
                        matches!(
                            event,
                            DomainEvent::TemporaryVitalityGranted { target_id, amount }
                                if target_id == &vitality.target_id
                                    && amount == &(vitality.after - vitality.before)
                        )
                    })
            })
        }
        "operation.applyModifier" => {
            receipt
                .target_results
                .iter()
                .any(|result| result.modifier.is_some())
                || receipt.modifier.is_some()
        }
        "operation.move" => receipt
            .target_results
            .iter()
            .any(|result| result.movement.is_some()),
        "operation.changeResource" => receipt
            .target_results
            .iter()
            .any(|result| !result.resource_changes.is_empty()),
        "operation.openReactionWindow" => reaction_window_opens(scenario, intent, receipt),
        "targeting.singleCombatant" => {
            receipt.target_results.len() == 1 || receipt.target_legality.is_some()
        }
        "targeting.multipleCombatants" | "targeting.manhattanBurstArea" => {
            receipt.target_results.len() > 1
                && receipt
                    .target_results
                    .windows(2)
                    .all(|pair| pair[0].target_id < pair[1].target_id)
        }
        "targeting.cellMovement" => receipt
            .events
            .iter()
            .any(|event| matches!(event, DomainEvent::PositionChanged { .. })),
        _ => false,
    }
}

fn reaction_window_opens(
    scenario: &rulebench_content::RulebenchScenario,
    intent: &UseActionIntent,
    receipt: &RulebenchReceipt,
) -> bool {
    let has_hook = scenario
        .actions
        .iter()
        .find(|action| action.id == intent.action_id)
        .is_some_and(|action| {
            action
                .hit
                .operations
                .iter()
                .any(|operation| matches!(operation, HitEffectOperation::OpenReactionWindow(_)))
        });
    if !has_hook {
        return false;
    }
    let mut session = CombatSessionState::new("conformance-reaction", scenario.clone());
    let result = session.submit_intent_command(CombatSessionIntentCommandSpec::new(
        "conformance-reaction",
        "Conformance reaction",
        "Prove the registered reaction window opens through the session owner.",
        intent.clone(),
        receipt
            .roll_consumption
            .iter()
            .filter_map(|roll| roll.supplied_value)
            .collect(),
    ));
    result.receipt.accepted && session.snapshot().current_reaction_window.is_some()
}

fn execute_rejection_probes(
    case: &ScenarioCatalogCase,
    capabilities: &[CapabilityIdentity],
) -> (Vec<String>, bool) {
    let action = case
        .scenario
        .actions
        .iter()
        .find(|action| action.id == case.intent.action_id)
        .expect("catalog action exists");
    if action.movement.is_some() {
        return (Vec::new(), true);
    }
    let target_id = case
        .intent
        .target_ids
        .first()
        .cloned()
        .unwrap_or_else(|| case.intent.target_id.clone());
    if target_id.is_empty() {
        return (Vec::new(), true);
    }
    let mut codes = Vec::new();
    let invalid = resolve_use_action(
        &case.scenario,
        UseActionIntent::new(
            &case.intent.actor_id,
            &case.intent.action_id,
            "missing-target",
        ),
        &case.roll_stream,
    );
    codes.push(
        invalid
            .rejection
            .map(RulebenchRejection::code)
            .unwrap_or("accepted")
            .to_string(),
    );
    let mut defeated_scenario = case.scenario.clone();
    if let Some(target) = defeated_scenario
        .combatants
        .iter_mut()
        .find(|combatant| combatant.id == target_id)
    {
        target.hit_points.current = 0;
    }
    let defeated = resolve_use_action(&defeated_scenario, case.intent.clone(), &case.roll_stream);
    codes.push(
        defeated
            .rejection
            .map(RulebenchRejection::code)
            .unwrap_or("accepted")
            .to_string(),
    );
    let defeated_behavior_is_classified = if defeated.accepted {
        defeated.projection.is_some() && !defeated.events.is_empty()
    } else {
        defeated.rejection.is_some() && defeated.events.is_empty()
    };
    let mut passed = invalid.rejection == Some(RulebenchRejection::InvalidTarget)
        && invalid.events.is_empty()
        && defeated_behavior_is_classified;

    if capabilities
        .iter()
        .any(|identity| identity.id == "targeting.multipleCombatants")
    {
        let duplicate = resolve_use_action(
            &case.scenario,
            UseActionIntent::for_targets(
                &case.intent.actor_id,
                &case.intent.action_id,
                vec![target_id.clone(), target_id],
            ),
            &case.roll_stream,
        );
        codes.push(
            duplicate
                .rejection
                .map(RulebenchRejection::code)
                .unwrap_or("accepted")
                .to_string(),
        );
        passed &= duplicate.rejection == Some(RulebenchRejection::DuplicateTarget)
            && duplicate.events.is_empty();

        let mut out_of_range_scenario = case.scenario.clone();
        out_of_range_scenario
            .actions
            .iter_mut()
            .find(|action| action.id == case.intent.action_id)
            .expect("conformance action exists")
            .targeting
            .maximum_range = 0;
        let out_of_range = resolve_use_action(
            &out_of_range_scenario,
            case.intent.clone(),
            &case.roll_stream,
        );
        codes.push(
            out_of_range
                .rejection
                .map(RulebenchRejection::code)
                .unwrap_or("accepted")
                .to_string(),
        );
        passed &= out_of_range.rejection == Some(RulebenchRejection::TargetOutOfRange)
            && out_of_range.events.is_empty();

        let accepted = resolve_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
        if let Some(destination) = accepted
            .target_results
            .iter()
            .find_map(|target| target.movement.as_ref().map(|movement| movement.to))
        {
            let mut blocked_scenario = case.scenario.clone();
            blocked_scenario
                .grid
                .cells
                .iter_mut()
                .find(|cell| cell.position == destination)
                .expect("effect movement destination is registered")
                .terrain_tags = vec!["blocked".to_string()];
            let initial = rulebench_combat::CombatState::from_scenario(&blocked_scenario)
                .project("rollback-initial");
            let blocked =
                resolve_use_action(&blocked_scenario, case.intent.clone(), &case.roll_stream);
            codes.push(
                blocked
                    .rejection
                    .map(RulebenchRejection::code)
                    .unwrap_or("accepted")
                    .to_string(),
            );
            passed &= blocked.rejection
                == Some(RulebenchRejection::EffectMovementDestinationBlocked)
                && blocked.events.is_empty()
                && blocked
                    .projection
                    .as_ref()
                    .is_some_and(|projection| projection.combatants == initial.combatants);
        } else {
            passed = false;
        }
    }
    (codes, passed)
}

fn execute_replay_proof(case: &ScenarioCatalogCase) -> (bool, bool) {
    let mut scenario = case.scenario.clone();
    scenario.content_pack_set = crate::content_import_examples()
        .into_iter()
        .find_map(|example| match example.outcome {
            ContentImportExampleOutcome::Accepted(imported) => {
                Some(imported.resolved_set.reference)
            }
            ContentImportExampleOutcome::Rejected { .. } => None,
        });
    let Some(ruleset) = scenario
        .selected_ruleset()
        .map(|ruleset| ruleset.artifact_provenance())
    else {
        return (false, false);
    };
    let package = record_replay_package(
        format!("conformance-{}", case.summary.id),
        CombatSessionCreateRequest::new(format!("conformance-{}", case.summary.id), scenario),
        ruleset,
        vec![ReplayCommandRecordingSpec::new(
            case.summary.id.clone(),
            ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
                case.summary.id.clone(),
                case.summary.title.clone(),
                "Registered capability conformance replay.",
                case.intent.clone(),
                case.roll_stream.clone(),
            )),
        )],
    );
    let verified = verify_replay_package(&package).accepted;
    let mut mismatched = package;
    if let Some(command) = mismatched.commands.first_mut() {
        if let ReplayCommand::Intent(intent) = &mut command.command {
            if intent.intent.target_cell.is_some() {
                intent.intent.target_cell = Some(rpg_core::GridPosition { x: 0, y: 0 });
            } else if intent.intent.destination_cell.is_some() {
                intent.intent.destination_cell = Some(rpg_core::GridPosition { x: 0, y: 0 });
            } else if intent.intent.target_ids.len() > 1 {
                intent.intent.target_ids.pop();
                intent.intent.target_id = intent
                    .intent
                    .target_ids
                    .first()
                    .cloned()
                    .unwrap_or_default();
            } else {
                intent.intent.target_id = intent.intent.actor_id.clone();
            }
        }
    }
    let mismatch = verify_replay_package(&mismatched);
    (verified, !mismatch.accepted && mismatch.mismatch.is_some())
}

fn append_policy_cases(
    registry: &ScenarioPackageRegistry,
    filter: &CapabilityConformanceFilter,
    executable_by_id: &BTreeMap<&str, &CapabilityIdentity>,
    cases: &mut Vec<CapabilityConformanceCaseReadout>,
    failures: &mut Vec<CapabilityConformanceFailure>,
) {
    for registration in registry.registrations() {
        let package = &registration.package;
        if !matches_filter(filter.package_id.as_deref(), &package.identity.id)
            || !matches_filter(filter.ruleset_id.as_deref(), &package.ruleset.id)
        {
            continue;
        }
        let replay_readouts = registration.automatic_run_replay_readouts();
        for readout in registration.automatic_run_readouts() {
            let capability = CapabilityIdentity {
                id: format!("policy.{}", readout.policy.id),
                version: readout.policy.version.to_string(),
            };
            if !matches_filter(filter.capability_id.as_deref(), &capability.id)
                || !matches_filter(filter.scenario_id.as_deref(), &readout.id)
            {
                continue;
            }
            let matching_replay = replay_readouts.iter().find(|replay| {
                replay.replayed_run.id == readout.id && replay.replayed_run.policy == readout.policy
            });
            let replay_verified = matching_replay.is_some_and(|replay| replay.accepted);
            let replay_mismatch_classified = matching_replay.is_some_and(|replay| {
                execute_policy_replay_mismatch(&package.initial_state.scenario, replay)
            });
            let accepted = readout.accepted
                && readout.executed_step_count > 0
                && !readout.policy_decisions.is_empty()
                && replay_verified
                && replay_mismatch_classified;
            let case = CapabilityConformanceCaseReadout {
                case_id: format!("{}@1", readout.id),
                package_id: package.identity.id.clone(),
                package_version: package.identity.version.clone(),
                ruleset_id: package.ruleset.id.clone(),
                ruleset_version: package.ruleset.version.clone(),
                scenario_id: readout.id,
                capabilities: vec![capability.clone()],
                accepted,
                event_count: readout.steps.len(),
                trace_count: readout.policy_decisions.len(),
                final_state_fingerprint: readout.final_snapshot.current_state_fingerprint.value,
                replay_verified,
                replay_mismatch_classified,
                rejection_codes: Vec::new(),
            };
            validate_case_references(&case, executable_by_id, failures);
            if !case.accepted {
                failures.push(CapabilityConformanceFailure {
                    kind: CapabilityConformanceFailureKind::CaseEvidenceMismatch,
                    case_id: Some(case.case_id.clone()),
                    capability,
                    detail: "policy authority execution lacks its own verified replay or an executed replay mismatch classification".to_string(),
                });
            }
            cases.push(case);
        }
    }
}

fn execute_policy_replay_mismatch(
    scenario: &rulebench_content::RulebenchScenario,
    replay: &CombatSessionAutomaticRunReplayReadout,
) -> bool {
    let run = &replay.replayed_run;
    if run.policy_decisions.is_empty() {
        return false;
    }
    let materialized_rolls = automatic_run_materialized_rolls(run);
    let run_spec = CombatSessionAutomaticRunSpec::new(
        run.id.clone(),
        run.title.clone(),
        run.summary.clone(),
        run.max_steps,
        materialized_rolls,
    )
    .with_policy(run.policy.clone());
    let mut mismatched_policy_decisions = run.policy_decisions.clone();
    mismatched_policy_decisions.clear();
    let mismatch = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
        format!("{}-policy-mismatch", replay.id),
        "Policy conformance mismatch probe",
        "Re-executes the exact policy run against deliberately changed expected decision evidence.",
        format!("{}-policy-mismatch-session", replay.id),
        scenario.clone(),
        run_spec,
        run.final_snapshot.current_state_fingerprint.clone(),
        run.final_snapshot.finalization.clone(),
        run.decision_kind,
        run.executed_step_count,
        mismatched_policy_decisions,
        run.final_snapshot.action_resource_transition_log.clone(),
        run.final_snapshot.equipment_ledger.clone(),
        run.final_snapshot.class_build_ledger.clone(),
        run.final_snapshot.equipment_transition_log.clone(),
        run.final_snapshot.reaction_window_lifecycle_log.clone(),
        run.final_snapshot.reaction_audit_log.clone(),
        run.final_snapshot.modifier_duration_expiration_log.clone(),
    ));
    !mismatch.accepted && !mismatch.policy_decisions_match
}

fn automatic_run_materialized_rolls(readout: &CombatSessionAutomaticRunReadout) -> Vec<i32> {
    readout
        .steps
        .iter()
        .filter_map(|step| {
            step.auto_candidate
                .as_ref()
                .and_then(|execution| execution.submitted_step.as_ref())
        })
        .flat_map(|submitted| submitted.command.roll_stream.iter().copied())
        .collect()
}

fn validate_case_references(
    case: &CapabilityConformanceCaseReadout,
    executable_by_id: &BTreeMap<&str, &CapabilityIdentity>,
    failures: &mut Vec<CapabilityConformanceFailure>,
) {
    for capability in &case.capabilities {
        let Some(owner_identity) = executable_by_id.get(capability.id.as_str()) else {
            failures.push(CapabilityConformanceFailure {
                kind: CapabilityConformanceFailureKind::UnknownCapabilityReference,
                case_id: Some(case.case_id.clone()),
                capability: capability.clone(),
                detail: "conformance case references no executable owner-registry identity"
                    .to_string(),
            });
            continue;
        };
        if owner_identity.version != capability.version {
            failures.push(CapabilityConformanceFailure {
                kind: CapabilityConformanceFailureKind::VersionMismatch,
                case_id: Some(case.case_id.clone()),
                capability: capability.clone(),
                detail: format!(
                    "owner registry version is {}, conformance case version is {}",
                    owner_identity.version, capability.version
                ),
            });
        }
    }
}

fn matches_filter(filter: Option<&str>, value: &str) -> bool {
    filter.is_none_or(|filter| filter == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_hexing_policy_replay_only() -> Vec<CombatSessionAutomaticRunReplayReadout> {
        let mut readouts =
            crate::scenarios::hexing_bolt::combat_session_automatic_run_replay_readouts();
        readouts.truncate(1);
        readouts
    }

    fn registry_missing_lowest_vitality_replay() -> ScenarioPackageRegistry {
        ScenarioPackageRegistry::new(vec![crate::ScenarioPackageRegistration::new(
            crate::scenarios::hexing_bolt::hexing_bolt_scenario_package(),
            crate::ScenarioPackageReadbackFactories {
                catalog_cases: crate::scenarios::hexing_bolt::scenario_catalog_cases,
                ruleset_catalog_readout: crate::scenarios::hexing_bolt::ruleset_catalog_readout,
                content_validation_readouts:
                    crate::scenarios::hexing_bolt::content_validation_readouts,
                session_transcripts: crate::scenarios::hexing_bolt::combat_session_transcripts,
                control_history_readouts:
                    crate::scenarios::hexing_bolt::combat_session_control_history_readouts,
                script_readouts: crate::scenarios::hexing_bolt::combat_session_script_readouts,
                automatic_run_readouts:
                    crate::scenarios::hexing_bolt::combat_session_automatic_run_readouts,
                automatic_run_replay_readouts: first_hexing_policy_replay_only,
            },
        )])
        .expect("isolated package registry is valid")
    }

    #[test]
    fn executed_cases_cover_every_owner_registry_capability() {
        let report = run_capability_conformance(
            &crate::scenario_package_registry(),
            &CapabilityConformanceFilter::default(),
        );

        assert!(report.accepted, "{:?}", report.failures);
        assert_eq!(
            report.covered_capabilities,
            executable_conformance_capabilities()
        );
        for capability_id in [
            "targeting.multipleCombatants",
            "operation.heal",
            "operation.grantTemporaryVitality",
        ] {
            assert!(report.cases.iter().any(|case| case.accepted
                && case
                    .capabilities
                    .iter()
                    .any(|identity| identity.id == capability_id)
                && case.replay_verified
                && case.replay_mismatch_classified));
        }
    }

    #[test]
    fn missing_and_version_drift_are_classified_against_owner_registries() {
        let capability = executable_conformance_capabilities()
            .into_iter()
            .find(|identity| identity.id == "operation.heal")
            .expect("healing is executable");
        let mut missing_report = run_capability_conformance(
            &crate::ScenarioPackageRegistry::new(Vec::new()).expect("empty registry is valid"),
            &CapabilityConformanceFilter {
                capability_id: Some(capability.id.clone()),
                ..Default::default()
            },
        );
        assert!(missing_report.failures.iter().any(|failure| {
            failure.kind == CapabilityConformanceFailureKind::MissingExecutableCapability
        }));

        let mut drifted = CapabilityConformanceCaseReadout {
            case_id: "drift@1".to_string(),
            package_id: "test".to_string(),
            package_version: "1".to_string(),
            ruleset_id: "test".to_string(),
            ruleset_version: "1".to_string(),
            scenario_id: "test".to_string(),
            capabilities: vec![CapabilityIdentity {
                id: capability.id,
                version: "future".to_string(),
            }],
            accepted: true,
            event_count: 1,
            trace_count: 1,
            final_state_fingerprint: "test".to_string(),
            replay_verified: true,
            replay_mismatch_classified: true,
            rejection_codes: Vec::new(),
        };
        let executable = executable_conformance_capabilities();
        let executable_by_id = executable
            .iter()
            .map(|identity| (identity.id.as_str(), identity))
            .collect::<BTreeMap<_, _>>();
        missing_report.failures.clear();
        validate_case_references(&drifted, &executable_by_id, &mut missing_report.failures);
        assert_eq!(
            missing_report.failures[0].kind,
            CapabilityConformanceFailureKind::VersionMismatch
        );
        drifted.capabilities[0].id = "operation.renamed".to_string();
        missing_report.failures.clear();
        validate_case_references(&drifted, &executable_by_id, &mut missing_report.failures);
        assert_eq!(
            missing_report.failures[0].kind,
            CapabilityConformanceFailureKind::UnknownCapabilityReference
        );
    }

    #[test]
    fn policy_coverage_requires_the_matching_policy_replay_and_mismatch_probe() {
        let registry = registry_missing_lowest_vitality_replay();
        let first = run_capability_conformance(
            &registry,
            &CapabilityConformanceFilter {
                capability_id: Some("policy.firstAcceptedCandidate".to_string()),
                ..Default::default()
            },
        );
        assert!(first.accepted, "{:?}", first.failures);
        assert_eq!(first.cases.len(), 1);
        assert!(first.cases[0].replay_verified);
        assert!(first.cases[0].replay_mismatch_classified);

        let lowest = run_capability_conformance(
            &registry,
            &CapabilityConformanceFilter {
                capability_id: Some("policy.lowestVitalityTarget".to_string()),
                ..Default::default()
            },
        );
        assert!(!lowest.accepted);
        assert_eq!(lowest.cases.len(), 1);
        assert!(!lowest.cases[0].replay_verified);
        assert!(!lowest.cases[0].replay_mismatch_classified);
        assert!(lowest.failures.iter().any(|failure| {
            failure.kind == CapabilityConformanceFailureKind::CaseEvidenceMismatch
                && failure.capability.id == "policy.lowestVitalityTarget"
        }));
    }
}
