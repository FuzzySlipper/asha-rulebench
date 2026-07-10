use super::super::test_support::{hexing_bolt_scenario_package, ScenarioPackageValidationError};

#[test]
fn hexing_bolt_conforms_to_the_fixture_owned_scenario_package_contract() {
    let package = hexing_bolt_scenario_package();

    assert_eq!(package.identity.id, "asha-rulebench.hexing-bolt");
    assert!(package.validate().is_ok());
}

#[test]
fn scenario_package_validation_codes_are_stable_for_reference_and_evidence_failures() {
    let mut package = hexing_bolt_scenario_package();
    package.ruleset.version = "missing-version".to_string();
    package
        .expected_evidence
        .push(package.expected_evidence[0].clone());

    let errors = package
        .validate()
        .expect_err("package should reject invalid data");
    let codes = errors
        .iter()
        .map(ScenarioPackageValidationError::code)
        .collect::<Vec<_>>();

    assert_eq!(
        codes,
        vec!["referencedRulesetVersionMismatch", "duplicateEvidenceId"]
    );
}
