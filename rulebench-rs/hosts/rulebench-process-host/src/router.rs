use std::path::{Path, PathBuf};

use crate::{ArtifactRepositoryIssue, FileReplayArchiveStorage};
use crate::{HttpMethod, HttpRequest, HttpResponse};
use rulebench_bridge::replay_storage::{
    ContentPackStorage, ReplayArchiveEntry, ReplayArchiveStorage,
};
use rulebench_bridge::{BridgeError, BridgeErrorKind, BridgeScenario, RulebenchBridge};
use rulebench_fixtures::{aggregated_scenario_catalog_cases, replay_review_packages};
use rulebench_protocol::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatControlCommandDto,
    CombatSessionCreateRequestDto, CombatSessionHandleDto, CombatSessionIntentCommandDto,
    LiveAutomaticRunDto, LiveAutomaticStepDto, LiveCandidateSummaryDto, LiveCommandExecutionDto,
    LiveControlExecutionDto, LivePreflightDto, LiveReactionExecutionDto, LiveSessionSnapshotDto,
    LiveTransportErrorDto, ProtocolRequestContextDto, ReactionCommandSpecDto,
    ReplayComparisonRequestDto, UseActionIntentDto,
};

const API_PREFIX: &str = "/api/rulebench/v1";
const PROTOCOL_VERSION_HEADER: &str = "x-rulebench-protocol-version";

pub fn build_rulebench_bridge() -> Result<RulebenchBridge, BridgeError> {
    RulebenchBridge::new_with_replays(bridge_scenarios(), replay_review_packages())
}

fn bridge_scenarios() -> Vec<BridgeScenario> {
    aggregated_scenario_catalog_cases()
        .into_iter()
        .map(|case| {
            BridgeScenario::new(
                case.summary.id,
                case.summary.title,
                case.summary.summary,
                case.scenario,
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRepositoryStatus {
    pub mode: String,
    pub root: Option<String>,
    pub content_artifact_count: usize,
    pub replay_artifact_count: usize,
    pub issues: Vec<ArtifactRepositoryIssue>,
}

impl ArtifactRepositoryStatus {
    fn in_memory(replay_artifact_count: usize) -> Self {
        Self {
            mode: "memory".to_string(),
            root: None,
            content_artifact_count: 0,
            replay_artifact_count,
            issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRepositoryConfig {
    root: PathBuf,
}

impl ArtifactRepositoryConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn open(&self) -> Result<ProcessHostRouter, String> {
        open_durable_rulebench_router(&self.root)
    }
}

pub fn build_durable_rulebench_router(root: &Path) -> Result<ProcessHostRouter, String> {
    ArtifactRepositoryConfig::new(root).open()
}

fn open_durable_rulebench_router(root: &Path) -> Result<ProcessHostRouter, String> {
    let scenarios = bridge_scenarios();
    let replay_packages = replay_review_packages();
    let mut storage_scenarios = scenarios.clone();
    for (index, package) in replay_packages.iter().enumerate() {
        let scenario = package.initial_session.scenario.clone();
        if !storage_scenarios
            .iter()
            .any(|registered| registered.scenario.metadata.id == scenario.metadata.id)
        {
            let title = scenario.metadata.title.clone();
            let summary = scenario.metadata.summary.clone();
            storage_scenarios.push(BridgeScenario::new(
                format!("replay-storage-fixture-{index:04}"),
                title,
                summary,
                scenario,
            ));
        }
    }
    let replay_root = root.join("replays");
    let report = FileReplayArchiveStorage::open(replay_root, storage_scenarios)
        .map_err(|issue| issue.message)?;
    let mut storage = report.storage;
    for (index, package) in replay_packages.into_iter().enumerate() {
        storage
            .write(ReplayArchiveEntry::new(
                package,
                format!("fixture-{index:04}"),
            ))
            .map_err(|error| format!("Could not seed durable replay repository: {error:?}"))?;
    }
    let replay_artifact_count = storage
        .list()
        .map_err(|error| format!("Could not list durable replay repository: {error:?}"))?
        .len();
    let (content, content_issues) = ContentPackStorage::open_quarantining(root.join("content"))
        .map_err(|error| format!("Could not open durable content repository: {error:?}"))?;
    let content_artifact_count = content.list().len();
    let mut issues = report.issues;
    issues.extend(
        content_issues
            .into_iter()
            .map(|issue| ArtifactRepositoryIssue {
                artifact_kind: "content".to_string(),
                code: issue.code,
                path: issue.path.display().to_string(),
                message: issue.reason,
            }),
    );
    let status = ArtifactRepositoryStatus {
        mode: "filesystem".to_string(),
        root: Some(root.display().to_string()),
        content_artifact_count,
        replay_artifact_count,
        issues,
    };
    let bridge = RulebenchBridge::new_with_replay_storage(scenarios, Box::new(storage))
        .map_err(|error| error.to_string())?;
    Ok(ProcessHostRouter::new_with_repository(
        bridge, content, status,
    ))
}

#[derive(Debug)]
pub struct ProcessHostRouter {
    bridge: RulebenchBridge,
    content_storage: Option<ContentPackStorage>,
    repository_status: ArtifactRepositoryStatus,
}

impl ProcessHostRouter {
    pub fn new(bridge: RulebenchBridge) -> Self {
        let replay_artifact_count = bridge
            .list_replay_packages(&ProtocolRequestContextDto::current())
            .map_or(0, |packages| packages.len());
        Self {
            bridge,
            content_storage: None,
            repository_status: ArtifactRepositoryStatus::in_memory(replay_artifact_count),
        }
    }

    pub fn new_with_repository(
        bridge: RulebenchBridge,
        content_storage: ContentPackStorage,
        repository_status: ArtifactRepositoryStatus,
    ) -> Self {
        Self {
            bridge,
            content_storage: Some(content_storage),
            repository_status,
        }
    }

    pub fn repository_status(&self) -> &ArtifactRepositoryStatus {
        &self.repository_status
    }

    pub fn content_storage(&self) -> Option<&ContentPackStorage> {
        self.content_storage.as_ref()
    }

    pub fn content_storage_mut(&mut self) -> Option<&mut ContentPackStorage> {
        self.content_storage.as_mut()
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
                    Ok(archive) => {
                        if let Ok(packages) = self.bridge.list_replay_packages(&context) {
                            self.repository_status.replay_artifact_count = packages.len();
                        }
                        json_ok(LiveSessionSnapshotDto::from(&archive.snapshot))
                    }
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
            (HttpMethod::Post, ["sessions", session_id, "reactions"]) => {
                let command = match decode_body::<ReactionCommandSpecDto>(request) {
                    Ok(command) => command,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                let reaction = match self.bridge.submit_reaction(&context, &handle, &command) {
                    Ok(reaction) => reaction,
                    Err(error) => return bridge_error(error),
                };
                match self.bridge.get_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveReactionExecutionDto::new(&reaction, &snapshot)),
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
            (HttpMethod::Get, ["replays"]) => {
                bridge_result(self.bridge.list_replay_packages(&context))
            }
            (HttpMethod::Post, ["replays", "compare"]) => {
                let comparison = match decode_body::<ReplayComparisonRequestDto>(request) {
                    Ok(comparison) => comparison,
                    Err(response) => return response,
                };
                bridge_result(self.bridge.compare_replay_packages(
                    &context,
                    &comparison.expected_package_id,
                    &comparison.actual_package_id,
                ))
            }
            (HttpMethod::Get, ["replays", package_id]) => {
                bridge_result(self.bridge.load_replay_package(&context, package_id))
            }
            (HttpMethod::Post, ["replays", package_id, "verify"]) => {
                bridge_result(self.bridge.verify_replay_package(&context, package_id))
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
        BridgeErrorKind::ReplayArchive if error.code == "unknownReplayPackage" => 404,
        BridgeErrorKind::ReplayArchive if error.retryable => 503,
        BridgeErrorKind::ReplayArchive => 422,
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
