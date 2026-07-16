use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rulebench_protocol::{
    AutomaticRunRequestDto, AutomationPolicyCatalogEntryDto,
    CombatAutomationNoCandidateBehaviorDto, CombatAutomationPolicyDto, CombatControlCommandDto,
    CombatControlCommandKindDto, CombatSessionCreateRequestDto, CombatSessionIntentCommandDto,
    CommandRollModeDto, ContentImportAttemptDto, ContentWorkspaceDto,
    ExperimentComparisonReadoutDto, ExperimentComparisonRequestDto, ExperimentMatrixRequestDto,
    ExperimentReadoutDto, LiveAutomaticRunDto, LiveCommandExecutionDto, LiveControlExecutionDto,
    LiveReactionExecutionDto, LiveSessionSnapshotDto, ProtocolHandshakeDto, ReactionCommandSpecDto,
    ReactionResponseKindDto, ReplayArchiveMetadataDto, ReplayComparisonReadoutDto,
    ReplayComparisonRequestDto, ReplayPackageReviewDto, ReplayVerificationReadoutDto,
    RulebenchCapabilityManifestDto, ScenarioOptionDto, SessionRecoveryCatalogDto,
    SessionRecoveryForkRequestDto, UseActionIntentDto, ViewerScenarioReadoutDto,
    ViewerScenarioSummaryDto, ViewerSessionStepReadoutDto, ViewerSessionSummaryDto,
    PROTOCOL_VERSION,
};

use crate::{
    build_durable_rulebench_router, build_rulebench_bridge, serve_until, HttpMethod, HttpRequest,
    ProcessHostRouter,
};

static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn router() -> ProcessHostRouter {
    ProcessHostRouter::new(build_rulebench_bridge().expect("fixture catalog is valid"))
}

fn request(method: HttpMethod, path: &str) -> HttpRequest {
    HttpRequest::new(method, path)
        .with_header("x-rulebench-protocol-version", PROTOCOL_VERSION.to_string())
}

fn json_request<T>(method: HttpMethod, path: &str, body: &T) -> HttpRequest
where
    T: serde::Serialize,
{
    request(method, path)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_vec(body).expect("test DTO serializes"))
}

#[test]
fn capability_route_reports_registry_and_actual_host_composition() {
    let mut memory_router = router();
    let response =
        memory_router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/capabilities"));
    assert_eq!(response.status, 200);
    let manifest: RulebenchCapabilityManifestDto =
        serde_json::from_slice(&response.body).expect("capability manifest is JSON");

    assert_eq!(manifest.host.storage_mode, "memory");
    assert_eq!(
        manifest.host.session_recovery_mode,
        "processLocalCheckpoints"
    );
    assert_eq!(manifest.host.authority_viewer_mode, "liveAuthorityReadback");
    assert_eq!(manifest.providers.len(), 2);
    assert_eq!(manifest.rulesets.len(), 2);
    assert_eq!(manifest.packages.len(), 4);
    assert_eq!(manifest.scenarios.len(), 11);
    assert!(manifest.providers.iter().any(|provider| {
        provider.provider.id == "provider.asha-rulebench.turn-control"
            && provider.ruleset.id == "asha-rulebench.turn-control.v0"
            && provider
                .capabilities
                .iter()
                .any(|capability| capability.id == "check.savingThrow")
    }));
    let hexing_provider = manifest
        .providers
        .iter()
        .find(|provider| provider.provider.id == "provider.asha-rulebench.hexing-bolt")
        .expect("hexing provider is present");
    assert!(hexing_provider
        .capabilities
        .iter()
        .any(|capability| capability.id == "policy.lowestVitalityTarget"));
    assert!(!hexing_provider
        .capabilities
        .iter()
        .any(|capability| capability.id == "policy.objectiveSidePressure"));
    let turn_provider = manifest
        .providers
        .iter()
        .find(|provider| provider.provider.id == "provider.asha-rulebench.turn-control")
        .expect("turn-control provider is present");
    assert!(turn_provider
        .capabilities
        .iter()
        .any(|capability| capability.id == "policy.objectiveSidePressure"));
    for capability_id in [
        "policy.firstAcceptedCandidate",
        "policy.lowestVitalityTarget",
        "policy.objectiveSidePressure",
    ] {
        assert!(manifest.capabilities.iter().any(|capability| {
            capability.id == capability_id && capability.support.regression_covered
        }));
    }
    assert!(manifest.capabilities.iter().any(|capability| {
        capability.id == "targeting.multipleCombatants"
            && capability.support.runtime_executable
            && capability.support.regression_covered
    }));
    for capability_id in ["operation.heal", "operation.grantTemporaryVitality"] {
        assert!(manifest.capabilities.iter().any(|capability| {
            capability.id == capability_id
                && capability.support.runtime_executable
                && capability.support.regression_covered
        }));
    }
    assert!(manifest.capabilities.iter().any(|capability| {
        capability.id == "content.authored-pack"
            && !capability.support.runtime_executable
            && !capability.support.live_host_exposed
    }));
    assert!(manifest.capabilities.iter().any(|capability| {
        capability.id == "viewer.authority-readback"
            && capability.support.protocol_exposed
            && capability.support.live_host_exposed
            && capability.support.ui_exposed
            && capability.support.regression_covered
            && !capability.support.durable_across_restart
    }));
}

#[test]
fn policy_laboratory_routes_validate_advance_compare_cancel_and_archive() {
    let mut router = router();
    let catalog = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/automation-policies",
    ));
    assert_eq!(catalog.status, 200);
    let catalog: Vec<AutomationPolicyCatalogEntryDto> =
        serde_json::from_slice(&catalog.body).expect("policy catalog is JSON");
    assert_eq!(catalog.len(), 3);
    let objective = catalog
        .iter()
        .find(|policy| policy.id == "objectiveSidePressure")
        .expect("objective policy registration");
    assert!(objective.compatibility.iter().any(|compatibility| {
        compatibility.ruleset_id == "asha-rulebench.turn-control.v0" && compatibility.compatible
    }));
    assert!(objective.compatibility.iter().any(|compatibility| {
        compatibility.ruleset_id == "asha-rulebench.hexing-bolt.v0" && !compatibility.compatible
    }));

    let matrix = ExperimentMatrixRequestDto {
        id: "host-policy-lab".to_string(),
        scenario_ids: vec!["hexing-bolt-hit".to_string()],
        policies: vec![CombatAutomationPolicyDto {
            id: "lowestVitalityTarget".to_string(),
            version: 1,
            no_candidate_behavior: CombatAutomationNoCandidateBehaviorDto::AdvanceTurn,
        }],
        seeds: vec![7, 7],
        max_steps: 8,
    };
    let created = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/experiments",
        &matrix,
    ));
    assert_eq!(created.status, 200);
    let created: ExperimentReadoutDto =
        serde_json::from_slice(&created.body).expect("experiment creation is JSON");
    assert_eq!(created.planned_trial_count, 2);

    let first = router.handle(&request(
        HttpMethod::Post,
        "/api/rulebench/v1/experiments/host-policy-lab/advance",
    ));
    assert_eq!(first.status, 200);
    let first: ExperimentReadoutDto =
        serde_json::from_slice(&first.body).expect("first progress readout is JSON");
    assert_eq!(first.status, "running");
    assert!(first.trials[0].replay_verified);

    let completed = router.handle(&request(
        HttpMethod::Post,
        "/api/rulebench/v1/experiments/host-policy-lab/advance",
    ));
    let completed: ExperimentReadoutDto =
        serde_json::from_slice(&completed.body).expect("completed experiment is JSON");
    assert_eq!(completed.status, "completed");
    let comparison = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/experiments/compare",
        &ExperimentComparisonRequestDto {
            expected_experiment_id: completed.id.clone(),
            expected_trial_id: completed.trials[0].id.clone(),
            actual_experiment_id: completed.id.clone(),
            actual_trial_id: completed.trials[1].id.clone(),
        },
    ));
    let comparison: ExperimentComparisonReadoutDto =
        serde_json::from_slice(&comparison.body).expect("comparison is JSON");
    assert!(comparison.identical);

    let replay = router.handle(&request(
        HttpMethod::Get,
        &format!(
            "/api/rulebench/v1/replays/{}",
            completed.trials[0].replay_package_id
        ),
    ));
    assert_eq!(replay.status, 200);
}

#[test]
fn completed_experiment_trial_replay_survives_durable_host_restart() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-policy-lab-restart-{}-{sequence}",
        std::process::id()
    ));
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let matrix = ExperimentMatrixRequestDto {
        id: "restart-policy-lab".to_string(),
        scenario_ids: vec!["hexing-bolt-hit".to_string()],
        policies: vec![CombatAutomationPolicyDto {
            id: "firstAcceptedCandidate".to_string(),
            version: 1,
            no_candidate_behavior: CombatAutomationNoCandidateBehaviorDto::AdvanceTurn,
        }],
        seeds: vec![7],
        max_steps: 8,
    };
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/experiments",
                &matrix,
            ))
            .status,
        200
    );
    let completed = router.handle(&request(
        HttpMethod::Post,
        "/api/rulebench/v1/experiments/restart-policy-lab/advance",
    ));
    let completed: ExperimentReadoutDto =
        serde_json::from_slice(&completed.body).expect("completed trial is JSON");
    let replay_id = completed.trials[0].replay_package_id.clone();
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("durable router restarts");
    let replay = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/{replay_id}"),
    ));
    assert_eq!(replay.status, 200);
    let verified = restarted.handle(&request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/replays/{replay_id}/verify"),
    ));
    let verified: ReplayVerificationReadoutDto =
        serde_json::from_slice(&verified.body).expect("replay verification is JSON");
    assert!(verified.accepted);

    fs::remove_dir_all(&directory).expect("durable policy lab directory is removed");
}

#[test]
fn viewer_routes_expose_registry_driven_authority_readbacks() {
    let mut router = router();
    let scenarios = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/viewer/scenarios",
    ));
    assert_eq!(scenarios.status, 200);
    let scenarios: Vec<ViewerScenarioSummaryDto> =
        serde_json::from_slice(&scenarios.body).expect("viewer scenarios are JSON");
    assert_eq!(scenarios.len(), 11);
    let provider_scenario = scenarios
        .iter()
        .find(|scenario| scenario.id == "watchtower-storm-pulse-multiple")
        .expect("provider-registered scenario is exposed");

    let scenario = router.handle(&request(
        HttpMethod::Get,
        &format!(
            "/api/rulebench/v1/viewer/scenarios/{}",
            provider_scenario.id
        ),
    ));
    assert_eq!(scenario.status, 200);
    let scenario: ViewerScenarioReadoutDto =
        serde_json::from_slice(&scenario.body).expect("viewer scenario is JSON");
    assert_eq!(scenario.identity.id, provider_scenario.id);
    assert!(!scenario.domain_events.is_empty());
    assert!(!scenario.trace.is_empty());
    assert_eq!(scenario.board.width, 10);

    let sessions = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/viewer/sessions",
    ));
    assert_eq!(sessions.status, 200);
    let sessions: Vec<ViewerSessionSummaryDto> =
        serde_json::from_slice(&sessions.body).expect("viewer sessions are JSON");
    let provider_session = sessions
        .iter()
        .find(|session| session.id == "objective-turn-control-opening")
        .expect("provider-owned transcript is exposed");
    let step = provider_session
        .steps
        .first()
        .expect("provider transcript has a step");
    let step = router.handle(&request(
        HttpMethod::Get,
        &format!(
            "/api/rulebench/v1/viewer/sessions/{}/steps/{}",
            provider_session.id, step.id
        ),
    ));
    assert_eq!(step.status, 200);
    let step: ViewerSessionStepReadoutDto =
        serde_json::from_slice(&step.body).expect("viewer session step is JSON");
    assert_eq!(step.session_id, provider_session.id);
    assert!(!step.scenario.trace.is_empty());

    let missing_scenario = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/viewer/scenarios/missing",
    ));
    assert_eq!(missing_scenario.status, 404);
    let missing_step = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/viewer/sessions/objective-turn-control-opening/steps/missing",
    ));
    assert_eq!(missing_step.status, 400);
}

#[test]
fn router_serializes_lifecycle_and_isolates_multiple_sessions() {
    let mut router = router();
    let handshake = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/handshake"));
    assert_eq!(handshake.status, 200);
    let handshake: ProtocolHandshakeDto =
        serde_json::from_slice(&handshake.body).expect("handshake is JSON");
    assert_eq!(handshake.protocol_version, PROTOCOL_VERSION);

    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = &scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-hit")
        .expect("Hexing Bolt fixture scenario exists")
        .id;

    for session_id in ["first", "second"] {
        let created = router.handle(&json_request(
            HttpMethod::Post,
            "/api/rulebench/v1/sessions",
            &CombatSessionCreateRequestDto {
                session_id: session_id.to_string(),
                scenario_id: scenario_id.clone(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        ));
        assert_eq!(created.status, 200);
    }

    let started = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions/first/controls",
        &CombatControlCommandDto {
            kind: CombatControlCommandKindDto::ExplicitStart,
        },
    ));
    assert_eq!(started.status, 200);
    let started: LiveControlExecutionDto =
        serde_json::from_slice(&started.body).expect("control readout is JSON");
    assert!(started.accepted);

    let first = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/sessions/first",
    ));
    let second = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/sessions/second",
    ));
    let first: LiveSessionSnapshotDto =
        serde_json::from_slice(&first.body).expect("first snapshot is JSON");
    let second: LiveSessionSnapshotDto =
        serde_json::from_slice(&second.body).expect("second snapshot is JSON");
    assert_eq!(first.lifecycle_phase, "inProgress");
    assert_eq!(second.lifecycle_phase, "ready");
    assert_eq!(first.board.width, 6);
    assert_eq!(first.board.height, 4);
    assert_eq!(first.participants[0].position.x, 1);
    assert!(first.board.cells.iter().any(|cell| {
        cell.position.x == 1 && cell.position.y == 1 && cell.occupant_ids == vec!["entity-adept"]
    }));
}

#[test]
fn router_classifies_version_serialization_handle_and_lifecycle_errors() {
    let mut router = router();
    let version = router.handle(
        &HttpRequest::new(HttpMethod::Get, "/api/rulebench/v1/handshake")
            .with_header("x-rulebench-protocol-version", "999"),
    );
    assert_eq!(version.status, 409);

    let malformed = router
        .handle(&request(HttpMethod::Post, "/api/rulebench/v1/sessions").with_body(b"{}".to_vec()));
    assert_eq!(malformed.status, 400);

    let unknown = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/sessions/missing",
    ));
    assert_eq!(unknown.status, 404);

    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let created = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: "ready".to_string(),
            scenario_id: scenarios[0].id.clone(),
            participant_order: Vec::new(),
            content_pack: None,
        },
    ));
    assert_eq!(created.status, 200);
    let close = router.handle(&request(
        HttpMethod::Delete,
        "/api/rulebench/v1/sessions/ready",
    ));
    assert_eq!(close.status, 422);
}

#[test]
fn router_exposes_rust_replay_review_verification_and_comparison() {
    let mut router = router();
    let listed = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/replays"));
    assert_eq!(listed.status, 200);
    let packages: Vec<ReplayArchiveMetadataDto> =
        serde_json::from_slice(&listed.body).expect("replay archive list is JSON");
    assert_eq!(packages.len(), 3);

    let expected_id = &packages
        .iter()
        .find(|package| package.package_id == "hexing-bolt-replay")
        .expect("expected replay is listed")
        .package_id;
    let actual_id = &packages
        .iter()
        .find(|package| package.package_id != **expected_id)
        .expect("comparison replay is listed")
        .package_id;
    let loaded = router.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/{expected_id}"),
    ));
    let review: ReplayPackageReviewDto =
        serde_json::from_slice(&loaded.body).expect("replay review is JSON");
    assert!(!review.commands.is_empty());
    assert!(!review.commands[0].actual.accepted_events.is_empty());
    assert!(!review.commands[0].actual.rolls.is_empty());
    assert!(!review.commands[0].actual.trace.is_empty());
    assert!(!review.commands[0].snapshot.combat_log.is_empty());

    let verified = router.handle(&request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/replays/{expected_id}/verify"),
    ));
    let verification: ReplayVerificationReadoutDto =
        serde_json::from_slice(&verified.body).expect("verification is JSON");
    assert!(verification.accepted);
    assert!(verification.finalized);

    let compared = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/replays/compare",
        &ReplayComparisonRequestDto {
            expected_package_id: expected_id.clone(),
            actual_package_id: actual_id.clone(),
        },
    ));
    let comparison: ReplayComparisonReadoutDto =
        serde_json::from_slice(&compared.body).expect("comparison is JSON");
    assert!(!comparison.matches);
    assert!(comparison.first_difference.is_some());

    let missing = router.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/replays/missing",
    ));
    assert_eq!(missing.status, 404);
}

#[test]
fn durable_router_reports_repository_state_and_reloads_a_finalized_live_replay() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-durable-host-{}-{sequence}",
        std::process::id()
    ));
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    assert_eq!(router.repository_status().mode, "filesystem");
    assert_eq!(router.repository_status().replay_artifact_count, 3);
    assert!(router.content_storage().is_some());

    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-hit")
        .expect("finalizable fixture exists")
        .id
        .clone();
    let session_id = "durable-live-session";
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/sessions",
                &CombatSessionCreateRequestDto {
                    session_id: session_id.to_string(),
                    scenario_id,
                    participant_order: Vec::new(),
                    content_pack: None,
                },
            ))
            .status,
        200
    );
    for kind in [
        CombatControlCommandKindDto::ExplicitStart,
        CombatControlCommandKindDto::ExplicitEnd,
    ] {
        assert_eq!(
            router
                .handle(&json_request(
                    HttpMethod::Post,
                    &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
                    &CombatControlCommandDto { kind },
                ))
                .status,
            200
        );
    }
    assert_eq!(
        router
            .handle(&request(
                HttpMethod::Delete,
                &format!("/api/rulebench/v1/sessions/{session_id}"),
            ))
            .status,
        200
    );
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("durable router restarts");
    let listed = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/replays"));
    let packages: Vec<ReplayArchiveMetadataDto> =
        serde_json::from_slice(&listed.body).expect("replay list is JSON");
    assert!(
        packages
            .iter()
            .any(|package| package.package_id == format!("live-{session_id}")),
        "packages={packages:?} status={:?}",
        restarted.repository_status()
    );
    let loaded = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/live-{session_id}"),
    ));
    assert_eq!(loaded.status, 200);
    let verified = restarted.handle(&request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/replays/live-{session_id}/verify"),
    ));
    let verified: ReplayVerificationReadoutDto =
        serde_json::from_slice(&verified.body).expect("restarted replay verification is JSON");
    assert!(verified.accepted);
    assert!(verified.finalized);
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn durable_router_reconstructs_an_active_session_and_continues_it_exactly() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-active-recovery-{}-{sequence}",
        std::process::id()
    ));
    let session_id = "restart-safe-active";
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-hit")
        .expect("recoverable fixture exists")
        .id
        .clone();
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/sessions",
                &CombatSessionCreateRequestDto {
                    session_id: session_id.to_string(),
                    scenario_id,
                    participant_order: Vec::new(),
                    content_pack: None,
                },
            ))
            .status,
        200
    );
    let started = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
        &CombatControlCommandDto {
            kind: CombatControlCommandKindDto::ExplicitStart,
        },
    ));
    assert_eq!(started.status, 200);
    let before_restart = router.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    let before_restart: LiveSessionSnapshotDto =
        serde_json::from_slice(&before_restart.body).expect("active snapshot is JSON");
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("active session reconstructs");
    let after_restart = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    assert_eq!(after_restart.status, 200);
    let after_restart: LiveSessionSnapshotDto =
        serde_json::from_slice(&after_restart.body).expect("restored snapshot is JSON");
    assert_eq!(after_restart, before_restart);
    let recovery = restarted.handle(&request(
        HttpMethod::Get,
        "/api/rulebench/v1/session-recovery",
    ));
    let recovery: SessionRecoveryCatalogDto =
        serde_json::from_slice(&recovery.body).expect("recovery catalog is JSON");
    let restored_entry = recovery
        .sessions
        .iter()
        .find(|entry| entry.session_id == session_id)
        .expect("restored session is classified");
    assert_eq!(restored_entry.origin, "restored");
    assert_eq!(restored_entry.state, "recoverable");
    assert_eq!(restored_entry.generation, 1);

    let fork_id = "restart-safe-active-fork";
    let forked = restarted.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/session-recovery/{session_id}/fork"),
        &SessionRecoveryForkRequestDto {
            new_session_id: fork_id.to_string(),
        },
    ));
    let forked: LiveSessionSnapshotDto =
        serde_json::from_slice(&forked.body).expect("forked snapshot is JSON");
    assert_eq!(forked.session_id, fork_id);
    assert_eq!(forked.state_fingerprint, after_restart.state_fingerprint);
    assert_eq!(
        restarted
            .handle(&request(
                HttpMethod::Delete,
                &format!("/api/rulebench/v1/session-recovery/{fork_id}"),
            ))
            .status,
        200
    );

    let ended = restarted.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
        &CombatControlCommandDto {
            kind: CombatControlCommandKindDto::ExplicitEnd,
        },
    ));
    assert_eq!(ended.status, 200);
    assert_eq!(
        restarted
            .handle(&request(
                HttpMethod::Delete,
                &format!("/api/rulebench/v1/sessions/{session_id}"),
            ))
            .status,
        200
    );
    drop(restarted);

    let mut finalized =
        build_durable_rulebench_router(&directory).expect("finalized repository reopens");
    let sessions = finalized.handle(&request(HttpMethod::Get, "/api/rulebench/v1/sessions"));
    let sessions: Vec<LiveSessionSnapshotDto> =
        serde_json::from_slice(&sessions.body).expect("session list is JSON");
    assert!(sessions.is_empty());
    let replay = finalized.handle(&request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/replays/live-{session_id}/verify"),
    ));
    let replay: ReplayVerificationReadoutDto =
        serde_json::from_slice(&replay.body).expect("recovered replay verification is JSON");
    assert!(replay.accepted);
    assert!(replay.finalized);
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn authored_content_survives_restart_binds_a_session_and_replay_to_the_exact_pack() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-authored-content-{}-{sequence}",
        std::process::id()
    ));
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario = scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-hit")
        .expect("authored-content compatible fixture exists");
    let payload = authored_content_payload(scenario, "Durable Authored Pack", 1);
    let imported = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/import",
        &serde_json::json!({
            "authoredPayload": payload,
            "replacementPolicy": "reject"
        }),
    ));
    assert_eq!(imported.status, 200);
    let imported: ContentImportAttemptDto =
        serde_json::from_slice(&imported.body).expect("import attempt is JSON");
    assert!(imported.accepted, "{imported:?}");
    let reference = imported
        .outcome
        .as_ref()
        .expect("accepted import has outcome")
        .review
        .pack
        .reference
        .clone();

    let activated = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/activate",
        &serde_json::json!({ "reference": reference }),
    ));
    assert_eq!(activated.status, 200);
    let activated: ContentWorkspaceDto =
        serde_json::from_slice(&activated.body).expect("activation is JSON");
    assert!(activated.packs.iter().all(|pack| pack.active));

    let unsupported = authored_content_payload(scenario, "Rejected Replacement", 99);
    let rejected = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/import",
        &serde_json::json!({
            "authoredPayload": unsupported,
            "replacementPolicy": "replaceSameIdentity"
        }),
    ));
    let rejected: ContentImportAttemptDto =
        serde_json::from_slice(&rejected.body).expect("rejection is JSON");
    assert!(!rejected.accepted);
    assert_eq!(
        rejected.error_code.as_deref(),
        Some("unsupportedAuthoredContentVersion")
    );
    let retained = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let retained: ContentWorkspaceDto =
        serde_json::from_slice(&retained.body).expect("workspace is JSON");
    assert_eq!(retained.packs.len(), 1);
    assert!(retained.packs[0].active);
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("content repository restarts");
    let workspace = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace.body).expect("restarted workspace is JSON");
    assert_eq!(workspace.packs.len(), 1);
    assert!(workspace.packs[0].active);
    assert_eq!(workspace.packs[0].reference, reference);

    let second_ruleset_scenario = scenarios
        .iter()
        .find(|candidate| candidate.id == "binding-glyph-failed-save")
        .expect("second provider scenario exists");
    let incompatible = restarted.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: "incompatible-authored-content-session".to_string(),
            scenario_id: second_ruleset_scenario.id.clone(),
            participant_order: Vec::new(),
            content_pack: Some(reference.clone()),
        },
    ));
    assert_eq!(incompatible.status, 422);
    let incompatible: serde_json::Value =
        serde_json::from_slice(&incompatible.body).expect("compatibility error is JSON");
    assert_eq!(incompatible["code"], "incompatibleSessionRuleset");
    let sessions = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/sessions"));
    let sessions: Vec<LiveSessionSnapshotDto> =
        serde_json::from_slice(&sessions.body).expect("session list is JSON");
    assert!(sessions.is_empty());

    let session_id = "authored-content-session";
    let created = restarted.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: session_id.to_string(),
            scenario_id: scenario.id.clone(),
            participant_order: Vec::new(),
            content_pack: Some(reference.clone()),
        },
    ));
    assert_eq!(
        created.status,
        200,
        "{}",
        String::from_utf8_lossy(&created.body)
    );
    for kind in [
        CombatControlCommandKindDto::ExplicitStart,
        CombatControlCommandKindDto::ExplicitEnd,
    ] {
        assert_eq!(
            restarted
                .handle(&json_request(
                    HttpMethod::Post,
                    &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
                    &CombatControlCommandDto { kind },
                ))
                .status,
            200
        );
    }
    let closed = restarted.handle(&request(
        HttpMethod::Delete,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    assert_eq!(
        closed.status,
        200,
        "{}",
        String::from_utf8_lossy(&closed.body)
    );
    let replay = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/live-{session_id}"),
    ));
    let replay: ReplayPackageReviewDto =
        serde_json::from_slice(&replay.body).expect("replay review is JSON");
    assert_eq!(replay.content_pack_root, Some(reference.clone()));
    assert_eq!(replay.content_pack_references, vec![reference]);
    assert!(replay.content_pack_set_fingerprint.is_some());

    let workspace = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace.body).expect("audited workspace is JSON");
    assert!(workspace
        .audit
        .iter()
        .any(|entry| entry.operation == "contentBoundToSession"));
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn shipped_authored_content_versions_import_through_the_same_rust_workspace() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-authored-version-fixtures-{}-{sequence}",
        std::process::id()
    ));
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let fixtures = [
        include_str!("fixtures/authored-content-v1.json"),
        include_str!("fixtures/authored-content-v2.json"),
        include_str!("fixtures/authored-content-v3.json"),
    ];

    for payload in fixtures {
        let imported = router.handle(&json_request(
            HttpMethod::Post,
            "/api/rulebench/v1/content/import",
            &serde_json::json!({
                "authoredPayload": payload,
                "replacementPolicy": "reject"
            }),
        ));
        assert_eq!(
            imported.status,
            200,
            "{}",
            String::from_utf8_lossy(&imported.body)
        );
        let imported: ContentImportAttemptDto =
            serde_json::from_slice(&imported.body).expect("fixture import is JSON");
        assert!(imported.accepted, "{imported:?}");
    }

    let workspace = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace.body).expect("workspace is JSON");
    assert_eq!(workspace.packs.len(), 3);
    let v1 = workspace
        .packs
        .iter()
        .find(|pack| pack.reference.id == "pack.fixture.authored.v1")
        .expect("v1 receipt exists");
    assert!(v1
        .definitions
        .iter()
        .any(|definition| definition.kind == "entity"));
    assert_eq!(v1.reference.fingerprint.value, "673a6a29efafa979");
    let v2 = workspace
        .packs
        .iter()
        .find(|pack| pack.reference.id == "pack.fixture.authored.v2")
        .expect("v2 receipt exists");
    assert!(v2.definitions.iter().any(|definition| {
        definition.kind == "ability" && definition.id == "ability.binding-glyph"
    }));
    assert_eq!(v2.reference.fingerprint.value, "938acf1dca484c9c");
    let v3 = workspace
        .packs
        .iter()
        .find(|pack| pack.reference.id == "pack.fixture.authored.v3")
        .expect("v3 receipt exists");
    assert_eq!(
        v3.reference.fingerprint.algorithm,
        "fnv1a64.rulebench-content-pack.v1"
    );
    assert_eq!(v3.reference.fingerprint.value, "cf89ac91fa911871");
    assert!(v3.definitions.iter().any(|definition| {
        definition.kind == "modifier" && definition.id == "modifier.binding-glyph.anchored"
    }));
    assert!(v3.definitions.iter().any(|definition| {
        definition.kind == "action" && definition.id == "action.binding-glyph"
    }));
    let references = workspace
        .packs
        .iter()
        .map(|pack| pack.reference.clone())
        .collect::<Vec<_>>();
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("all shipped versions reload");
    let workspace = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace.body).expect("restarted workspace is JSON");
    assert_eq!(
        workspace
            .packs
            .iter()
            .map(|pack| pack.reference.clone())
            .collect::<Vec<_>>(),
        references
    );
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn authored_content_validation_returns_a_receipt_without_mutating_the_workspace() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-authored-validation-{}-{sequence}",
        std::process::id()
    ));
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let before = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let before: ContentWorkspaceDto =
        serde_json::from_slice(&before.body).expect("initial workspace is JSON");

    let validated = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/validate",
        &serde_json::json!({
            "authoredPayload": include_str!("fixtures/authored-content-v3.json")
        }),
    ));
    assert_eq!(validated.status, 200);
    let validated: ContentImportAttemptDto =
        serde_json::from_slice(&validated.body).expect("validation attempt is JSON");
    assert!(validated.accepted, "{validated:?}");
    assert!(validated.outcome.is_none());
    assert_eq!(
        validated
            .pack
            .fingerprint
            .as_ref()
            .expect("accepted validation has a canonical receipt")
            .algorithm,
        "fnv1a64.rulebench-content-pack.v1"
    );

    let mut invalid: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/authored-content-v3.json"))
            .expect("v3 fixture is JSON");
    invalid["pack"]["catalogs"]["actions"][0]["effects"][1]["modifierId"] =
        serde_json::Value::String("modifier.missing".to_string());
    let invalid_payload = serde_json::to_string(&invalid).expect("invalid fixture serializes");
    let dry_run_rejection = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/validate",
        &serde_json::json!({ "authoredPayload": invalid_payload }),
    ));
    let dry_run_rejection: ContentImportAttemptDto =
        serde_json::from_slice(&dry_run_rejection.body).expect("dry-run rejection is JSON");
    let import_rejection = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/import",
        &serde_json::json!({
            "authoredPayload": invalid_payload,
            "replacementPolicy": "reject"
        }),
    ));
    let import_rejection: ContentImportAttemptDto =
        serde_json::from_slice(&import_rejection.body).expect("import rejection is JSON");
    assert!(!dry_run_rejection.accepted);
    assert_eq!(dry_run_rejection.pack, import_rejection.pack);
    assert_eq!(dry_run_rejection.diagnostics, import_rejection.diagnostics);
    assert_eq!(dry_run_rejection.error_code, import_rejection.error_code);
    assert_eq!(
        dry_run_rejection.error_message,
        import_rejection.error_message
    );

    let after = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let after: ContentWorkspaceDto =
        serde_json::from_slice(&after.body).expect("final workspace is JSON");
    assert_eq!(after, before);
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("workspace restarts after dry-run");
    let restarted = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let restarted: ContentWorkspaceDto =
        serde_json::from_slice(&restarted.body).expect("restarted workspace is JSON");
    assert_eq!(restarted, before);
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn authored_ability_v2_survives_restart_and_binds_second_provider_replay() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-authored-ability-v2-{}-{sequence}",
        std::process::id()
    ));
    let payload = include_str!("fixtures/authored-content-v2.json");
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let imported = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/import",
        &serde_json::json!({
            "authoredPayload": payload,
            "replacementPolicy": "reject"
        }),
    ));
    let imported: ContentImportAttemptDto =
        serde_json::from_slice(&imported.body).expect("v2 import is JSON");
    assert!(imported.accepted, "{imported:?}");
    let reference = imported
        .outcome
        .as_ref()
        .expect("accepted import has review")
        .review
        .pack
        .reference
        .clone();
    assert!(imported
        .outcome
        .as_ref()
        .expect("accepted import has review")
        .review
        .pack
        .definitions
        .iter()
        .any(|definition| {
            definition.kind == "ability" && definition.id == "ability.binding-glyph"
        }));

    let activated = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/activate",
        &serde_json::json!({ "reference": reference }),
    ));
    assert_eq!(activated.status, 200);

    let mut invalid_replacement: serde_json::Value =
        serde_json::from_str(payload).expect("fixture is JSON");
    invalid_replacement["pack"]["catalogs"]["abilities"][0]["summary"] =
        serde_json::Value::String(String::new());
    let rejected = router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/content/import",
        &serde_json::json!({
            "authoredPayload": serde_json::to_string_pretty(&invalid_replacement).expect("replacement serializes"),
            "replacementPolicy": "replaceSameIdentity"
        }),
    ));
    let rejected: ContentImportAttemptDto =
        serde_json::from_slice(&rejected.body).expect("rejection is JSON");
    assert!(!rejected.accepted);
    assert_eq!(
        rejected.error_code.as_deref(),
        Some("emptyContentImportField")
    );
    assert!(rejected.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "emptyContentImportField"
            && diagnostic.path == "catalogs.abilities[0].summary"
            && diagnostic.definition_kind.as_deref() == Some("ability")
            && diagnostic.reference_id.as_deref() == Some("ability.binding-glyph")
    }));
    let retained = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let retained: ContentWorkspaceDto =
        serde_json::from_slice(&retained.body).expect("retained workspace is JSON");
    assert_eq!(retained.packs.len(), 1);
    assert!(retained.packs[0].active);
    assert_eq!(retained.packs[0].reference, reference);
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("v2 repository reopens and revalidates");
    let workspace = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/content"));
    let workspace: ContentWorkspaceDto =
        serde_json::from_slice(&workspace.body).expect("restarted workspace is JSON");
    assert_eq!(workspace.packs.len(), 1);
    assert!(workspace.packs[0].active);
    assert_eq!(workspace.packs[0].reference, reference);
    assert!(workspace.packs[0].definitions.iter().any(|definition| {
        definition.kind == "ability" && definition.id == "ability.binding-glyph"
    }));

    let scenarios = restarted.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario = scenarios
        .iter()
        .find(|scenario| scenario.id == "binding-glyph-failed-save")
        .expect("second provider scenario exists");
    let session_id = "authored-ability-v2-session";
    let created = restarted.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: session_id.to_string(),
            scenario_id: scenario.id.clone(),
            participant_order: Vec::new(),
            content_pack: Some(reference.clone()),
        },
    ));
    assert_eq!(
        created.status,
        200,
        "{}",
        String::from_utf8_lossy(&created.body)
    );
    for kind in [
        CombatControlCommandKindDto::ExplicitStart,
        CombatControlCommandKindDto::ExplicitEnd,
    ] {
        assert_eq!(
            restarted
                .handle(&json_request(
                    HttpMethod::Post,
                    &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
                    &CombatControlCommandDto { kind },
                ))
                .status,
            200
        );
    }
    assert_eq!(
        restarted
            .handle(&request(
                HttpMethod::Delete,
                &format!("/api/rulebench/v1/sessions/{session_id}"),
            ))
            .status,
        200
    );
    let replay = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/live-{session_id}"),
    ));
    let replay: ReplayPackageReviewDto =
        serde_json::from_slice(&replay.body).expect("v2-bound replay is JSON");
    assert_eq!(replay.content_pack_root, Some(reference.clone()));
    assert_eq!(replay.content_pack_references, vec![reference]);
    assert!(replay.content_pack_set_fingerprint.is_some());
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

fn authored_content_payload(
    scenario: &ScenarioOptionDto,
    title: &str,
    format_version: u32,
) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "format": "asha-rulebench.content-pack",
        "formatVersion": format_version,
        "pack": {
            "id": "pack.authored.durable",
            "version": "1.0.0",
            "title": title,
            "summary": "Authored host lifecycle integration fixture.",
            "tags": ["authored", "integration"],
            "provenance": {
                "sourceKind": "authoredFile",
                "sourceId": "fixture:authored-content",
                "authoredBy": "Rulebench integration test"
            },
            "rulesetId": scenario.ruleset_id,
            "dependencies": [],
            "catalogs": {
                "rulesets": [{
                    "id": scenario.ruleset_id,
                    "name": "Authored Compatible Ruleset",
                    "version": scenario.ruleset_version,
                    "summary": "Matches the selected live scenario authority modules.",
                    "modules": [{
                        "module": "actionResolution",
                        "version": "1",
                        "configuration": {
                            "module": "actionResolution",
                            "targetingPolicy": "declaredTargetsAndLineOfSight",
                            "supportedCheckHandlers": ["attackVsDefense"]
                        }
                    }]
                }],
                "entities": [{
                    "id": "entity.authored-review",
                    "name": "Authored Review Entity",
                    "summary": "Proves generic canonical definition review.",
                    "tags": ["review"],
                    "damageAdjustments": [{
                        "damageType": "arcane",
                        "policy": "resistance"
                    }]
                }]
            }
        }
    }))
    .expect("authored fixture serializes")
}

#[test]
fn router_completes_rejects_stale_and_archives_the_live_reaction_workflow() {
    let mut router = router();
    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-reaction")
        .expect("reaction scenario exists")
        .id
        .clone();
    let session_id = "reaction-route";
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/sessions",
                &CombatSessionCreateRequestDto {
                    session_id: session_id.to_owned(),
                    scenario_id,
                    participant_order: Vec::new(),
                    content_pack: None,
                },
            ))
            .status,
        200
    );
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
                &CombatControlCommandDto {
                    kind: CombatControlCommandKindDto::ExplicitStart,
                },
            ))
            .status,
        200
    );
    let submitted = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/intents"),
        &CombatSessionIntentCommandDto {
            id: "reaction-action".to_owned(),
            title: "Hexing Bolt".to_owned(),
            summary: "Open the authored reaction window.".to_owned(),
            intent: UseActionIntentDto {
                actor_id: "entity-adept".to_owned(),
                action_id: "hexing_bolt".to_owned(),
                target_id: "entity-raider".to_owned(),
                target_ids: Vec::new(),
                target_cell: None,
                destination_cell: None,
                observed_origin: None,
            },
            roll_stream: vec![17, 5],
            roll_mode: CommandRollModeDto::Supplied,
            generated_seed: None,
        },
    ));
    let submitted: LiveCommandExecutionDto =
        serde_json::from_slice(&submitted.body).expect("intent execution is JSON");
    let opened = submitted
        .snapshot
        .current_reaction_window
        .expect("reaction window is open");
    assert_eq!(opened.current_reactor_id.as_deref(), Some("entity-adept"));

    let stopped = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/automatic-run"),
        &AutomaticRunRequestDto {
            id: "reaction-paused-automatic-run".to_string(),
            title: "Reaction-paused run".to_string(),
            summary: "Automation must yield to explicit reaction response.".to_string(),
            max_steps: 8,
            roll_stream: vec![17, 5],
            policy: CombatAutomationPolicyDto {
                id: "firstAcceptedCandidate".to_string(),
                version: 1,
                no_candidate_behavior: CombatAutomationNoCandidateBehaviorDto::AdvanceTurn,
            },
            roll_mode: CommandRollModeDto::Supplied,
            generated_seed: None,
        },
    ));
    let stopped: LiveAutomaticRunDto =
        serde_json::from_slice(&stopped.body).expect("reaction stop is JSON");
    assert!(stopped.accepted);
    assert_eq!(stopped.decision_kind, "stoppedReactionWindow");
    assert_eq!(stopped.executed_step_count, 1);
    assert_eq!(
        stopped.final_snapshot.state_fingerprint,
        submitted.snapshot.state_fingerprint
    );

    let pass = ReactionCommandSpecDto {
        window_id: opened.id.clone(),
        reactor_id: "entity-adept".to_owned(),
        response_kind: ReactionResponseKindDto::Pass,
        option_id: None,
    };
    let passed = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/reactions"),
        &pass,
    ));
    let passed: LiveReactionExecutionDto =
        serde_json::from_slice(&passed.body).expect("reaction execution is JSON");
    assert!(passed.reaction.accepted);
    assert_eq!(
        passed
            .snapshot
            .current_reaction_window
            .as_ref()
            .and_then(|window| window.current_reactor_id.as_deref()),
        Some("entity-raider")
    );

    let fingerprint_before_stale = passed.snapshot.state_fingerprint.clone();
    let stale = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/reactions"),
        &pass,
    ));
    let stale: LiveReactionExecutionDto =
        serde_json::from_slice(&stale.body).expect("stale reaction execution is JSON");
    assert!(!stale.reaction.accepted);
    assert_eq!(stale.reaction.decision_kind, "rejectedOutOfOrder");
    assert_eq!(stale.snapshot.state_fingerprint, fingerprint_before_stale);

    let accepted = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/reactions"),
        &ReactionCommandSpecDto {
            window_id: opened.id,
            reactor_id: "entity-raider".to_owned(),
            response_kind: ReactionResponseKindDto::Accept,
            option_id: Some("raider-ward".to_owned()),
        },
    ));
    let accepted: LiveReactionExecutionDto =
        serde_json::from_slice(&accepted.body).expect("accepted reaction execution is JSON");
    assert!(accepted.reaction.accepted);
    assert!(accepted.reaction.resumed_pending_resolution);
    assert!(accepted.snapshot.current_reaction_window.is_none());
    assert_eq!(accepted.snapshot.gameplay_fabric.pending_decision_count, 0);
    assert_eq!(
        accepted
            .snapshot
            .participants
            .iter()
            .find(|participant| participant.id == "entity-raider")
            .expect("Raider remains in snapshot")
            .current_hit_points,
        11
    );

    router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
        &CombatControlCommandDto {
            kind: CombatControlCommandKindDto::ExplicitEnd,
        },
    ));
    assert_eq!(
        router
            .handle(&request(
                HttpMethod::Delete,
                &format!("/api/rulebench/v1/sessions/{session_id}"),
            ))
            .status,
        200
    );
    let loaded = router.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/replays/live-{session_id}"),
    ));
    let replay: ReplayPackageReviewDto =
        serde_json::from_slice(&loaded.body).expect("live reaction replay is JSON");
    assert_eq!(
        replay
            .commands
            .iter()
            .filter(|command| command.command_kind == "reaction")
            .count(),
        3
    );
    let verified = router.handle(&request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/replays/live-{session_id}/verify"),
    ));
    let verified: ReplayVerificationReadoutDto =
        serde_json::from_slice(&verified.body).expect("live reaction replay verification is JSON");
    assert!(verified.accepted);
    assert!(verified.finalized);
}

#[test]
fn durable_router_reconstructs_a_suspended_reaction_before_resuming_it() {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "asha-rulebench-reaction-recovery-{}-{sequence}",
        std::process::id()
    ));
    let session_id = "recovered-reaction";
    let mut router = build_durable_rulebench_router(&directory).expect("durable router opens");
    let scenarios = router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = scenarios
        .iter()
        .find(|scenario| scenario.id == "hexing-bolt-reaction")
        .expect("reaction scenario exists")
        .id
        .clone();
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/sessions",
                &CombatSessionCreateRequestDto {
                    session_id: session_id.to_owned(),
                    scenario_id,
                    participant_order: Vec::new(),
                    content_pack: None,
                },
            ))
            .status,
        200
    );
    assert_eq!(
        router
            .handle(&json_request(
                HttpMethod::Post,
                &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
                &CombatControlCommandDto {
                    kind: CombatControlCommandKindDto::ExplicitStart,
                },
            ))
            .status,
        200
    );
    let submitted = router.handle(&json_request(
        HttpMethod::Post,
        &format!("/api/rulebench/v1/sessions/{session_id}/intents"),
        &CombatSessionIntentCommandDto {
            id: "recovery-reaction-action".to_owned(),
            title: "Hexing Bolt".to_owned(),
            summary: "Open the restart-safe reaction window.".to_owned(),
            intent: UseActionIntentDto {
                actor_id: "entity-adept".to_owned(),
                action_id: "hexing_bolt".to_owned(),
                target_id: "entity-raider".to_owned(),
                target_ids: Vec::new(),
                target_cell: None,
                destination_cell: None,
                observed_origin: None,
            },
            roll_stream: vec![17, 5],
            roll_mode: CommandRollModeDto::Supplied,
            generated_seed: None,
        },
    ));
    let before_restart: LiveCommandExecutionDto =
        serde_json::from_slice(&submitted.body).expect("suspended execution is JSON");
    let opened = before_restart
        .snapshot
        .current_reaction_window
        .clone()
        .expect("reaction window opens");
    assert_eq!(
        before_restart
            .snapshot
            .gameplay_fabric
            .pending_decision_count,
        1
    );
    drop(router);

    let mut restarted =
        build_durable_rulebench_router(&directory).expect("reaction checkpoint reconstructs");
    let restored = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    let restored: LiveSessionSnapshotDto =
        serde_json::from_slice(&restored.body).expect("restored reaction snapshot is JSON");
    assert_eq!(restored, before_restart.snapshot);

    for (reactor_id, response_kind, option_id) in [
        ("entity-adept", ReactionResponseKindDto::Pass, None),
        (
            "entity-raider",
            ReactionResponseKindDto::Accept,
            Some("raider-ward".to_owned()),
        ),
    ] {
        let response = restarted.handle(&json_request(
            HttpMethod::Post,
            &format!("/api/rulebench/v1/sessions/{session_id}/reactions"),
            &ReactionCommandSpecDto {
                window_id: opened.id.clone(),
                reactor_id: reactor_id.to_owned(),
                response_kind,
                option_id,
            },
        ));
        assert_eq!(
            response.status,
            200,
            "{}",
            String::from_utf8_lossy(&response.body)
        );
    }
    let resumed = restarted.handle(&request(
        HttpMethod::Get,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    let resumed: LiveSessionSnapshotDto =
        serde_json::from_slice(&resumed.body).expect("resumed snapshot is JSON");
    assert!(resumed.current_reaction_window.is_none());
    assert_eq!(resumed.gameplay_fabric.pending_decision_count, 0);
    fs::remove_dir_all(directory).expect("test repository cleans up");
}

#[test]
fn router_closes_recreates_and_restarts_with_isolated_in_memory_state() {
    let mut active_router = router();
    let scenarios = active_router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/scenarios"));
    let scenarios: Vec<ScenarioOptionDto> =
        serde_json::from_slice(&scenarios.body).expect("scenario options are JSON");
    let scenario_id = scenarios[0].id.clone();
    let session_id = "reusable-session";

    for control in [
        CombatControlCommandKindDto::ExplicitStart,
        CombatControlCommandKindDto::ExplicitEnd,
    ] {
        if control == CombatControlCommandKindDto::ExplicitStart {
            let created = active_router.handle(&json_request(
                HttpMethod::Post,
                "/api/rulebench/v1/sessions",
                &CombatSessionCreateRequestDto {
                    session_id: session_id.to_string(),
                    scenario_id: scenario_id.clone(),
                    participant_order: Vec::new(),
                    content_pack: None,
                },
            ));
            assert_eq!(created.status, 200);
        }
        let controlled = active_router.handle(&json_request(
            HttpMethod::Post,
            &format!("/api/rulebench/v1/sessions/{session_id}/controls"),
            &CombatControlCommandDto { kind: control },
        ));
        assert_eq!(controlled.status, 200);
    }

    let closed = active_router.handle(&request(
        HttpMethod::Delete,
        &format!("/api/rulebench/v1/sessions/{session_id}"),
    ));
    assert_eq!(closed.status, 200);
    let active = active_router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/sessions"));
    let active: Vec<LiveSessionSnapshotDto> =
        serde_json::from_slice(&active.body).expect("active sessions are JSON");
    assert!(active.is_empty());

    let retained_archive = active_router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: session_id.to_string(),
            scenario_id: scenario_id.clone(),
            participant_order: Vec::new(),
            content_pack: None,
        },
    ));
    assert_eq!(retained_archive.status, 409);

    let mut restarted_router = router();
    let recreated = restarted_router.handle(&json_request(
        HttpMethod::Post,
        "/api/rulebench/v1/sessions",
        &CombatSessionCreateRequestDto {
            session_id: session_id.to_string(),
            scenario_id,
            participant_order: Vec::new(),
            content_pack: None,
        },
    ));
    assert_eq!(recreated.status, 200);
    let recreated: LiveSessionSnapshotDto =
        serde_json::from_slice(&recreated.body).expect("restarted session is JSON");
    assert_eq!(recreated.lifecycle_phase, "ready");
    assert_eq!(recreated.next_step_index, 0);
    assert!(recreated.combat_log.is_empty());
    assert!(recreated.audit_log.is_empty());

    let restarted =
        restarted_router.handle(&request(HttpMethod::Get, "/api/rulebench/v1/sessions"));
    let restarted: Vec<LiveSessionSnapshotDto> =
        serde_json::from_slice(&restarted.body).expect("restarted session list is JSON");
    assert_eq!(restarted.len(), 1);
    assert_eq!(restarted[0].session_id, session_id);
}

#[test]
fn tcp_server_starts_serves_json_and_stops_cleanly() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener binds");
    let address = listener.local_addr().expect("listener has address");
    let shutdown = Arc::new(AtomicBool::new(false));
    let server_shutdown = Arc::clone(&shutdown);
    let server = thread::spawn(move || serve_until(listener, router(), server_shutdown));

    let mut stream = connect_with_retry(address);
    write!(
        stream,
        "GET /api/rulebench/v1/handshake HTTP/1.1\r\nHost: 127.0.0.1\r\nx-rulebench-protocol-version: {}\r\nConnection: close\r\n\r\n",
        PROTOCOL_VERSION
    )
    .expect("request writes");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("response reads");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("\"protocolId\":\"asha-rulebench.protocol\""));

    shutdown.store(true, Ordering::Release);
    server
        .join()
        .expect("server thread joins")
        .expect("server stops without error");
}

#[test]
fn tcp_server_survives_clients_that_disconnect_before_a_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener binds");
    let address = listener.local_addr().expect("listener has address");
    let shutdown = Arc::new(AtomicBool::new(false));
    let server_shutdown = Arc::clone(&shutdown);
    let server = thread::spawn(move || serve_until(listener, router(), server_shutdown));

    for _ in 0..8 {
        let mut abandoned = connect_with_retry(address);
        abandoned
            .write_all(b"GET /api/rulebench/v1/handshake HTTP/1.1\r\n")
            .expect("partial request writes");
        drop(abandoned);
    }

    let mut subsequent = connect_with_retry(address);
    write!(
        subsequent,
        "GET /api/rulebench/v1/handshake HTTP/1.1\r\nHost: 127.0.0.1\r\nx-rulebench-protocol-version: {}\r\nConnection: close\r\n\r\n",
        PROTOCOL_VERSION
    )
    .expect("subsequent request writes");
    let mut response = String::new();
    subsequent
        .read_to_string(&mut response)
        .expect("subsequent response reads");
    assert!(response.starts_with("HTTP/1.1 200 OK"));

    shutdown.store(true, Ordering::Release);
    server
        .join()
        .expect("server thread joins")
        .expect("per-connection failures do not stop the listener");
}

fn connect_with_retry(address: std::net::SocketAddr) -> TcpStream {
    for _ in 0..50 {
        if let Ok(stream) = TcpStream::connect(address) {
            return stream;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("test server did not accept connections");
}
