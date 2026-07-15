mod catalog;
mod fixture;
mod session;

pub use catalog::{content_validation_readouts, ruleset_catalog_readout, scenario_catalog_cases};
pub use fixture::{
    accepted_hexing_bolt_fixture_receipt, hexing_bolt_fixture_scenario,
    hexing_bolt_scenario_package, rejected_target_fixture_receipt, turn_control_fixture_scenario,
};
pub use session::{
    combat_session_automatic_run_readouts, combat_session_automatic_run_replay_readouts,
    combat_session_control_history_readouts, combat_session_script_readouts,
    combat_session_transcripts,
};

pub fn registration() -> crate::ScenarioPackageRegistration {
    crate::ScenarioPackageRegistration::new(
        hexing_bolt_scenario_package(),
        crate::ScenarioPackageReadbackFactories {
            catalog_cases: scenario_catalog_cases,
            ruleset_catalog_readout,
            content_validation_readouts,
            session_transcripts: combat_session_transcripts,
            control_history_readouts: combat_session_control_history_readouts,
            script_readouts: combat_session_script_readouts,
            automatic_run_readouts: combat_session_automatic_run_readouts,
            automatic_run_replay_readouts: combat_session_automatic_run_replay_readouts,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexing_bolt_package_owns_valid_data_and_expected_evidence() {
        let package = hexing_bolt_scenario_package();

        assert!(package.validate().is_ok());
        assert_eq!(package.identity.id, "asha-rulebench.hexing-bolt");
        assert_eq!(package.scripts.len(), 1);
        assert_eq!(package.expected_evidence.len(), 11);
        assert_eq!(package.golden_manifest.artifacts.len(), 11);
    }

    #[test]
    fn hexing_bolt_package_owns_deterministic_receipts_and_transcript_evidence() {
        let accepted_receipt = accepted_hexing_bolt_fixture_receipt();
        let rejected_receipt = rejected_target_fixture_receipt();

        assert!(accepted_receipt.accepted);
        assert!(!rejected_receipt.accepted);
        assert_eq!(scenario_catalog_cases().len(), 4);
        assert_eq!(combat_session_transcripts().len(), 1);
    }

    #[test]
    fn golden_manifest_rejects_an_expected_evidence_gap() {
        let mut package = hexing_bolt_scenario_package();
        package.golden_manifest.artifacts.pop();

        let errors = package
            .validate()
            .expect_err("package should reject incomplete golden coverage");
        let codes = errors.iter().map(|error| error.code()).collect::<Vec<_>>();

        assert_eq!(codes, vec!["expectedEvidenceMissingGoldenArtifact"]);
    }
}
