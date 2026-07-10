use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rulebench_protocol::{
    CombatControlCommandDto, CombatControlCommandKindDto, CombatSessionCreateRequestDto,
    LiveControlExecutionDto, LiveSessionSnapshotDto, ProtocolHandshakeDto, ScenarioOptionDto,
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

fn connect_with_retry(address: std::net::SocketAddr) -> TcpStream {
    for _ in 0..50 {
        if let Ok(stream) = TcpStream::connect(address) {
            return stream;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("test server did not accept connections");
}
