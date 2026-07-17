use rpg_ir::RulesetMetadata;
use rulebench_combat::RulebenchReceipt;
use rulebench_content::{ContentValidationReport, RulebenchScenario, UseActionIntent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
}

impl ScenarioOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            ScenarioOutcomeClass::AcceptedHit => "acceptedHit",
            ScenarioOutcomeClass::AcceptedMiss => "acceptedMiss",
            ScenarioOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub outcome_class: ScenarioOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogCase {
    pub summary: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogResolution {
    pub case: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetCatalogReadout {
    pub selected_ruleset_id: String,
    pub rulesets: Vec<RulesetMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationReadout {
    pub scenario_id: String,
    pub scenario_title: String,
    pub report: ContentValidationReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioCatalogError {
    UnknownScenarioId,
}
