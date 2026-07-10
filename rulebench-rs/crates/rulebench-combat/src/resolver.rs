//! Deterministic authoritative action resolution.

use crate::model::*;
use crate::modifiers::effective_stats_for_combatant;
use crate::state::CombatState;

struct CheckResolutionContext<'a> {
    scenario: &'a RulebenchScenario,
    intent: UseActionIntent,
    actor: &'a Combatant,
    action: &'a ActionDefinition,
    action_resolution: &'a ActionResolutionModuleConfiguration,
    roll_stream: &'a [i32],
    trace: Vec<TraceEntry>,
}

struct CheckEffectResolution<'a> {
    scenario: &'a RulebenchScenario,
    intent: UseActionIntent,
    target: &'a Combatant,
    target_legality: TargetLegality,
    check_event: DomainEvent,
    hit_operations: HitOperations<'a>,
    damage_roll: i32,
    trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
}

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

    let Some(ruleset) = scenario.selected_ruleset() else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidRulesetModules,
            None,
            trace,
        );
    };
    let Ok(module_registry) = ruleset.validate_modules() else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidRulesetModules,
            None,
            trace,
        );
    };
    let action_resolution = module_registry.action_resolution();

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
    if action.ruleset_id != scenario.selected_ruleset_id {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    }
    if action.actor_id != intent.actor_id {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    }
    if !action_resolution.supports_check(&action.check) {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    }

    match &action.check {
        CheckDeclaration::SavingThrow(save) => {
            return resolve_saving_throw_action(
                CheckResolutionContext {
                    scenario,
                    intent,
                    actor,
                    action,
                    action_resolution,
                    roll_stream,
                    trace,
                },
                save,
            );
        }
        CheckDeclaration::Contested(contested) => {
            return resolve_contested_action(
                CheckResolutionContext {
                    scenario,
                    intent,
                    actor,
                    action,
                    action_resolution,
                    roll_stream,
                    trace,
                },
                contested,
            );
        }
        CheckDeclaration::Attack(_) => {}
    }

    let Some(attack) = action.attack_check() else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    };

    let Some(attack_modifier) = attack_modifier(scenario, actor, attack) else {
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

    let target_legality =
        validate_target_legality_for_module(actor, target, action, action_resolution);
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
        attack,
        attack_modifier,
        hit_operations,
        target_legality,
        roll_stream,
    )
}

fn resolve_saving_throw_action(
    context: CheckResolutionContext<'_>,
    save: &SavingThrowCheckDeclaration,
) -> RulebenchReceipt {
    let CheckResolutionContext {
        scenario,
        intent,
        actor,
        action,
        action_resolution,
        roll_stream,
        mut trace,
    } = context;
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
    let target_legality =
        validate_target_legality_for_module(actor, target, action, action_resolution);
    if !target_legality.accepted {
        return rejected_with_projection(
            scenario,
            intent,
            target_legality_rejection(&target_legality),
            Some(target_legality),
            trace,
        );
    }
    let Some(modifier) = effective_stat_value(scenario, &target.id, &save.save_stat_id) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    let Some(hit_operations) = hit_operations(action) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    let Some(save_roll) = roll_stream.first().copied() else {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingCheckRoll,
            Some(target_legality),
            trace,
            vec![missing_roll_consumption(
                0,
                RollRequestKind::SavingThrowRoll,
                "Saving throw roll was requested but no roll value was supplied.",
            )],
        );
    };
    let total = save_roll + modifier;
    let outcome = if total >= save.difficulty_class {
        SavingThrowOutcome::Saved
    } else {
        SavingThrowOutcome::Failed
    };
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Validation,
        TraceStatus::Accepted,
        "Target legality accepted.",
        target_legality.reason.clone(),
    ));
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Saving throw resolved.",
        format!(
            "Target roll {} plus {} equals {} against DC {}; ties save.",
            save_roll, modifier, total, save.difficulty_class
        ),
    ));

    let event = DomainEvent::SavingThrowResolved {
        actor_id: intent.actor_id.clone(),
        target_id: intent.target_id.clone(),
        total,
        difficulty_class: save.difficulty_class,
        outcome,
    };
    if outcome == SavingThrowOutcome::Saved {
        trace.push(TraceEntry::new(
            trace.len() as u32 + 1,
            TracePhase::Commit,
            TraceStatus::Accepted,
            "DomainEvents committed.",
            "ActionUsed and SavingThrowResolved became accepted facts; effects were avoided.",
        ));
        return accepted_non_effect_receipt(
            scenario,
            intent,
            target_legality,
            event,
            trace,
            vec![consumed_roll(
                0,
                RollRequestKind::SavingThrowRoll,
                save_roll,
                "Saving throw roll value was consumed for save resolution.",
            )],
        );
    }

    let Some(damage_roll) = roll_stream.get(1).copied() else {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingDamageRoll,
            Some(target_legality),
            trace,
            vec![
                consumed_roll(
                    0,
                    RollRequestKind::SavingThrowRoll,
                    save_roll,
                    "Saving throw roll value was consumed for save resolution.",
                ),
                missing_roll_consumption(
                    1,
                    RollRequestKind::DamageRoll,
                    "Damage roll was requested after a failed saving throw but no roll value was supplied.",
                ),
            ],
        );
    };
    resolve_check_effects(CheckEffectResolution {
        scenario,
        intent,
        target,
        target_legality,
        check_event: event,
        hit_operations,
        damage_roll,
        trace,
        roll_consumption: vec![
            consumed_roll(
                0,
                RollRequestKind::SavingThrowRoll,
                save_roll,
                "Saving throw roll value was consumed for save resolution.",
            ),
            consumed_roll(
                1,
                RollRequestKind::DamageRoll,
                damage_roll,
                "Damage roll value was consumed after a failed saving throw.",
            ),
        ],
    })
}

fn resolve_contested_action(
    context: CheckResolutionContext<'_>,
    contested: &ContestedCheckDeclaration,
) -> RulebenchReceipt {
    let CheckResolutionContext {
        scenario,
        intent,
        actor,
        action,
        action_resolution,
        roll_stream,
        mut trace,
    } = context;
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
    let target_legality =
        validate_target_legality_for_module(actor, target, action, action_resolution);
    if !target_legality.accepted {
        return rejected_with_projection(
            scenario,
            intent,
            target_legality_rejection(&target_legality),
            Some(target_legality),
            trace,
        );
    }
    let Some(actor_modifier) = effective_stat_value(scenario, &actor.id, &contested.actor_stat_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    let Some(target_modifier) =
        effective_stat_value(scenario, &target.id, &contested.target_stat_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    let Some(hit_operations) = hit_operations(action) else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    let Some(actor_roll) = roll_stream.first().copied() else {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingCheckRoll,
            Some(target_legality),
            trace,
            vec![missing_roll_consumption(
                0,
                RollRequestKind::ContestedActorRoll,
                "Contested actor roll was requested but no roll value was supplied.",
            )],
        );
    };
    let Some(target_roll) = roll_stream.get(1).copied() else {
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingCheckRoll,
            Some(target_legality),
            trace,
            vec![
                consumed_roll(
                    0,
                    RollRequestKind::ContestedActorRoll,
                    actor_roll,
                    "Contested actor roll value was consumed.",
                ),
                missing_roll_consumption(
                    1,
                    RollRequestKind::ContestedTargetRoll,
                    "Contested target roll was requested but no roll value was supplied.",
                ),
            ],
        );
    };
    let actor_total = actor_roll + actor_modifier;
    let target_total = target_roll + target_modifier;
    let outcome = if actor_total > target_total {
        ContestedCheckOutcome::ActorWins
    } else {
        ContestedCheckOutcome::TargetWins
    };
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Validation,
        TraceStatus::Accepted,
        "Target legality accepted.",
        target_legality.reason.clone(),
    ));
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Contested check resolved.",
        format!(
            "Actor total {} versus target total {}; ties favor the target.",
            actor_total, target_total
        ),
    ));
    let event = DomainEvent::ContestedCheckResolved {
        actor_id: intent.actor_id.clone(),
        target_id: intent.target_id.clone(),
        actor_total,
        target_total,
        outcome,
    };
    let contested_rolls = vec![
        consumed_roll(
            0,
            RollRequestKind::ContestedActorRoll,
            actor_roll,
            "Contested actor roll value was consumed.",
        ),
        consumed_roll(
            1,
            RollRequestKind::ContestedTargetRoll,
            target_roll,
            "Contested target roll value was consumed.",
        ),
    ];
    if outcome == ContestedCheckOutcome::TargetWins {
        trace.push(TraceEntry::new(
            trace.len() as u32 + 1,
            TracePhase::Commit,
            TraceStatus::Accepted,
            "DomainEvents committed.",
            "ActionUsed and ContestedCheckResolved became accepted facts; effects were avoided.",
        ));
        return accepted_non_effect_receipt(
            scenario,
            intent,
            target_legality,
            event,
            trace,
            contested_rolls,
        );
    }
    let Some(damage_roll) = roll_stream.get(2).copied() else {
        let mut rolls = contested_rolls;
        rolls.push(missing_roll_consumption(
            2,
            RollRequestKind::DamageRoll,
            "Damage roll was requested after a winning contested check but no roll value was supplied.",
        ));
        return rejected_with_projection_and_rolls(
            scenario,
            intent,
            RulebenchRejection::MissingDamageRoll,
            Some(target_legality),
            trace,
            rolls,
        );
    };
    let mut rolls = contested_rolls;
    rolls.push(consumed_roll(
        2,
        RollRequestKind::DamageRoll,
        damage_roll,
        "Damage roll value was consumed after a winning contested check.",
    ));
    resolve_check_effects(CheckEffectResolution {
        scenario,
        intent,
        target,
        target_legality,
        check_event: event,
        hit_operations,
        damage_roll,
        trace,
        roll_consumption: rolls,
    })
}

fn validate_target_legality_for_module(
    actor: &Combatant,
    target: &Combatant,
    action: &ActionDefinition,
    configuration: &ActionResolutionModuleConfiguration,
) -> TargetLegality {
    match configuration.targeting_policy {
        ActionResolutionTargetingPolicy::DeclaredTargetsAndLineOfSight => {
            validate_target_legality(actor, target, action)
        }
    }
}

fn resolve_accepted_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target: &Combatant,
    attack: &AttackCheckDeclaration,
    attack_modifier: i32,
    hit_operations: HitOperations<'_>,
    target_legality: TargetLegality,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let defense_value = defense_value(target, &attack.defense.id);
    let total = roll_stream[0] + attack_modifier;
    let attack_roll = AttackRollResult {
        roll: roll_stream[0],
        modifier: attack_modifier,
        total,
        defense_id: attack.defense.id.clone(),
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

    if scenario
        .stat_definition_by_id(&attack.modifier_stat_id)
        .is_some_and(|definition| definition.kind == StatDefinitionKind::Derived)
    {
        if let Some(readout) =
            effective_stats_for_combatant(scenario, &intent.actor_id).and_then(|readout| {
                readout
                    .stats
                    .into_iter()
                    .find(|stat| stat.stat_id == attack.modifier_stat_id)
            })
        {
            trace.push(TraceEntry::new(
                3,
                TracePhase::Resolution,
                TraceStatus::Info,
                "Derived attack stat evaluated.",
                format!(
                    "{} formula produced effective value {} for attack resolution.",
                    readout
                        .formula
                        .as_ref()
                        .map_or("derived", DerivedStatFormula::code),
                    readout.effective_value
                ),
            ));
        }
    }

    let resolution_sequence = trace.len() as u32 + 1;

    if attack_roll.outcome == AttackOutcome::Miss {
        trace.push(TraceEntry::new(
            resolution_sequence,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Miss branch selected.",
            format!(
                "Roll stream supplied {}; total {} misses {} {}.",
                attack_roll.roll, attack_roll.total, attack.defense.label, defense_value
            ),
        ));
        trace.push(TraceEntry::new(
            resolution_sequence + 1,
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
    let vitality_effects = apply_vitality_effects(scenario, target, damage_roll, hit_operations);
    let damage = vitality_effects.damage.clone();
    let Some(modifier) =
        modifier_outcome(scenario, target, &intent.action_id, hit_operations.modifier)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };

    trace.push(TraceEntry::new(
        resolution_sequence,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Hit branch selected.",
        format!(
            "Roll stream supplied {}; total {} beats {} {}.",
            attack_roll.roll, attack_roll.total, attack.defense.label, defense_value
        ),
    ));
    append_vitality_trace(&mut trace, &vitality_effects);
    trace.push(TraceEntry::new(
        resolution_sequence + 1,
        TracePhase::Commit,
        TraceStatus::Accepted,
        "DomainEvents committed.",
        "ActionUsed, AttackRolled, vitality effects, and ModifierApplied became accepted facts.",
    ));

    accepted_hit_receipt(
        scenario,
        intent,
        target_legality,
        attack_roll,
        damage,
        vitality_effects.healing,
        vitality_effects.temporary_vitality,
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

fn effective_stat_value(
    scenario: &RulebenchScenario,
    combatant_id: &str,
    stat_id: &str,
) -> Option<i32> {
    effective_stats_for_combatant(scenario, combatant_id)?
        .stats
        .into_iter()
        .find(|stat| stat.stat_id == stat_id)
        .map(|stat| stat.effective_value)
}

fn accepted_non_effect_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target_legality: TargetLegality,
    check_event: DomainEvent,
    trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: None,
        damage: None,
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption,
        events: vec![
            DomainEvent::ActionUsed {
                actor_id: intent.actor_id,
                action_id: intent.action_id,
                target_id: intent.target_id,
            },
            check_event,
        ],
        trace,
        projection: Some(
            CombatState::from_scenario(scenario)
                .project("Check prevented effects; no authority state changed."),
        ),
    }
}

fn resolve_check_effects(resolution: CheckEffectResolution<'_>) -> RulebenchReceipt {
    let CheckEffectResolution {
        scenario,
        intent,
        target,
        target_legality,
        check_event,
        hit_operations,
        damage_roll,
        mut trace,
        roll_consumption,
    } = resolution;
    let vitality_effects = apply_vitality_effects(scenario, target, damage_roll, hit_operations);
    let damage = vitality_effects.damage.clone();
    let Some(modifier) =
        modifier_outcome(scenario, target, &intent.action_id, hit_operations.modifier)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            Some(target_legality),
            trace,
        );
    };
    append_vitality_trace(&mut trace, &vitality_effects);
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Commit,
        TraceStatus::Accepted,
        "DomainEvents committed.",
        "ActionUsed, check resolution, vitality effects, and ModifierApplied became accepted facts.",
    ));
    let mut state = CombatState::from_scenario(scenario);
    state.apply_hit(&damage, &modifier);
    if let Some(healing) = &vitality_effects.healing {
        state.apply_healing(healing);
    }
    if let Some(temporary_vitality) = &vitality_effects.temporary_vitality {
        state.apply_temporary_vitality(temporary_vitality);
    }

    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: None,
        damage: Some(damage.clone()),
        healing: vitality_effects.healing.clone(),
        temporary_vitality: vitality_effects.temporary_vitality.clone(),
        modifier: Some(modifier.clone()),
        roll_consumption,
        events: accepted_check_effect_events(
            &intent,
            check_event,
            &damage,
            vitality_effects.healing.as_ref(),
            vitality_effects.temporary_vitality.as_ref(),
            &modifier,
        ),
        trace,
        projection: Some(state.project("Check failed and effects were applied.")),
    }
}

fn accepted_check_effect_events(
    intent: &UseActionIntent,
    check_event: DomainEvent,
    damage: &DamageOutcome,
    healing: Option<&HealingOutcome>,
    temporary_vitality: Option<&TemporaryVitalityOutcome>,
    modifier: &ModifierOutcome,
) -> Vec<DomainEvent> {
    let mut events = vec![
        DomainEvent::ActionUsed {
            actor_id: intent.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: intent.target_id.clone(),
        },
        check_event,
        DomainEvent::DamageApplied {
            target_id: damage.target_id.clone(),
            amount: damage.amount,
            damage_type: damage.damage_type.clone(),
        },
    ];
    if let Some(healing) = healing {
        events.push(DomainEvent::HealingApplied {
            target_id: healing.target_id.clone(),
            amount: healing.amount,
            healing_type: healing.healing_type.clone(),
        });
    }
    if let Some(temporary_vitality) = temporary_vitality {
        events.push(DomainEvent::TemporaryVitalityGranted {
            target_id: temporary_vitality.target_id.clone(),
            amount: temporary_vitality.after - temporary_vitality.before,
        });
    }
    events.push(DomainEvent::ModifierApplied {
        target_id: modifier.target_id.clone(),
        modifier_id: modifier.modifier_id.clone(),
        duration: modifier.duration.clone(),
    });
    events
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
        healing: None,
        temporary_vitality: None,
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
        healing: None,
        temporary_vitality: None,
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
    healing: Option<HealingOutcome>,
    temporary_vitality: Option<TemporaryVitalityOutcome>,
    modifier: ModifierOutcome,
    trace: Vec<TraceEntry>,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    let mut state = CombatState::from_scenario(scenario);
    state.apply_hit(&damage, &modifier);
    if let Some(healing) = &healing {
        state.apply_healing(healing);
    }
    if let Some(temporary_vitality) = &temporary_vitality {
        state.apply_temporary_vitality(temporary_vitality);
    }

    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: Some(attack_roll.clone()),
        damage: Some(damage.clone()),
        healing: healing.clone(),
        temporary_vitality: temporary_vitality.clone(),
        modifier: Some(modifier.clone()),
        roll_consumption,
        events: accepted_hit_events(
            &intent,
            &attack_roll,
            &damage,
            healing.as_ref(),
            temporary_vitality.as_ref(),
            &modifier,
        ),
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
        healing: None,
        temporary_vitality: None,
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
    healing: Option<&HealingOutcome>,
    temporary_vitality: Option<&TemporaryVitalityOutcome>,
    modifier: &ModifierOutcome,
) -> Vec<DomainEvent> {
    let mut events = vec![
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
    ];
    if let Some(healing) = healing {
        events.push(DomainEvent::HealingApplied {
            target_id: healing.target_id.clone(),
            amount: healing.amount,
            healing_type: healing.healing_type.clone(),
        });
    }
    if let Some(temporary_vitality) = temporary_vitality {
        events.push(DomainEvent::TemporaryVitalityGranted {
            target_id: temporary_vitality.target_id.clone(),
            amount: temporary_vitality.after - temporary_vitality.before,
        });
    }
    events.push(DomainEvent::ModifierApplied {
        target_id: modifier.target_id.clone(),
        modifier_id: modifier.modifier_id.clone(),
        duration: modifier.duration.clone(),
    });
    events
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
        healing: None,
        temporary_vitality: None,
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
    if action.targeting.target_kind != TargetKind::Combatant
        || action.targeting.selection != TargetSelection::Single
    {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target declaration is not supported.".to_string(),
        };
    }
    if action.targeting.team_constraint == TargetTeamConstraint::Hostile
        && actor.team == target.team
    {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is not hostile.".to_string(),
        };
    }
    if action.targeting.team_constraint == TargetTeamConstraint::Ally && actor.team != target.team {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is not allied.".to_string(),
        };
    }
    if range_between(actor.position, target.position) > action.targeting.maximum_range {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is outside range.".to_string(),
        };
    }
    if action.targeting.visibility_requirement == VisibilityRequirement::Required
        && !action.targeting.visible_target_ids.contains(&target.id)
    {
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
    attack: &AttackCheckDeclaration,
) -> Option<i32> {
    effective_stats_for_combatant(scenario, &actor.id)?
        .stats
        .into_iter()
        .find(|stat| stat.stat_id == attack.modifier_stat_id)
        .map(|stat| stat.effective_value)
}

#[derive(Debug, Clone, Copy)]
struct HitOperations<'a> {
    damage: &'a DamageEffectOperation,
    healing: Option<&'a HealingEffectOperation>,
    temporary_vitality: Option<&'a TemporaryVitalityEffectOperation>,
    modifier: &'a ModifierEffectOperation,
}

struct AppliedVitalityEffects {
    damage: DamageOutcome,
    healing: Option<HealingOutcome>,
    temporary_vitality: Option<TemporaryVitalityOutcome>,
}

fn hit_operations(action: &ActionDefinition) -> Option<HitOperations<'_>> {
    if action.hit.operations.is_empty()
        || action
            .hit
            .operations
            .iter()
            .any(|operation| !operation.is_currently_supported())
    {
        return None;
    }

    Some(HitOperations {
        damage: action.hit.damage_operation()?,
        healing: action
            .hit
            .operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::Heal(healing) => Some(healing),
                _ => None,
            }),
        temporary_vitality: action
            .hit
            .operations
            .iter()
            .find_map(|operation| match operation {
                HitEffectOperation::GrantTemporaryVitality(vitality) => Some(vitality),
                _ => None,
            }),
        modifier: action.hit.modifier_operation()?,
    })
}

fn apply_damage(
    scenario: &RulebenchScenario,
    target: &Combatant,
    amount: i32,
    damage_type: &str,
) -> DamageOutcome {
    let before = target.hit_points;
    let requested_amount = amount.max(0);
    let adjusted_amount = match scenario
        .entity_by_id(&target.entity_id)
        .and_then(|entity| {
            entity
                .damage_adjustments
                .iter()
                .find(|adjustment| adjustment.damage_type == damage_type)
        })
        .map(|adjustment| adjustment.policy)
    {
        Some(DamageAdjustmentPolicy::Immunity) => 0,
        Some(DamageAdjustmentPolicy::Resistance) => requested_amount / 2,
        Some(DamageAdjustmentPolicy::Vulnerability) => requested_amount.saturating_mul(2),
        None => requested_amount,
    };
    let temporary_vitality_absorbed = target.temporary_vitality.min(adjusted_amount);
    let remaining_damage = adjusted_amount - temporary_vitality_absorbed;
    let next = before.current.saturating_sub(remaining_damage).max(0);
    DamageOutcome {
        target_id: target.id.clone(),
        damage_type: damage_type.to_string(),
        requested_amount,
        amount: adjusted_amount,
        temporary_vitality_absorbed,
        temporary_vitality_after: target.temporary_vitality - temporary_vitality_absorbed,
        before,
        after: BoundedValue {
            current: next,
            max: before.max,
        },
    }
}

fn apply_vitality_effects(
    scenario: &RulebenchScenario,
    target: &Combatant,
    damage_roll: i32,
    operations: HitOperations<'_>,
) -> AppliedVitalityEffects {
    // The operation order is fixed: mitigate damage, let temporary vitality
    // absorb it, cap healing at max HP, then replace lower temporary vitality.
    let damage = apply_damage(
        scenario,
        target,
        damage_roll + operations.damage.damage_bonus,
        &operations.damage.damage_type,
    );
    let mut after_damage = target.clone();
    after_damage.hit_points = damage.after;
    after_damage.temporary_vitality = damage.temporary_vitality_after;
    let healing = operations
        .healing
        .map(|operation| apply_healing(&after_damage, operation));
    let mut after_healing = after_damage;
    if let Some(outcome) = &healing {
        after_healing.hit_points = outcome.after;
    }
    let temporary_vitality = operations
        .temporary_vitality
        .map(|operation| grant_temporary_vitality(&after_healing, operation));

    AppliedVitalityEffects {
        damage,
        healing,
        temporary_vitality,
    }
}

fn modifier_outcome(
    scenario: &RulebenchScenario,
    target: &Combatant,
    source_id: &str,
    operation: &ModifierEffectOperation,
) -> Option<ModifierOutcome> {
    let definition = scenario.modifier_by_id(&operation.modifier_id)?;
    let remaining_turns = match definition.duration_policy {
        ModifierDurationPolicy::Turns(turns) => Some(turns),
        ModifierDurationPolicy::Permanent
        | ModifierDurationPolicy::Rounds(_)
        | ModifierDurationPolicy::UntilEvent(_) => None,
    };
    let remaining_rounds = match definition.duration_policy {
        ModifierDurationPolicy::Rounds(rounds) => Some(rounds),
        ModifierDurationPolicy::Permanent
        | ModifierDurationPolicy::Turns(_)
        | ModifierDurationPolicy::UntilEvent(_) => None,
    };
    Some(ModifierOutcome {
        target_id: target.id.clone(),
        modifier_id: operation.modifier_id.clone(),
        source_id: source_id.to_string(),
        label: operation.modifier_label.clone(),
        duration: operation.modifier_duration.clone(),
        stacking_group: definition.stacking_group.clone(),
        stacking_policy: definition.stacking_policy,
        duration_policy: definition.duration_policy.clone(),
        remaining_turns,
        remaining_rounds,
    })
}

fn append_vitality_trace(trace: &mut Vec<TraceEntry>, effects: &AppliedVitalityEffects) {
    trace.push(TraceEntry::new(
        trace.len() as u32 + 1,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Damage vitality resolved.",
        format!(
            "Requested {} {}; {} applied after mitigation, with {} absorbed by temporary vitality.",
            effects.damage.requested_amount,
            effects.damage.damage_type,
            effects.damage.amount,
            effects.damage.temporary_vitality_absorbed
        ),
    ));
    if let Some(healing) = &effects.healing {
        trace.push(TraceEntry::new(
            trace.len() as u32 + 1,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Healing vitality resolved.",
            format!(
                "Requested {} {} healing; {} applied within the hit point cap.",
                healing.requested_amount, healing.healing_type, healing.amount
            ),
        ));
    }
    if let Some(temporary_vitality) = &effects.temporary_vitality {
        trace.push(TraceEntry::new(
            trace.len() as u32 + 1,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Temporary vitality resolved.",
            format!(
                "Requested {}; temporary vitality changed from {} to {}.",
                temporary_vitality.requested_amount,
                temporary_vitality.before,
                temporary_vitality.after
            ),
        ));
    }
}

fn apply_healing(target: &Combatant, operation: &HealingEffectOperation) -> HealingOutcome {
    let before = target.hit_points;
    let requested_amount = operation.healing_bonus.max(0);
    let next = before
        .current
        .saturating_add(requested_amount)
        .min(before.max);
    HealingOutcome {
        target_id: target.id.clone(),
        healing_type: operation.healing_type.clone(),
        requested_amount,
        amount: next - before.current,
        before,
        after: BoundedValue {
            current: next,
            max: before.max,
        },
    }
}

fn grant_temporary_vitality(
    target: &Combatant,
    operation: &TemporaryVitalityEffectOperation,
) -> TemporaryVitalityOutcome {
    let requested_amount = operation.vitality_bonus.max(0);
    TemporaryVitalityOutcome {
        target_id: target.id.clone(),
        requested_amount,
        before: target.temporary_vitality,
        after: target.temporary_vitality.max(requested_amount),
    }
}
