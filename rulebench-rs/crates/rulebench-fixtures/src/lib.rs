//! Rulebench-local scenarios, fixtures, and regression packs.
//!
//! This reserved boundary will own authored testbench content such as Hexing
//! Bolt. Game repositories should not need this crate to execute shared rules.

#![forbid(unsafe_code)]

mod package;

pub use package::{
    ScenarioPackage, ScenarioPackageContentReference, ScenarioPackageDisplayMetadata,
    ScenarioPackageEvidenceExpectation, ScenarioPackageEvidenceKind, ScenarioPackageIdentity,
    ScenarioPackageInitialState, ScenarioPackageRulesetReference, ScenarioPackageScript,
    ScenarioPackageValidationError,
};
