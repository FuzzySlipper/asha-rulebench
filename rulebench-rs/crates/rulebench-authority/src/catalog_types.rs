//! Compatibility re-exports for Rulebench-local catalog and scenario types.
//!
//! The owning definitions live in `rulebench-fixtures`; this facade remains so
//! existing workbench callers can migrate without changing authority behavior.

pub use rulebench_fixtures::{
    ContentValidationReadout, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogError,
    ScenarioCatalogResolution, ScenarioCatalogSummary, ScenarioOutcomeClass,
};
