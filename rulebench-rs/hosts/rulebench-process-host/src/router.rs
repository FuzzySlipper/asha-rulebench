use rulebench_bridge::{BridgeError, BridgeErrorKind, BridgeScenario, RulebenchBridge};
use rulebench_fixtures::aggregated_scenario_catalog_cases;
use rulebench_protocol::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatControlCommandDto,
    CombatSessionCreateRequestDto, CombatSessionHandleDto, CombatSessionIntentCommandDto,
    LiveAutomaticRunDto, LiveAutomaticStepDto, LiveCandidateSummaryDto, LiveCommandExecutionDto,
    LiveControlExecutionDto, LivePreflightDto, LiveSessionSnapshotDto, LiveTransportErrorDto,
    ProtocolRequestContextDto, UseActionIntentDto,
};

use crate::{HttpMethod, HttpRequest, HttpResponse};

const API_PREFIX: &str = "/api/rulebench/v1";
const PROTOCOL_VERSION_HEADER: &str = "x-rulebench-protocol-version";

pub fn build_rulebench_bridge() -> Result<RulebenchBridge, BridgeError> {
    RulebenchBridge::new(aggregated_scenario_catalog_cases().into_iter().map(|case| {
        BridgeScenario::new(
            case.summary.id,
            case.summary.title,
            case.summary.summary,
            case.scenario,
        )
    }))
}

#[derive(Debug)]
pub struct ProcessHostRouter {
    bridge: RulebenchBridge,
}

impl ProcessHostRouter {
    pub fn new(bridge: RulebenchBridge) -> Self {
        Self { bridge }
    }

    pub fn handle(&mut self, request: &HttpRequest) -> HttpResponse {
        let context = match request_context(request) {
            Ok(context) => context,
            Err(response) => return response,
        };
        let path = request.path.split('?').next().unwrap_or(&request.path);
        let relative = path.strip_prefix(API_PREFIX).unwrap_or(path);
        let segments = relative
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();

        match (request.method, segments.as_slice()) {
            (HttpMethod::Get, ["handshake"]) => bridge_result(self.bridge.handshake(&context)),
            (HttpMethod::Get, ["scenarios"]) => bridge_result(self.bridge.list_scenarios(&context)),
            (HttpMethod::Get, ["sessions"]) => match self.bridge.list_sessions(&context) {
                Ok(snapshots) => json_ok(
                    snapshots
                        .iter()
                        .map(LiveSessionSnapshotDto::from)
                        .collect::<Vec<_>>(),
                ),
                Err(error) => bridge_error(error),
            },
            (HttpMethod::Post, ["sessions"]) => {
                let request = match decode_body::<CombatSessionCreateRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                match self.bridge.create_session(&context, &request) {
                    Ok(created) => json_ok(LiveSessionSnapshotDto::from(&created.snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Get, ["sessions", session_id]) => {
                let handle = session_handle(session_id);
                match self.bridge.get_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveSessionSnapshotDto::from(&snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Delete, ["sessions", session_id]) => {
                let handle = session_handle(session_id);
                match self.bridge.close_session(&context, &handle) {
                    Ok(archive) => json_ok(LiveSessionSnapshotDto::from(&archive.snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Get, ["sessions", session_id, "options"]) => {
                let handle = session_handle(session_id);
                match self.bridge.current_actor_options(&context, &handle) {
                    Ok(options) => json_ok(rulebench_protocol::LiveCurrentActorOptionsDto::from(
                        &options,
                    )),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["sessions", session_id, "preflight"]) => {
                let intent = match decode_body::<UseActionIntentDto>(request) {
                    Ok(intent) => intent,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                match self.bridge.preflight_command(&context, &handle, &intent) {
                    Ok(preflight) => json_ok(LivePreflightDto::from(&preflight)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Get, ["sessions", session_id, "candidates"]) => {
                let handle = session_handle(session_id);
                match self.bridge.command_candidates(&context, &handle) {
                    Ok(candidates) => json_ok(LiveCandidateSummaryDto::from(&candidates)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["sessions", session_id, "intents"]) => {
                let command = match decode_body::<CombatSessionIntentCommandDto>(request) {
                    Ok(command) => command,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                let step = match self.bridge.submit_intent(&context, &handle, &command) {
                    Ok(step) => step,
                    Err(error) => return bridge_error(error),
                };
                match self.bridge.get_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveCommandExecutionDto::new(&step, &snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["sessions", session_id, "controls"]) => {
                let command = match decode_body::<CombatControlCommandDto>(request) {
                    Ok(command) => command,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                let control = match self.bridge.submit_control(&context, &handle, &command) {
                    Ok(control) => control,
                    Err(error) => return bridge_error(error),
                };
                match self.bridge.get_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveControlExecutionDto::new(&control, &snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["sessions", session_id, "automatic-step"]) => {
                let command = match decode_body::<AutomaticStepRequestDto>(request) {
                    Ok(command) => command,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                let execution = match self.bridge.automatic_step(&context, &handle, &command) {
                    Ok(execution) => execution,
                    Err(error) => return bridge_error(error),
                };
                match self.bridge.get_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveAutomaticStepDto::new(&execution, &snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["sessions", session_id, "automatic-run"]) => {
                let command = match decode_body::<AutomaticRunRequestDto>(request) {
                    Ok(command) => command,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                match self.bridge.automatic_run(&context, &handle, &command) {
                    Ok(readout) => json_ok(LiveAutomaticRunDto::from(&readout)),
                    Err(error) => bridge_error(error),
                }
            }
            _ => error_response(
                404,
                "transport",
                "routeNotFound",
                format!("Rulebench host route not found: {}", request.path),
                false,
            ),
        }
    }
}

fn request_context(request: &HttpRequest) -> Result<ProtocolRequestContextDto, HttpResponse> {
    let Some(raw_version) = request.header(PROTOCOL_VERSION_HEADER) else {
        return Err(error_response(
            400,
            "protocol",
            "missingProtocolVersion",
            format!("Missing required header {PROTOCOL_VERSION_HEADER}."),
            false,
        ));
    };
    let protocol_version = raw_version.parse::<u32>().map_err(|_| {
        error_response(
            400,
            "protocol",
            "invalidProtocolVersion",
            "Protocol version header must be an unsigned integer.".to_string(),
            false,
        )
    })?;
    Ok(ProtocolRequestContextDto { protocol_version })
}

fn session_handle(session_id: &str) -> CombatSessionHandleDto {
    CombatSessionHandleDto {
        id: session_id.to_string(),
    }
}

fn decode_body<T>(request: &HttpRequest) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(&request.body).map_err(|error| {
        error_response(
            400,
            "serialization",
            "invalidJsonBody",
            format!("Request body did not match the protocol DTO: {error}"),
            false,
        )
    })
}

fn bridge_result<T>(result: Result<T, BridgeError>) -> HttpResponse
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => json_ok(value),
        Err(error) => bridge_error(error),
    }
}

fn json_ok<T>(value: T) -> HttpResponse
where
    T: serde::Serialize,
{
    match serde_json::to_vec(&value) {
        Ok(body) => HttpResponse::json(200, body),
        Err(error) => error_response(
            500,
            "serialization",
            "responseSerializationFailed",
            format!("Rust host could not serialize its response: {error}"),
            false,
        ),
    }
}

fn bridge_error(error: BridgeError) -> HttpResponse {
    let status = match error.kind {
        BridgeErrorKind::ProtocolVersionMismatch | BridgeErrorKind::DuplicateSession => 409,
        BridgeErrorKind::UnknownScenario | BridgeErrorKind::UnknownSession => 404,
        BridgeErrorKind::InvalidScenario | BridgeErrorKind::InvalidLifecycle => 422,
        BridgeErrorKind::InvalidRequest => 400,
    };
    error_response(status, "bridge", error.code, error.message, error.retryable)
}

fn error_response(
    status: u16,
    kind: impl Into<String>,
    code: impl Into<String>,
    message: impl Into<String>,
    retryable: bool,
) -> HttpResponse {
    let error = LiveTransportErrorDto::new(kind, code, message, retryable);
    match serde_json::to_vec(&error) {
        Ok(body) => HttpResponse::json(status, body),
        Err(_) => HttpResponse::json(
            500,
            br#"{"kind":"serialization","code":"errorSerializationFailed","message":"Rust host could not serialize an error response.","retryable":false}"#.to_vec(),
        ),
    }
}
