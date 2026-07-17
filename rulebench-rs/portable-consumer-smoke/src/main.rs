use rulebench_rules::*;

fn main() {
    let scenario = scenario();
    let providers = portable_provider_catalog(
        scenario
            .selected_ruleset()
            .expect("portable scenario selects a ruleset")
            .clone(),
    );
    providers
        .validate_ruleset(
            scenario
                .selected_ruleset()
                .expect("portable scenario selects a ruleset"),
        )
        .expect("portable provider accepts its exact ruleset definition");
    let replay_ruleset = scenario
        .selected_ruleset()
        .expect("portable scenario selects a replay ruleset")
        .artifact_provenance();
    let replay_package = record_replay_package(
        "portable-consumer-replay",
        CombatSessionCreateRequest::new("consumer-replay-session", scenario.clone()),
        replay_ruleset,
        vec![
            ReplayCommandRecordingSpec::new(
                "start",
                ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
            ),
            ReplayCommandRecordingSpec::new(
                "multi-target-action",
                ReplayCommand::Intent(CombatSessionIntentCommandSpec::new(
                    "consumer-replay-command",
                    "Consumer replay command",
                    "Portable replay records a multi-target action.",
                    UseActionIntent::for_targets(
                        "actor",
                        "bolt",
                        vec!["target-2".to_string(), "target".to_string()],
                    ),
                    vec![17, 5, 18, 4],
                )),
            ),
            ReplayCommandRecordingSpec::new(
                "end",
                ReplayCommand::Control(CombatControlCommandSpec::explicit_end()),
            ),
        ],
    );
    let replay_entry = ReplayArchiveEntry::new(replay_package, "portable-consumer");
    let canonical_bytes = canonical_replay_archive_payload(&replay_entry);
    assert!(!canonical_bytes.is_empty());
    assert_eq!(
        replay_entry.payload_encoding_version,
        REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION
    );
    assert!(replay_entry.is_self_consistent());

    let mut api = CombatSessionApi::new();
    let created = api
        .create_session(CombatSessionCreateRequest::new("consumer-session", scenario))
        .expect("consumer-authored scenario validates");
    let session = created.session;
    api.start_session(&session).expect("session starts");
    let step = api
        .submit_intent(
            &session,
            CombatSessionIntentCommandSpec::new(
                "consumer-command",
                "Consumer command",
                "External consumer submits one action.",
                UseActionIntent::for_targets(
                    "actor",
                    "bolt",
                    vec!["target-2".to_string(), "target".to_string()],
                ),
                vec![17, 5, 18, 4],
            ),
        )
        .expect("command is accepted");

    assert!(step.receipt.accepted);
    assert_eq!(step.receipt.target_results.len(), 2);
    assert_eq!(step.receipt.target_results[0].target_id, "target");
    assert!(!step.receipt.events.is_empty());
    let snapshot = api.snapshot(&session).expect("snapshot");
    assert_eq!(snapshot.combat_log.len(), 1);
    assert_eq!(snapshot.current_state.board.width, 6);
    assert_eq!(snapshot.current_state.combatants[0].position, GridPosition { x: 0, y: 0 });
}

fn portable_provider_catalog(ruleset: RulesetMetadata) -> RulesetProviderCatalog {
    RulesetProviderCatalog::new(vec![RulesetProviderDescriptor {
        provider_id: "provider.portable-consumer".to_string(),
        provider_version: "1".to_string(),
        ruleset,
        operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        effect_operation_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
        capabilities: vec![
            RulesetProviderCapability {
                id: "check.attackVsDefense".to_string(),
                version: "1".to_string(),
            },
            RulesetProviderCapability {
                id: "operation.applyModifier".to_string(),
                version: EffectOperationId::VOCABULARY_VERSION.to_string(),
            },
            RulesetProviderCapability {
                id: "operation.damage".to_string(),
                version: EffectOperationId::VOCABULARY_VERSION.to_string(),
            },
            RulesetProviderCapability {
                id: "targeting.multipleCombatants".to_string(),
                version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
            },
        ],
    }])
    .expect("portable provider metadata is compatible")
}

fn scenario() -> RulebenchScenario {
    let action = ActionDefinition {
        id: "bolt".to_string(),
        ruleset_id: "consumer-rules".to_string(),
        ability_id: "bolt-ability".to_string(),
        name: "Consumer Bolt".to_string(),
        actor_id: "actor".to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Multiple,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 10,
            visibility_requirement: VisibilityRequirement::Required,
            target_ids: vec!["target".to_string(), "target-2".to_string()],
            visible_target_ids: vec!["target".to_string(), "target-2".to_string()],
            operation_pipeline: Some(OperationPipelineV2 {
                maximum_targets: 2,
                area: None,
                roll_policy: ActionRollPolicy::PerTarget,
                failure_policy: TargetFailurePolicy::Atomic,
                target_order: TargetOrderPolicy::CanonicalId,
            }),
        },
        check: CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 4,
            modifier_stat_id: "mind".to_string(),
            defense: DefenseReference { id: "nerve".to_string(), label: "Nerve".to_string() },
        }),
        hit: HitEffect {
            damage_bonus: 4,
            damage_type: "psychic".to_string(),
            modifier_id: "rattled".to_string(),
            modifier_label: "Rattled".to_string(),
            modifier_duration: "one turn".to_string(),
            operations: vec![
                HitEffectOperation::Damage(DamageEffectOperation { damage_bonus: 4, damage_type: "psychic".to_string() }),
                HitEffectOperation::ApplyModifier(ModifierEffectOperation { modifier_id: "rattled".to_string(), modifier_label: "Rattled".to_string(), modifier_duration: "one turn".to_string() }),
            ],
        },
        resource_costs: vec![ActionResourceCost::standard_action()],
        movement: None,
        action_text: "Mind vs Nerve.".to_string(),
        effect_text: "Psychic damage and rattled.".to_string(),
    };
    RulebenchScenario {
        metadata: ScenarioMetadata { id: "consumer-scenario".to_string(), title: "Portable Consumer".to_string(), summary: "Standalone authority smoke.".to_string(), seed_label: "17,5".to_string() },
        content_pack_set: None,
        authored_action_binding: None,
        authored_scenario_binding: None,
        rulesets: vec![RulesetMetadata { id: "consumer-rules".to_string(), name: "Consumer Rules".to_string(), version: "0.1.0".to_string(), summary: "Standalone rules.".to_string(), modules: vec![RuleModuleDeclaration::action_resolution(ActionResolutionModuleConfiguration::declared_targets_and_line_of_sight())] }],
        selected_ruleset_id: "consumer-rules".to_string(),
        grid: Grid { width: 6, height: 1, cells: vec![
            GridCell { position: GridPosition { x: 0, y: 0 }, terrain_tags: vec!["clear".to_string()] },
            GridCell { position: GridPosition { x: 3, y: 0 }, terrain_tags: vec!["clear".to_string()] },
            GridCell { position: GridPosition { x: 5, y: 0 }, terrain_tags: vec!["clear".to_string()] },
        ] },
        combatants: vec![combatant("actor", "entity.actor", Team::Ally, 0, 20, 4, 15, true), combatant("target", "entity.target", Team::Enemy, 3, 18, 1, 13, false), combatant("target-2", "entity.target-2", Team::Enemy, 5, 16, 1, 12, false)],
        entities: vec![entity("entity.actor"), entity("entity.target"), entity("entity.target-2")],
        abilities: vec![AbilityDefinition { id: "bolt-ability".to_string(), name: "Consumer Bolt".to_string(), kind: AbilityDefinitionKind::Ability, summary: "Standalone action ability.".to_string(), tags: vec![] }],
        selected_ability_id: Some("bolt-ability".to_string()),
        classes: vec![], selected_class_id: None,
        stat_definitions: vec![StatDefinition { id: "mind".to_string(), label: "Mind".to_string(), kind: StatDefinitionKind::Base, formula: None, summary: "Attack stat.".to_string() }],
        modifiers: vec![ModifierDefinition { id: "rattled".to_string(), label: "Rattled".to_string(), summary: "Consumer modifier.".to_string(), default_tenure: ModifierTenure::Temporary, stacking_group: "rattled".to_string(), stacking_policy: ModifierStackingPolicy::Refresh, duration_policy: ModifierDurationPolicy::Turns(1), stat_adjustments: vec![] }],
        items: vec![], selected_item_id: None,
        actions: vec![action.clone()], selected_action: action,
    }
}

fn entity(id: &str) -> EntityDefinition { EntityDefinition { id: id.to_string(), name: id.to_string(), summary: id.to_string(), tags: vec![], damage_adjustments: vec![] } }

fn combatant(id: &str, entity_id: &str, team: Team, x: u32, hit_points: i32, mind: i32, nerve: i32, is_actor: bool) -> Combatant {
    let side_id = match team { Team::Ally => "ally", Team::Enemy => "enemy" };
    Combatant {
        id: id.to_string(), entity_id: entity_id.to_string(), name: id.to_string(), team, side_id: side_id.to_string(), initiative: 0, position: GridPosition { x, y: 0 }, hit_points: BoundedValue { current: hit_points, max: hit_points }, temporary_vitality: 0,
        class_inputs: vec![], stats: StatBlock { base_stats: vec![NamedNumber { id: "mind".to_string(), label: "Mind".to_string(), value: mind }], derived_stats: vec![] },
        defenses: vec![NamedNumber { id: "nerve".to_string(), label: "Nerve".to_string(), value: nerve }], resource_pools: vec![ActionResourcePool::standard_action()], inventory_item_ids: vec![], equipped_item_ids: vec![], base_ability_ids: if is_actor { vec!["bolt-ability".to_string()] } else { vec![] }, active_modifiers: vec![], conditions: vec![], is_actor,
    }
}
