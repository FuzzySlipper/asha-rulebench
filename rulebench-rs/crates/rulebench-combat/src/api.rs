use std::collections::BTreeMap;

use rulebench_content::{validate_scenario_content_report, ContentValidationReport};

use crate::model::{
    CombatControlCommandSpec, CombatControlReadout, CombatSessionSnapshot,
    CombatSessionStepReadout, CommandCandidateSummary, CommandPreflightReadout,
    CurrentActorOptionSummary, EquipmentCommandReadout, EquipmentCommandSpec,
    ReactionCommandReadout, ReactionCommandSpec, RulebenchScenario, UseActionIntent,
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
    SessionNotFinalized { session_id: String },
    InvalidScenario { report: ContentValidationReport },
}

impl CombatSessionApiError {
    pub const fn code(&self) -> &'static str {
        match self {
            CombatSessionApiError::EmptySessionId => "emptySessionId",
            CombatSessionApiError::DuplicateSessionId { .. } => "duplicateSessionId",
            CombatSessionApiError::UnknownSessionId { .. } => "unknownSessionId",
            CombatSessionApiError::SessionNotFinalized { .. } => "sessionNotFinalized",
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

    /// Install a session that was reconstructed and verified by the replay
    /// owner. Private runtime state still never crosses this API boundary.
    pub fn restore_session(
        &mut self,
        state: CombatSessionState,
    ) -> Result<CombatSessionCreateReadout, CombatSessionApiError> {
        let snapshot = state.snapshot();
        let session = CombatSessionHandle::new(&snapshot.session_id);
        if session.id.is_empty() {
            return Err(CombatSessionApiError::EmptySessionId);
        }
        if self.active_sessions.contains_key(&session.id)
            || self.archived_sessions.contains_key(&session.id)
        {
            return Err(CombatSessionApiError::DuplicateSessionId {
                session_id: session.id,
            });
        }
        self.active_sessions.insert(session.id.clone(), state);
        Ok(CombatSessionCreateReadout { session, snapshot })
    }

    /// Replace an existing active session with a replay-verified reconstruction.
    ///
    /// This rollback seam cannot create a new identity or replace an archived
    /// session.
    pub fn replace_active_session(
        &mut self,
        state: CombatSessionState,
    ) -> Result<CombatSessionSnapshot, CombatSessionApiError> {
        let snapshot = state.snapshot();
        if !self.active_sessions.contains_key(&snapshot.session_id) {
            return Err(CombatSessionApiError::UnknownSessionId {
                session_id: snapshot.session_id,
            });
        }
        self.active_sessions
            .insert(snapshot.session_id.clone(), state);
        Ok(snapshot)
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

    pub fn submit_equipment(
        &mut self,
        handle: &CombatSessionHandle,
        command: EquipmentCommandSpec,
    ) -> Result<EquipmentCommandReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(handle)?
            .submit_equipment_command(command))
    }

    pub fn submit_reaction(
        &mut self,
        handle: &CombatSessionHandle,
        command: ReactionCommandSpec,
    ) -> Result<ReactionCommandReadout, CombatSessionApiError> {
        Ok(self
            .active_session_mut(handle)?
            .submit_reaction_command(command))
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
        let active_session = self.active_session(session)?;
        if active_session.finalization().is_none() {
            return Err(CombatSessionApiError::SessionNotFinalized {
                session_id: session.id.clone(),
            });
        }
        let Some(active_session) = self.active_sessions.remove(&session.id) else {
            unreachable!("active session was checked before removal");
        };
        let archive = CombatSessionArchive {
            session: session.clone(),
            snapshot: active_session.snapshot(),
        };
        self.archived_sessions
            .insert(session.id.clone(), archive.clone());
        Ok(archive)
    }

    pub fn discard_session(
        &mut self,
        session: &CombatSessionHandle,
    ) -> Result<CombatSessionSnapshot, CombatSessionApiError> {
        let state = self.active_sessions.remove(&session.id).ok_or_else(|| {
            CombatSessionApiError::UnknownSessionId {
                session_id: session.id.clone(),
            }
        })?;
        Ok(state.snapshot())
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
    use rpg_ir::ActionResourceCost;

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
            api.close_session(&request.session)
                .expect_err("active combat must finalize before archival")
                .code(),
            "sessionNotFinalized"
        );
        assert_eq!(api.list_active_sessions().len(), 1);
        assert_eq!(
            api.create_session(request)
                .expect_err("duplicate session id is rejected")
                .code(),
            "duplicateSessionId"
        );
    }

    #[test]
    fn api_lethal_intent_completes_combat_through_authoritative_readbacks() {
        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("manual", valid_scenario()))
            .expect("valid combat session is created");
        let session = created.session;

        assert!(
            api.start_session(&session)
                .expect("session starts")
                .accepted
        );
        let options = api
            .current_actor_options(&session)
            .expect("options are externally readable");
        assert_eq!(options.current_actor_id, Some("adept".to_string()));

        let intent = UseActionIntent::new("adept", "api_bolt", "raider");
        assert!(
            api.preflight_command(&session, intent.clone())
                .expect("preflight readback")
                .accepted
        );
        assert!(
            api.submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "manual-hit",
                    "Manual API hit",
                    "A caller submits the selected intent.",
                    intent,
                    vec![20, 20],
                ),
            )
            .expect("intent submission")
            .receipt
            .accepted
        );

        let finalized = api.snapshot(&session).expect("lethal intent finalizes");
        let end = api
            .submit_control(&session, CombatControlCommandSpec::end_if_condition_met())
            .expect("conditional end readback");
        assert!(!end.accepted);
        let snapshot = api.snapshot(&session).expect("final snapshot");
        assert_eq!(snapshot, finalized);
        assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(snapshot.combat_log.len(), 1);
        assert_eq!(snapshot.audit_log.len(), 1);
        assert!(snapshot.combat_end_condition.combat_should_end);
        assert_eq!(
            snapshot.current_actor_options.lifecycle_phase,
            CombatLifecyclePhase::Ended
        );
    }

    #[test]
    fn api_rejects_unaffordable_declared_cost_without_resource_mutation() {
        let mut scenario = valid_scenario();
        let costly_resource = ActionResourceCost {
            resource_id: "standard-action".to_string(),
            amount: 2,
        };
        scenario.actions[0].resource_costs = vec![costly_resource.clone()];
        scenario.selected_action.resource_costs = vec![costly_resource.clone()];

        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("unaffordable", scenario))
            .expect("resource costs are structurally valid authored content");
        let session = created.session;
        api.start_session(&session).expect("session starts");

        let intent = UseActionIntent::new("adept", "api_bolt", "raider");
        let before = api.snapshot(&session).expect("initial snapshot");
        let preflight = api
            .preflight_command(&session, intent.clone())
            .expect("preflight readback");
        assert!(!preflight.accepted);
        assert_eq!(
            preflight.decision_kind,
            CommandPreflightDecisionKind::RejectedByActionResource
        );
        assert_eq!(preflight.resource_costs, vec![costly_resource]);
        assert!(api
            .command_candidates(&session)
            .expect("candidate readback")
            .candidates
            .is_empty());

        let step = api
            .submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "unaffordable-command",
                    "Unaffordable command",
                    "The declared standard-action cost exceeds the available resource.",
                    intent,
                    vec![20, 20],
                ),
            )
            .expect("rejected command still produces an authoritative readback");
        let after = api.snapshot(&session).expect("post-rejection snapshot");

        assert!(!step.receipt.accepted);
        assert_eq!(
            step.audit_entry.state_before_fingerprint,
            step.audit_entry.state_after_fingerprint
        );
        assert_eq!(before.action_resource_ledger, after.action_resource_ledger);
        assert!(after.action_resource_transition_log.is_empty());
    }

    #[test]
    fn api_refreshes_standard_action_when_its_turn_returns() {
        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("refresh", valid_scenario()))
            .expect("valid session is created");
        let session = created.session;
        api.start_session(&session).expect("session starts");

        let intent = UseActionIntent::new("adept", "api_bolt", "raider");
        let step = api
            .submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "spend-standard-action",
                    "Spend standard action",
                    "Spend the authored standard-action cost.",
                    intent,
                    vec![20, 1],
                ),
            )
            .expect("intent submission");
        assert!(step.receipt.accepted);
        assert_eq!(
            step.action_resource_ledger.combatants[0].resources[0].current,
            0
        );

        api.submit_control(&session, CombatControlCommandSpec::advance_turn())
            .expect("advance to raider");
        api.submit_control(&session, CombatControlCommandSpec::advance_turn())
            .expect("advance back to adept");
        let snapshot = api.snapshot(&session).expect("refreshed snapshot");

        assert_eq!(
            snapshot.turn_order.current_actor_id,
            Some("adept".to_string())
        );
        assert_eq!(
            snapshot.action_resource_ledger.combatants[0].resources[0].current,
            1
        );
        let transitions = &snapshot.action_resource_transition_log;
        assert_eq!(
            transitions[0].transition_kind,
            ActionResourceTransitionKind::Spent
        );
        assert_eq!(transitions[0].amount, 1);
        let refresh = transitions.last().expect("turn return refresh is logged");
        assert_eq!(
            refresh.transition_kind,
            ActionResourceTransitionKind::Refreshed
        );
        assert_eq!(refresh.combatant_id, "adept");
        assert_eq!(refresh.amount, 1);
        assert!(
            api.preflight_command(
                &session,
                UseActionIntent::new("adept", "api_bolt", "raider")
            )
            .expect("refreshed preflight")
            .accepted
        );
    }

    #[test]
    fn api_enforces_typed_pools_and_reopens_a_timed_cooldown() {
        let mut scenario = valid_scenario();
        scenario.combatants[0].resource_pools = vec![
            ActionResourcePool::standard_action(),
            ActionResourcePool {
                id: "spell-slot-1".to_string(),
                kind: ActionResourceKind::SpellSlot,
                initial: 2,
                maximum: 2,
                refresh_policy: ActionResourceRefreshPolicy::CombatStart,
            },
            ActionResourcePool {
                id: "arcane-charge".to_string(),
                kind: ActionResourceKind::Charge,
                initial: 2,
                maximum: 2,
                refresh_policy: ActionResourceRefreshPolicy::Never,
            },
            ActionResourcePool {
                id: "api-bolt-cooldown".to_string(),
                kind: ActionResourceKind::Cooldown,
                initial: 1,
                maximum: 1,
                refresh_policy: ActionResourceRefreshPolicy::Turns(2),
            },
        ];
        let costs = vec![
            ActionResourceCost::standard_action(),
            ActionResourceCost {
                resource_id: "spell-slot-1".to_string(),
                amount: 1,
            },
            ActionResourceCost {
                resource_id: "arcane-charge".to_string(),
                amount: 1,
            },
            ActionResourceCost {
                resource_id: "api-bolt-cooldown".to_string(),
                amount: 1,
            },
        ];
        scenario.actions[0].resource_costs = costs.clone();
        scenario.selected_action.resource_costs = costs;

        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("typed-pools", scenario))
            .expect("typed resource scenario is valid");
        let session = created.session;
        api.start_session(&session).expect("session starts");

        let intent = UseActionIntent::new("adept", "api_bolt", "raider");
        let accepted = api
            .submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "typed-pool-command",
                    "Typed resource command",
                    "Spend one action, slot, charge, and cooldown resource.",
                    intent.clone(),
                    vec![20, 1],
                ),
            )
            .expect("first typed resource command is accepted");
        assert!(accepted.receipt.accepted);
        let after_spend = api.snapshot(&session).expect("spent resource snapshot");
        let adept_resources = &after_spend.action_resource_ledger.combatants[0].resources;
        assert_eq!(resource_current(adept_resources, "spell-slot-1"), Some(1));
        assert_eq!(resource_current(adept_resources, "arcane-charge"), Some(1));
        assert_eq!(
            resource_current(adept_resources, "api-bolt-cooldown"),
            Some(0)
        );
        assert_eq!(
            resource_remaining_turns(adept_resources, "api-bolt-cooldown"),
            Some(Some(2))
        );
        assert!(!after_spend.current_actor_options.available);
        assert_eq!(
            after_spend.current_actor_options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::NoAvailableResources)
        );
        let spent_action_option = &after_spend.current_actor_options.actions[0];
        assert!(!spent_action_option.available);
        assert_eq!(spent_action_option.resource_costs.len(), 4);
        assert_eq!(spent_action_option.resource_states.len(), 4);
        assert_eq!(
            resource_current(&spent_action_option.resource_states, "api-bolt-cooldown"),
            Some(0)
        );

        let rejected = api
            .submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "typed-pool-retry",
                    "Rejected typed resource retry",
                    "The same action cannot reuse its spent cooldown or action resource.",
                    intent,
                    vec![20, 1],
                ),
            )
            .expect("rejected command returns readback");
        let after_rejection = api.snapshot(&session).expect("rejected resource snapshot");
        assert!(!rejected.receipt.accepted);
        assert_eq!(
            after_spend.action_resource_ledger,
            after_rejection.action_resource_ledger
        );

        for _ in 0..4 {
            api.submit_control(&session, CombatControlCommandSpec::advance_turn())
                .expect("turn advances for cooldown timing");
        }
        let refreshed = api.snapshot(&session).expect("cooldown refresh snapshot");
        let resources = &refreshed.action_resource_ledger.combatants[0].resources;
        assert_eq!(resource_current(resources, "api-bolt-cooldown"), Some(1));
        assert_eq!(
            resource_remaining_turns(resources, "api-bolt-cooldown"),
            Some(None)
        );
        assert!(refreshed
            .action_resource_transition_log
            .iter()
            .any(|entry| entry.transition_kind == ActionResourceTransitionKind::CooldownAdvanced));
        assert!(refreshed.current_actor_options.actions[0].available);
        assert!(refreshed.current_actor_options.actions[0].available);
        assert!(
            api.preflight_command(
                &session,
                UseActionIntent::new("adept", "api_bolt", "raider")
            )
            .expect("cooldown-refreshed preflight")
            .accepted
        );
    }

    #[test]
    fn api_equipment_commands_apply_and_remove_shared_item_grants() {
        let mut scenario = valid_scenario();
        scenario.combatants[0].base_ability_ids.clear();
        scenario.combatants[0].inventory_item_ids = vec!["item.api-focus".to_string()];
        scenario.items.push(ItemDefinition {
            id: "item.api-focus".to_string(),
            name: "API Focus".to_string(),
            summary: "Equipment integration fixture.".to_string(),
            tags: vec!["focus".to_string()],
            equipment_slot: "implement".to_string(),
            requirements: vec![StatRequirement {
                stat_id: "mind".to_string(),
                minimum: 1,
            }],
            granted_modifier_ids: vec!["marked".to_string()],
            granted_ability_ids: vec!["ability.api".to_string()],
            granted_resource_pools: vec![ActionResourcePool {
                id: "focus-charge".to_string(),
                kind: ActionResourceKind::Charge,
                initial: 2,
                maximum: 2,
                refresh_policy: ActionResourceRefreshPolicy::Never,
            }],
        });
        let focus_cost = ActionResourceCost {
            resource_id: "focus-charge".to_string(),
            amount: 1,
        };
        scenario.actions[0].resource_costs.push(focus_cost.clone());
        scenario.selected_action.resource_costs.push(focus_cost);

        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("equipment", scenario))
            .expect("unequipped owned item scenario is valid");
        let session = created.session;
        let unavailable = api
            .preflight_command(
                &session,
                UseActionIntent::new("adept", "api_bolt", "raider"),
            )
            .expect("preflight before equipment");
        assert_eq!(
            unavailable.decision_kind,
            CommandPreflightDecisionKind::RejectedByAbilityAvailability
        );

        let equipped = api
            .submit_equipment(
                &session,
                EquipmentCommandSpec::equip("adept", "item.api-focus"),
            )
            .expect("equipment command readout");
        assert!(equipped.accepted);
        let equipped_snapshot = api.snapshot(&session).expect("equipped snapshot");
        assert!(equipped_snapshot.current_actor_options.actions[0].available);
        let focus_resource = equipped_snapshot.action_resource_ledger.combatants[0]
            .resources
            .iter()
            .find(|resource| resource.resource_id == "focus-charge")
            .expect("item resource is present");
        assert_eq!(focus_resource.source_id, "item.api-focus");
        assert!(equipped_snapshot.current_state.combatants[0]
            .conditions
            .contains(&"marked".to_string()));
        assert_eq!(equipped_snapshot.equipment_transition_log.len(), 1);
        assert_eq!(
            equipped_snapshot.equipment_transition_log[0].item_id,
            "item.api-focus"
        );
        let duplicate = api
            .submit_equipment(
                &session,
                EquipmentCommandSpec::equip("adept", "item.api-focus"),
            )
            .expect("duplicate equipment readout");
        assert!(!duplicate.accepted);
        assert_eq!(
            duplicate.decision_kind,
            EquipmentDecisionKind::RejectedByEquippedState
        );
        assert_eq!(
            api.snapshot(&session)
                .expect("duplicate rejection snapshot")
                .equipment_transition_log
                .len(),
            1
        );

        let unequipped = api
            .submit_equipment(
                &session,
                EquipmentCommandSpec::unequip("adept", "item.api-focus"),
            )
            .expect("unequipment command readout");
        assert!(unequipped.accepted);
        let unequipped_snapshot = api.snapshot(&session).expect("unequipped snapshot");
        assert!(!unequipped_snapshot.current_actor_options.actions[0].available);
        assert!(!unequipped_snapshot.action_resource_ledger.combatants[0]
            .resources
            .iter()
            .any(|resource| resource.resource_id == "focus-charge"));
        assert!(!unequipped_snapshot.current_state.combatants[0]
            .conditions
            .contains(&"marked".to_string()));
        assert_eq!(unequipped_snapshot.equipment_transition_log.len(), 2);

        api.end_session(&session).expect("session ends");
        let after_end = api
            .submit_equipment(
                &session,
                EquipmentCommandSpec::equip("adept", "item.api-focus"),
            )
            .expect("post-end equipment readout");
        assert_eq!(
            after_end.decision_kind,
            EquipmentDecisionKind::RejectedByLifecycle
        );
    }

    #[test]
    fn api_applies_versioned_cumulative_class_grants() {
        let mut scenario = valid_scenario();
        scenario.combatants[0].base_ability_ids.clear();
        scenario.combatants[0].class_inputs = vec![ClassLevelInput {
            class_id: "class.api-adept".to_string(),
            version: "1.2.0".to_string(),
            level: 2,
        }];
        scenario.classes.push(ClassDefinition {
            id: "class.api-adept".to_string(),
            name: "API Adept".to_string(),
            version: "1.2.0".to_string(),
            summary: "Class grant integration fixture.".to_string(),
            tags: vec!["caster".to_string()],
            prerequisites: vec![StatRequirement {
                stat_id: "mind".to_string(),
                minimum: 1,
            }],
            level_grants: vec![
                ClassLevelGrant {
                    level: 1,
                    granted_modifier_ids: Vec::new(),
                    granted_ability_ids: vec!["ability.api".to_string()],
                    granted_resource_pools: Vec::new(),
                },
                ClassLevelGrant {
                    level: 2,
                    granted_modifier_ids: vec!["marked".to_string()],
                    granted_ability_ids: Vec::new(),
                    granted_resource_pools: vec![ActionResourcePool {
                        id: "class-charge".to_string(),
                        kind: ActionResourceKind::Charge,
                        initial: 2,
                        maximum: 2,
                        refresh_policy: ActionResourceRefreshPolicy::Never,
                    }],
                },
            ],
        });
        let class_cost = ActionResourceCost {
            resource_id: "class-charge".to_string(),
            amount: 1,
        };
        scenario.actions[0].resource_costs.push(class_cost.clone());
        scenario.selected_action.resource_costs.push(class_cost);
        scenario.modifiers[0]
            .stat_adjustments
            .push(ModifierStatAdjustment {
                stat_id: "mind".to_string(),
                stat_label: "Mind".to_string(),
                delta: 1,
            });

        let granted_scenario =
            crate::CombatState::from_scenario(&scenario).apply_to_scenario(scenario.clone());
        let granted_stats = crate::effective_stats_for_combatant(&granted_scenario, "adept")
            .expect("class modifier feeds effective stat evaluation");
        assert_eq!(
            granted_stats
                .stats
                .iter()
                .find(|stat| stat.stat_id == "mind")
                .map(|stat| stat.effective_value),
            Some(2)
        );

        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new("class-grants", scenario))
            .expect("versioned class build is valid");
        let snapshot = api
            .snapshot(&created.session)
            .expect("class build snapshot");
        let build = &snapshot.class_build_ledger.combatants[0].class_inputs[0];
        assert_eq!(build.class_id, "class.api-adept");
        assert_eq!(build.version, "1.2.0");
        assert_eq!(build.level, 2);
        assert_eq!(build.applied_grant_levels, vec![1, 2]);
        assert_eq!(
            build.source_ids,
            vec![
                "class:class.api-adept@1.2.0:1".to_string(),
                "class:class.api-adept@1.2.0:2".to_string(),
            ]
        );
        let class_resource = snapshot.action_resource_ledger.combatants[0]
            .resources
            .iter()
            .find(|resource| resource.resource_id == "class-charge")
            .expect("class resource exists");
        assert_eq!(class_resource.source_id, "class:class.api-adept@1.2.0:2");
        assert!(snapshot.current_state.combatants[0]
            .conditions
            .contains(&"marked".to_string()));
        assert!(
            api.preflight_command(
                &created.session,
                UseActionIntent::new("adept", "api_bolt", "raider"),
            )
            .expect("class-granted action preflight")
            .accepted
        );
    }

    #[test]
    fn api_orders_nested_reactions_and_resumes_before_effect_resolution() {
        let mut scenario = valid_scenario();
        let hook = ReactionHookEffectOperation {
            hook_id: "api-before-effect".to_string(),
            window: ReactionWindow::BeforeEffect,
            eligible_reactor_ids: vec!["raider".to_string(), "adept".to_string()],
            options: vec![
                ReactionOptionDeclaration {
                    id: "adept-counter".to_string(),
                    reactor_id: "adept".to_string(),
                    opens_nested_window: true,
                },
                ReactionOptionDeclaration {
                    id: "raider-guard".to_string(),
                    reactor_id: "raider".to_string(),
                    opens_nested_window: false,
                },
            ],
            maximum_nested_depth: 1,
        };
        scenario.actions[0]
            .hit
            .operations
            .push(HitEffectOperation::OpenReactionWindow(hook.clone()));
        scenario
            .selected_action
            .hit
            .operations
            .push(HitEffectOperation::OpenReactionWindow(hook));

        let mut api = CombatSessionApi::new();
        let created = api
            .create_session(CombatSessionCreateRequest::new(
                "reactions",
                scenario.clone(),
            ))
            .expect("reaction scenario is valid");
        let session = created.session;
        let command = api
            .submit_intent(
                &session,
                CombatSessionIntentCommandSpec::new(
                    "reaction-trigger",
                    "Reaction trigger",
                    "Pause before effects for ordered reactions.",
                    UseActionIntent::new("adept", "api_bolt", "raider"),
                    vec![20, 20],
                ),
            )
            .expect("trigger command readout");
        assert!(command.receipt.accepted);
        assert_eq!(command.state_after.combatants[1].hit_points.current, 10);
        assert_eq!(
            command
                .receipt
                .projection
                .as_ref()
                .expect("resolver projection remains visible")
                .combatants[1]
                .hit_points
                .current,
            0
        );
        let pending = api.snapshot(&session).expect("pending reaction snapshot");
        assert_eq!(pending.gameplay_fabric.pending_decision_count, 1);
        assert_eq!(pending.gameplay_fabric.decisions.len(), 1);
        assert_eq!(pending.gameplay_fabric.decisions[0].status, "Suspended");
        assert!(pending.gameplay_fabric.reaction_frame_hashes.is_empty());
        assert_eq!(pending.current_state.combatants[1].hit_points.current, 10);
        assert_eq!(
            resource_current(
                &pending.action_resource_ledger.combatants[0].resources,
                "standard-action"
            ),
            Some(1)
        );
        assert_eq!(
            pending.current_actor_options.unavailable_reason,
            Some(CurrentActorOptionsUnavailableReason::ReactionWindowOpen)
        );
        let root = pending
            .current_reaction_window
            .expect("root reaction window is visible");
        assert_eq!(root.current_reactor_id.as_deref(), Some("adept"));

        let rejected_advance = api
            .submit_control(&session, CombatControlCommandSpec::advance_turn())
            .expect("paused turn-control readout");
        assert_eq!(
            rejected_advance.decision_kind,
            CombatControlDecisionKind::RejectedByReactionWindow
        );
        let rejected_end = api
            .submit_control(&session, CombatControlCommandSpec::end_if_condition_met())
            .expect("paused end-control readout");
        assert_eq!(
            rejected_end.decision_kind,
            CombatControlDecisionKind::RejectedByReactionWindow
        );
        let rejected_equipment = api
            .submit_equipment(
                &session,
                EquipmentCommandSpec::equip("adept", "item.not-present"),
            )
            .expect("paused equipment readout");
        assert_eq!(
            rejected_equipment.decision_kind,
            EquipmentDecisionKind::RejectedByReactionWindow
        );

        let out_of_order = api
            .submit_reaction(
                &session,
                ReactionCommandSpec::pass(root.id.clone(), "raider"),
            )
            .expect("out-of-order reaction readout");
        assert_eq!(
            out_of_order.decision_kind,
            ReactionDecisionKind::RejectedOutOfOrder
        );
        let nested_open = api
            .submit_reaction(
                &session,
                ReactionCommandSpec::accept(root.id.clone(), "adept", "adept-counter"),
            )
            .expect("nested reaction readout");
        let nested = nested_open
            .opened_nested_window
            .expect("nested window opens");
        assert_eq!(nested.depth, 1);
        assert_eq!(nested.parent_window_id.as_deref(), Some(root.id.as_str()));

        let beyond_limit = api
            .submit_reaction(
                &session,
                ReactionCommandSpec::accept(nested.id.clone(), "adept", "adept-counter"),
            )
            .expect("bounded nesting rejection");
        assert_eq!(
            beyond_limit.decision_kind,
            ReactionDecisionKind::RejectedNestedLimit
        );
        api.submit_reaction(
            &session,
            ReactionCommandSpec::pass(nested.id.clone(), "adept"),
        )
        .expect("nested adept pass");
        api.submit_reaction(&session, ReactionCommandSpec::pass(nested.id, "raider"))
            .expect("nested raider pass");
        let resumed = api
            .submit_reaction(
                &session,
                ReactionCommandSpec::pass(root.id.clone(), "raider"),
            )
            .expect("root raider pass");
        assert!(resumed.resumed_pending_resolution);

        let resolved = api.snapshot(&session).expect("resolved reaction snapshot");
        assert_eq!(resolved.gameplay_fabric.pending_decision_count, 0);
        assert_eq!(resolved.gameplay_fabric.decisions.len(), 2);
        assert_eq!(resolved.gameplay_fabric.decisions[1].status, "Accepted");
        assert!(resolved.gameplay_fabric.decisions[1].routing_hash.is_some());
        assert_eq!(resolved.gameplay_fabric.reaction_frame_hashes.len(), 1);
        assert!(resolved.current_reaction_window.is_none());
        assert_eq!(resolved.current_state.combatants[1].hit_points.current, 0);
        assert_eq!(resolved.lifecycle.phase, CombatLifecyclePhase::Ended);
        assert_eq!(
            resolved
                .finalization
                .as_ref()
                .map(|finalization| finalization.outcome_kind),
            Some(CombatOutcomeKind::Victory)
        );
        assert_eq!(
            resource_current(
                &resolved.action_resource_ledger.combatants[0].resources,
                "standard-action"
            ),
            Some(0)
        );
        assert!(resolved.reaction_window_lifecycle_log.iter().any(|entry| {
            entry.lifecycle_kind == ReactionWindowLifecycleKind::ResolutionResumed
                && entry.window_id == root.id
        }));
        assert_eq!(resolved.reaction_audit_log.len(), 6);

        let before_stale = api.snapshot(&session).expect("final snapshot is readable");
        let stale = api
            .submit_reaction(&session, ReactionCommandSpec::pass(root.id, "adept"))
            .expect("stale reaction readout");
        assert_eq!(
            stale.decision_kind,
            ReactionDecisionKind::RejectedNoOpenWindow
        );
        assert_eq!(
            api.snapshot(&session).expect("post-end snapshot is stable"),
            before_stale
        );

        let miss_session = api
            .create_session(CombatSessionCreateRequest::new("reaction-miss", scenario))
            .expect("miss reaction scenario is valid")
            .session;
        let miss = api
            .submit_intent(
                &miss_session,
                CombatSessionIntentCommandSpec::new(
                    "reaction-miss",
                    "Reaction miss",
                    "A missed hit effect must not open its reaction window.",
                    UseActionIntent::new("adept", "api_bolt", "raider"),
                    vec![1, 1],
                ),
            )
            .expect("miss command readout");
        assert_eq!(miss.step.outcome_class, CommandOutcomeClass::AcceptedMiss);
        assert!(api
            .snapshot(&miss_session)
            .expect("miss snapshot")
            .current_reaction_window
            .is_none());
    }

    fn resource_current(resources: &[ActionResourceState], resource_id: &str) -> Option<i32> {
        resources
            .iter()
            .find(|resource| resource.resource_id == resource_id)
            .map(|resource| resource.current)
    }

    fn resource_remaining_turns(
        resources: &[ActionResourceState],
        resource_id: &str,
    ) -> Option<Option<u32>> {
        resources
            .iter()
            .find(|resource| resource.resource_id == resource_id)
            .map(|resource| resource.remaining_refresh_turns)
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
            content_pack_set: None,
            authored_action_binding: None,
            authored_scenario_binding: None,
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
                stacking_group: "marked".to_string(),
                stacking_policy: ModifierStackingPolicy::Refresh,
                duration_policy: ModifierDurationPolicy::Turns(1),
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
            side_id: match team {
                Team::Ally => "ally",
                Team::Enemy => "enemy",
            }
            .to_string(),
            initiative: 0,
            position: GridPosition { x, y: 0 },
            hit_points: BoundedValue {
                current: hit_points,
                max: hit_points,
            },
            temporary_vitality: 0,
            class_inputs: Vec::new(),
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
            resource_pools: vec![ActionResourcePool::standard_action()],
            inventory_item_ids: Vec::new(),
            equipped_item_ids: Vec::new(),
            base_ability_ids: vec!["ability.api".to_string()],
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
                operation_pipeline: None,
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
                operations: vec![
                    rpg_ir::HitEffectOperation::Damage(rpg_ir::DamageEffectOperation {
                        damage_bonus: 1,
                        damage_type: "force".to_string(),
                    }),
                    rpg_ir::HitEffectOperation::ApplyModifier(rpg_ir::ModifierEffectOperation {
                        modifier_id: "marked".to_string(),
                        modifier_label: "marked".to_string(),
                        modifier_duration: "one turn".to_string(),
                    }),
                ],
            },
            resource_costs: vec![ActionResourceCost::standard_action()],
            movement: None,
            action_text: "Mind versus Nerve.".to_string(),
            effect_text: "Minimal hit effect.".to_string(),
        }
    }
}
