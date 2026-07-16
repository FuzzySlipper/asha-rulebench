use std::path::{Path, PathBuf};

use crate::content_workspace::{ContentWorkspace, ContentWorkspaceError};
use crate::{ArtifactRepositoryIssue, FileReplayArchiveStorage, FileSessionRecoveryStorage};
use crate::{HttpMethod, HttpRequest, HttpResponse};
use rulebench_bridge::replay_storage::{
    ContentPackStorage, ReplayArchiveEntry, ReplayArchiveStorage,
};
use rulebench_bridge::{BridgeError, BridgeErrorKind, BridgeScenario, RulebenchBridge};
use rulebench_fixtures::{
    aggregated_combat_session_transcripts, aggregated_scenario_catalog_cases,
    replay_review_packages, resolve_catalog_scenario,
};
use rulebench_fixtures::{
    assemble_capability_manifest, capability_registry_input, scenario_package_registry,
    HostCapabilityProfile,
};
use rulebench_protocol::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatControlCommandDto,
    CombatSessionCreateRequestDto, CombatSessionHandleDto, CombatSessionIntentCommandDto,
    ContentImportRequestDto, ContentPayloadRequestDto, ContentReferenceRequestDto,
    ExperimentComparisonRequestDto, ExperimentMatrixRequestDto, LiveAutomaticRunDto,
    LiveAutomaticStepDto, LiveCandidateSummaryDto, LiveCommandExecutionDto,
    LiveControlExecutionDto, LivePreflightDto, LiveReactionExecutionDto, LiveSessionSnapshotDto,
    LiveTransportErrorDto, ProtocolRequestContextDto, ReactionCommandSpecDto,
    ReplayComparisonRequestDto, RulebenchCapabilityManifestDto, SessionRecoveryCatalogDto,
    SessionRecoveryForkRequestDto, SessionRecoveryIssueDto, UseActionIntentDto,
    ViewerScenarioReadoutDto, ViewerScenarioSummaryDto, ViewerSessionTranscriptDto,
};

const API_PREFIX: &str = "/api/rulebench/v1";
const PROTOCOL_VERSION_HEADER: &str = "x-rulebench-protocol-version";

pub fn build_rulebench_bridge() -> Result<RulebenchBridge, BridgeError> {
    RulebenchBridge::new_with_replays_and_viewer_sessions(
        bridge_scenarios(),
        replay_review_packages(),
        viewer_session_transcripts(),
    )
}

fn bridge_scenarios() -> Vec<BridgeScenario> {
    aggregated_scenario_catalog_cases()
        .into_iter()
        .map(|case| {
            let resolution = resolve_catalog_scenario(&case.summary.id)
                .expect("registered viewer scenario resolves through Rust authority");
            let projection = resolution
                .receipt
                .projection
                .as_ref()
                .expect("registered viewer scenario has an authority projection");
            let identity = ViewerScenarioSummaryDto {
                id: case.summary.id.clone(),
                title: case.summary.title.clone(),
                summary: case.summary.summary.clone(),
                seed_label: case.summary.seed_label.clone(),
                outcome_class: case.summary.outcome_class.code().to_string(),
            };
            BridgeScenario::new(
                case.summary.id,
                case.summary.title,
                case.summary.summary,
                case.scenario,
            )
            .with_viewer_readout(ViewerScenarioReadoutDto::new(
                identity,
                &resolution.scenario,
                &resolution.receipt,
                projection,
            ))
        })
        .collect()
}

fn viewer_session_transcripts() -> Vec<ViewerSessionTranscriptDto> {
    aggregated_combat_session_transcripts()
        .iter()
        .map(ViewerSessionTranscriptDto::from)
        .collect()
}

fn process_host_capability_manifest(
    repository_status: &ArtifactRepositoryStatus,
    authored_content_enabled: bool,
) -> RulebenchCapabilityManifestDto {
    let mut registry = capability_registry_input(&scenario_package_registry());
    registry
        .regression_capability_ids
        .push("content.authored-pack".to_string());
    registry
        .regression_capability_ids
        .push("replay.finalized-archive".to_string());
    registry
        .regression_capability_ids
        .push("viewer.authority-readback".to_string());
    registry
        .regression_capability_ids
        .push("session.active-recovery".to_string());
    let filesystem = repository_status.mode == "filesystem";
    let manifest = assemble_capability_manifest(
        registry,
        HostCapabilityProfile {
            adapter_id: "rulebench-process-host".to_string(),
            storage_mode: repository_status.mode.clone(),
            content_storage_adapter: if filesystem {
                "versionedFilesystem".to_string()
            } else {
                "none".to_string()
            },
            replay_storage_adapter: if filesystem {
                "versionedFilesystem".to_string()
            } else {
                "inMemory".to_string()
            },
            replay_recovery_mode: "finalizedArchive".to_string(),
            session_recovery_mode: if filesystem {
                "replayVerifiedCheckpoints".to_string()
            } else {
                "processLocalCheckpoints".to_string()
            },
            authority_viewer_mode: "liveAuthorityReadback".to_string(),
            authored_content_enabled,
            exposes_capabilities_through_protocol: true,
            exposes_capabilities_through_live_host: true,
            exposes_capabilities_in_ui: true,
            durable_content: filesystem && authored_content_enabled,
            durable_finalized_replays: filesystem,
            durable_active_sessions: filesystem,
        },
    )
    .expect("compiled owner registries form a valid capability manifest");
    RulebenchCapabilityManifestDto::from(&manifest)
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
    let report = FileReplayArchiveStorage::open(replay_root, storage_scenarios.clone())
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
    let recovery_report =
        FileSessionRecoveryStorage::open(root.join("session-recovery"), storage_scenarios.clone())
            .map_err(|issue| issue.message)?;
    let (content, content_issues) = ContentPackStorage::open_quarantining(root.join("content"))
        .map_err(|error| format!("Could not open durable content repository: {error:?}"))?;
    let (content, workspace_issues) = ContentWorkspace::open(content);
    let content_artifact_count = content.storage().list().len();
    let mut issues = report.issues;
    issues.extend(recovery_report.issues);
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
    issues.extend(workspace_issues);
    let status = ArtifactRepositoryStatus {
        mode: "filesystem".to_string(),
        root: Some(root.display().to_string()),
        content_artifact_count,
        replay_artifact_count,
        issues,
    };
    let bridge = RulebenchBridge::new_with_durable_storage(
        scenarios,
        Box::new(storage),
        Box::new(recovery_report.storage),
        viewer_session_transcripts(),
    )
    .map_err(|error| error.to_string())?;
    Ok(ProcessHostRouter::new_with_repository(
        bridge, content, status,
    ))
}

#[derive(Debug)]
pub struct ProcessHostRouter {
    bridge: RulebenchBridge,
    content_workspace: Option<ContentWorkspace>,
    repository_status: ArtifactRepositoryStatus,
    capability_manifest: RulebenchCapabilityManifestDto,
}

impl ProcessHostRouter {
    pub fn new(bridge: RulebenchBridge) -> Self {
        let replay_artifact_count = bridge
            .list_replay_packages(&ProtocolRequestContextDto::current())
            .map_or(0, |packages| packages.len());
        let repository_status = ArtifactRepositoryStatus::in_memory(replay_artifact_count);
        Self {
            bridge,
            content_workspace: None,
            capability_manifest: process_host_capability_manifest(&repository_status, false),
            repository_status,
        }
    }

    pub fn new_with_repository(
        bridge: RulebenchBridge,
        content_workspace: ContentWorkspace,
        repository_status: ArtifactRepositoryStatus,
    ) -> Self {
        Self {
            bridge,
            content_workspace: Some(content_workspace),
            capability_manifest: process_host_capability_manifest(&repository_status, true),
            repository_status,
        }
    }

    pub fn repository_status(&self) -> &ArtifactRepositoryStatus {
        &self.repository_status
    }

    pub fn capability_manifest(&self) -> &RulebenchCapabilityManifestDto {
        &self.capability_manifest
    }

    pub fn content_storage(&self) -> Option<&ContentPackStorage> {
        self.content_workspace
            .as_ref()
            .map(ContentWorkspace::storage)
    }

    pub fn content_storage_mut(&mut self) -> Option<&mut ContentPackStorage> {
        self.content_workspace
            .as_mut()
            .map(ContentWorkspace::storage_mut)
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
            (HttpMethod::Get, ["capabilities"]) => json_ok(&self.capability_manifest),
            (HttpMethod::Get, ["automation-policies"]) => {
                bridge_result(self.bridge.automation_policy_catalog(&context))
            }
            (HttpMethod::Get, ["experiments"]) => {
                bridge_result(self.bridge.list_experiments(&context))
            }
            (HttpMethod::Post, ["experiments"]) => {
                let matrix = match decode_body::<ExperimentMatrixRequestDto>(request) {
                    Ok(matrix) => matrix,
                    Err(response) => return response,
                };
                bridge_result(self.bridge.create_experiment(&context, &matrix))
            }
            (HttpMethod::Post, ["experiments", "compare"]) => {
                let comparison = match decode_body::<ExperimentComparisonRequestDto>(request) {
                    Ok(comparison) => comparison,
                    Err(response) => return response,
                };
                bridge_result(self.bridge.compare_experiment_trials(&context, &comparison))
            }
            (HttpMethod::Get, ["experiments", experiment_id]) => {
                bridge_result(self.bridge.get_experiment(&context, experiment_id))
            }
            (HttpMethod::Post, ["experiments", experiment_id, "advance"]) => {
                bridge_result(self.bridge.advance_experiment(&context, experiment_id))
            }
            (HttpMethod::Post, ["experiments", experiment_id, "cancel"]) => {
                bridge_result(self.bridge.cancel_experiment(&context, experiment_id))
            }
            (HttpMethod::Get, ["viewer", "scenarios"]) => {
                bridge_result(self.bridge.list_viewer_scenarios(&context))
            }
            (HttpMethod::Get, ["viewer", "scenarios", scenario_id]) => {
                bridge_result(self.bridge.get_viewer_scenario(&context, scenario_id))
            }
            (HttpMethod::Get, ["viewer", "sessions"]) => {
                bridge_result(self.bridge.list_viewer_sessions(&context))
            }
            (HttpMethod::Get, ["viewer", "sessions", session_id, "steps", step_id]) => {
                bridge_result(
                    self.bridge
                        .get_viewer_session_step(&context, session_id, step_id),
                )
            }
            (HttpMethod::Get, ["scenarios"]) => bridge_result(self.bridge.list_scenarios(&context)),
            (HttpMethod::Get, ["session-recovery"]) => {
                match self.bridge.list_session_recovery(&context) {
                    Ok(sessions) => json_ok(SessionRecoveryCatalogDto {
                        sessions,
                        issues: self
                            .repository_status
                            .issues
                            .iter()
                            .filter(|issue| issue.artifact_kind == "sessionRecovery")
                            .map(|issue| SessionRecoveryIssueDto {
                                code: issue.code.clone(),
                                message: issue.message.clone(),
                                path: issue.path.clone(),
                            })
                            .collect(),
                    }),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Delete, ["session-recovery", session_id]) => {
                let handle = session_handle(session_id);
                match self.bridge.discard_recovery_session(&context, &handle) {
                    Ok(snapshot) => json_ok(LiveSessionSnapshotDto::from(&snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
            (HttpMethod::Post, ["session-recovery", session_id, "fork"]) => {
                let request = match decode_body::<SessionRecoveryForkRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                let handle = session_handle(session_id);
                match self
                    .bridge
                    .fork_recovery_session(&context, &handle, &request.new_session_id)
                {
                    Ok(created) => json_ok(LiveSessionSnapshotDto::from(&created.snapshot)),
                    Err(error) => bridge_error(error),
                }
            }
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
                let selected_content = match &request.content_pack {
                    Some(reference) => {
                        let reference = reference.to_authority();
                        let Some(workspace) = self.content_workspace.as_ref() else {
                            return error_response(
                                409,
                                "content",
                                "durableContentRepositoryRequired",
                                "Authored content sessions require the configured durable host.",
                                false,
                            );
                        };
                        let scenario = match self.bridge.list_scenarios(&context) {
                            Ok(scenarios) => scenarios
                                .into_iter()
                                .find(|scenario| scenario.id == request.scenario_id),
                            Err(error) => return bridge_error(error),
                        };
                        let Some(scenario) = scenario else {
                            return error_response(
                                404,
                                "bridge",
                                "unknownScenario",
                                format!("Scenario does not exist: {}", request.scenario_id),
                                false,
                            );
                        };
                        let (ruleset_id, ruleset_version) = match workspace.ruleset_for(&reference)
                        {
                            Ok(ruleset) => ruleset,
                            Err(error) => return content_error(error),
                        };
                        if ruleset_id != scenario.ruleset_id
                            || ruleset_version != scenario.ruleset_version
                        {
                            return error_response(
                                422,
                                "content",
                                "incompatibleSessionRuleset",
                                format!(
                                    "Content requires {ruleset_id} {ruleset_version}; scenario uses {} {}.",
                                    scenario.ruleset_id, scenario.ruleset_version
                                ),
                                false,
                            );
                        }
                        let provenance = match workspace.ruleset_provenance_for(&reference) {
                            Ok(provenance) => provenance,
                            Err(error) => return content_error(error),
                        };
                        match workspace.active_pack_set(&reference) {
                            Ok(set) => Some((reference, set, provenance)),
                            Err(error) => return content_error(error),
                        }
                    }
                    None => None,
                };
                let created = match &selected_content {
                    Some((_, set, provenance)) => self.bridge.create_session_with_content_pack_set(
                        &context,
                        &request,
                        Some(set.clone()),
                        Some(provenance.clone()),
                    ),
                    None => self.bridge.create_session(&context, &request),
                };
                match created {
                    Ok(created) => {
                        if let (Some((reference, _, _)), Some(workspace)) =
                            (selected_content, self.content_workspace.as_mut())
                        {
                            if let Err(error) =
                                workspace.record_session_use(&reference, &request.session_id)
                            {
                                return content_error(error);
                            }
                        }
                        json_ok(LiveSessionSnapshotDto::from(&created.snapshot))
                    }
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
            (HttpMethod::Get, ["content"]) => match self.content_workspace.as_ref() {
                Some(workspace) => json_ok(workspace.snapshot()),
                None => content_repository_required(),
            },
            (HttpMethod::Post, ["content", "import"]) => {
                let request = match decode_body::<ContentImportRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                match self.content_workspace.as_mut() {
                    Some(workspace) => json_ok(
                        workspace.import(&request.authored_payload, request.replacement_policy),
                    ),
                    None => content_repository_required(),
                }
            }
            (HttpMethod::Post, ["content", "review"]) => {
                let request = match decode_body::<ContentReferenceRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                match self.content_workspace.as_ref() {
                    Some(workspace) => match workspace.review(&request.reference.to_authority()) {
                        Ok(review) => json_ok(review),
                        Err(error) => content_error(error),
                    },
                    None => content_repository_required(),
                }
            }
            (HttpMethod::Post, ["content", "compare"]) => {
                let request = match decode_body::<ContentPayloadRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                match self.content_workspace.as_ref() {
                    Some(workspace) => match workspace.compare(&request.authored_payload) {
                        Ok(diff) => json_ok(diff),
                        Err(error) => content_error(error),
                    },
                    None => content_repository_required(),
                }
            }
            (HttpMethod::Post, ["content", operation @ ("activate" | "deactivate" | "delete")]) => {
                let request = match decode_body::<ContentReferenceRequestDto>(request) {
                    Ok(request) => request,
                    Err(response) => return response,
                };
                let reference = request.reference.to_authority();
                let Some(workspace) = self.content_workspace.as_mut() else {
                    return content_repository_required();
                };
                let result = match *operation {
                    "activate" => workspace.activate(&reference),
                    "deactivate" => workspace.deactivate(&reference),
                    "delete" => workspace.delete(&reference),
                    _ => unreachable!(),
                };
                match result {
                    Ok(snapshot) => json_ok(snapshot),
                    Err(error) => content_error(error),
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
        BridgeErrorKind::SessionRecovery if error.retryable => 503,
        BridgeErrorKind::SessionRecovery => 422,
    };
    error_response(status, "bridge", error.code, error.message, error.retryable)
}

fn content_repository_required() -> HttpResponse {
    error_response(
        409,
        "content",
        "durableContentRepositoryRequired",
        "Authored content operations require the configured durable process host.",
        false,
    )
}

fn content_error(error: ContentWorkspaceError) -> HttpResponse {
    let status = match error.code.as_str() {
        "contentPackNotFound" | "contentReplacementTargetNotFound" => 404,
        "contentStorageUnavailable" | "contentAuditStorageUnavailable" => 503,
        "contentReplacementConfirmationRequired" | "contentPackAlreadyStored" => 409,
        _ => 422,
    };
    error_response(
        status,
        "content",
        error.code,
        error.message,
        error.retryable,
    )
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
