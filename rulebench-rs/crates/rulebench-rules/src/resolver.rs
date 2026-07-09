use crate::model::*;
use crate::modifiers::effective_stats_for_combatant;
use crate::state::CombatState;

pub fn validate_intent_shape(intent: &UseActionIntent) -> RulebenchReceipt {
    let trace = vec![TraceEntry::new(
        1,
        TracePhase::Proposal,
        TraceStatus::Info,
        "UseActionIntent received.",
        "Structural intent validation started.",
    )];

    if intent.actor_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyActorId, trace);
    }
    if intent.action_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyActionId, trace);
    }
    if intent.target_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyTargetId, trace);
    }

    accepted_shape(intent.clone(), trace)
}

/// Resolve a single action against the supplied scenario.
///
/// The resolver is intentionally narrow and deterministic. It consumes a
/// scenario, a typed intent, and an explicit roll stream. It returns accepted
/// DomainEvents plus final projection, or a typed rejection with no accepted
/// events and unchanged projection.
pub fn resolve_use_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let trace = vec![TraceEntry::new(
        1,
        TracePhase::Proposal,
        TraceStatus::Info,
        "UseActionIntent received.",
        format!(
            "Actor {} proposed action {} against {}.",
            intent.actor_id, intent.action_id, intent.target_id
        ),
    )];

    if intent.actor_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyActorId,
            None,
            trace,
        );
    }
    if intent.action_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyActionId,
            None,
            trace,
        );
    }
    if intent.target_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyTargetId,
            None,
            trace,
        );
    }

    let Some(actor) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.actor_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidActor,
            None,
            trace,
        );
    };

    let Some(action) = scenario.action_by_id(&intent.action_id) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    };
    if action.actor_id != intent.actor_id {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    }

    let Some(attack_modifier) = attack_modifier(scenario, actor, action) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    };

    let Some(hit_operations) = hit_operations(action) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    };

    let Some(target) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.target_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidTarget,
            None,
            trace,
        );
    };

    let target_legality = validate_target_legality(actor, target, action);
    if !target_legality.accepted {
        let rejection = target_legality_rejection(&target_legality);
        return rejected_with_projection(scenario, intent, rejection, Some(target_legality), trace);
    }

    if roll_stream.is_empty() {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingAttackRoll,
            Some(target_legality),
            trace,
            vec![missing_roll_consumption(
                0,
                RollRequestKind::AttackRoll,
                "Attack roll was requested but no roll value was supplied.",
            )],
        );
    }
    if roll_stream.len() < 2 {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingDamageRoll,
            Some(target_legality),
            trace,
            vec![
                consumed_roll(
                    0,
                    RollRequestKind::AttackRoll,
                    roll_stream[0],
                    "Attack roll value was consumed for hit resolution.",
                ),
                missing_roll_consumption(
                    1,
                    RollRequestKind::DamageRoll,
                    "Damage roll was requested after a hit but no roll value was supplied.",
                ),
            ],
        );
    }

    resolve_accepted_action(
        scenario,
        intent,
        target,
        action,
        attack_modifier,
        hit_operations,
        target_legality,
        roll_stream,
    )
}

fn resolve_accepted_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target: &Combatant,
    action: &ActionDefinition,
    attack_modifier: i32,
    hit_operations: HitOperations<'_>,
    target_legality: TargetLegality,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let defense_value = defense_value(target, &action.attack.defense_id);
    let total = roll_stream[0] + attack_modifier;
    let attack_roll = AttackRollResult {
        roll: roll_stream[0],
        modifier: attack_modifier,
        total,
        defense_id: action.attack.defense_id.clone(),
        defense_value,
        outcome: if total >= defense_value {
            AttackOutcome::Hit
        } else {
            AttackOutcome::Miss
        },
    };

    let mut trace = vec![
        TraceEntry::new(
            1,
            TracePhase::Proposal,
            TraceStatus::Info,
            "UseActionIntent received.",
            format!(
                "Actor {} proposed action {} against {}.",
                intent.actor_id, intent.action_id, intent.target_id
            ),
        ),
        TraceEntry::new(
            2,
            TracePhase::Validation,
            TraceStatus::Accepted,
            "Target legality accepted.",
            target_legality.reason.clone(),
        ),
    ];

    if attack_roll.outcome == AttackOutcome::Miss {
        trace.push(TraceEntry::new(
            3,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Miss branch selected.",
            format!(
                "Roll stream supplied {}; total {} misses {} {}.",
                attack_roll.roll, attack_roll.total, action.attack.defense_label, defense_value
            ),
        ));
        trace.push(TraceEntry::new(
            4,
            TracePhase::Commit,
            TraceStatus::Accepted,
            "DomainEvents committed.",
            "ActionUsed and AttackRolled became accepted facts.",
        ));
        return accepted_miss_receipt(
            scenario,
            intent,
            target_legality,
            attack_roll,
            trace,
            vec![
                consumed_roll(
                    0,
                    RollRequestKind::AttackRoll,
                    roll_stream[0],
                    "Attack roll value was consumed for miss resolution.",
                ),
                unconsumed_roll(
                    1,
                    RollRequestKind::DamageRoll,
                    roll_stream.get(1).copied(),
                    "Damage roll value was supplied but not consumed because the attack missed.",
                ),
            ],
        );
    }

    let damage_roll = roll_stream[1];
    let damage = apply_damage(
        target,
        damage_roll + hit_operations.damage.damage_bonus,
        &hit_operations.damage.damage_type,
    );
    let modifier = ModifierOutcome {
        target_id: target.id.clone(),
        modifier_id: hit_operations.modifier.modifier_id.clone(),
        label: hit_operations.modifier.modifier_label.clone(),
        duration: hit_operations.modifier.modifier_duration.clone(),
    };

    trace.push(TraceEntry::new(
        3,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Hit branch selected.",
        format!(
            "Roll stream supplied {}; total {} beats {} {}.",
            attack_roll.roll, attack_roll.total, action.attack.defense_label, defense_value
        ),
    ));
    trace.push(TraceEntry::new(
        4,
        TracePhase::Commit,
        TraceStatus::Accepted,
        "DomainEvents committed.",
        "ActionUsed, AttackRolled, DamageApplied, and ModifierApplied became accepted facts.",
    ));

    accepted_hit_receipt(
        scenario,
        intent,
        target_legality,
        attack_roll,
        damage,
        modifier,
        trace,
        vec![
            consumed_roll(
                0,
                RollRequestKind::AttackRoll,
                roll_stream[0],
                "Attack roll value was consumed for hit resolution.",
            ),
            consumed_roll(
                1,
                RollRequestKind::DamageRoll,
                damage_roll,
                "Damage roll value was consumed for damage resolution.",
            ),
        ],
    )
}

fn accepted_shape(intent: UseActionIntent, mut trace: Vec<TraceEntry>) -> RulebenchReceipt {
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Accepted,
        "Intent shape accepted.",
        "Actor, action, and target ids are present.",
    ));
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        roll_consumption: Vec::new(),
        events: vec![DomainEvent::IntentShapeAccepted {
            actor_id: intent.actor_id,
            action_id: intent.action_id,
            target_id: intent.target_id,
        }],
        trace,
        projection: None,
    }
}

fn rejected(
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    mut trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Rejected,
        "Intent shape rejected.",
        rejection.code(),
    ));
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        roll_consumption: Vec::new(),
        events: Vec::new(),
        trace,
        projection: None,
    }
}

fn accepted_hit_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target_legality: TargetLegality,
    attack_roll: AttackRollResult,
    damage: DamageOutcome,
    modifier: ModifierOutcome,
    trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    let mut state = CombatState::from_scenario(scenario);
    state.apply_hit(&damage, &modifier);

    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: Some(attack_roll.clone()),
        damage: Some(damage.clone()),
        modifier: Some(modifier.clone()),
        roll_consumption,
        events: accepted_hit_events(&intent, &attack_roll, &damage, &modifier),
        trace,
        projection: Some(state.project("Raider is damaged and rattled; Adept is unchanged.")),
    }
}

fn accepted_miss_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target_legality: TargetLegality,
    attack_roll: AttackRollResult,
    trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: Some(attack_roll.clone()),
        damage: None,
        modifier: None,
        roll_consumption,
        events: vec![
            DomainEvent::ActionUsed {
                actor_id: intent.actor_id.clone(),
                action_id: intent.action_id.clone(),
                target_id: intent.target_id.clone(),
            },
            DomainEvent::AttackRolled {
                actor_id: intent.actor_id,
                target_id: intent.target_id,
                total: attack_roll.total,
                defense_id: attack_roll.defense_id,
                defense_value: attack_roll.defense_value,
                outcome: attack_roll.outcome,
            },
        ],
        trace,
        projection: Some(
            CombatState::from_scenario(scenario)
                .project("Attack missed; no authority state changed."),
        ),
    }
}

fn accepted_hit_events(
    intent: &UseActionIntent,
    attack_roll: &AttackRollResult,
    damage: &DamageOutcome,
    modifier: &ModifierOutcome,
) -> Vec<DomainEvent> {
    vec![
        DomainEvent::ActionUsed {
            actor_id: intent.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: intent.target_id.clone(),
        },
        DomainEvent::AttackRolled {
            actor_id: intent.actor_id.clone(),
            target_id: intent.target_id.clone(),
            total: attack_roll.total,
            defense_id: attack_roll.defense_id.clone(),
            defense_value: attack_roll.defense_value,
            outcome: attack_roll.outcome,
        },
        DomainEvent::DamageApplied {
            target_id: damage.target_id.clone(),
            amount: damage.amount,
            damage_type: damage.damage_type.clone(),
        },
        DomainEvent::ModifierApplied {
            target_id: modifier.target_id.clone(),
            modifier_id: modifier.modifier_id.clone(),
            duration: modifier.duration.clone(),
        },
    ]
}

fn rejected_with_projection(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    target_legality: Option<TargetLegality>,
    trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    rejected_with_projection_and_rolls(
        scenario,
        intent,
        rejection,
        target_legality,
        trace,
        Vec::new(),
    )
}

fn rejected_with_projection_and_rolls(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    target_legality: Option<TargetLegality>,
    mut trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    let detail = target_legality.as_ref().map_or_else(
        || rejection.code().to_string(),
        |legality| legality.reason.clone(),
    );
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Rejected,
        "Intent rejected.",
        detail,
    ));
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality,
        attack_roll: None,
        damage: None,
        modifier: None,
        roll_consumption,
        events: Vec::new(),
        trace,
        projection: Some(
            CombatState::from_scenario(scenario)
                .project("No authority state changed; intent rejected."),
        ),
    }
}

fn consumed_roll(
    sequence: u32,
    request_kind: RollRequestKind,
    supplied_value: i32,
    reason: impl Into<String>,
) -> RollConsumptionEntry {
    RollConsumptionEntry {
        sequence,
        request_kind,
        supplied_value: Some(supplied_value),
        consumed: true,
        reason: reason.into(),
    }
}

fn unconsumed_roll(
    sequence: u32,
    request_kind: RollRequestKind,
    supplied_value: Option<i32>,
    reason: impl Into<String>,
) -> RollConsumptionEntry {
    RollConsumptionEntry {
        sequence,
        request_kind,
        supplied_value,
        consumed: false,
        reason: reason.into(),
    }
}

fn missing_roll_consumption(
    sequence: u32,
    request_kind: RollRequestKind,
    reason: impl Into<String>,
) -> RollConsumptionEntry {
    unconsumed_roll(sequence, request_kind, None, reason)
}

pub(crate) fn validate_target_legality(
    actor: &Combatant,
    target: &Combatant,
    action: &ActionDefinition,
) -> TargetLegality {
    if actor.team == target.team {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is not hostile.".to_string(),
        };
    }
    if range_between(actor.position, target.position) > action.range {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is outside range.".to_string(),
        };
    }
    if action.line_of_sight_required && !action.visible_target_ids.contains(&target.id) {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Line of sight is blocked.".to_string(),
        };
    }
    TargetLegality {
        target_id: target.id.clone(),
        accepted: true,
        reason: "Target is hostile, within range, and line of sight is clear.".to_string(),
    }
}

pub(crate) fn target_legality_rejection(target_legality: &TargetLegality) -> RulebenchRejection {
    match target_legality.reason.as_str() {
        "Target is outside range." => RulebenchRejection::TargetOutOfRange,
        "Line of sight is blocked." => RulebenchRejection::TargetNotVisible,
        _ => RulebenchRejection::TargetLegalityFailed,
    }
}

fn range_between(from: GridPosition, to: GridPosition) -> u32 {
    from.x.abs_diff(to.x) + from.y.abs_diff(to.y)
}

fn defense_value(target: &Combatant, defense_id: &str) -> i32 {
    target
        .defenses
        .iter()
        .find(|defense| defense.id == defense_id)
        .map_or(0, |defense| defense.value)
}

fn attack_modifier(
    scenario: &RulebenchScenario,
    actor: &Combatant,
    action: &ActionDefinition,
) -> Option<i32> {
    effective_stats_for_combatant(scenario, &actor.id)?
        .stats
        .into_iter()
        .find(|stat| stat.stat_id == action.attack.modifier_stat_id)
        .map(|stat| stat.effective_value)
}

#[derive(Debug, Clone, Copy)]
struct HitOperations<'a> {
    damage: &'a DamageEffectOperation,
    modifier: &'a ModifierEffectOperation,
}

fn hit_operations(action: &ActionDefinition) -> Option<HitOperations<'_>> {
    Some(HitOperations {
        damage: action.hit.damage_operation()?,
        modifier: action.hit.modifier_operation()?,
    })
}

fn apply_damage(target: &Combatant, amount: i32, damage_type: &str) -> DamageOutcome {
    let before = target.hit_points;
    let next = before.current.saturating_sub(amount).max(0);
    DamageOutcome {
        target_id: target.id.clone(),
        damage_type: damage_type.to_string(),
        amount: before.current - next,
        before,
        after: BoundedValue {
            current: next,
            max: before.max,
        },
    }
}
