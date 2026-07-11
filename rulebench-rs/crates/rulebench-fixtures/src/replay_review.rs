use rulebench_rules::{
    record_replay_package, CombatControlCommandSpec, CombatSessionCreateRequest,
    CombatSessionIntentCommandSpec, ReplayCommand, ReplayCommandRecordingSpec, ReplayNarration,
    ReplayPackage, UseActionIntent,
};

use crate::{content_import_examples, hexing_bolt_fixture_scenario, ContentImportExampleOutcome};

pub fn replay_review_packages() -> Vec<ReplayPackage> {
    vec![
        replay_package("hexing-bolt-replay", "replay-session-expected", false),
        replay_package(
            "hexing-bolt-replay-explicit-start",
            "replay-session-actual",
            true,
        ),
    ]
}

fn replay_package(package_id: &str, session_id: &str, explicit_start: bool) -> ReplayPackage {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.content_pack_set = Some(
        content_import_examples()
            .into_iter()
            .find_map(|example| match example.outcome {
                ContentImportExampleOutcome::Accepted(imported) => {
                    Some(imported.resolved_set.reference)
                }
                ContentImportExampleOutcome::Rejected { .. } => None,
            })
            .expect("fixture content import includes an accepted pack set"),
    );
    let ruleset = scenario
        .selected_ruleset()
        .expect("fixture scenario selects a ruleset")
        .artifact_provenance();
    let mut commands = Vec::new();
    if explicit_start {
        commands.push(ReplayCommandRecordingSpec::new(
            "explicit-start",
            ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
        ));
    }
    commands.push(ReplayCommandRecordingSpec::new(
        "hexing-bolt-hit",
        ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
            "hexing-bolt-hit",
            "Adept hits Raider",
            "Replay fixture resolves Hexing Bolt through Rust authority.",
            UseActionIntent {
                actor_id: "entity-adept".to_string(),
                action_id: "hexing_bolt".to_string(),
                target_id: "entity-raider".to_string(),
                destination_cell: None,
            },
            vec![17, 5],
        )),
    ));
    commands.push(ReplayCommandRecordingSpec::new(
        "explicit-end",
        ReplayCommand::Control(CombatControlCommandSpec::explicit_end()),
    ));
    let command_summaries = if explicit_start {
        vec![
            "Combat starts explicitly.".to_string(),
            "Adept hits Raider with Hexing Bolt.".to_string(),
            "Combat ends explicitly.".to_string(),
        ]
    } else {
        vec![
            "Adept hits Raider with Hexing Bolt and starts combat implicitly.".to_string(),
            "Combat ends explicitly.".to_string(),
        ]
    };
    record_replay_package(
        package_id,
        CombatSessionCreateRequest::new(session_id, scenario),
        ruleset,
        commands,
    )
    .with_narration(ReplayNarration {
        title: "Hexing Bolt Replay".to_string(),
        summary: "Deterministic replay evidence for post-combat review.".to_string(),
        command_summaries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_rules::{compare_replay_packages, verify_replay_package};

    #[test]
    fn review_packages_verify_and_expose_a_structural_comparison() {
        let packages = replay_review_packages();

        let verifications = packages
            .iter()
            .map(verify_replay_package)
            .collect::<Vec<_>>();
        assert!(
            verifications
                .iter()
                .all(|verification| { verification.accepted && verification.finalized }),
            "{verifications:#?}"
        );
        let comparison = compare_replay_packages(&packages[0], &packages[1]);
        assert!(!comparison.matches);
        assert_eq!(
            comparison.first_difference.expect("first difference").path,
            "commands.length"
        );
    }
}
