//! Rulebench-local scenarios, fixtures, and regression packs.
//!
//! This reserved boundary will own authored testbench content such as Hexing
//! Bolt. Game repositories should not need this crate to execute shared rules.

#![forbid(unsafe_code)]

mod catalog;
mod package;
pub mod scenarios;

pub use catalog::*;
pub use package::{
    ScenarioPackage, ScenarioPackageContentReference, ScenarioPackageDisplayMetadata,
    ScenarioPackageEvidenceExpectation, ScenarioPackageEvidenceKind, ScenarioPackageIdentity,
    ScenarioPackageInitialState, ScenarioPackageRulesetReference, ScenarioPackageScript,
    ScenarioPackageValidationError,
};
pub use scenarios::hexing_bolt::{
    accepted_hexing_bolt_fixture_receipt, combat_session_automatic_run_readouts,
    combat_session_automatic_run_replay_readouts, combat_session_control_history_readouts,
    combat_session_script_readouts, combat_session_transcripts, content_validation_readouts,
    hexing_bolt_fixture_scenario, hexing_bolt_scenario_package, rejected_target_fixture_receipt,
    ruleset_catalog_readout, scenario_catalog_cases, turn_control_fixture_scenario,
};
