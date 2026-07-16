use crate::{
    FixtureGoldenArtifact, FixtureGoldenArtifactKind, FixtureGoldenManifest, ScenarioPackage,
    ScenarioPackageContentReference, ScenarioPackageDisplayMetadata,
    ScenarioPackageEvidenceExpectation, ScenarioPackageEvidenceKind, ScenarioPackageIdentity,
    ScenarioPackageInitialState, ScenarioPackageRulesetReference, ScenarioPackageScript,
};
use rulebench_rules::*;

pub fn hexing_bolt_fixture_scenario() -> RulebenchScenario {
    let selected_action = hexing_bolt_action();
    RulebenchScenario {
        metadata: ScenarioMetadata {
            id: "two-combatant-hexing-bolt".to_string(),
            title: "Hexing Bolt Opening".to_string(),
            summary: "A focused two-combatant fixture for proving board, event, trace, and final-state readouts.".to_string(),
            seed_label: "roll-stream:17,5".to_string(),
        },
        content_pack_set: None,
        authored_action_binding: None,
        rulesets: vec![crate::hexing_bolt_ruleset()],
        selected_ruleset_id: crate::HEXING_BOLT_RULESET_ID.to_string(),
        grid: Grid {
            width: 6,
            height: 4,
            cells: vec![
                GridCell {
                    position: GridPosition { x: 1, y: 1 },
                    terrain_tags: vec!["clear".to_string()],
                },
                GridCell {
                    position: GridPosition { x: 4, y: 1 },
                    terrain_tags: vec!["clear".to_string()],
                },
                GridCell {
                    position: GridPosition { x: 2, y: 2 },
                    terrain_tags: vec!["cover".to_string()],
                },
            ],
        },
        combatants: vec![adept_initial(), raider_initial()],
        entities: hexing_bolt_entities(),
        abilities: hexing_bolt_abilities(),
        selected_ability_id: Some("ability.hexing-bolt".to_string()),
        classes: hexing_bolt_classes(),
        selected_class_id: Some("class.hex-adept".to_string()),
        stat_definitions: hexing_bolt_stat_definitions(),
        modifiers: hexing_bolt_modifiers(),
        items: hexing_bolt_items(),
        selected_item_id: Some("item.hex-focus".to_string()),
        actions: vec![
            selected_action.clone(),
            move_action("entity-adept"),
            basic_attack_action(
                "entity-adept",
                "Focus Shot",
                "mind",
                4,
                "force",
                vec!["entity-raider"],
            ),
            move_action("entity-raider"),
            basic_attack_action(
                "entity-raider",
                "Raider Blade",
                "body",
                1,
                "slashing",
                vec!["entity-adept"],
            ),
        ],
        selected_action,
    }
}

pub fn turn_control_fixture_scenario() -> RulebenchScenario {
    let mut scenario = hexing_bolt_fixture_scenario();
    let ruleset = crate::turn_control_ruleset();
    let ruleset_id = ruleset.id.clone();
    scenario.rulesets[0] = ruleset;
    scenario.selected_ruleset_id = ruleset_id.clone();
    for action in &mut scenario.actions {
        action.ruleset_id = ruleset_id.clone();
    }
    scenario.selected_action.ruleset_id = ruleset_id;
    scenario
}

pub fn hexing_bolt_scenario_package() -> ScenarioPackage {
    let scenario = hexing_bolt_fixture_scenario();
    ScenarioPackage {
        identity: ScenarioPackageIdentity {
            id: "asha-rulebench.hexing-bolt".to_string(),
            version: "0.1.0".to_string(),
        },
        display: ScenarioPackageDisplayMetadata {
            title: "Hexing Bolt Package".to_string(),
            summary:
                "Rulebench-local scenario package for deterministic Hexing Bolt authority evidence."
                    .to_string(),
            tags: vec!["combat".to_string(), "hexing-bolt".to_string()],
        },
        ruleset: ScenarioPackageRulesetReference {
            id: scenario.selected_ruleset_id.clone(),
            version: scenario
                .selected_ruleset()
                .expect("Hexing Bolt scenario selects a declared ruleset")
                .version
                .clone(),
        },
        content_references: vec![ScenarioPackageContentReference {
            id: "asha-rulebench.hexing-bolt.content".to_string(),
            version: "0.1.0".to_string(),
        }],
        initial_state: ScenarioPackageInitialState {
            participant_ids: scenario
                .combatants
                .iter()
                .map(|combatant| combatant.id.clone())
                .collect(),
            scenario,
        },
        scripts: vec![ScenarioPackageScript {
            session_id: "hexing-bolt-mixed-control-script".to_string(),
            script: super::session::hexing_bolt_mixed_script_spec(),
        }],
        expected_evidence: vec![
            evidence("hexing-bolt-hit", ScenarioPackageEvidenceKind::CatalogCase),
            evidence(
                "hexing-bolt-reaction",
                ScenarioPackageEvidenceKind::CatalogCase,
            ),
            evidence("hexing-bolt-miss", ScenarioPackageEvidenceKind::CatalogCase),
            evidence(
                "hexing-bolt-self-target-rejected",
                ScenarioPackageEvidenceKind::CatalogCase,
            ),
            evidence(
                "hexing-bolt-opening-exchange",
                ScenarioPackageEvidenceKind::SessionTranscript,
            ),
            evidence(
                "hexing-bolt-control-sequence",
                ScenarioPackageEvidenceKind::ControlHistory,
            ),
            evidence(
                "hexing-bolt-mixed-control-script",
                ScenarioPackageEvidenceKind::Script,
            ),
            evidence(
                "hexing-bolt-bounded-automatic-run",
                ScenarioPackageEvidenceKind::AutomaticRun,
            ),
            evidence(
                "hexing-bolt-bounded-automatic-run-replay",
                ScenarioPackageEvidenceKind::ReplayVerification,
            ),
            evidence(
                "hexing-bolt-accepted-receipt",
                ScenarioPackageEvidenceKind::Receipt,
            ),
            evidence(
                "hexing-bolt-rejected-target-receipt",
                ScenarioPackageEvidenceKind::Receipt,
            ),
        ],
        golden_manifest: FixtureGoldenManifest {
            package_id: "asha-rulebench.hexing-bolt".to_string(),
            artifacts: hexing_bolt_golden_artifacts(),
        },
    }
}

fn hexing_bolt_golden_artifacts() -> Vec<FixtureGoldenArtifact> {
    vec![
        golden(
            "hexing-bolt-hit",
            FixtureGoldenArtifactKind::ScenarioCatalog,
            "pnpm run catalog:check",
        ),
        golden(
            "hexing-bolt-reaction",
            FixtureGoldenArtifactKind::ScenarioCatalog,
            "pnpm run catalog:check",
        ),
        golden(
            "hexing-bolt-miss",
            FixtureGoldenArtifactKind::ScenarioCatalog,
            "pnpm run catalog:check",
        ),
        golden(
            "hexing-bolt-self-target-rejected",
            FixtureGoldenArtifactKind::ScenarioCatalog,
            "pnpm run catalog:check",
        ),
        golden(
            "hexing-bolt-opening-exchange",
            FixtureGoldenArtifactKind::SessionTranscript,
            "pnpm run session:check",
        ),
        golden(
            "hexing-bolt-control-sequence",
            FixtureGoldenArtifactKind::ControlHistory,
            "pnpm run session:check",
        ),
        golden(
            "hexing-bolt-mixed-control-script",
            FixtureGoldenArtifactKind::ScriptReadout,
            "pnpm run session:check",
        ),
        golden(
            "hexing-bolt-bounded-automatic-run",
            FixtureGoldenArtifactKind::AutomaticRun,
            "pnpm run session:check",
        ),
        golden(
            "hexing-bolt-bounded-automatic-run-replay",
            FixtureGoldenArtifactKind::ReplayVerification,
            "pnpm run session:check",
        ),
        golden(
            "hexing-bolt-accepted-receipt",
            FixtureGoldenArtifactKind::Receipt,
            "cargo test --manifest-path rulebench-rs/Cargo.toml -p rulebench-fixtures",
        ),
        golden(
            "hexing-bolt-rejected-target-receipt",
            FixtureGoldenArtifactKind::Receipt,
            "cargo test --manifest-path rulebench-rs/Cargo.toml -p rulebench-fixtures",
        ),
    ]
}

fn golden(id: &str, kind: FixtureGoldenArtifactKind, check_command: &str) -> FixtureGoldenArtifact {
    FixtureGoldenArtifact {
        id: id.to_string(),
        kind,
        check_command: check_command.to_string(),
    }
}

fn evidence(id: &str, kind: ScenarioPackageEvidenceKind) -> ScenarioPackageEvidenceExpectation {
    ScenarioPackageEvidenceExpectation {
        id: id.to_string(),
        kind,
    }
}

fn hexing_bolt_action() -> ActionDefinition {
    ActionDefinition {
        id: "hexing_bolt".to_string(),
        ruleset_id: crate::HEXING_BOLT_RULESET_ID.to_string(),
        ability_id: "ability.hexing-bolt".to_string(),
        name: "Hexing Bolt".to_string(),
        actor_id: "entity-adept".to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 10,
            visibility_requirement: VisibilityRequirement::Required,
            target_ids: vec!["entity-raider".to_string()],
            visible_target_ids: vec!["entity-raider".to_string()],
            operation_pipeline: None,
        },
        check: CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 4,
            modifier_stat_id: "mind".to_string(),
            defense: DefenseReference {
                id: "nerve".to_string(),
                label: "Nerve".to_string(),
            },
        }),
        hit: HitEffect {
            damage_bonus: 4,
            damage_type: "psychic".to_string(),
            modifier_id: "rattled".to_string(),
            modifier_label: "rattled".to_string(),
            modifier_duration: "until end of next turn".to_string(),
            operations: vec![
                HitEffectOperation::Damage(DamageEffectOperation {
                    damage_bonus: 4,
                    damage_type: "psychic".to_string(),
                }),
                HitEffectOperation::ApplyModifier(ModifierEffectOperation {
                    modifier_id: "rattled".to_string(),
                    modifier_label: "rattled".to_string(),
                    modifier_duration: "until end of next turn".to_string(),
                }),
            ],
        },
        resource_costs: vec![ActionResourceCost::standard_action()],
        movement: None,
        action_text: "Mind vs Nerve at range 10".to_string(),
        effect_text: "1d8 + Mind psychic damage and rattled until end of next turn on hit"
            .to_string(),
    }
}

fn move_action(actor_id: &str) -> ActionDefinition {
    let mut action = hexing_bolt_action();
    action.id = format!("move.{actor_id}");
    action.ability_id = "ability.move".to_string();
    action.name = "Move".to_string();
    action.actor_id = actor_id.to_string();
    action.targeting = TargetingDeclaration {
        target_kind: TargetKind::Area,
        selection: TargetSelection::Single,
        team_constraint: TargetTeamConstraint::Any,
        maximum_range: 6,
        visibility_requirement: VisibilityRequirement::Ignored,
        target_ids: Vec::new(),
        visible_target_ids: Vec::new(),
        operation_pipeline: None,
    };
    action.resource_costs = Vec::new();
    action.movement = Some(MovementActionDeclaration {
        allowance: 6,
        topology: MovementTopology::OrthogonalManhattan,
        blocking_terrain_tags: vec!["wall".to_string(), "blocked".to_string()],
        difficult_terrain_tags: vec!["difficult".to_string()],
    });
    action.action_text = "Move to a legal orthogonal destination.".to_string();
    action.effect_text =
        "Spend movement equal to direct Manhattan distance plus destination terrain cost."
            .to_string();
    action
}

fn basic_attack_action(
    actor_id: &str,
    name: &str,
    attack_stat_id: &str,
    maximum_range: u32,
    damage_type: &str,
    target_ids: Vec<&str>,
) -> ActionDefinition {
    let targets = target_ids
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    ActionDefinition {
        id: format!("basic-attack.{actor_id}"),
        ruleset_id: crate::HEXING_BOLT_RULESET_ID.to_string(),
        ability_id: "ability.basic-attack".to_string(),
        name: name.to_string(),
        actor_id: actor_id.to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range,
            visibility_requirement: VisibilityRequirement::Required,
            target_ids: targets.clone(),
            visible_target_ids: targets,
            operation_pipeline: None,
        },
        check: CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 0,
            modifier_stat_id: attack_stat_id.to_string(),
            defense: DefenseReference {
                id: "guard".to_string(),
                label: "Guard".to_string(),
            },
        }),
        hit: HitEffect {
            damage_bonus: 0,
            damage_type: damage_type.to_string(),
            modifier_id: String::new(),
            modifier_label: String::new(),
            modifier_duration: String::new(),
            operations: vec![HitEffectOperation::Damage(DamageEffectOperation {
                damage_bonus: 0,
                damage_type: damage_type.to_string(),
            })],
        },
        resource_costs: vec![ActionResourceCost::standard_action()],
        movement: None,
        action_text: format!("{attack_stat_id} vs Guard at range {maximum_range}."),
        effect_text: format!("1d8 {damage_type} damage on hit."),
    }
}

fn hexing_bolt_abilities() -> Vec<AbilityDefinition> {
    vec![
        AbilityDefinition {
            id: "ability.hexing-bolt".to_string(),
            name: "Hexing Bolt".to_string(),
            kind: AbilityDefinitionKind::Spell,
            summary: "A focused spell entry that owns the Hexing Bolt action content.".to_string(),
            tags: vec![
                "spell".to_string(),
                "attack".to_string(),
                "psychic".to_string(),
            ],
        },
        AbilityDefinition {
            id: "ability.move".to_string(),
            name: "Move".to_string(),
            kind: AbilityDefinitionKind::Ability,
            summary: "Content-declared movement behavior.".to_string(),
            tags: vec!["movement".to_string()],
        },
        AbilityDefinition {
            id: "ability.basic-attack".to_string(),
            name: "Basic Attack".to_string(),
            kind: AbilityDefinitionKind::Ability,
            summary: "Content-defined ordinary weapon attack behavior.".to_string(),
            tags: vec!["attack".to_string(), "weapon".to_string()],
        },
    ]
}

fn hexing_bolt_entities() -> Vec<EntityDefinition> {
    vec![
        EntityDefinition {
            id: "entity.adept".to_string(),
            name: "Adept".to_string(),
            summary: "A focused caster entity used as the Hexing Bolt actor.".to_string(),
            tags: vec!["ally".to_string(), "caster".to_string()],
            damage_adjustments: Vec::new(),
        },
        EntityDefinition {
            id: "entity.raider".to_string(),
            name: "Raider".to_string(),
            summary: "A hostile raider entity used as the Hexing Bolt target.".to_string(),
            tags: vec!["enemy".to_string(), "skirmisher".to_string()],
            damage_adjustments: Vec::new(),
        },
    ]
}

fn hexing_bolt_items() -> Vec<ItemDefinition> {
    vec![
        ItemDefinition {
            id: "item.hex-focus".to_string(),
            name: "Hex Focus".to_string(),
            summary: "A small focus that supports the Adept's hexing magic.".to_string(),
            tags: vec!["focus".to_string(), "implement".to_string()],
            equipment_slot: "implement".to_string(),
            requirements: vec![StatRequirement {
                stat_id: "mind".to_string(),
                minimum: 3,
            }],
            granted_modifier_ids: Vec::new(),
            granted_ability_ids: Vec::new(),
            granted_resource_pools: Vec::new(),
        },
        ItemDefinition {
            id: "item.raider-mail".to_string(),
            name: "Raider Mail".to_string(),
            summary: "Rough armor worn by the Raider.".to_string(),
            tags: vec!["armor".to_string()],
            equipment_slot: "armor".to_string(),
            requirements: vec![StatRequirement {
                stat_id: "body".to_string(),
                minimum: 2,
            }],
            granted_modifier_ids: Vec::new(),
            granted_ability_ids: Vec::new(),
            granted_resource_pools: Vec::new(),
        },
    ]
}

fn hexing_bolt_classes() -> Vec<ClassDefinition> {
    vec![
        ClassDefinition {
            id: "class.hex-adept".to_string(),
            name: "Hex Adept".to_string(),
            version: "1.0.0".to_string(),
            summary: "A focused caster class that grants Hexing Bolt.".to_string(),
            tags: vec!["caster".to_string()],
            prerequisites: vec![StatRequirement {
                stat_id: "mind".to_string(),
                minimum: 3,
            }],
            level_grants: vec![ClassLevelGrant {
                level: 1,
                granted_modifier_ids: Vec::new(),
                granted_ability_ids: vec!["ability.hexing-bolt".to_string()],
                granted_resource_pools: Vec::new(),
            }],
        },
        ClassDefinition {
            id: "class.raider".to_string(),
            name: "Raider".to_string(),
            version: "1.0.0".to_string(),
            summary: "A hostile skirmisher class input.".to_string(),
            tags: vec!["martial".to_string()],
            prerequisites: Vec::new(),
            level_grants: Vec::new(),
        },
    ]
}

fn hexing_bolt_stat_definitions() -> Vec<StatDefinition> {
    vec![
        StatDefinition {
            id: "mind".to_string(),
            label: "Mind".to_string(),
            kind: StatDefinitionKind::Base,
            formula: None,
            summary: "Mental force used by Hexing Bolt attack rolls.".to_string(),
        },
        StatDefinition {
            id: "body".to_string(),
            label: "Body".to_string(),
            kind: StatDefinitionKind::Base,
            formula: None,
            summary: "Physical force for future melee and durability checks.".to_string(),
        },
        StatDefinition {
            id: "initiative".to_string(),
            label: "Initiative".to_string(),
            kind: StatDefinitionKind::Derived,
            formula: Some(DerivedStatFormula::Difference {
                minuend: Box::new(DerivedStatFormula::Sum {
                    operands: vec![
                        DerivedStatFormula::StatReference {
                            stat_id: "mind".to_string(),
                        },
                        DerivedStatFormula::StatReference {
                            stat_id: "body".to_string(),
                        },
                    ],
                }),
                subtrahend: Box::new(DerivedStatFormula::Constant { value: 3 }),
            }),
            summary: "Turn-order readiness derived from mind and body.".to_string(),
        },
    ]
}

fn hexing_bolt_modifiers() -> Vec<ModifierDefinition> {
    vec![
        ModifierDefinition {
            id: "rattled".to_string(),
            label: "rattled".to_string(),
            summary: "A temporary condition-like modifier applied by Hexing Bolt.".to_string(),
            default_tenure: ModifierTenure::Temporary,
            stacking_group: "rattled".to_string(),
            stacking_policy: ModifierStackingPolicy::Refresh,
            duration_policy: ModifierDurationPolicy::Turns(1),
            stat_adjustments: vec![ModifierStatAdjustment {
                stat_id: "mind".to_string(),
                stat_label: "Mind".to_string(),
                delta: -1,
            }],
        },
        ModifierDefinition {
            id: "battle-drilled".to_string(),
            label: "battle drilled".to_string(),
            summary: "A permanent training marker for stat-adjustment readouts.".to_string(),
            default_tenure: ModifierTenure::Permanent,
            stacking_group: "battle-drilled".to_string(),
            stacking_policy: ModifierStackingPolicy::Replace,
            duration_policy: ModifierDurationPolicy::Permanent,
            stat_adjustments: vec![ModifierStatAdjustment {
                stat_id: "initiative".to_string(),
                stat_label: "Initiative".to_string(),
                delta: 1,
            }],
        },
    ]
}

pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    rulebench_rules::resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    )
}

pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    rulebench_rules::resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        &[17, 5],
    )
}

fn adept_initial() -> Combatant {
    Combatant {
        id: "entity-adept".to_string(),
        entity_id: "entity.adept".to_string(),
        name: "Adept".to_string(),
        team: Team::Ally,
        side_id: "ally".to_string(),
        initiative: 15,
        position: GridPosition { x: 1, y: 1 },
        hit_points: BoundedValue {
            current: 24,
            max: 24,
        },
        temporary_vitality: 0,
        class_inputs: vec![ClassLevelInput {
            class_id: "class.hex-adept".to_string(),
            version: "1.0.0".to_string(),
            level: 1,
        }],
        stats: StatBlock {
            base_stats: vec![
                NamedNumber {
                    id: "mind".to_string(),
                    label: "Mind".to_string(),
                    value: 4,
                },
                NamedNumber {
                    id: "body".to_string(),
                    label: "Body".to_string(),
                    value: 2,
                },
            ],
            derived_stats: Vec::new(),
        },
        defenses: vec![
            NamedNumber {
                id: "guard".to_string(),
                label: "Guard".to_string(),
                value: 16,
            },
            NamedNumber {
                id: "nerve".to_string(),
                label: "Nerve".to_string(),
                value: 15,
            },
        ],
        resource_pools: vec![ActionResourcePool::standard_action()],
        inventory_item_ids: vec!["item.hex-focus".to_string()],
        equipped_item_ids: vec!["item.hex-focus".to_string()],
        base_ability_ids: vec![
            "ability.move".to_string(),
            "ability.basic-attack".to_string(),
        ],
        active_modifiers: Vec::new(),
        conditions: Vec::new(),
        is_actor: true,
    }
}

fn raider_initial() -> Combatant {
    Combatant {
        id: "entity-raider".to_string(),
        entity_id: "entity.raider".to_string(),
        name: "Raider".to_string(),
        team: Team::Enemy,
        side_id: "enemy".to_string(),
        initiative: 10,
        position: GridPosition { x: 4, y: 1 },
        hit_points: BoundedValue {
            current: 18,
            max: 18,
        },
        temporary_vitality: 0,
        class_inputs: vec![ClassLevelInput {
            class_id: "class.raider".to_string(),
            version: "1.0.0".to_string(),
            level: 1,
        }],
        stats: StatBlock {
            base_stats: vec![
                NamedNumber {
                    id: "mind".to_string(),
                    label: "Mind".to_string(),
                    value: 1,
                },
                NamedNumber {
                    id: "body".to_string(),
                    label: "Body".to_string(),
                    value: 3,
                },
            ],
            derived_stats: Vec::new(),
        },
        defenses: vec![
            NamedNumber {
                id: "guard".to_string(),
                label: "Guard".to_string(),
                value: 14,
            },
            NamedNumber {
                id: "nerve".to_string(),
                label: "Nerve".to_string(),
                value: 13,
            },
        ],
        resource_pools: vec![ActionResourcePool::standard_action()],
        inventory_item_ids: vec!["item.raider-mail".to_string()],
        equipped_item_ids: vec!["item.raider-mail".to_string()],
        base_ability_ids: vec![
            "ability.move".to_string(),
            "ability.basic-attack".to_string(),
        ],
        active_modifiers: Vec::new(),
        conditions: Vec::new(),
        is_actor: false,
    }
}
