use crate::model::*;
use crate::resolver::resolve_use_action;

pub fn hexing_bolt_fixture_scenario() -> RulebenchScenario {
    let selected_action = hexing_bolt_action();
    RulebenchScenario {
        metadata: ScenarioMetadata {
            id: "two-combatant-hexing-bolt".to_string(),
            title: "Hexing Bolt Opening".to_string(),
            summary: "A focused two-combatant fixture for proving board, event, trace, and final-state readouts.".to_string(),
            seed_label: "roll-stream:17,5".to_string(),
        },
        ruleset: hexing_bolt_ruleset(),
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
        items: hexing_bolt_items(),
        selected_item_id: Some("item.hex-focus".to_string()),
        actions: vec![selected_action.clone()],
        selected_action,
    }
}

fn hexing_bolt_ruleset() -> RulesetMetadata {
    RulesetMetadata {
        id: "asha-rulebench.hexing-bolt.v0".to_string(),
        name: "Hexing Bolt Fixture Rules".to_string(),
        version: "0.0.0".to_string(),
        summary: "Local single-action fixture ruleset for authority incubation.".to_string(),
    }
}

fn hexing_bolt_action() -> ActionDefinition {
    ActionDefinition {
        id: "hexing_bolt".to_string(),
        name: "Hexing Bolt".to_string(),
        actor_id: "entity-adept".to_string(),
        target_ids: vec!["entity-raider".to_string()],
        range: 10,
        line_of_sight_required: true,
        visible_target_ids: vec!["entity-raider".to_string()],
        attack: AttackSpec {
            modifier: 4,
            modifier_stat_id: "mind".to_string(),
            defense_id: "nerve".to_string(),
            defense_label: "Nerve".to_string(),
        },
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
        action_text: "Mind vs Nerve at range 10".to_string(),
        effect_text: "1d8 + Mind psychic damage and rattled until end of next turn on hit"
            .to_string(),
    }
}

fn hexing_bolt_items() -> Vec<ItemDefinition> {
    vec![
        ItemDefinition {
            id: "item.hex-focus".to_string(),
            name: "Hex Focus".to_string(),
            summary: "A small focus carried by the Adept; structural content only.".to_string(),
            tags: vec!["focus".to_string(), "implement".to_string()],
        },
        ItemDefinition {
            id: "item.raider-mail".to_string(),
            name: "Raider Mail".to_string(),
            summary: "Rough armor worn by the Raider; structural content only.".to_string(),
            tags: vec!["armor".to_string()],
        },
    ]
}

pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    )
}

pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        &[17, 5],
    )
}

fn adept_initial() -> Combatant {
    Combatant {
        id: "entity-adept".to_string(),
        name: "Adept".to_string(),
        team: Team::Ally,
        position: GridPosition { x: 1, y: 1 },
        hit_points: BoundedValue {
            current: 24,
            max: 24,
        },
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
            derived_stats: vec![NamedNumber {
                id: "initiative".to_string(),
                label: "Initiative".to_string(),
                value: 3,
            }],
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
        equipped_item_ids: vec!["item.hex-focus".to_string()],
        active_modifiers: Vec::new(),
        conditions: Vec::new(),
        is_actor: true,
    }
}

fn raider_initial() -> Combatant {
    Combatant {
        id: "entity-raider".to_string(),
        name: "Raider".to_string(),
        team: Team::Enemy,
        position: GridPosition { x: 4, y: 1 },
        hit_points: BoundedValue {
            current: 18,
            max: 18,
        },
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
            derived_stats: vec![NamedNumber {
                id: "initiative".to_string(),
                label: "Initiative".to_string(),
                value: 1,
            }],
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
        equipped_item_ids: vec!["item.raider-mail".to_string()],
        active_modifiers: Vec::new(),
        conditions: Vec::new(),
        is_actor: false,
    }
}
