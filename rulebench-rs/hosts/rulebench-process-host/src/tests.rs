use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rulebench_protocol::{
    CombatControlCommandDto, CombatControlCommandKindDto, CombatSessionCreateRequestDto,
    CombatSessionIntentCommandDto, CommandRollModeDto, LiveCommandExecutionDto,
    LiveControlExecutionDto, LiveReactionExecutionDto, LiveSessionSnapshotDto,
    ProtocolHandshakeDto, ReactionCommandSpecDto, ReactionResponseKindDto,
    ReplayArchiveMetadataDto, ReplayComparisonReadoutDto, ReplayComparisonRequestDto,
    ReplayPackageReviewDto, ReplayVerificationReadoutDto, ScenarioOptionDto, UseActionIntentDto,
    PROTOCOL_VERSION,
};

use crate::{build_rulebench_bridge, serve_until, HttpMethod, HttpRequest, ProcessHostRouter};

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
    let scenario_id = &scenarios.first().expect("fixture scenario exists").id;

    for session_id in ["first", "second"] {
        let created = router.handle(&json_request(
            HttpMethod::Post,
            "/api/rulebench/v1/sessions",
            &CombatSessionCreateRequestDto {
                session_id: session_id.to_string(),
                scenario_id: scenario_id.clone(),
                participant_order: Vec::new(),
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
    assert_eq!(packages.len(), 2);

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
