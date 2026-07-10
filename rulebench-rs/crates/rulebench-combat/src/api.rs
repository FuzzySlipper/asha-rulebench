use std::collections::BTreeMap;

use rulebench_content::{validate_scenario_content_report, ContentValidationReport};

use crate::model::{
    CombatControlCommandSpec, CombatControlReadout, CombatSessionSnapshot,
    CombatSessionStepReadout, CommandCandidateSummary, CommandPreflightReadout,
    CurrentActorOptionSummary, RulebenchScenario, UseActionIntent,
};
use crate::{
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticRunSpec,
    CombatSessionAutomaticStepExecutionReadout, CombatSessionAutomaticStepSpec,
    CombatSessionIntentCommandSpec, CombatSessionState,
};

/// Opaque identity for one active or archived combat session.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombatSessionHandle {
    pub id: String,
}

impl CombatSessionHandle {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Validated input for creating a host-neutral combat session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCreateRequest {
    pub session: CombatSessionHandle,
    pub scenario: RulebenchScenario,
}

impl CombatSessionCreateRequest {
    pub fn new(session_id: impl Into<String>, scenario: RulebenchScenario) -> Self {
        Self {
            session: CombatSessionHandle::new(session_id),
            scenario,
        }
    }
}

/// Immutable readback emitted after a session is accepted for execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCreateReadout {
    pub session: CombatSessionHandle,
    pub snapshot: CombatSessionSnapshot,
}

/// Immutable archived handoff produced when an active session is closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionArchive {
    pub session: CombatSessionHandle,
    pub snapshot: CombatSessionSnapshot,
}

/// Stable host-neutral failures for session API calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatSessionApiError {
    EmptySessionId,
    DuplicateSessionId { session_id: String },
    UnknownSessionId { session_id: String },
    InvalidScenario { report: ContentValidationReport },
}

impl CombatSessionApiError {
    pub const fn code(&self) -> &'static str {
        match self {
            CombatSessionApiError::EmptySessionId => "emptySessionId",
            CombatSessionApiError::DuplicateSessionId { .. } => "duplicateSessionId",
            CombatSessionApiError::UnknownSessionId { .. } => "unknownSessionId",
            CombatSessionApiError::InvalidScenario { .. } => "invalidScenario",
        }
    }
}

/// Owner for active combat sessions and immutable archived readbacks.
///
/// The contained `CombatSessionState` never escapes this API. Host adapters can
/// therefore invoke authority behavior only through typed commands and
/// immutable results.
#[derive(Debug, Default)]
pub struct CombatSessionApi {
    active_sessions: BTreeMap<String, CombatSessionState>,
    archived_sessions: BTreeMap<String, CombatSessionArchive>,
}

impl CombatSessionApi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(
        &mut self,
        request: CombatSessionCreateRequest,
    ) -> Result<CombatSessionCreateReadout, CombatSessionApiError> {
        if request.session.id.is_empty() {
            return Err(CombatSessionApiError::EmptySessionId);
        }
        if self.active_sessions.contains_key(&request.session.id)
            || self.archived_sessions.contains_key(&request.session.id)
        {
            return Err(CombatSessionApiError::DuplicateSessionId {
                session_id: request.session.id,
            });
        }

        let report = validate_scenario_content_report(&request.scenario);
        if !report.accepted {
            return Err(CombatSessionApiError::InvalidScenario { report });
        }

        let session = CombatSessionState::new(request.session.id.clone(), request.scenario);
        let snapshot = session.snapshot();
        self.active_sessions
            .insert(request.session.id.clone(), session);

        Ok(CombatSessionCreateReadout {
            session: request.session,
            snapshot,
        })
    }

    pub fn list_active_sessions(&self) -> Vec<CombatSessionSnapshot> {
        self.active_sessions
            .values()
            .map(CombatSessionState::snapshot)
            .collect()
    }

    pub fn snapshot(
        &self,
        session: &CombatSessionHandle,
    ) -> Result<CombatSessionSnapshot, CombatSessionApiError> {
        Ok(self.active_session(session)?.snapshot())
    }

    pub fn start_session(
        &mut self,
        session: &CombatSessionHandle,
    ) -> Result<CombatControlReadout, CombatSessionApiError> {
        self.submit_control(session, CombatControlCommandSpec::explicit_start())
    }

    pub fn end_session(
        &mut self,
        session: &CombatSessionHandle,
    ) -> Result<CombatControlReadout, CombatSessionApiError> {
        self.submit_control(session, CombatControlCommandSpec::explicit_end())
    }

    pub fn submit_control(
        &mut self,
        session: &CombatSessionHandle,
        command: CombatControlCommandSpec,
    ) -> Result<CombatControlReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(session)?
            .submit_control_command(command))
    }

    pub fn current_actor_options(
        &self,
        session: &CombatSessionHandle,
    ) -> Result<CurrentActorOptionSummary, CombatSessionApiError> {
        Ok(self.active_session(session)?.current_actor_options())
    }

    pub fn preflight_command(
        &self,
        session: &CombatSessionHandle,
        intent: UseActionIntent,
    ) -> Result<CommandPreflightReadout, CombatSessionApiError> {
        Ok(self.active_session(session)?.preflight_command(intent))
    }

    pub fn command_candidates(
        &self,
        session: &CombatSessionHandle,
    ) -> Result<CommandCandidateSummary, CombatSessionApiError> {
        Ok(self
            .active_session(session)?
            .current_actor_command_candidates())
    }

    pub fn submit_intent(
        &mut self,
        session: &CombatSessionHandle,
        command: CombatSessionIntentCommandSpec,
    ) -> Result<CombatSessionStepReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(session)?
            .submit_intent_command(command))
    }

    pub fn automatic_step(
        &mut self,
        session: &CombatSessionHandle,
        command: CombatSessionAutomaticStepSpec,
    ) -> Result<CombatSessionAutomaticStepExecutionReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(session)?
            .submit_automatic_step(command))
    }

    pub fn automatic_run(
        &mut self,
        session: &CombatSessionHandle,
        command: CombatSessionAutomaticRunSpec,
    ) -> Result<CombatSessionAutomaticRunReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(session)?
            .run_automatic_combat(command))
    }

    pub fn close_session(
        &mut self,
        session: &CombatSessionHandle,
    ) -> Result<CombatSessionArchive, CombatSessionApiError> {
        let Some(active_session) = self.active_sessions.remove(&session.id) else {
            return Err(CombatSessionApiError::UnknownSessionId {
                session_id: session.id.clone(),
            });
        };
        let archive = CombatSessionArchive {
            session: session.clone(),
            snapshot: active_session.snapshot(),
        };
        self.archived_sessions
            .insert(session.id.clone(), archive.clone());
        Ok(archive)
    }

    pub fn archived_session(&self, session: &CombatSessionHandle) -> Option<&CombatSessionArchive> {
        self.archived_sessions.get(&session.id)
    }

    fn active_session(
        &self,
        session: &CombatSessionHandle,
    ) -> Result<&CombatSessionState, CombatSessionApiError> {
        self.active_sessions.get(&session.id).ok_or_else(|| {
            CombatSessionApiError::UnknownSessionId {
                session_id: session.id.clone(),
            }
        })
    }

    fn active_session_mut(
        &mut self,
        session: &CombatSessionHandle,
    ) -> Result<&mut CombatSessionState, CombatSessionApiError> {
        self.active_sessions.get_mut(&session.id).ok_or_else(|| {
            CombatSessionApiError::UnknownSessionId {
                session_id: session.id.clone(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn api_owns_lifecycle_and_archives_immutable_session_readbacks() {
        let mut api = CombatSessionApi::new();
        let request = CombatSessionCreateRequest::new("api-session", valid_scenario());

        let created = api.create_session(request).expect("session is valid");
        assert_eq!(
            created.snapshot.lifecycle.phase,
            CombatLifecyclePhase::Ready
        );
        assert_eq!(api.list_active_sessions().len(), 1);

        let started = api.start_session(&created.session).expect("active session");
        assert!(started.accepted);
        let ended = api.end_session(&created.session).expect("active session");
        assert!(ended.accepted);

        let repeated_start = api.start_session(&created.session).expect("active session");
        assert!(!repeated_start.accepted);
        assert_eq!(
            repeated_start.decision_kind,
            CombatControlDecisionKind::RejectedByLifecycle
        );

        let archive = api
            .close_session(&created.session)
            .expect("active session closes");
        assert_eq!(
            archive.snapshot.lifecycle.phase,
            CombatLifecyclePhase::Ended
        );
        assert!(api.list_active_sessions().is_empty());
        assert_eq!(api.archived_session(&created.session), Some(&archive));
        assert_eq!(
            api.snapshot(&created.session)
                .expect_err("closed session is not active")
                .code(),
            "unknownSessionId"
        );
    }

    #[test]
    fn api_rejects_invalid_creation_and_unknown_call_ordering() {
        let mut api = CombatSessionApi::new();
        let missing = CombatSessionHandle::new("missing");

        assert_eq!(
            api.start_session(&missing)
                .expect_err("missing session cannot start")
                .code(),
            "unknownSessionId"
        );
        assert_eq!(
            api.create_session(CombatSessionCreateRequest::new(
                "invalid",
                invalid_scenario()
            ))
            .expect_err("invalid content cannot create a session")
            .code(),
            "invalidScenario"
        );

        let request = CombatSessionCreateRequest::new("duplicate", valid_scenario());
        api.create_session(request.clone())
            .expect("first create succeeds");
        assert_eq!(
            api.create_session(request)
                .expect_err("duplicate session id is rejected")
                .code(),
            "duplicateSessionId"
        );
    }

    fn invalid_scenario() -> RulebenchScenario {
        let mut scenario = valid_scenario();
        scenario.selected_ruleset_id = "missing-ruleset".to_string();
        scenario
    }

    fn valid_scenario() -> RulebenchScenario {
        let selected_action = action_definition();
        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "combat-api".to_string(),
                title: "Combat API".to_string(),
                summary: "Minimal valid session API scenario.".to_string(),
                seed_label: "combat-api".to_string(),
            },
            rulesets: vec![RulesetMetadata {
                id: "combat-api.v0".to_string(),
                name: "Combat API Rules".to_string(),
                version: "0.0.0".to_string(),
                summary: "Minimal validated ruleset.".to_string(),
                modules: vec![RuleModuleDeclaration::action_resolution(
                    ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
                )],
            }],
            selected_ruleset_id: "combat-api.v0".to_string(),
            grid: Grid {
                width: 2,
                height: 1,
                cells: vec![
                    GridCell {
                        position: GridPosition { x: 0, y: 0 },
                        terrain_tags: Vec::new(),
                    },
                    GridCell {
                        position: GridPosition { x: 1, y: 0 },
                        terrain_tags: Vec::new(),
                    },
                ],
            },
            combatants: vec![
                combatant("adept", Team::Ally, 0, "nerve", 12),
                combatant("raider", Team::Enemy, 1, "nerve", 10),
            ],
            entities: vec![entity("adept"), entity("raider")],
            abilities: vec![AbilityDefinition {
                id: "ability.api".to_string(),
                name: "API Bolt".to_string(),
                kind: AbilityDefinitionKind::Ability,
                summary: "Minimal action ability.".to_string(),
                tags: Vec::new(),
            }],
            selected_ability_id: None,
            classes: Vec::new(),
            selected_class_id: None,
            stat_definitions: vec![StatDefinition {
                id: "mind".to_string(),
                label: "Mind".to_string(),
                kind: StatDefinitionKind::Base,
                formula: None,
                summary: "Attack stat.".to_string(),
            }],
            modifiers: vec![ModifierDefinition {
                id: "marked".to_string(),
                label: "marked".to_string(),
                summary: "Minimal hit modifier.".to_string(),
                default_tenure: ModifierTenure::Temporary,
                stat_adjustments: Vec::new(),
            }],
            items: Vec::new(),
            selected_item_id: None,
            actions: vec![selected_action.clone()],
            selected_action,
        }
    }

    fn entity(id: &str) -> EntityDefinition {
        EntityDefinition {
            id: id.to_string(),
            name: id.to_string(),
            summary: "Minimal entity.".to_string(),
            tags: Vec::new(),
            damage_adjustments: Vec::new(),
        }
    }

    fn combatant(id: &str, team: Team, x: u32, defense_id: &str, hit_points: i32) -> Combatant {
        Combatant {
            id: id.to_string(),
            entity_id: id.to_string(),
            name: id.to_string(),
            team,
            position: GridPosition { x, y: 0 },
            hit_points: BoundedValue {
                current: hit_points,
                max: hit_points,
            },
            temporary_vitality: 0,
            class_ids: Vec::new(),
            stats: StatBlock {
                base_stats: vec![NamedNumber {
                    id: "mind".to_string(),
                    label: "Mind".to_string(),
                    value: 1,
                }],
                derived_stats: Vec::new(),
            },
            defenses: vec![NamedNumber {
                id: defense_id.to_string(),
                label: "Nerve".to_string(),
                value: 10,
            }],
            equipped_item_ids: Vec::new(),
            active_modifiers: Vec::new(),
            conditions: Vec::new(),
            is_actor: id == "adept",
        }
    }

    fn action_definition() -> ActionDefinition {
        ActionDefinition {
            id: "api_bolt".to_string(),
            ruleset_id: "combat-api.v0".to_string(),
            ability_id: "ability.api".to_string(),
            name: "API Bolt".to_string(),
            actor_id: "adept".to_string(),
            targeting: TargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 2,
                visibility_requirement: VisibilityRequirement::Ignored,
                target_ids: vec!["raider".to_string()],
                visible_target_ids: Vec::new(),
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 1,
                modifier_stat_id: "mind".to_string(),
                defense: DefenseReference {
                    id: "nerve".to_string(),
                    label: "Nerve".to_string(),
                },
            }),
            hit: HitEffect {
                damage_bonus: 1,
                damage_type: "force".to_string(),
                modifier_id: "marked".to_string(),
                modifier_label: "marked".to_string(),
                modifier_duration: "one turn".to_string(),
                operations: Vec::new(),
            },
            action_text: "Mind versus Nerve.".to_string(),
            effect_text: "Minimal hit effect.".to_string(),
        }
    }
}
