//! Second package using the existing Hexing Bolt ruleset without new rule semantics.

use crate::{
    ContentValidationReadout, FixtureGoldenArtifact, FixtureGoldenArtifactKind,
    FixtureGoldenManifest, RulesetCatalogReadout, ScenarioCatalogCase, ScenarioCatalogSummary,
    ScenarioOutcomeClass, ScenarioPackage, ScenarioPackageContentReference,
    ScenarioPackageDisplayMetadata, ScenarioPackageEvidenceExpectation,
    ScenarioPackageEvidenceKind, ScenarioPackageIdentity, ScenarioPackageInitialState,
    ScenarioPackageReadbackFactories, ScenarioPackageRegistration, ScenarioPackageRulesetReference,
};
use rpg_core::*;
use rulebench_combat::*;
use rulebench_replay::*;

const PACKAGE_ID: &str = "asha-rulebench.hexing-bolt-veteran";
const PACKAGE_VERSION: &str = "0.1.0";
const CATALOG_CASE_ID: &str = "hexing-bolt-veteran-hit";
const SESSION_ID: &str = "hexing-bolt-veteran-opening";
const RECEIPT_ID: &str = "hexing-bolt-veteran-accepted-receipt";

pub fn registration() -> ScenarioPackageRegistration {
    ScenarioPackageRegistration::new(
        scenario_package(),
        ScenarioPackageReadbackFactories {
            catalog_cases: scenario_catalog_cases,
            ruleset_catalog_readout,
            content_validation_readouts,
            session_transcripts,
            control_history_readouts,
            script_readouts,
            automatic_run_readouts,
            automatic_run_replay_readouts,
        },
    )
}

pub fn scenario_package() -> ScenarioPackage {
    let scenario = scenario();
    ScenarioPackage {
        identity: ScenarioPackageIdentity {
            id: PACKAGE_ID.to_string(),
            version: PACKAGE_VERSION.to_string(),
        },
        display: ScenarioPackageDisplayMetadata {
            title: "Veteran Hexing Bolt Package".to_string(),
            summary: "A second package that reuses Hexing Bolt rule behavior with a veteran actor and altered starting state.".to_string(),
            tags: vec!["combat".to_string(), "hexing-bolt".to_string(), "veteran".to_string()],
        },
        ruleset: ScenarioPackageRulesetReference {
            id: scenario.selected_ruleset_id.clone(),
            version: scenario
                .selected_ruleset()
                .expect("veteran package selects the existing ruleset")
                .version
                .clone(),
        },
        content_references: vec![ScenarioPackageContentReference {
            id: "asha-rulebench.hexing-bolt.veteran.content".to_string(),
            version: PACKAGE_VERSION.to_string(),
        }],
        initial_state: ScenarioPackageInitialState {
            participant_ids: scenario
                .combatants
                .iter()
                .map(|combatant| combatant.id.clone())
                .collect(),
            scenario,
        },
        scripts: Vec::new(),
        expected_evidence: vec![
            evidence(CATALOG_CASE_ID, ScenarioPackageEvidenceKind::CatalogCase),
            evidence(SESSION_ID, ScenarioPackageEvidenceKind::SessionTranscript),
            evidence(RECEIPT_ID, ScenarioPackageEvidenceKind::Receipt),
        ],
        golden_manifest: FixtureGoldenManifest {
            package_id: PACKAGE_ID.to_string(),
            artifacts: vec![
                golden(
                    CATALOG_CASE_ID,
                    FixtureGoldenArtifactKind::ScenarioCatalog,
                    "pnpm run catalog:check",
                ),
                golden(
                    SESSION_ID,
                    FixtureGoldenArtifactKind::SessionTranscript,
                    "pnpm run session:check",
                ),
                golden(
                    RECEIPT_ID,
                    FixtureGoldenArtifactKind::Receipt,
                    "cargo test --manifest-path rulebench-rs/Cargo.toml -p rulebench-fixtures",
                ),
            ],
        },
    }
}

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    let scenario = scenario_with_metadata(
        CATALOG_CASE_ID,
        "Veteran Hexing Bolt Hit",
        "Veteran Adept hits Raider with the same registered Rust action behavior.",
        "roll-stream:17,5",
    );
    vec![ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: scenario.metadata.id.clone(),
            title: scenario.metadata.title.clone(),
            summary: scenario.metadata.summary.clone(),
            seed_label: scenario.metadata.seed_label.clone(),
            outcome_class: ScenarioOutcomeClass::AcceptedHit,
        },
        scenario,
        intent: veteran_intent(),
        roll_stream: vec![17, 5],
    }]
}

pub fn ruleset_catalog_readout() -> RulesetCatalogReadout {
    let scenario = scenario();
    RulesetCatalogReadout {
        selected_ruleset_id: scenario.selected_ruleset_id,
        rulesets: scenario.rulesets,
    }
}

pub fn content_validation_readouts() -> Vec<ContentValidationReadout> {
    let scenario = scenario();
    vec![ContentValidationReadout {
        scenario_id: scenario.metadata.id.clone(),
        scenario_title: scenario.metadata.title.clone(),
        report: validate_scenario_content_report(&scenario),
    }]
}

pub fn session_transcripts() -> Vec<CombatSessionTranscript> {
    let scenario = scenario_with_metadata(
        SESSION_ID,
        "Veteran Hexing Bolt Opening",
        "A deterministic opening transcript for the veteran package.",
        "roll-stream:17,5",
    );
    let mut session = CombatSessionState::new(SESSION_ID, scenario);
    let readout = session.submit_command(CombatSessionCommandSpec::new(
        "veteran-hexing-bolt-hit",
        "Veteran Adept hits Raider",
        "The registered Rust resolver accepts the veteran actor's Hexing Bolt intent.",
        CommandOutcomeClass::AcceptedHit,
        veteran_intent(),
        vec![17, 5],
    ));
    vec![CombatSessionTranscript {
        summary: CombatSessionSummary {
            id: SESSION_ID.to_string(),
            title: "Veteran Hexing Bolt Opening".to_string(),
            summary: "A deterministic opening transcript for the veteran package.".to_string(),
            seed_label: "roll-stream:17,5".to_string(),
            steps: vec![readout.step.clone()],
        },
        steps: vec![readout],
    }]
}

pub fn control_history_readouts() -> Vec<CombatControlHistoryReadout> {
    Vec::new()
}

pub fn script_readouts() -> Vec<CombatSessionScriptReadout> {
    Vec::new()
}

pub fn automatic_run_readouts() -> Vec<CombatSessionAutomaticRunReadout> {
    Vec::new()
}

pub fn automatic_run_replay_readouts() -> Vec<CombatSessionAutomaticRunReplayReadout> {
    Vec::new()
}

pub fn accepted_receipt() -> RulebenchReceipt {
    let scenario = scenario();
    resolve_use_action(&scenario, veteran_intent(), &[17, 5])
}

fn scenario() -> RulebenchScenario {
    scenario_with_metadata(
        "two-combatant-veteran-hexing-bolt",
        "Veteran Hexing Bolt Opening",
        "A second package with a veteran actor and different opening state on the existing ruleset.",
        "roll-stream:17,5",
    )
}

fn scenario_with_metadata(
    id: &str,
    title: &str,
    summary: &str,
    seed_label: &str,
) -> RulebenchScenario {
    let mut scenario = super::hexing_bolt::hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        seed_label: seed_label.to_string(),
    };
    scenario.entities[0].id = "entity.veteran-adept".to_string();
    scenario.entities[0].name = "Veteran Adept".to_string();
    scenario.combatants[0].id = "entity-veteran-adept".to_string();
    scenario.combatants[0].entity_id = "entity.veteran-adept".to_string();
    scenario.combatants[0].name = "Veteran Adept".to_string();
    scenario.combatants[0].position = GridPosition { x: 1, y: 2 };
    scenario.combatants[0].hit_points.current = 30;
    scenario.combatants[0].hit_points.max = 30;
    for action in &mut scenario.actions {
        if action.actor_id == "entity-adept" {
            action.actor_id = "entity-veteran-adept".to_string();
            action.id = action.id.replace("entity-adept", "entity-veteran-adept");
        }
        for target_id in &mut action.targeting.target_ids {
            if target_id == "entity-adept" {
                *target_id = "entity-veteran-adept".to_string();
            }
        }
        for target_id in &mut action.targeting.visible_target_ids {
            if target_id == "entity-adept" {
                *target_id = "entity-veteran-adept".to_string();
            }
        }
    }
    scenario.selected_action.actor_id = "entity-veteran-adept".to_string();
    scenario
}

fn veteran_intent() -> UseActionIntent {
    UseActionIntent::new("entity-veteran-adept", "hexing_bolt", "entity-raider")
}

fn evidence(id: &str, kind: ScenarioPackageEvidenceKind) -> ScenarioPackageEvidenceExpectation {
    ScenarioPackageEvidenceExpectation {
        id: id.to_string(),
        kind,
    }
}

fn golden(id: &str, kind: FixtureGoldenArtifactKind, check_command: &str) -> FixtureGoldenArtifact {
    FixtureGoldenArtifact {
        id: id.to_string(),
        kind,
        check_command: check_command.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veteran_package_reuses_the_hexing_bolt_ruleset_with_distinct_participants() {
        let package = scenario_package();

        assert!(package.validate().is_ok());
        assert_eq!(package.ruleset.id, "asha-rulebench.hexing-bolt.v0");
        assert_eq!(
            package.initial_state.scenario.combatants[0].id,
            "entity-veteran-adept"
        );
        assert_eq!(
            package.initial_state.scenario.combatants[0].hit_points.max,
            30
        );
        assert!(accepted_receipt().accepted);
    }
}
