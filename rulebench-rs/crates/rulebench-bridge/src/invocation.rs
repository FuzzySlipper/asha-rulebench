use std::collections::BTreeMap;

use rulebench_protocol::{
    AutomaticRunRequestDto, AutomaticStepRequestDto, CombatControlCommandDto,
    CombatSessionCreateRequestDto, CombatSessionHandleDto, CombatSessionIntentCommandDto,
    ProtocolHandshakeDto, ProtocolRequestContextDto, ScenarioOptionDto,
    ScenarioParticipantOptionDto, UseActionIntentDto, PROTOCOL_ID, PROTOCOL_VERSION,
};
use rulebench_rules::{
    CombatControlReadout, CombatSessionApi, CombatSessionArchive, CombatSessionAutomaticRunReadout,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionCreateReadout, CombatSessionSnapshot,
    CombatSessionStepReadout, CommandCandidateSummary, CommandPreflightReadout,
    CurrentActorOptionSummary, RulebenchScenario, AUTHORITY_SURFACE,
};

use crate::{BridgeError, BridgeErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeScenario {
    pub option: ScenarioOptionDto,
    pub scenario: RulebenchScenario,
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
        }
    }
}

#[derive(Debug, Default)]
pub struct RulebenchBridge {
    scenarios: BTreeMap<String, BridgeScenario>,
    sessions: CombatSessionApi,
}

impl RulebenchBridge {
    pub fn new(scenarios: impl IntoIterator<Item = BridgeScenario>) -> Result<Self, BridgeError> {
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
        Ok(Self {
            scenarios: indexed,
            sessions: CombatSessionApi::new(),
        })
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

    pub fn create_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        request: &CombatSessionCreateRequestDto,
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
        let configured_scenario =
            configure_participant_order(scenario.scenario.clone(), &request.participant_order)?;
        self.sessions
            .create_session(rulebench_rules::CombatSessionCreateRequest::new(
                &request.session_id,
                configured_scenario,
            ))
            .map_err(BridgeError::from_session_error)
    }

    pub fn list_sessions(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<CombatSessionSnapshot>, BridgeError> {
        self.check_version(context)?;
        Ok(self.sessions.list_active_sessions())
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
        self.sessions
            .submit_intent(&session.to_combat_session_handle(), command.to_authority())
            .map_err(BridgeError::from_session_error)
    }

    pub fn submit_control(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &CombatControlCommandDto,
    ) -> Result<CombatControlReadout, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .submit_control(&session.to_combat_session_handle(), command.to_authority())
            .map_err(BridgeError::from_session_error)
    }

    pub fn automatic_step(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
        command: &AutomaticStepRequestDto,
    ) -> Result<CombatSessionAutomaticStepExecutionReadout, BridgeError> {
        self.check_version(context)?;
        require_command_id(&command.id)?;
        self.sessions
            .automatic_step(&session.to_combat_session_handle(), command.to_authority())
            .map_err(BridgeError::from_session_error)
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
        self.sessions
            .automatic_run(&session.to_combat_session_handle(), command.to_authority())
            .map_err(BridgeError::from_session_error)
    }

    pub fn close_session(
        &mut self,
        context: &ProtocolRequestContextDto,
        session: &CombatSessionHandleDto,
    ) -> Result<CombatSessionArchive, BridgeError> {
        self.check_version(context)?;
        self.sessions
            .close_session(&session.to_combat_session_handle())
            .map_err(BridgeError::from_session_error)
    }

    fn check_version(&self, context: &ProtocolRequestContextDto) -> Result<(), BridgeError> {
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
