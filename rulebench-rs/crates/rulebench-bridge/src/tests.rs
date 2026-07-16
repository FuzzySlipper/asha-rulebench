use rulebench_protocol::{
    CombatAutomationNoCandidateBehaviorDto, CombatAutomationPolicyDto, CombatControlCommandDto,
    CombatControlCommandKindDto, CombatSessionCreateRequestDto, CombatSessionHandleDto,
    CombatSessionIntentCommandDto, CommandRollModeDto, ExperimentComparisonRequestDto,
    ExperimentMatrixRequestDto, ProtocolRequestContextDto, UseActionIntentDto, PROTOCOL_VERSION,
};
use rulebench_rules::*;
use std::collections::BTreeMap;

use crate::{BridgeErrorKind, BridgeScenario, RulebenchBridge};

fn bridge() -> RulebenchBridge {
    RulebenchBridge::new([BridgeScenario::new(
        "hexing-bolt",
        "Hexing Bolt",
        "Bridge contract fixture.",
        minimal_scenario(),
    )])
    .expect("fixture registry is valid")
}

#[derive(Debug)]
struct FailOnWriteRecoveryStorage {
    fail_on_write: usize,
    write_count: usize,
    packages: BTreeMap<String, SessionRecoveryPackage>,
}

impl FailOnWriteRecoveryStorage {
    fn new(fail_on_write: usize) -> Self {
        Self {
            fail_on_write,
            write_count: 0,
            packages: BTreeMap::new(),
        }
    }
}

impl SessionRecoveryStorage for FailOnWriteRecoveryStorage {
    fn write(
        &mut self,
        package: SessionRecoveryPackage,
    ) -> Result<(), SessionRecoveryStorageError> {
        self.write_count += 1;
        if self.write_count == self.fail_on_write {
            return Err(SessionRecoveryStorageError::WriteFailed {
                session_id: package.session_id().to_string(),
            });
        }
        self.packages
            .insert(package.session_id().to_string(), package);
        Ok(())
    }

    fn list(&self) -> Result<Vec<SessionRecoveryPackage>, SessionRecoveryStorageError> {
        Ok(self.packages.values().cloned().collect())
    }

    fn delete(&mut self, session_id: &str) -> Result<(), SessionRecoveryStorageError> {
        self.packages.remove(session_id);
        Ok(())
    }
}

#[derive(Debug)]
struct FailOnceReplayStorage {
    fail_next_write: bool,
    entries: BTreeMap<String, ReplayArchiveEntry>,
}

impl FailOnceReplayStorage {
    fn new() -> Self {
        Self {
            fail_next_write: true,
            entries: BTreeMap::new(),
        }
    }
}

impl ReplayArchiveStorage for FailOnceReplayStorage {
    fn write(&mut self, entry: ReplayArchiveEntry) -> Result<(), ReplayArchiveStorageError> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(ReplayArchiveStorageError::WriteFailed {
                package_id: entry.metadata.package_id,
            });
        }
        self.entries
            .insert(entry.metadata.package_id.clone(), entry);
        Ok(())
    }

    fn read(
        &self,
        package_id: &str,
    ) -> Result<Option<ReplayArchiveEntry>, ReplayArchiveStorageError> {
        Ok(self.entries.get(package_id).cloned())
    }

    fn list(&self) -> Result<Vec<ReplayArchiveMetadata>, ReplayArchiveStorageError> {
        Ok(self
            .entries
            .values()
            .map(|entry| entry.metadata.clone())
            .collect())
    }

    fn clear(&mut self) -> Result<(), ReplayArchiveStorageError> {
        self.entries.clear();
        Ok(())
    }
}

#[derive(Debug, Default)]
struct FailOnceDeleteRecoveryStorage {
    fail_next_delete: bool,
    packages: BTreeMap<String, SessionRecoveryPackage>,
}

impl FailOnceDeleteRecoveryStorage {
    fn new() -> Self {
        Self {
            fail_next_delete: true,
            packages: BTreeMap::new(),
        }
    }
}

impl SessionRecoveryStorage for FailOnceDeleteRecoveryStorage {
    fn write(
        &mut self,
        package: SessionRecoveryPackage,
    ) -> Result<(), SessionRecoveryStorageError> {
        self.packages
            .insert(package.session_id().to_string(), package);
        Ok(())
    }

    fn list(&self) -> Result<Vec<SessionRecoveryPackage>, SessionRecoveryStorageError> {
        Ok(self.packages.values().cloned().collect())
    }

    fn delete(&mut self, session_id: &str) -> Result<(), SessionRecoveryStorageError> {
        if self.fail_next_delete {
            self.fail_next_delete = false;
            return Err(SessionRecoveryStorageError::DeleteFailed {
                session_id: session_id.to_string(),
            });
        }
        self.packages.remove(session_id);
        Ok(())
    }
}

fn bridge_with_recovery_storage(recovery: Box<dyn SessionRecoveryStorage>) -> RulebenchBridge {
    bridge_with_storages(Box::new(InMemoryReplayArchiveStorage::new()), recovery)
}

fn bridge_with_storages(
    replay: Box<dyn ReplayArchiveStorage>,
    recovery: Box<dyn SessionRecoveryStorage>,
) -> RulebenchBridge {
    RulebenchBridge::new_with_durable_storage(
        [BridgeScenario::new(
            "hexing-bolt",
            "Hexing Bolt",
            "Bridge contract fixture.",
            minimal_scenario(),
        )],
        replay,
        recovery,
        Vec::new(),
    )
    .expect("fixture registry is valid")
}

fn minimal_scenario() -> RulebenchScenario {
    let selected_action = action_definition();
    RulebenchScenario {
        metadata: ScenarioMetadata {
            id: "bridge-contract".to_string(),
            title: "Bridge Contract".to_string(),
            summary: "Minimal valid bridge scenario.".to_string(),
            seed_label: "bridge-contract".to_string(),
        },
        content_pack_set: None,
        rulesets: vec![RulesetMetadata {
            id: "bridge.v0".to_string(),
            name: "Bridge Rules".to_string(),
            version: "0.0.0".to_string(),
            summary: "Minimal validated ruleset.".to_string(),
            modules: vec![RuleModuleDeclaration::action_resolution(
                ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight(),
            )],
        }],
        selected_ruleset_id: "bridge.v0".to_string(),
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
            id: "ability.bridge".to_string(),
            name: "Bridge Bolt".to_string(),
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
        base_ability_ids: vec!["ability.bridge".to_string()],
        active_modifiers: Vec::new(),
        conditions: Vec::new(),
        is_actor: id == "adept",
    }
}

fn action_definition() -> ActionDefinition {
    ActionDefinition {
        id: "bridge_bolt".to_string(),
        ruleset_id: "bridge.v0".to_string(),
        ability_id: "ability.bridge".to_string(),
        name: "Bridge Bolt".to_string(),
        actor_id: "adept".to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 2,
            visibility_requirement: VisibilityRequirement::Ignored,
            target_ids: vec!["raider".to_string()],
            visible_target_ids: vec!["raider".to_string()],
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
                HitEffectOperation::Damage(DamageEffectOperation {
                    damage_bonus: 1,
                    damage_type: "force".to_string(),
                }),
                HitEffectOperation::ApplyModifier(ModifierEffectOperation {
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

fn context() -> ProtocolRequestContextDto {
    ProtocolRequestContextDto::current()
}

fn create(bridge: &mut RulebenchBridge, session_id: &str) -> CombatSessionHandleDto {
    let created = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: session_id.to_string(),
                scenario_id: "hexing-bolt".to_string(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        )
        .expect("fixture session is valid");
    CombatSessionHandleDto::from(&created.session)
}

#[test]
fn bridge_rolls_back_creation_when_the_initial_recovery_checkpoint_fails() {
    let mut bridge = bridge_with_recovery_storage(Box::new(FailOnWriteRecoveryStorage::new(1)));

    let error = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: "uncommitted".to_string(),
                scenario_id: "hexing-bolt".to_string(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        )
        .expect_err("failed initial checkpoint rejects the session");

    assert_eq!(error.kind, BridgeErrorKind::SessionRecovery);
    assert!(bridge
        .list_sessions(&context())
        .expect("session catalog remains readable")
        .is_empty());
    assert!(bridge
        .list_session_recovery(&context())
        .expect("recovery catalog remains readable")
        .is_empty());
}

#[test]
fn bridge_restores_the_previous_frame_when_a_command_checkpoint_fails() {
    let mut bridge = bridge_with_recovery_storage(Box::new(FailOnWriteRecoveryStorage::new(2)));
    let session = create(&mut bridge, "rollback-command");

    let error = bridge
        .submit_control(
            &context(),
            &session,
            &CombatControlCommandDto {
                kind: CombatControlCommandKindDto::ExplicitStart,
            },
        )
        .expect_err("failed command checkpoint rejects the command");

    assert_eq!(error.kind, BridgeErrorKind::SessionRecovery);
    let snapshot = bridge
        .get_session(&context(), &session)
        .expect("previous verified frame remains active");
    assert_eq!(snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
    let recovery = bridge
        .list_session_recovery(&context())
        .expect("recovery catalog remains readable");
    assert_eq!(recovery[0].generation, 0);
}

#[test]
fn bridge_calls_real_authority_through_a_complete_manual_lifecycle() {
    let mut bridge = bridge();
    let handshake = bridge.handshake(&context()).expect("version is supported");
    assert_eq!(handshake.protocol_version, PROTOCOL_VERSION);

    let session = create(&mut bridge, "manual");
    let created = bridge
        .get_session(&context(), &session)
        .expect("session is active");
    assert_eq!(created.lifecycle.phase, CombatLifecyclePhase::Ready);

    let started = bridge
        .submit_control(
            &context(),
            &session,
            &CombatControlCommandDto {
                kind: CombatControlCommandKindDto::ExplicitStart,
            },
        )
        .expect("session exists");
    assert!(started.accepted);

    let options = bridge
        .current_actor_options(&context(), &session)
        .expect("options are readable");
    assert!(
        options.available,
        "options unavailable: {:?}",
        options.unavailable_reason
    );
    let candidate = bridge
        .command_candidates(&context(), &session)
        .expect("candidates are readable")
        .candidates
        .into_iter()
        .find(|candidate| candidate.accepted)
        .expect("fixture has an accepted candidate");
    let submitted = bridge
        .submit_intent(
            &context(),
            &session,
            &CombatSessionIntentCommandDto {
                id: "manual-step".to_string(),
                title: "Manual action".to_string(),
                summary: "Submit the bridge-selected candidate.".to_string(),
                intent: UseActionIntentDto {
                    actor_id: candidate.intent.actor_id,
                    action_id: candidate.intent.action_id,
                    target_id: candidate.intent.target_id,
                    target_ids: Vec::new(),
                    target_cell: None,
                    destination_cell: None,
                    observed_origin: None,
                },
                roll_stream: vec![17, 5],
                roll_mode: CommandRollModeDto::Supplied,
                generated_seed: None,
            },
        )
        .expect("command reaches authority");
    assert!(submitted.audit_entry.accepted);

    let ended = bridge
        .submit_control(
            &context(),
            &session,
            &CombatControlCommandDto {
                kind: CombatControlCommandKindDto::ExplicitEnd,
            },
        )
        .expect("session exists");
    assert!(ended.accepted);
    let archive = bridge
        .close_session(&context(), &session)
        .expect("ended session can close");
    assert_eq!(archive.session.id, "manual");
    assert_eq!(
        bridge
            .close_session(&context(), &session)
            .expect("closing an archived session is idempotent"),
        archive
    );
    let packages = bridge
        .list_replay_packages(&context())
        .expect("recorded replay is listed");
    assert_eq!(
        packages
            .iter()
            .filter(|package| package.package_id == "live-manual")
            .count(),
        1
    );
    let review = bridge
        .load_replay_package(&context(), "live-manual")
        .expect("recorded replay loads");
    assert_eq!(review.session_id, "manual");
    let verification = bridge
        .verify_replay_package(&context(), "live-manual")
        .expect("recorded replay verifies");
    assert!(verification.accepted);
    assert!(verification.finalized);
}

#[test]
fn bridge_keeps_session_handles_isolated() {
    let mut bridge = bridge();
    let first = create(&mut bridge, "first");
    let second = create(&mut bridge, "second");

    bridge
        .submit_control(
            &context(),
            &first,
            &CombatControlCommandDto {
                kind: CombatControlCommandKindDto::ExplicitStart,
            },
        )
        .expect("first exists");

    let first_snapshot = bridge
        .get_session(&context(), &first)
        .expect("first exists");
    let second_snapshot = bridge
        .get_session(&context(), &second)
        .expect("second exists");
    assert_eq!(
        first_snapshot.lifecycle.phase,
        CombatLifecyclePhase::InProgress
    );
    assert_eq!(second_snapshot.lifecycle.phase, CombatLifecyclePhase::Ready);
}

#[test]
fn bridge_fails_closed_for_versions_handles_commands_and_lifecycle() {
    let mut bridge = bridge();
    let version = bridge
        .handshake(&ProtocolRequestContextDto {
            protocol_version: PROTOCOL_VERSION + 1,
        })
        .expect_err("unsupported version must fail");
    assert_eq!(version.kind, BridgeErrorKind::ProtocolVersionMismatch);

    let missing = bridge
        .get_session(
            &context(),
            &CombatSessionHandleDto {
                id: "missing".to_string(),
            },
        )
        .expect_err("unknown handle must fail");
    assert_eq!(missing.kind, BridgeErrorKind::UnknownSession);

    let session = create(&mut bridge, "invalid");
    let invalid_command = bridge
        .submit_intent(
            &context(),
            &session,
            &CombatSessionIntentCommandDto {
                id: String::new(),
                title: "Invalid".to_string(),
                summary: "Missing identity.".to_string(),
                intent: UseActionIntentDto {
                    actor_id: "entity-adept".to_string(),
                    action_id: "hexing_bolt".to_string(),
                    target_id: "entity-raider".to_string(),
                    target_ids: Vec::new(),
                    target_cell: None,
                    destination_cell: None,
                    observed_origin: None,
                },
                roll_stream: vec![17, 5],
                roll_mode: CommandRollModeDto::Supplied,
                generated_seed: None,
            },
        )
        .expect_err("empty command id must fail");
    assert_eq!(invalid_command.kind, BridgeErrorKind::InvalidRequest);

    let close = bridge
        .close_session(&context(), &session)
        .expect_err("ready session cannot close");
    assert_eq!(close.kind, BridgeErrorKind::InvalidLifecycle);
}

#[test]
fn bridge_rejects_duplicate_and_unknown_scenario_requests() {
    let mut bridge = bridge();
    create(&mut bridge, "duplicate");
    let duplicate = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: "duplicate".to_string(),
                scenario_id: "hexing-bolt".to_string(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        )
        .expect_err("duplicate session must fail");
    assert_eq!(duplicate.kind, BridgeErrorKind::DuplicateSession);

    let unknown = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: "unknown".to_string(),
                scenario_id: "missing".to_string(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        )
        .expect_err("unknown scenario must fail");
    assert_eq!(unknown.kind, BridgeErrorKind::UnknownScenario);
}

#[test]
fn bridge_exposes_setup_metadata_and_validates_participant_order() {
    let mut bridge = bridge();
    let options = bridge
        .list_scenarios(&context())
        .expect("scenario options load");
    let option = &options[0];
    assert_eq!(option.ruleset_id, "bridge.v0");
    assert_eq!(option.participants.len(), 2);

    let created = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: "reordered".to_string(),
                scenario_id: "hexing-bolt".to_string(),
                participant_order: vec!["raider".to_string(), "adept".to_string()],
                content_pack: None,
            },
        )
        .expect("complete participant order is valid");
    assert_eq!(
        created.snapshot.turn_order.participant_order,
        vec!["raider".to_string(), "adept".to_string()]
    );

    let invalid = bridge
        .create_session(
            &context(),
            &CombatSessionCreateRequestDto {
                session_id: "invalid-setup".to_string(),
                scenario_id: "hexing-bolt".to_string(),
                participant_order: vec!["adept".to_string()],
                content_pack: None,
            },
        )
        .expect_err("incomplete participant order is rejected");
    assert_eq!(invalid.kind, BridgeErrorKind::InvalidRequest);
    assert!(invalid
        .message
        .contains("include all 2 scenario participants"));
}

#[test]
fn policy_catalog_reports_real_ruleset_compatibility() {
    let bridge = bridge();
    let catalog = bridge
        .automation_policy_catalog(&context())
        .expect("policy catalog loads");

    assert_eq!(catalog.len(), 3);
    let objective = catalog
        .iter()
        .find(|policy| policy.id == OBJECTIVE_SIDE_PRESSURE_POLICY_ID)
        .expect("objective policy is registered");
    assert_eq!(objective.requirement, "objectiveSidePolicy");
    assert_eq!(objective.compatibility.len(), 1);
    assert!(!objective.compatibility[0].compatible);
    assert_eq!(
        objective.compatibility[0].code,
        "incompatibleRulesetCapability"
    );
}

#[test]
fn experiment_matrix_advances_one_trial_at_a_time_and_archives_verified_replays() {
    let mut bridge = bridge();
    let created = bridge
        .create_experiment(&context(), &experiment_request("bounded-lab", vec![7, 7]))
        .expect("compatible bounded matrix is created");
    assert_eq!(created.status, "planned");
    assert_eq!(created.planned_trial_count, 2);
    assert!(created.trials.is_empty());

    let first = bridge
        .advance_experiment(&context(), "bounded-lab")
        .expect("first trial advances");
    assert_eq!(first.status, "running");
    assert_eq!(first.completed_trial_count, 1);
    assert!(first.trials[0].replay_verified);
    assert!(!first.trials[0].decisions.is_empty());
    assert!(!first.trials[0].materialized_rolls.is_empty());

    let completed = bridge
        .advance_experiment(&context(), "bounded-lab")
        .expect("second trial completes matrix");
    assert_eq!(completed.status, "completed");
    assert_eq!(completed.completed_trial_count, 2);
    assert_eq!(
        completed.trials[0].final_state_fingerprint,
        completed.trials[1].final_state_fingerprint
    );
    assert_eq!(completed.trials[0].decisions, completed.trials[1].decisions);
    let replay_ids = bridge
        .list_replay_packages(&context())
        .expect("archived experiment replay list");
    assert!(replay_ids
        .iter()
        .any(|replay| replay.package_id == completed.trials[0].replay_package_id));
}

#[test]
fn failed_experiment_checkpoint_cleans_up_and_the_same_trial_retries() {
    let mut bridge = bridge_with_recovery_storage(Box::new(FailOnWriteRecoveryStorage::new(2)));
    bridge
        .create_experiment(&context(), &experiment_request("checkpoint-retry", vec![7]))
        .expect("matrix creates");

    bridge
        .advance_experiment(&context(), "checkpoint-retry")
        .expect_err("automatic-run checkpoint fails once");
    assert_experiment_trial_cleanup(&bridge);

    let completed = bridge
        .advance_experiment(&context(), "checkpoint-retry")
        .expect("same deterministic trial retries");
    assert_eq!(completed.status, "completed");
    assert_experiment_trial_cleanup(&bridge);
}

#[test]
fn failed_experiment_replay_save_cleans_up_and_the_same_trial_retries() {
    let mut bridge = bridge_with_storages(
        Box::new(FailOnceReplayStorage::new()),
        Box::new(InMemorySessionRecoveryStorage::new()),
    );
    bridge
        .create_experiment(&context(), &experiment_request("replay-retry", vec![7]))
        .expect("matrix creates");

    bridge
        .advance_experiment(&context(), "replay-retry")
        .expect_err("replay save fails once");
    assert_experiment_trial_cleanup(&bridge);
    assert!(bridge
        .replays
        .list(&ReplayArchiveQuery::default())
        .expect("archive lists")
        .is_empty());

    let completed = bridge
        .advance_experiment(&context(), "replay-retry")
        .expect("same deterministic trial retries");
    assert_eq!(completed.status, "completed");
    assert_eq!(
        bridge
            .replays
            .list(&ReplayArchiveQuery::default())
            .expect("archive lists")
            .len(),
        1
    );
    assert_experiment_trial_cleanup(&bridge);
}

#[test]
fn failed_experiment_recovery_delete_preserves_replay_and_retries_idempotently() {
    let mut bridge = bridge_with_storages(
        Box::new(InMemoryReplayArchiveStorage::new()),
        Box::new(FailOnceDeleteRecoveryStorage::new()),
    );
    bridge
        .create_experiment(&context(), &experiment_request("delete-retry", vec![7]))
        .expect("matrix creates");

    bridge
        .advance_experiment(&context(), "delete-retry")
        .expect_err("recovery deletion fails once");
    assert_experiment_trial_cleanup(&bridge);
    assert_eq!(
        bridge
            .replays
            .list(&ReplayArchiveQuery::default())
            .expect("committed replay lists")
            .len(),
        1
    );

    let completed = bridge
        .advance_experiment(&context(), "delete-retry")
        .expect("same trial accepts its exact committed replay");
    assert_eq!(completed.status, "completed");
    assert_eq!(
        bridge
            .replays
            .list(&ReplayArchiveQuery::default())
            .expect("archive remains singular")
            .len(),
        1
    );
    assert_experiment_trial_cleanup(&bridge);
}

fn assert_experiment_trial_cleanup(bridge: &RulebenchBridge) {
    assert!(bridge.sessions.list_active_sessions().is_empty());
    assert!(bridge.recordings.is_empty());
    assert!(bridge
        .recovery
        .list()
        .expect("recovery storage lists")
        .is_empty());
}

#[test]
fn experiment_comparison_classifies_identical_and_divergent_evidence() {
    let mut bridge = bridge();
    bridge
        .create_experiment(&context(), &experiment_request("compare-a", vec![7]))
        .expect("first matrix");
    let first = bridge
        .advance_experiment(&context(), "compare-a")
        .expect("first trial");
    bridge
        .create_experiment(&context(), &experiment_request("compare-b", vec![7]))
        .expect("second matrix");
    let second = bridge
        .advance_experiment(&context(), "compare-b")
        .expect("second trial");
    let identical = bridge
        .compare_experiment_trials(
            &context(),
            &ExperimentComparisonRequestDto {
                expected_experiment_id: "compare-a".to_string(),
                expected_trial_id: first.trials[0].id.clone(),
                actual_experiment_id: "compare-b".to_string(),
                actual_trial_id: second.trials[0].id.clone(),
            },
        )
        .expect("identical trials compare");
    assert!(identical.identical);
    assert_eq!(identical.first_divergence_index, None);

    bridge
        .create_experiment(&context(), &experiment_request("compare-c", vec![11]))
        .expect("third matrix");
    let third = bridge
        .advance_experiment(&context(), "compare-c")
        .expect("third trial");
    let divergent = bridge
        .compare_experiment_trials(
            &context(),
            &ExperimentComparisonRequestDto {
                expected_experiment_id: "compare-a".to_string(),
                expected_trial_id: first.trials[0].id.clone(),
                actual_experiment_id: "compare-c".to_string(),
                actual_trial_id: third.trials[0].id.clone(),
            },
        )
        .expect("different seed trials compare");
    assert!(!divergent.identical);
    assert!(divergent.first_divergence_index.is_some());
}

#[test]
fn experiment_matrix_rejects_incompatible_policy_and_supports_cancellation() {
    let mut bridge = bridge();
    let mut incompatible = experiment_request("incompatible-lab", vec![7]);
    incompatible.policies[0] = policy(OBJECTIVE_SIDE_PRESSURE_POLICY_ID);
    let rejected = bridge
        .create_experiment(&context(), &incompatible)
        .expect_err("incompatible matrix fails before state creation");
    assert_eq!(rejected.kind, BridgeErrorKind::InvalidRequest);
    assert!(bridge
        .list_experiments(&context())
        .expect("experiment list")
        .is_empty());

    bridge
        .create_experiment(&context(), &experiment_request("cancelled-lab", vec![3, 5]))
        .expect("cancellable matrix");
    let cancelled = bridge
        .cancel_experiment(&context(), "cancelled-lab")
        .expect("matrix cancellation");
    assert_eq!(cancelled.status, "cancelled");
    let unchanged = bridge
        .advance_experiment(&context(), "cancelled-lab")
        .expect("cancelled matrix is stable");
    assert!(unchanged.trials.is_empty());
}

fn experiment_request(id: &str, seeds: Vec<u32>) -> ExperimentMatrixRequestDto {
    ExperimentMatrixRequestDto {
        id: id.to_string(),
        scenario_ids: vec!["hexing-bolt".to_string()],
        policies: vec![policy(FIRST_ACCEPTED_CANDIDATE_POLICY_ID)],
        seeds,
        max_steps: 8,
    }
}

fn policy(id: &str) -> CombatAutomationPolicyDto {
    CombatAutomationPolicyDto {
        id: id.to_string(),
        version: 1,
        no_candidate_behavior: CombatAutomationNoCandidateBehaviorDto::AdvanceTurn,
    }
}
