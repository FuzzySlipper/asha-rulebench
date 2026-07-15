use rulebench_protocol::{
    CombatControlCommandDto, CombatControlCommandKindDto, CombatSessionCreateRequestDto,
    CombatSessionHandleDto, CombatSessionIntentCommandDto, CommandRollModeDto,
    ProtocolRequestContextDto, UseActionIntentDto, PROTOCOL_VERSION,
};
use rulebench_rules::*;

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
