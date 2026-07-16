use std::collections::BTreeMap;

use rulebench_protocol::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatControlCommandDto,
    CombatSessionCreateRequestDto, CombatSessionHandleDto, CombatSessionIntentCommandDto,
    ProtocolHandshakeDto, ProtocolRequestContextDto, ReactionCommandSpecDto,
    ReplayArchiveMetadataDto, ReplayComparisonReadoutDto, ReplayPackageReviewDto,
    ReplayVerificationReadoutDto, ScenarioOptionDto, ScenarioParticipantOptionDto,
    SessionRecoveryEntryDto, UseActionIntentDto, ViewerScenarioReadoutDto,
    ViewerScenarioSummaryDto, ViewerSessionStepReadoutDto, ViewerSessionSummaryDto,
    ViewerSessionTranscriptDto, PROTOCOL_ID, PROTOCOL_VERSION,
};
use rulebench_rules::{
    compare_replay_packages, record_replay_package, verify_replay_package, CombatControlReadout,
    CombatSessionApi, CombatSessionArchive, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionCreateReadout, CombatSessionSnapshot,
    CombatSessionStepReadout, CommandCandidateSummary, CommandPreflightReadout,
    ContentPackSetReference, CurrentActorOptionSummary, InMemoryReplayArchiveStorage,
    InMemorySessionRecoveryStorage, ReactionCommandReadout, ReplayArchive, ReplayArchiveQuery,
    ReplayArchiveStorage, ReplayCommand, ReplayCommandRecordingSpec, ReplayPackage,
    RulebenchScenario, RulesetArtifactProvenance, SessionRecoveryPackage, SessionRecoveryStorage,
    AUTHORITY_SURFACE,
};

use crate::{BridgeError, BridgeErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeScenario {
    pub option: ScenarioOptionDto,
    pub scenario: RulebenchScenario,
    pub viewer_readout: Option<ViewerScenarioReadoutDto>,
}

impl BridgeScenario {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        scenario: RulebenchScenario,
    ) -> Self {
        let ruleset_version = scenario
            .selected_ruleset()
            .map(|ruleset| ruleset.version.clone())
            .unwrap_or_default();
        let content_pack_id = scenario
            .content_pack_set
            .as_ref()
            .map(|set| set.root.id.clone());
        let content_pack_version = scenario
            .content_pack_set
            .as_ref()
            .map(|set| set.root.version.clone());
        let participants = scenario
            .combatants
            .iter()
            .map(|combatant| ScenarioParticipantOptionDto {
                id: combatant.id.clone(),
                name: combatant.name.clone(),
                side_id: combatant.side_id.clone(),
                initiative: combatant.initiative,
            })
            .collect();
        Self {
            option: ScenarioOptionDto {
                id: id.into(),
                title: title.into(),
                summary: summary.into(),
                ruleset_id: scenario.selected_ruleset_id.clone(),
                ruleset_version,
                content_pack_id,
                content_pack_version,
                participants,
            },
            scenario,
            viewer_readout: None,
        }
    }

    pub fn with_viewer_readout(mut self, readout: ViewerScenarioReadoutDto) -> Self {
        self.viewer_readout = Some(readout);
        self
    }
}

pub struct RulebenchBridge {
    pub(crate) scenarios: BTreeMap<String, BridgeScenario>,
    viewer_sessions: BTreeMap<String, ViewerSessionTranscriptDto>,
    pub(crate) sessions: CombatSessionApi,
    pub(crate) replays: ReplayArchive<Box<dyn ReplayArchiveStorage>>,
    pub(crate) recovery: Box<dyn SessionRecoveryStorage>,
    pub(crate) recordings: BTreeMap<String, LiveReplayRecording>,
    pub(crate) experiments: BTreeMap<String, crate::experiment::ExperimentRecord>,
}

impl std::fmt::Debug for RulebenchBridge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RulebenchBridge")
            .field("scenarios", &self.scenarios)
            .field("sessions", &self.sessions)
            .field("recordings", &self.recordings)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LiveReplayRecording {
    pub(crate) initial_session: rulebench_rules::CombatSessionCreateRequest,
    pub(crate) commands: Vec<ReplayCommandRecordingSpec>,
    pub(crate) origin: String,
}

impl Default for RulebenchBridge {
    fn default() -> Self {
        Self {
            scenarios: BTreeMap::new(),
            viewer_sessions: BTreeMap::new(),
            sessions: CombatSessionApi::new(),
            replays: ReplayArchive::new(Box::new(InMemoryReplayArchiveStorage::new())),
            recovery: Box::new(InMemorySessionRecoveryStorage::new()),
            recordings: BTreeMap::new(),
            experiments: crate::experiment::empty_experiment_registry(),
        }
    }
}

impl RulebenchBridge {
    pub fn new(scenarios: impl IntoIterator<Item = BridgeScenario>) -> Result<Self, BridgeError> {
        Self::new_with_replays(scenarios, Vec::new())
    }

    pub fn new_with_replays(
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        replay_packages: impl IntoIterator<Item = ReplayPackage>,
    ) -> Result<Self, BridgeError> {
        Self::new_with_replays_and_viewer_sessions(scenarios, replay_packages, Vec::new())
    }

    pub fn new_with_replays_and_viewer_sessions(
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        replay_packages: impl IntoIterator<Item = ReplayPackage>,
        viewer_sessions: impl IntoIterator<Item = ViewerSessionTranscriptDto>,
    ) -> Result<Self, BridgeError> {
        let mut indexed = BTreeMap::new();
        for scenario in scenarios {
            if scenario.option.id.is_empty() {
                return Err(BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Bridge scenario id must not be empty.",
                ));
            }
            if indexed
                .insert(scenario.option.id.clone(), scenario)
                .is_some()
            {
                return Err(BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Bridge scenario ids must be unique.",
                ));
            }
        }
        let mut replays = ReplayArchive::new(
            Box::new(InMemoryReplayArchiveStorage::new()) as Box<dyn ReplayArchiveStorage>
        );
        for (index, replay) in replay_packages.into_iter().enumerate() {
            replays
                .save(replay, format!("fixture-{index:04}"))
                .map_err(BridgeError::from_replay_error)?;
        }
        let viewer_sessions = index_viewer_sessions(viewer_sessions)?;
        Ok(Self {
            scenarios: indexed,
            viewer_sessions,
            sessions: CombatSessionApi::new(),
            replays,
            recovery: Box::new(InMemorySessionRecoveryStorage::new()),
            recordings: BTreeMap::new(),
            experiments: crate::experiment::empty_experiment_registry(),
        })
    }

    pub fn new_with_replay_storage(
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        replay_storage: Box<dyn ReplayArchiveStorage>,
    ) -> Result<Self, BridgeError> {
        Self::new_with_replay_storage_and_viewer_sessions(scenarios, replay_storage, Vec::new())
    }

    pub fn new_with_replay_storage_and_viewer_sessions(
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        replay_storage: Box<dyn ReplayArchiveStorage>,
        viewer_sessions: impl IntoIterator<Item = ViewerSessionTranscriptDto>,
    ) -> Result<Self, BridgeError> {
        let mut bridge =
            Self::new_with_replays_and_viewer_sessions(scenarios, Vec::new(), viewer_sessions)?;
        bridge.replays = ReplayArchive::new(replay_storage);
        Ok(bridge)
    }

    pub fn new_with_durable_storage(
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        replay_storage: Box<dyn ReplayArchiveStorage>,
        recovery_storage: Box<dyn SessionRecoveryStorage>,
        viewer_sessions: impl IntoIterator<Item = ViewerSessionTranscriptDto>,
    ) -> Result<Self, BridgeError> {
        let mut bridge = Self::new_with_replay_storage_and_viewer_sessions(
            scenarios,
            replay_storage,
            viewer_sessions,
        )?;
        let packages = recovery_storage
            .list()
            .map_err(BridgeError::from_recovery_storage_error)?;
        bridge.recovery = recovery_storage;
        for package in packages {
            bridge.restore_recovery_package(package)?;
        }
        Ok(bridge)
    }

    pub fn handshake(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<ProtocolHandshakeDto, BridgeError> {
        self.check_version(context)?;
        Ok(ProtocolHandshakeDto {
            protocol_id: PROTOCOL_ID.to_string(),
            protocol_version: PROTOCOL_VERSION,
            authority_surface: AUTHORITY_SURFACE.to_string(),
        })
    }

    pub fn list_scenarios(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<ScenarioOptionDto>, BridgeError> {
        self.check_version(context)?;
        Ok(self
            .scenarios
            .values()
            .map(|scenario| scenario.option.clone())
            .collect())
    }

    pub fn list_viewer_scenarios(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<ViewerScenarioSummaryDto>, BridgeError> {
        self.check_version(context)?;
        Ok(self
            .scenarios
            .values()
            .filter_map(|scenario| {
                scenario
                    .viewer_readout
                    .as_ref()
                    .map(|readout| readout.identity.clone())
            })
            .collect())
    }

    pub fn get_viewer_scenario(
        &self,
        context: &ProtocolRequestContextDto,
        scenario_id: &str,
    ) -> Result<ViewerScenarioReadoutDto, BridgeError> {
        self.check_version(context)?;
        self.scenarios
            .get(scenario_id)
            .and_then(|scenario| scenario.viewer_readout.clone())
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::UnknownScenario,
                    format!("Viewer scenario does not exist: {scenario_id}"),
                )
            })
    }

    pub fn list_viewer_sessions(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<ViewerSessionSummaryDto>, BridgeError> {
        self.check_version(context)?;
        Ok(self
            .viewer_sessions
            .values()
            .map(|transcript| transcript.summary.clone())
            .collect())
    }

    pub fn get_viewer_session_step(
        &self,
        context: &ProtocolRequestContextDto,
        session_id: &str,
        step_id: &str,
    ) -> Result<ViewerSessionStepReadoutDto, BridgeError> {
        self.check_version(context)?;
        let transcript = self.viewer_sessions.get(session_id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::UnknownSession,
                format!("Viewer session does not exist: {session_id}"),
            )
        })?;
        transcript
            .steps
            .iter()
            .find(|step| step.step.id == step_id)
            .cloned()
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    format!("Viewer session step does not exist: {session_id}/{step_id}"),
                )
            })
    }

    pub fn create_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        request: &CombatSessionCreateRequestDto,
    ) -> Result<CombatSessionCreateReadout, BridgeError> {
        self.create_session_with_content_pack_set(context, request, None, None)
    }

    pub fn create_session_with_content_pack_set(
        &mut self,
        context: &ProtocolRequestContextDto,
        request: &CombatSessionCreateRequestDto,
        content_pack_set: Option<ContentPackSetReference>,
        content_ruleset: Option<RulesetArtifactProvenance>,
    ) -> Result<CombatSessionCreateReadout, BridgeError> {
        self.check_version(context)?;
        if request.session_id.is_empty() || request.scenario_id.is_empty() {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Session id and scenario id must not be empty.",
            ));
        }
        let scenario = self.scenarios.get(&request.scenario_id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::UnknownScenario,
                format!("Scenario does not exist: {}", request.scenario_id),
            )
        })?;
        let mut configured_scenario =
            configure_participant_order(scenario.scenario.clone(), &request.participant_order)?;
        if let Some(content_ruleset) = &content_ruleset {
            let ruleset = configured_scenario.selected_ruleset().ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidScenario,
                    "Scenario selected ruleset does not exist.",
                )
            })?;
            ruleset
                .validate_artifact_provenance(content_ruleset)
                .map_err(|error| {
                    BridgeError::new(
                        BridgeErrorKind::InvalidScenario,
                        format!("Authored content ruleset is incompatible: {error:?}"),
                    )
                })?;
        }
        if let Some(content_pack_set) = content_pack_set {
            configured_scenario.content_pack_set = Some(content_pack_set);
        }
        let initial_session = rulebench_rules::CombatSessionCreateRequest::new(
            &request.session_id,
            prepare_replay_scenario(configured_scenario.clone()),
        );
        let readout = self
            .sessions
            .create_session(rulebench_rules::CombatSessionCreateRequest::new(
                &request.session_id,
                configured_scenario,
            ))
            .map_err(BridgeError::from_session_error)?;
        self.recordings.insert(
            request.session_id.clone(),
            LiveReplayRecording {
                initial_session,
                commands: Vec::new(),
                origin: "new".to_string(),
            },
        );
        if let Err(error) = self.persist_recovery_checkpoint(&request.session_id) {
            let _ = self.sessions.discard_session(&readout.session);
            self.recordings.remove(&request.session_id);
            return Err(error);
        }
        Ok(readout)
    }

    pub fn list_sessions(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<CombatSessionSnapshot>, BridgeError> {
        self.check_version(context)?;
        Ok(self.sessions.list_active_sessions())
    }

    pub fn list_session_recovery(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<SessionRecoveryEntryDto>, BridgeError> {
        self.check_version(context)?;
        self.recordings
            .iter()
            .map(|(session_id, recording)| {
                let package = recovery_package(recording)?;
                Ok(SessionRecoveryEntryDto {
                    session_id: session_id.clone(),
                    origin: recording.origin.clone(),
                    state: "recoverable".to_string(),
                    generation: package.frame.generation,
                    last_verified_frame_id: format!(
                        "{}:{}",
                        package.frame.generation, package.frame.state_fingerprint.value
                    ),
                    pending_reaction_window_id: package.frame.pending_reaction_window_id,
                    actions: vec!["discard".to_string(), "fork".to_string()],
                })
            })
            .collect()
    }

    pub fn discard_recovery_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CombatSessionSnapshot, BridgeError> {
        self.check_version(context)?;
        let snapshot = self
            .sessions
            .snapshot(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)?;
        self.recovery
            .delete(&session.id)
            .map_err(BridgeError::from_recovery_storage_error)?;
        self.sessions
            .discard_session(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)?;
        self.recordings.remove(&session.id);
        Ok(snapshot)
    }

    pub fn fork_recovery_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        source: &CombatSessionHandleDto,
        new_session_id: &str,
    ) -> Result<CombatSessionCreateReadout, BridgeError> {
        self.check_version(context)?;
        if new_session_id.is_empty() {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Forked session id must not be empty.",
            ));
        }
        let source_recording = self.recordings.get(&source.id).cloned().ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::UnknownSession,
                format!("Session does not exist: {}", source.id),
            )
        })?;
        let mut recording = source_recording;
        recording.initial_session.session.id = new_session_id.to_string();
        recording.origin = "forked".to_string();
        let package = recovery_package(&recording)?;
        let restored = package
            .restore()
            .map_err(BridgeError::from_recovery_error)?;
        let readout = self
            .sessions
            .restore_session(restored.state)
            .map_err(BridgeError::from_session_error)?;
        self.recordings
            .insert(new_session_id.to_string(), recording);
        if let Err(error) = self.persist_recovery_checkpoint(new_session_id) {
            let _ = self.sessions.discard_session(&readout.session);
            self.recordings.remove(new_session_id);
            return Err(error);
        }
        Ok(readout)
    }

    pub fn get_session(
        &self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CombatSessionSnapshot, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .snapshot(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)
    }

    pub fn current_actor_options(
        &self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CurrentActorOptionSummary, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .current_actor_options(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)
    }

    pub fn preflight_command(
        &self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        intent: &UseActionIntentDto,
    ) -> Result<CommandPreflightReadout, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .preflight_command(&session.to_combat_session_handle(), intent.to_authority())
            .map_err(BridgeError::from_session_error)
    }

    pub fn command_candidates(
        &self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CommandCandidateSummary, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .command_candidates(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)
    }

    pub fn submit_intent(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &CombatSessionIntentCommandDto,
    ) -> Result<CombatSessionStepReadout, BridgeError> {
        self.check_version(context)?;
        require_command_id(&command.id)?;
        let authority = command.to_authority();
        let readout = self
            .sessions
            .submit_intent(&session.to_combat_session_handle(), authority.clone())
            .map_err(BridgeError::from_session_error)?;
        self.record_command(
            &session.id,
            command.id.clone(),
            ReplayCommand::Intent(authority),
        )?;
        Ok(readout)
    }

    pub fn submit_control(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &CombatControlCommandDto,
    ) -> Result<CombatControlReadout, BridgeError> {
        self.check_version(context)?;
        let authority = command.to_authority();
        let readout = self
            .sessions
            .submit_control(&session.to_combat_session_handle(), authority.clone())
            .map_err(BridgeError::from_session_error)?;
        let id = format!("control-{}", self.recording_command_count(&session.id)?);
        self.record_command(&session.id, id, ReplayCommand::Control(authority))?;
        Ok(readout)
    }

    pub fn submit_reaction(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &ReactionCommandSpecDto,
    ) -> Result<ReactionCommandReadout, BridgeError> {
        self.check_version(context)?;
        let authority = command.to_authority();
        let readout = self
            .sessions
            .submit_reaction(&session.to_combat_session_handle(), authority.clone())
            .map_err(BridgeError::from_session_error)?;
        let id = format!("reaction-{}", self.recording_command_count(&session.id)?);
        self.record_command(&session.id, id, ReplayCommand::Reaction(authority))?;
        Ok(readout)
    }

    pub fn automatic_step(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &AutomaticStepRequestDto,
    ) -> Result<CombatSessionAutomaticStepExecutionReadout, BridgeError> {
        self.check_version(context)?;
        require_command_id(&command.id)?;
        let authority = command.to_authority();
        let readout = self
            .sessions
            .automatic_step(&session.to_combat_session_handle(), authority.clone())
            .map_err(BridgeError::from_session_error)?;
        self.record_command(
            &session.id,
            command.id.clone(),
            ReplayCommand::AutomaticStep(authority),
        )?;
        Ok(readout)
    }

    pub fn automatic_run(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &AutomaticRunRequestDto,
    ) -> Result<CombatSessionAutomaticRunReadout, BridgeError> {
        self.check_version(context)?;
        require_command_id(&command.id)?;
        if command.max_steps == 0 {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Automatic run max steps must be greater than zero.",
            ));
        }
        let authority = command.to_authority();
        let readout = self
            .sessions
            .automatic_run(&session.to_combat_session_handle(), authority.clone())
            .map_err(BridgeError::from_session_error)?;
        self.record_command(
            &session.id,
            command.id.clone(),
            ReplayCommand::AutomaticRun(authority),
        )?;
        Ok(readout)
    }

    pub fn close_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CombatSessionArchive, BridgeError> {
        self.check_version(context)?;
        let handle = session.to_combat_session_handle();
        if let Some(archive) = self.sessions.archived_session(&handle) {
            return Ok(archive.clone());
        }
        if self
            .sessions
            .snapshot(&handle)
            .map_err(BridgeError::from_session_error)?
            .finalization
            .is_none()
        {
            return self
                .sessions
                .close_session(&handle)
                .map_err(BridgeError::from_session_error);
        }
        let recording = self.recordings.get(&session.id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Live session recording does not exist.",
            )
        })?;
        let ruleset = recording
            .initial_session
            .scenario
            .selected_ruleset()
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Live session ruleset does not exist.",
                )
            })?
            .artifact_provenance();
        let package_id = format!("live-{}", session.id);
        let package = record_replay_package(
            &package_id,
            recording.initial_session.clone(),
            ruleset,
            recording.commands.clone(),
        );
        self.replays
            .save(package, format!("session:{}", session.id))
            .map_err(BridgeError::from_replay_error)?;
        self.recovery
            .delete(&session.id)
            .map_err(BridgeError::from_recovery_storage_error)?;
        let archive = self
            .sessions
            .close_session(&handle)
            .map_err(BridgeError::from_session_error)?;
        self.recordings.remove(&session.id);
        Ok(archive)
    }

    pub fn list_replay_packages(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<ReplayArchiveMetadataDto>, BridgeError> {
        self.check_version(context)?;
        Ok(self
            .replays
            .list(&ReplayArchiveQuery::default())
            .map_err(BridgeError::from_replay_error)?
            .iter()
            .map(ReplayArchiveMetadataDto::from)
            .collect())
    }

    pub fn load_replay_package(
        &mut self,
        context: &ProtocolRequestContextDto,
        package_id: &str,
    ) -> Result<ReplayPackageReviewDto, BridgeError> {
        self.check_version(context)?;
        let package = self
            .replays
            .retrieve(package_id)
            .map_err(BridgeError::from_replay_error)?;
        Ok(ReplayPackageReviewDto::from(&package))
    }

    pub fn verify_replay_package(
        &mut self,
        context: &ProtocolRequestContextDto,
        package_id: &str,
    ) -> Result<ReplayVerificationReadoutDto, BridgeError> {
        self.check_version(context)?;
        let package = self
            .replays
            .retrieve(package_id)
            .map_err(BridgeError::from_replay_error)?;
        Ok(ReplayVerificationReadoutDto::from(&verify_replay_package(
            &package,
        )))
    }

    pub fn compare_replay_packages(
        &mut self,
        context: &ProtocolRequestContextDto,
        expected_package_id: &str,
        actual_package_id: &str,
    ) -> Result<ReplayComparisonReadoutDto, BridgeError> {
        self.check_version(context)?;
        let expected = self
            .replays
            .retrieve(expected_package_id)
            .map_err(BridgeError::from_replay_error)?;
        let actual = self
            .replays
            .retrieve(actual_package_id)
            .map_err(BridgeError::from_replay_error)?;
        Ok(ReplayComparisonReadoutDto::from(&compare_replay_packages(
            &expected, &actual,
        )))
    }

    pub(crate) fn check_version(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<(), BridgeError> {
        if context.protocol_version == PROTOCOL_VERSION {
            return Ok(());
        }
        Err(BridgeError::new(
            BridgeErrorKind::ProtocolVersionMismatch,
            format!(
                "Unsupported protocol version {}; expected {}.",
                context.protocol_version, PROTOCOL_VERSION
            ),
        ))
    }

    fn recording_command_count(&self, session_id: &str) -> Result<usize, BridgeError> {
        self.recordings
            .get(session_id)
            .map(|recording| recording.commands.len())
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Live session recording does not exist.",
                )
            })
    }

    fn record_command(
        &mut self,
        session_id: &str,
        id: String,
        command: ReplayCommand,
    ) -> Result<(), BridgeError> {
        let recording = self.recordings.get_mut(session_id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Live session recording does not exist.",
            )
        })?;
        recording
            .commands
            .push(ReplayCommandRecordingSpec::new(id, command));
        if let Err(error) = self.persist_recovery_checkpoint(session_id) {
            let recording = self.recordings.get_mut(session_id).ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Live session recording does not exist.",
                )
            })?;
            recording.commands.pop();
            let package = recovery_package(recording)?;
            let restored = package
                .restore()
                .map_err(BridgeError::from_recovery_error)?;
            self.sessions
                .replace_active_session(restored.state)
                .map_err(BridgeError::from_session_error)?;
            return Err(error);
        }
        Ok(())
    }

    fn persist_recovery_checkpoint(&mut self, session_id: &str) -> Result<(), BridgeError> {
        let recording = self.recordings.get(session_id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Live session recording does not exist.",
            )
        })?;
        let ruleset = recording
            .initial_session
            .scenario
            .selected_ruleset()
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidScenario,
                    "Live session ruleset does not exist.",
                )
            })?
            .artifact_provenance();
        let package = SessionRecoveryPackage::record(
            recording.commands.len() as u64,
            recording.initial_session.clone(),
            ruleset,
            recording.commands.clone(),
        )
        .map_err(BridgeError::from_recovery_error)?;
        self.recovery
            .write(package)
            .map_err(BridgeError::from_recovery_storage_error)
    }

    fn restore_recovery_package(
        &mut self,
        package: SessionRecoveryPackage,
    ) -> Result<(), BridgeError> {
        let restored = package
            .restore()
            .map_err(BridgeError::from_recovery_error)?;
        let session_id = package.session_id().to_string();
        self.sessions
            .restore_session(restored.state)
            .map_err(BridgeError::from_session_error)?;
        self.recordings.insert(
            session_id,
            LiveReplayRecording {
                initial_session: package.initial_session,
                commands: package
                    .commands
                    .into_iter()
                    .map(|record| ReplayCommandRecordingSpec::new(record.id, record.command))
                    .collect(),
                origin: "restored".to_string(),
            },
        );
        Ok(())
    }
}

fn recovery_package(
    recording: &LiveReplayRecording,
) -> Result<SessionRecoveryPackage, BridgeError> {
    let ruleset = recording
        .initial_session
        .scenario
        .selected_ruleset()
        .ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidScenario,
                "Live session ruleset does not exist.",
            )
        })?
        .artifact_provenance();
    SessionRecoveryPackage::record(
        recording.commands.len() as u64,
        recording.initial_session.clone(),
        ruleset,
        recording.commands.clone(),
    )
    .map_err(BridgeError::from_recovery_error)
}

fn index_viewer_sessions(
    sessions: impl IntoIterator<Item = ViewerSessionTranscriptDto>,
) -> Result<BTreeMap<String, ViewerSessionTranscriptDto>, BridgeError> {
    let mut indexed = BTreeMap::new();
    for transcript in sessions {
        if transcript.summary.id.is_empty() {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Viewer session id must not be empty.",
            ));
        }
        if indexed
            .insert(transcript.summary.id.clone(), transcript)
            .is_some()
        {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Viewer session ids must be unique.",
            ));
        }
    }
    Ok(indexed)
}

pub fn prepare_replay_scenario(mut scenario: RulebenchScenario) -> RulebenchScenario {
    if scenario.content_pack_set.is_some() {
        return scenario;
    }
    let root = rulebench_rules::ContentPackReference {
        id: format!("scenario.{}", scenario.metadata.id),
        version: "0.1.0".to_string(),
        fingerprint: rulebench_rules::ContentFingerprint {
            algorithm: "rulebench-scenario.v0".to_string(),
            value: scenario.metadata.id.clone(),
        },
    };
    let packs = vec![root.clone()];
    scenario.content_pack_set = Some(rulebench_rules::ContentPackSetReference {
        fingerprint: rulebench_rules::fingerprint_content_pack_set(&root, &packs),
        root,
        packs,
    });
    scenario
}

fn configure_participant_order(
    mut scenario: RulebenchScenario,
    participant_order: &[String],
) -> Result<RulebenchScenario, BridgeError> {
    if participant_order.is_empty() {
        return Ok(scenario);
    }
    if participant_order.len() != scenario.combatants.len() {
        return Err(BridgeError::new(
            BridgeErrorKind::InvalidRequest,
            format!(
                "Participant setup must include all {} scenario participants exactly once.",
                scenario.combatants.len()
            ),
        ));
    }
    let mut combatants = scenario
        .combatants
        .into_iter()
        .map(|combatant| (combatant.id.clone(), combatant))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::with_capacity(participant_order.len());
    for (index, participant_id) in participant_order.iter().enumerate() {
        let Some(mut combatant) = combatants.remove(participant_id) else {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                format!(
                    "Participant setup contains an unknown or duplicate participant id: {participant_id}."
                ),
            ));
        };
        combatant.initiative = i32::MAX
            - i32::try_from(index).map_err(|_| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Participant setup contains too many participants.",
                )
            })?;
        ordered.push(combatant);
    }
    if !combatants.is_empty() {
        return Err(BridgeError::new(
            BridgeErrorKind::InvalidRequest,
            "Participant setup omitted a scenario participant.",
        ));
    }
    scenario.combatants = ordered;
    Ok(scenario)
}

fn require_command_id(command_id: &str) -> Result<(), BridgeError> {
    if command_id.is_empty() {
        return Err(BridgeError::new(
            BridgeErrorKind::InvalidRequest,
            "Command id must not be empty.",
        ));
    }
    Ok(())
}
