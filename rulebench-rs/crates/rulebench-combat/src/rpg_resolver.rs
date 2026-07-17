use std::collections::BTreeMap;

use rpg_core::{
    DeterministicRandomStream, RpgCapabilityState, RpgDomainEvent, RpgEntityState, RpgIntent,
    RpgModifierStackingPolicy, RpgRandomRequest, RpgResolutionReceipt, RpgResolutionRejection,
    RpgTraceStep,
};
use rpg_ir::RpgIrCheck;
use rpg_runtime::{
    PreEffectWorkspace, RpgAuthoritySession, RpgGameplayContinuation, RpgGameplayFabricReadout,
    RpgPreEffectOwner,
};
use rulebench_content::{
    representative_rpg_content, RulebenchRpgContent, RulebenchRpgContentError,
};

use crate::model::*;
use crate::state::CombatState;

pub const ASHA_RPG_AUTHORITY_SURFACE: &str = "asha-rpg.semantic-kernel.v1";

#[derive(Debug)]
pub struct RulebenchRpgAuthority {
    content: RulebenchRpgContent,
    session: RpgAuthoritySession,
}

impl RulebenchRpgAuthority {
    pub fn new(scenario: &RulebenchScenario) -> Result<Self, RulebenchRpgContentError> {
        let content = representative_rpg_content()?;
        let compiled = content.compile()?;
        Ok(Self {
            content,
            session: RpgAuthoritySession::new(
                compiled,
                capability_state(scenario),
                DeterministicRandomStream::new(Vec::new()),
            ),
        })
    }

    pub fn resolve(
        &mut self,
        scenario: &RulebenchScenario,
        intent: UseActionIntent,
        source_action_id: &str,
        roll_stream: &[i32],
    ) -> RulebenchReceipt {
        let Some(action) = self
            .content
            .normalized_ir
            .actions
            .iter()
            .find(|action| action.id == source_action_id)
        else {
            return rejected_receipt(
                scenario,
                intent,
                RulebenchRejection::InvalidAction,
                None,
                "The action is not present in the compiled RPG content package.".to_string(),
                Vec::new(),
            );
        };
        let random_values = match random_values(roll_stream) {
            Ok(values) => values,
            Err(receipt) => {
                return rejected_receipt(
                    scenario,
                    intent,
                    receipt,
                    None,
                    "RPG authority random evidence must be a positive bounded integer.".to_string(),
                    Vec::new(),
                );
            }
        };
        let rpg_intent = rpg_intent(&intent, source_action_id);
        let mut result = match self.session.submit_with_random(&rpg_intent, random_values) {
            Ok(receipt) => accepted_receipt(scenario, intent, receipt, &action.check, roll_stream),
            Err(rejection) => {
                rejected_rpg_receipt(scenario, intent, rejection, &action.check, roll_stream)
            }
        };
        append_authored_binding_trace(scenario, &mut result);
        result
    }

    pub fn random_request(
        &self,
        intent: &UseActionIntent,
        source_action_id: &str,
        roll_stream: &[i32],
    ) -> Option<RpgRandomRequest> {
        let random_values = random_values(roll_stream).ok()?;
        self.session
            .preview_with_random(&rpg_intent(intent, source_action_id), random_values)
            .err()
            .and_then(|rejection| rejection.random_request)
    }

    pub fn preview(
        &self,
        intent: &UseActionIntent,
        source_action_id: &str,
        roll_stream: &[i32],
    ) -> Result<RpgResolutionReceipt, RpgResolutionRejection> {
        let random_values = random_values(roll_stream).map_err(|_| RpgResolutionRejection {
            code: "RPG_RANDOM_VALUE_OUT_OF_RANGE".to_string(),
            path: "$.random".to_string(),
            message: "random evidence is not a positive bounded integer".to_string(),
            trace: Vec::new(),
            random_attempted: 0,
            random_request: None,
        })?;
        self.session
            .preview_with_random(&rpg_intent(intent, source_action_id), random_values)
    }

    pub fn begin_before_effect(
        &mut self,
        workspace: PreEffectWorkspace,
        expected_owner_revision: String,
    ) -> Result<RpgGameplayContinuation, String> {
        self.session
            .begin_before_effect(workspace, expected_owner_revision)
    }

    pub fn resolve_before_effect(
        &mut self,
        pending: &RpgGameplayContinuation,
        accepted: bool,
        option_id: Option<String>,
        owner: &mut dyn RpgPreEffectOwner,
    ) -> Result<(), String> {
        let receipt = self
            .session
            .resolve_before_effect(pending, accepted, option_id, owner)?;
        if !receipt.accepted() {
            return Err(format!(
                "RPG gameplay pre-effect owner rejected: {:?}",
                receipt.diagnostics
            ));
        }
        Ok(())
    }

    pub fn gameplay_fabric_readout(&self) -> RpgGameplayFabricReadout {
        self.session.gameplay_fabric_readout()
    }
}

pub fn resolves_through_rpg_language(action_id: &str) -> bool {
    representative_rpg_content().is_ok_and(|content| content.binding(action_id).is_some())
}

fn resolves_through_rpg_language_for_scenario(
    scenario: &RulebenchScenario,
    action_id: &str,
) -> bool {
    representative_rpg_content().is_ok_and(|content| {
        content.binding(action_id).is_some_and(|binding| {
            binding
                .ruleset_ids
                .iter()
                .any(|ruleset_id| ruleset_id == &scenario.selected_ruleset_id)
        })
    })
}

/// Resolve a scenario-local runtime action identity to its TypeScript-authored
/// RPG source identity.
///
/// Returning an authored identity even when the generated RPG package does not
/// contain it is intentional: authored actions fail closed at this boundary
/// instead of falling through to Rulebench's legacy resolver.
pub fn rpg_dispatch_action_id(
    scenario: &RulebenchScenario,
    runtime_action_id: &str,
) -> Option<String> {
    if let Some(action_id) = scenario
        .authored_scenario_binding
        .iter()
        .flat_map(|binding| &binding.participants)
        .flat_map(|participant| &participant.action_grants)
        .find(|grant| grant.runtime_action_id == runtime_action_id)
        .map(|grant| grant.action_id.clone())
    {
        return Some(action_id);
    }

    if let Some(action_id) = scenario
        .authored_action_binding
        .as_ref()
        .filter(|_| scenario.action_by_id(runtime_action_id).is_some())
        .map(|binding| binding.action_id.clone())
    {
        return Some(action_id);
    }

    scenario
        .action_by_id(runtime_action_id)
        .is_none()
        .then(|| runtime_action_id.to_string())
        .filter(|action_id| resolves_through_rpg_language_for_scenario(scenario, action_id))
}

pub(crate) fn rpg_reaction_hook(
    scenario: &RulebenchScenario,
    runtime_action_id: &str,
    target_ids: &[String],
) -> Option<ReactionHookEffectOperation> {
    let source_action_id = rpg_dispatch_action_id(scenario, runtime_action_id)?;
    let content = representative_rpg_content().ok()?;
    let reaction = content.binding(&source_action_id)?.reaction.as_ref()?;
    let options = target_ids
        .iter()
        .map(|reactor_id| ReactionOptionDeclaration {
            id: format!("{}.{}", reaction.option_id, reactor_id),
            reactor_id: reactor_id.clone(),
            opens_nested_window: false,
        })
        .collect::<Vec<_>>();
    if options.is_empty() {
        return None;
    }

    Some(ReactionHookEffectOperation {
        hook_id: format!("rpg.{source_action_id}.before-effect"),
        window: ReactionWindow::BeforeEffect,
        eligible_reactor_ids: target_ids.to_vec(),
        options,
        maximum_nested_depth: 0,
    })
}

/// Stateless read-only adapter used by catalog previews, never by the product
/// command runtime. Product execution owns one `RulebenchRpgAuthority` per
/// `CombatSessionState`.
pub fn preview_rpg_use_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    source_action_id: &str,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let mut authority = match RulebenchRpgAuthority::new(scenario) {
        Ok(authority) => authority,
        Err(error) => {
            return rejected_receipt(
                scenario,
                intent,
                RulebenchRejection::InvalidAction,
                None,
                format!("Rulebench RPG content failed closed: {error:?}"),
                Vec::new(),
            );
        }
    };
    authority.resolve(scenario, intent, source_action_id, roll_stream)
}

fn random_values(roll_stream: &[i32]) -> Result<Vec<u32>, RulebenchRejection> {
    roll_stream
        .iter()
        .copied()
        .map(u32::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| RulebenchRejection::InvalidRollValue)
}

fn rpg_intent(intent: &UseActionIntent, source_action_id: &str) -> RpgIntent {
    RpgIntent {
        action_id: source_action_id.to_string(),
        actor_id: intent.actor_id.clone(),
        target_ids: if intent.target_ids.is_empty() {
            vec![intent.target_id.clone()]
        } else {
            intent.target_ids.clone()
        },
    }
}

fn append_authored_binding_trace(scenario: &RulebenchScenario, receipt: &mut RulebenchReceipt) {
    let Some(binding) = scenario.authored_action_binding.as_ref() else {
        return;
    };
    receipt.trace.push(TraceEntry::new(
        receipt.trace.len() as u32 + 1,
        TracePhase::Validation,
        TraceStatus::Info,
        "Authored action binding verified.",
        format!(
            "Pack {}@{} action {} ({}) granted ability {} to actor {}.",
            binding.content_pack_set.root.id,
            binding.content_pack_set.root.version,
            binding.action_id,
            binding.action_definition_fingerprint.value,
            binding.ability_id,
            binding.actor_id
        ),
    ));
}

fn capability_state(scenario: &RulebenchScenario) -> RpgCapabilityState {
    let mut state = RpgCapabilityState::default();
    for combatant in &scenario.combatants {
        let mut entity = RpgEntityState::new(
            &combatant.id,
            combatant.team,
            combatant.position,
            combatant.hit_points.max,
        );
        for stat in combatant
            .stats
            .base_stats
            .iter()
            .chain(&combatant.stats.derived_stats)
        {
            entity = entity
                .with_stat(&stat.id, stat.value)
                .with_defense(&stat.id, stat.value);
        }
        for defense in &combatant.defenses {
            entity = entity.with_defense(&defense.id, defense.value);
        }
        let resource_pools = if combatant.resource_pools.is_empty() {
            vec![ActionResourcePool::standard_action()]
        } else {
            combatant.resource_pools.clone()
        };
        for resource in &resource_pools {
            entity = entity.with_resource(
                &resource.id,
                i32::try_from(resource.initial).unwrap_or(i32::MAX),
                i32::try_from(resource.maximum).unwrap_or(i32::MAX),
            );
        }
        state.insert_entity(entity);
        let missing_vitality = combatant
            .hit_points
            .max
            .saturating_sub(combatant.hit_points.current);
        if missing_vitality > 0 {
            state
                .vitality_owner()
                .apply_damage(&combatant.id, missing_vitality)
                .expect("inserted RPG entity accepts current vitality projection");
        }
    }
    state
}

fn accepted_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    receipt: RpgResolutionReceipt,
    check: &RpgIrCheck,
    supplied_rolls: &[i32],
) -> RulebenchReceipt {
    let mut targets = receipt
        .target_ids
        .iter()
        .map(|target_id| {
            (
                target_id.clone(),
                TargetResolutionOutcome {
                    target_id: target_id.clone(),
                    target_legality: TargetLegality {
                        target_id: target_id.clone(),
                        accepted: true,
                        reason: "Accepted by the compiled Asha RPG target selector.".to_string(),
                    },
                    attack_roll: None,
                    damage: None,
                    healing: None,
                    temporary_vitality: None,
                    modifier: None,
                    movement: None,
                    resource_changes: Vec::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut events = vec![
        DomainEvent::IntentShapeAccepted {
            actor_id: receipt.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: receipt.target_ids.first().cloned().unwrap_or_default(),
        },
        DomainEvent::ActionUsed {
            actor_id: receipt.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: receipt.target_ids.first().cloned().unwrap_or_default(),
        },
    ];

    for event in &receipt.events {
        project_rpg_event(
            scenario,
            &receipt.actor_id,
            event,
            &mut targets,
            &mut events,
        );
    }

    let target_results = targets.into_values().collect::<Vec<_>>();
    let first = target_results.first();
    let mut projected_state = CombatState::from_scenario(scenario);
    apply_target_results(&mut projected_state, &target_results);
    let projection = projected_state.project(
        "State projected from Asha RPG accepted DomainEvents; Rulebench retains product presentation.",
    );
    let mut trace = vec![TraceEntry::new(
        1,
        TracePhase::Proposal,
        TraceStatus::Accepted,
        "RPG intent admitted.",
        format!(
            "Compiled package action {} resolved runtime action {} for {} target(s).",
            receipt.action_id,
            intent.action_id,
            receipt.target_ids.len()
        ),
    )];
    trace.extend(receipt.trace.iter().enumerate().map(|(index, step)| {
        TraceEntry::new(
            index as u32 + 2,
            if step.code == "RPG_RESOLUTION_COMMITTED" {
                TracePhase::Commit
            } else {
                TracePhase::Resolution
            },
            TraceStatus::Accepted,
            step.code.clone(),
            format!("{}: {}", step.path, step.detail),
        )
    }));

    RulebenchReceipt {
        accepted: true,
        authority_surface: ASHA_RPG_AUTHORITY_SURFACE,
        intent,
        rejection: None,
        target_legality: first.map(|target| target.target_legality.clone()),
        attack_roll: first.and_then(|target| target.attack_roll.clone()),
        damage: first.and_then(|target| target.damage.clone()),
        healing: first.and_then(|target| target.healing.clone()),
        temporary_vitality: None,
        modifier: first.and_then(|target| target.modifier.clone()),
        target_results,
        roll_consumption: roll_consumption(
            supplied_rolls,
            receipt.random_consumed,
            &receipt.trace,
            check,
            None,
        ),
        events,
        trace,
        projection: Some(projection),
    }
}

fn project_rpg_event(
    scenario: &RulebenchScenario,
    intent_actor_id: &str,
    event: &RpgDomainEvent,
    targets: &mut BTreeMap<String, TargetResolutionOutcome>,
    events: &mut Vec<DomainEvent>,
) {
    match event {
        RpgDomainEvent::ResourceSpent {
            entity_id,
            resource_id,
            amount,
            remaining,
        } => {
            let before = remaining.saturating_add(*amount);
            events.push(DomainEvent::ResourceChanged {
                target_id: entity_id.clone(),
                resource_id: resource_id.clone(),
                delta: -*amount,
                before,
                after: *remaining,
            });
        }
        RpgDomainEvent::AttackResolved {
            actor_id,
            target_id,
            roll,
            total,
            defense_id,
            defense,
            hit,
        } => {
            let attack = AttackRollResult {
                roll: i32::try_from(*roll).unwrap_or(i32::MAX),
                modifier: total.saturating_sub(i32::try_from(*roll).unwrap_or(i32::MAX)),
                total: *total,
                defense_id: defense_id.clone(),
                defense_value: *defense,
                outcome: if *hit {
                    AttackOutcome::Hit
                } else {
                    AttackOutcome::Miss
                },
            };
            if let Some(target) = targets.get_mut(target_id) {
                target.attack_roll = Some(attack);
            }
            events.push(DomainEvent::AttackRolled {
                actor_id: actor_id.clone(),
                target_id: target_id.clone(),
                total: *total,
                defense_id: defense_id.clone(),
                defense_value: *defense,
                outcome: if *hit {
                    AttackOutcome::Hit
                } else {
                    AttackOutcome::Miss
                },
            });
        }
        RpgDomainEvent::SavingThrowResolved {
            target_id,
            total,
            difficulty,
            saved,
            ..
        } => {
            events.push(DomainEvent::SavingThrowResolved {
                actor_id: intent_actor_id.to_string(),
                target_id: target_id.clone(),
                total: *total,
                difficulty_class: *difficulty,
                outcome: if *saved {
                    SavingThrowOutcome::Saved
                } else {
                    SavingThrowOutcome::Failed
                },
            });
        }
        RpgDomainEvent::DamageApplied {
            target_id,
            amount,
            damage_type,
            remaining_vitality,
            ..
        } => {
            let maximum = scenario
                .combatants
                .iter()
                .find(|combatant| combatant.id == *target_id)
                .map_or(*remaining_vitality, |combatant| combatant.hit_points.max);
            let before_current = remaining_vitality.saturating_add(*amount).min(maximum);
            let damage = DamageOutcome {
                target_id: target_id.clone(),
                damage_type: damage_type.clone(),
                requested_amount: *amount,
                amount: *amount,
                temporary_vitality_absorbed: 0,
                temporary_vitality_after: 0,
                before: BoundedValue {
                    current: before_current,
                    max: maximum,
                },
                after: BoundedValue {
                    current: *remaining_vitality,
                    max: maximum,
                },
            };
            if let Some(target) = targets.get_mut(target_id) {
                target.damage = Some(damage);
            }
            events.push(DomainEvent::DamageApplied {
                target_id: target_id.clone(),
                amount: *amount,
                damage_type: damage_type.clone(),
            });
        }
        RpgDomainEvent::HealingApplied {
            target_id,
            amount,
            current_vitality,
            ..
        } => {
            let maximum = scenario
                .combatants
                .iter()
                .find(|combatant| combatant.id == *target_id)
                .map_or(*current_vitality, |combatant| combatant.hit_points.max);
            let before_current = current_vitality.saturating_sub(*amount);
            let healing = HealingOutcome {
                target_id: target_id.clone(),
                healing_type: "rpg.healing".to_string(),
                requested_amount: *amount,
                amount: *amount,
                before: BoundedValue {
                    current: before_current,
                    max: maximum,
                },
                after: BoundedValue {
                    current: *current_vitality,
                    max: maximum,
                },
            };
            if let Some(target) = targets.get_mut(target_id) {
                target.healing = Some(healing);
            }
            events.push(DomainEvent::HealingApplied {
                target_id: target_id.clone(),
                amount: *amount,
                healing_type: "rpg.healing".to_string(),
            });
        }
        RpgDomainEvent::ResourceChanged {
            entity_id,
            resource_id,
            delta,
            current,
        } => {
            let before = current.saturating_sub(*delta);
            let maximum = scenario
                .combatants
                .iter()
                .find(|combatant| combatant.id == *entity_id)
                .and_then(|combatant| {
                    combatant
                        .resource_pools
                        .iter()
                        .find(|resource| resource.id == *resource_id)
                })
                .map_or((*current).max(before), |resource| {
                    i32::try_from(resource.maximum).unwrap_or(i32::MAX)
                });
            let outcome = ResourceChangeOutcome {
                target_id: entity_id.clone(),
                resource_id: resource_id.clone(),
                requested_delta: *delta,
                before,
                after: *current,
                maximum,
            };
            if let Some(target) = targets.get_mut(entity_id) {
                target.resource_changes.push(outcome);
            }
            events.push(DomainEvent::ResourceChanged {
                target_id: entity_id.clone(),
                resource_id: resource_id.clone(),
                delta: *delta,
                before,
                after: *current,
            });
        }
        RpgDomainEvent::ModifierApplied {
            source_id,
            target_id,
            modifier_id,
            stacking_group,
            stacking,
            remaining_turns,
            ..
        } => {
            let definition = scenario.modifier_by_id(modifier_id);
            let stacking_policy = match stacking {
                RpgModifierStackingPolicy::Replace => ModifierStackingPolicy::Replace,
                RpgModifierStackingPolicy::Refresh => ModifierStackingPolicy::Refresh,
            };
            let modifier = ModifierOutcome {
                target_id: target_id.clone(),
                modifier_id: modifier_id.clone(),
                source_id: source_id.clone(),
                label: definition.map_or_else(|| modifier_id.clone(), |value| value.label.clone()),
                duration: format!("{remaining_turns} turns"),
                stacking_group: stacking_group.clone(),
                stacking_policy,
                duration_policy: ModifierDurationPolicy::Turns(*remaining_turns),
                remaining_turns: Some(*remaining_turns),
                remaining_rounds: None,
            };
            if let Some(target) = targets.get_mut(target_id) {
                target.modifier = Some(modifier);
            }
            events.push(DomainEvent::ModifierApplied {
                target_id: target_id.clone(),
                modifier_id: modifier_id.clone(),
                duration: format!("{remaining_turns} turns"),
            });
        }
        RpgDomainEvent::PositionChanged {
            entity_id,
            previous,
            current,
            ..
        } => {
            let movement = EffectMovementOutcome {
                target_id: entity_id.clone(),
                movement_kind: MovementKind::Shift,
                from: *previous,
                to: *current,
                distance: previous
                    .x
                    .abs_diff(current.x)
                    .saturating_add(previous.y.abs_diff(current.y)),
            };
            if let Some(target) = targets.get_mut(entity_id) {
                target.movement = Some(movement);
            }
            events.push(DomainEvent::EffectMovementApplied {
                target_id: entity_id.clone(),
                movement_kind: MovementKind::Shift,
                from: *previous,
                to: *current,
            });
        }
    }
}

fn apply_target_results(state: &mut CombatState, targets: &[TargetResolutionOutcome]) {
    for target in targets {
        if let Some(damage) = &target.damage {
            state.apply_hit(damage, target.modifier.as_ref());
        } else if let Some(modifier) = &target.modifier {
            state.apply_modifier(modifier);
        }
        if let Some(healing) = &target.healing {
            state.apply_healing(healing);
        }
        if let Some(movement) = &target.movement {
            state.apply_effect_movement(&target.target_id, movement.to);
        }
        for resource in &target.resource_changes {
            state.apply_resource_change(resource);
        }
    }
}

fn rejected_rpg_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    rejection: RpgResolutionRejection,
    check: &RpgIrCheck,
    supplied_rolls: &[i32],
) -> RulebenchReceipt {
    let legacy_rejection = match rejection.code.as_str() {
        "RPG_RANDOM_EXHAUSTED" if rejection.path.contains("check") => match check {
            RpgIrCheck::Attack { .. } => RulebenchRejection::MissingAttackRoll,
            RpgIrCheck::SavingThrow { .. } => RulebenchRejection::MissingCheckRoll,
            RpgIrCheck::NoRoll => RulebenchRejection::MissingDamageRoll,
        },
        "RPG_RANDOM_EXHAUSTED" => RulebenchRejection::MissingDamageRoll,
        "RPG_RANDOM_VALUE_OUT_OF_RANGE" => RulebenchRejection::InvalidRollValue,
        "RPG_INTENT_ACTOR_UNKNOWN" => RulebenchRejection::InvalidActor,
        "RPG_INTENT_ACTION_UNKNOWN" => RulebenchRejection::InvalidAction,
        "RPG_INTENT_TARGET_UNKNOWN" => RulebenchRejection::InvalidTarget,
        "RPG_INTENT_TARGET_DUPLICATE" => RulebenchRejection::DuplicateTarget,
        "RPG_INTENT_TARGET_LIMIT_EXCEEDED" => RulebenchRejection::TargetLimitExceeded,
        "RPG_INTENT_TARGET_TEAM_INVALID" => RulebenchRejection::TargetLegalityFailed,
        "RPG_INTENT_TARGET_OUT_OF_RANGE" => RulebenchRejection::TargetOutOfRange,
        _ => RulebenchRejection::InvalidAction,
    };
    let target_legality = matches!(
        legacy_rejection,
        RulebenchRejection::InvalidTarget
            | RulebenchRejection::TargetLegalityFailed
            | RulebenchRejection::TargetOutOfRange
    )
    .then(|| TargetLegality {
        target_id: intent.target_id.clone(),
        accepted: false,
        reason: rejection.message.clone(),
    });
    let missing_kind =
        (rejection.code == "RPG_RANDOM_EXHAUSTED").then_some(match legacy_rejection {
            RulebenchRejection::MissingAttackRoll => RollRequestKind::AttackRoll,
            RulebenchRejection::MissingCheckRoll => RollRequestKind::SavingThrowRoll,
            _ => RollRequestKind::DamageRoll,
        });
    let rolls = roll_consumption(
        supplied_rolls,
        rejection.random_attempted,
        &rejection.trace,
        check,
        missing_kind,
    );
    let detail = format!(
        "{} {}: {}",
        rejection.code, rejection.path, rejection.message
    );
    rejected_receipt(
        scenario,
        intent,
        legacy_rejection,
        target_legality,
        detail,
        rolls,
    )
}

fn rejected_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    target_legality: Option<TargetLegality>,
    detail: String,
    roll_consumption: Vec<RollConsumptionEntry>,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: ASHA_RPG_AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality,
        attack_roll: None,
        damage: None,
        healing: None,
        temporary_vitality: None,
        modifier: None,
        target_results: Vec::new(),
        roll_consumption,
        events: Vec::new(),
        trace: vec![TraceEntry::new(
            1,
            TracePhase::Validation,
            TraceStatus::Rejected,
            "Compiled RPG intent rejected.",
            detail,
        )],
        projection: Some(
            CombatState::from_scenario(scenario)
                .project("No state changed because Asha RPG authority rejected the intent."),
        ),
    }
}

fn roll_consumption(
    supplied_rolls: &[i32],
    consumed: usize,
    trace: &[RpgTraceStep],
    check: &RpgIrCheck,
    missing: Option<RollRequestKind>,
) -> Vec<RollConsumptionEntry> {
    let random_steps = trace
        .iter()
        .filter(|step| step.code == "RPG_RANDOM_CONSUMED")
        .collect::<Vec<_>>();
    let mut entries = supplied_rolls
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let random_step = random_steps.get(index);
            let request_kind = random_step.map_or(RollRequestKind::DamageRoll, |step| {
                if step.path.contains(".check.") || step.path.contains(".check[") {
                    match check {
                        RpgIrCheck::Attack { .. } => RollRequestKind::AttackRoll,
                        RpgIrCheck::SavingThrow { .. } => RollRequestKind::SavingThrowRoll,
                        RpgIrCheck::NoRoll => RollRequestKind::DamageRoll,
                    }
                } else {
                    RollRequestKind::DamageRoll
                }
            });
            RollConsumptionEntry {
                sequence: index as u32,
                request_kind,
                supplied_value: Some(*value),
                consumed: index < consumed,
                reason: if let Some(step) = random_step {
                    format!(
                        "Consumed by compiled Asha RPG authority at {} ({}).",
                        step.path, step.detail
                    )
                } else {
                    "Not consumed after authority reached a terminal outcome.".to_string()
                },
            }
        })
        .collect::<Vec<_>>();
    if let Some(request_kind) = missing {
        entries.push(RollConsumptionEntry {
            sequence: entries.len() as u32,
            request_kind,
            supplied_value: None,
            consumed: false,
            reason: "Compiled Asha RPG authority requested another bounded random value."
                .to_string(),
        });
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representative_content_is_owned_by_the_compiled_language_boundary() {
        assert!(resolves_through_rpg_language("hexing_bolt"));
        assert!(!resolves_through_rpg_language("legacy-only-action"));
    }
}
