use rulebench_combat::{
    fingerprint_projected_state, resolve_use_action, AttackOutcome, CombatSessionIntentCommandSpec,
    CombatSessionState, ContestedCheckOutcome, DomainEvent, RulebenchReceipt, RulebenchRejection,
    SavingThrowOutcome,
};

use crate::{ScenarioCatalogCase, ScenarioOutcomeClass, ScenarioPackageRegistry};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScenarioRegressionFilter {
    pub package_id: Option<String>,
    pub package_version: Option<String>,
    pub ruleset_id: Option<String>,
    pub ruleset_version: Option<String>,
    pub scenario_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRegressionDifference {
    pub path: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRegressionCaseReadout {
    pub package_id: String,
    pub package_version: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub scenario_id: String,
    pub outcome_class: Option<ScenarioOutcomeClass>,
    pub event_count: usize,
    pub trace_count: usize,
    pub final_state_fingerprint: String,
    pub accepted: bool,
    pub first_difference: Option<ScenarioRegressionDifference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRegressionReport {
    pub accepted: bool,
    pub cases: Vec<ScenarioRegressionCaseReadout>,
    pub first_difference: Option<ScenarioRegressionDifference>,
}

pub fn run_scenario_regressions(
    registry: &ScenarioPackageRegistry,
    filter: &ScenarioRegressionFilter,
) -> ScenarioRegressionReport {
    let mut cases = Vec::new();
    for registration in registry.registrations() {
        let package = &registration.package;
        if !matches_filter(filter.package_id.as_deref(), &package.identity.id)
            || !matches_filter(filter.package_version.as_deref(), &package.identity.version)
            || !matches_filter(filter.ruleset_id.as_deref(), &package.ruleset.id)
            || !matches_filter(filter.ruleset_version.as_deref(), &package.ruleset.version)
        {
            continue;
        }

        for case in registration.scenario_catalog_cases() {
            if !matches_filter(filter.scenario_id.as_deref(), &case.summary.id) {
                continue;
            }
            let first = execute_catalog_case(&case);
            let second = execute_catalog_case(&case);
            let actual_outcome = classify_outcome(&first);
            let first_difference = compare_outcome(case.summary.outcome_class, actual_outcome)
                .or_else(|| compare_receipts(&first, &second));
            let final_state_fingerprint = first
                .projection
                .as_ref()
                .map(fingerprint_projected_state)
                .map(|fingerprint| fingerprint.value)
                .unwrap_or_default();
            cases.push(ScenarioRegressionCaseReadout {
                package_id: package.identity.id.clone(),
                package_version: package.identity.version.clone(),
                ruleset_id: package.ruleset.id.clone(),
                ruleset_version: package.ruleset.version.clone(),
                scenario_id: case.summary.id,
                outcome_class: actual_outcome,
                event_count: first.events.len(),
                trace_count: first.trace.len(),
                final_state_fingerprint,
                accepted: first_difference.is_none(),
                first_difference,
            });
        }
    }

    let selection_difference = cases.is_empty().then(|| ScenarioRegressionDifference {
        path: "selection".to_string(),
        expected: "at least one registered scenario".to_string(),
        actual: "no matching package/ruleset/scenario".to_string(),
    });
    let first_difference = selection_difference
        .or_else(|| cases.iter().find_map(|case| case.first_difference.clone()));
    ScenarioRegressionReport {
        accepted: first_difference.is_none(),
        cases,
        first_difference,
    }
}

fn execute_catalog_case(case: &ScenarioCatalogCase) -> RulebenchReceipt {
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
        format!("regression-{}", case.summary.id),
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

fn matches_filter(filter: Option<&str>, value: &str) -> bool {
    filter.is_none_or(|filter| filter == value)
}

fn classify_outcome(receipt: &RulebenchReceipt) -> Option<ScenarioOutcomeClass> {
    if receipt.rejection == Some(RulebenchRejection::TargetLegalityFailed) {
        return Some(ScenarioOutcomeClass::RejectedTargetLegality);
    }
    if let Some(attack) = &receipt.attack_roll {
        return match attack.outcome {
            AttackOutcome::Hit => Some(ScenarioOutcomeClass::AcceptedHit),
            AttackOutcome::Miss => Some(ScenarioOutcomeClass::AcceptedMiss),
        };
    }
    receipt.events.iter().find_map(|event| match event {
        DomainEvent::SavingThrowResolved { outcome, .. } => Some(match outcome {
            SavingThrowOutcome::Failed => ScenarioOutcomeClass::AcceptedHit,
            SavingThrowOutcome::Saved => ScenarioOutcomeClass::AcceptedMiss,
        }),
        DomainEvent::ContestedCheckResolved { outcome, .. } => Some(match outcome {
            ContestedCheckOutcome::ActorWins => ScenarioOutcomeClass::AcceptedHit,
            ContestedCheckOutcome::TargetWins => ScenarioOutcomeClass::AcceptedMiss,
        }),
        _ => None,
    })
}

fn compare_outcome(
    expected: ScenarioOutcomeClass,
    actual: Option<ScenarioOutcomeClass>,
) -> Option<ScenarioRegressionDifference> {
    let actual = actual
        .map(ScenarioOutcomeClass::code)
        .unwrap_or("unclassified");
    (expected.code() != actual).then(|| ScenarioRegressionDifference {
        path: "outcomeClass".to_string(),
        expected: expected.code().to_string(),
        actual: actual.to_string(),
    })
}

fn compare_receipts(
    expected: &RulebenchReceipt,
    actual: &RulebenchReceipt,
) -> Option<ScenarioRegressionDifference> {
    let comparisons = [
        (
            "decision.accepted",
            format!("{}", expected.accepted),
            format!("{}", actual.accepted),
        ),
        (
            "decision.rejection",
            format!("{:?}", expected.rejection),
            format!("{:?}", actual.rejection),
        ),
        (
            "acceptedEvents",
            format!("{:?}", expected.events),
            format!("{:?}", actual.events),
        ),
        (
            "rolls",
            format!("{:?}", expected.roll_consumption),
            format!("{:?}", actual.roll_consumption),
        ),
        (
            "trace",
            format!("{:?}", expected.trace),
            format!("{:?}", actual.trace),
        ),
        (
            "finalState",
            format!("{:?}", expected.projection),
            format!("{:?}", actual.projection),
        ),
    ];
    comparisons
        .into_iter()
        .find_map(|(path, expected, actual)| {
            (expected != actual).then(|| ScenarioRegressionDifference {
                path: path.to_string(),
                expected,
                actual,
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENTED_PROJECT_GATE_CASES: [&str; 3] = [
        "hexing-bolt-reaction",
        "watchtower-storm-pulse-multiple",
        "binding-glyph-failed-save",
    ];

    #[test]
    fn runner_executes_every_registered_package_reproducibly() {
        let report =
            run_scenario_regressions(&crate::scenario_package_registry(), &Default::default());

        assert!(report.accepted, "{:?}", report.first_difference);
        assert_eq!(report.cases.len(), 11);
        assert_eq!(
            report
                .cases
                .iter()
                .map(|case| case.package_id.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4,
        );
        assert!(report
            .cases
            .iter()
            .all(|case| !case.final_state_fingerprint.is_empty()));
    }

    #[test]
    fn runner_filters_exact_package_ruleset_version_and_scenario_identities() {
        let registry = crate::scenario_package_registry();
        let selected = &registry.registrations()[0];
        let selected_case = selected.scenario_catalog_cases()[0].summary.id.clone();
        let report = run_scenario_regressions(
            &registry,
            &ScenarioRegressionFilter {
                package_id: Some(selected.package.identity.id.clone()),
                package_version: Some(selected.package.identity.version.clone()),
                ruleset_id: Some(selected.package.ruleset.id.clone()),
                ruleset_version: Some(selected.package.ruleset.version.clone()),
                scenario_id: Some(selected_case.clone()),
            },
        );

        assert!(report.accepted);
        assert_eq!(report.cases.len(), 1);
        assert_eq!(report.cases[0].scenario_id, selected_case);
    }

    #[test]
    fn runner_filters_the_second_provider_by_exact_ruleset_identity() {
        let report = run_scenario_regressions(
            &crate::scenario_package_registry(),
            &ScenarioRegressionFilter {
                ruleset_id: Some(crate::TURN_CONTROL_RULESET_ID.to_string()),
                ruleset_version: Some(crate::TURN_CONTROL_RULESET_VERSION.to_string()),
                ..Default::default()
            },
        );

        assert!(report.accepted);
        assert_eq!(report.cases.len(), 2);
        assert!(report
            .cases
            .iter()
            .all(|case| case.ruleset_id == crate::TURN_CONTROL_RULESET_ID));
    }

    #[test]
    fn documented_project_gate_cases_are_independently_executable() {
        let registry = crate::scenario_package_registry();

        for scenario_id in DOCUMENTED_PROJECT_GATE_CASES {
            let regression = run_scenario_regressions(
                &registry,
                &ScenarioRegressionFilter {
                    scenario_id: Some(scenario_id.to_string()),
                    ..Default::default()
                },
            );
            assert!(
                regression.accepted,
                "documented gate scenario {scenario_id} failed regression selection: {:?}",
                regression.first_difference
            );
            assert_eq!(regression.cases.len(), 1, "{scenario_id}");

            let conformance = crate::run_capability_conformance(
                &registry,
                &crate::CapabilityConformanceFilter {
                    scenario_id: Some(scenario_id.to_string()),
                    ..Default::default()
                },
            );
            assert!(
                conformance.accepted,
                "documented gate scenario {scenario_id} failed conformance selection: {:?}",
                conformance.failures
            );
            assert_eq!(conformance.cases.len(), 1, "{scenario_id}");
        }
    }

    #[test]
    fn runner_reports_path_addressed_first_difference_and_empty_selection() {
        let mut cases = crate::aggregated_scenario_catalog_cases();
        let case = cases.remove(0);
        let expected = resolve_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
        let mut actual = expected.clone();
        actual.events.clear();
        assert_eq!(
            compare_receipts(&expected, &actual).unwrap().path,
            "acceptedEvents"
        );

        let report = run_scenario_regressions(
            &crate::scenario_package_registry(),
            &ScenarioRegressionFilter {
                package_id: Some("missing".to_string()),
                ..Default::default()
            },
        );
        assert!(!report.accepted);
        assert_eq!(report.first_difference.unwrap().path, "selection");
    }
}
