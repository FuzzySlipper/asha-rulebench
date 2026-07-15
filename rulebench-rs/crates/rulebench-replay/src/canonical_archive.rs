//! Versioned semantic encoding for portable replay archive identity.
//!
//! The encoding is independent of Rust `Debug`, struct layout, host JSON, and
//! TypeScript. Every value is length-prefixed and field order is fixed by this
//! module's version. Registered scenario content is referenced by exact
//! scenario/content/ruleset identity; commands and all recorded authority
//! evidence are encoded in full.

use rulebench_combat::{
    ActionDefinition, ActionResourcePool, ActionResourceRefreshPolicy, ActiveModifier,
    CheckDeclaration, CombatEndPolicy, Combatant, CommandAuditEntry, CommandRollMode,
    DerivedStatFormula, DomainEvent, HitEffectOperation, ModifierDefinition,
    ModifierDurationPolicy, RollConsumptionEntry, RuleModuleConfiguration, RulebenchScenario,
    StateFingerprint, TraceEntry, TracePhase, TraceStatus, UseActionIntent,
};

use crate::{
    ReplayArchiveEntry, ReplayCommand, ReplayCommandRandomnessProvenance, ReplayRandomnessSource,
    ReplayStepEvidence, REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION,
    REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM,
};

pub fn canonical_replay_archive_payload(entry: &ReplayArchiveEntry) -> Vec<u8> {
    let mut encoder = CanonicalEncoder::new();
    encoder.string(REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION);
    encoder.string("metadata");
    encoder.string(&entry.metadata.package_id);
    encoder.string(&entry.metadata.session_id);
    encoder.string(&entry.metadata.scenario_id);
    encoder.string(&entry.metadata.ruleset_id);
    encoder.string(&entry.metadata.ruleset_version);
    encoder.string(&entry.metadata.completed_at);

    let package = &entry.package;
    encoder.string("package");
    encoder.string(&package.package_version);
    encoder.string(&package.id);
    encoder.string(&package.initial_session.session.id);
    feed_scenario(&mut encoder, &package.initial_session.scenario);
    feed_ruleset(&mut encoder, &package.ruleset);
    encoder.sequence(&package.commands, |encoder, record| {
        encoder.u32(record.sequence);
        encoder.string(&record.id);
        feed_command(encoder, &record.command);
        feed_step_evidence(encoder, &record.expected);
    });
    encoder.sequence(&package.evidence.accepted_events, |encoder, value| {
        encoder.u32(value.command_sequence);
        encoder.sequence(&value.events, feed_event);
    });
    encoder.sequence(&package.evidence.command_audit, feed_audit);
    encoder.sequence(&package.evidence.rolls, |encoder, value| {
        encoder.u32(value.command_sequence);
        encoder.sequence(&value.consumption, feed_roll);
    });
    encoder.sequence(&package.evidence.trace, |encoder, value| {
        encoder.u32(value.command_sequence);
        encoder.sequence(&value.entries, feed_trace);
    });
    encoder.sequence(&package.evidence.randomness, feed_randomness);
    feed_state_fingerprint(&mut encoder, &package.final_state_fingerprint);
    encoder.string(&package.fingerprint_kind);
    match &package.narration {
        Some(narration) => {
            encoder.bool(true);
            encoder.string(&narration.title);
            encoder.string(&narration.summary);
            encoder.strings(&narration.command_summaries);
        }
        None => encoder.bool(false),
    }
    encoder.finish()
}

fn feed_scenario(encoder: &mut CanonicalEncoder, scenario: &RulebenchScenario) {
    encoder.string(&scenario.metadata.id);
    encoder.string(&scenario.metadata.title);
    encoder.string(&scenario.metadata.summary);
    encoder.string(&scenario.metadata.seed_label);
    match &scenario.content_pack_set {
        Some(set) => {
            encoder.bool(true);
            feed_content_reference(encoder, &set.root);
            encoder.sequence(&set.packs, feed_content_reference);
            feed_content_fingerprint(encoder, &set.fingerprint);
        }
        None => encoder.bool(false),
    }
    encoder.sequence(&scenario.rulesets, feed_ruleset_metadata);
    encoder.string(&scenario.selected_ruleset_id);
    encoder.u32(scenario.grid.width);
    encoder.u32(scenario.grid.height);
    encoder.sequence(&scenario.grid.cells, |encoder, cell| {
        encoder.position(cell.position);
        encoder.strings(&cell.terrain_tags);
    });
    encoder.sequence(&scenario.combatants, feed_combatant);
    encoder.sequence(&scenario.entities, |encoder, entity| {
        encoder.string(&entity.id);
        encoder.string(&entity.name);
        encoder.string(&entity.summary);
        encoder.strings(&entity.tags);
        encoder.sequence(&entity.damage_adjustments, |encoder, adjustment| {
            encoder.string(&adjustment.damage_type);
            encoder.string(adjustment.policy.code());
        });
    });
    encoder.sequence(&scenario.abilities, |encoder, ability| {
        encoder.string(&ability.id);
        encoder.string(&ability.name);
        encoder.string(ability.kind.code());
        encoder.string(&ability.summary);
        encoder.strings(&ability.tags);
    });
    encoder.optional_string(scenario.selected_ability_id.as_deref());
    encoder.sequence(&scenario.classes, |encoder, class| {
        encoder.string(&class.id);
        encoder.string(&class.name);
        encoder.string(&class.version);
        encoder.string(&class.summary);
        encoder.strings(&class.tags);
        encoder.sequence(&class.prerequisites, feed_stat_requirement);
        encoder.sequence(&class.level_grants, |encoder, grant| {
            encoder.u32(grant.level);
            encoder.strings(&grant.granted_modifier_ids);
            encoder.strings(&grant.granted_ability_ids);
            encoder.sequence(&grant.granted_resource_pools, feed_resource_pool);
        });
    });
    encoder.optional_string(scenario.selected_class_id.as_deref());
    encoder.sequence(&scenario.stat_definitions, |encoder, definition| {
        encoder.string(&definition.id);
        encoder.string(&definition.label);
        encoder.string(definition.kind.code());
        encoder.optional(definition.formula.as_ref(), feed_formula);
        encoder.string(&definition.summary);
    });
    encoder.sequence(&scenario.modifiers, feed_modifier_definition);
    encoder.sequence(&scenario.items, |encoder, item| {
        encoder.string(&item.id);
        encoder.string(&item.name);
        encoder.string(&item.summary);
        encoder.strings(&item.tags);
        encoder.string(&item.equipment_slot);
        encoder.sequence(&item.requirements, feed_stat_requirement);
        encoder.strings(&item.granted_modifier_ids);
        encoder.strings(&item.granted_ability_ids);
        encoder.sequence(&item.granted_resource_pools, feed_resource_pool);
    });
    encoder.optional_string(scenario.selected_item_id.as_deref());
    encoder.sequence(&scenario.actions, feed_action);
    feed_action(encoder, &scenario.selected_action);
}

pub fn canonical_replay_archive_payload_fingerprint(entry: &ReplayArchiveEntry) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM
        .bytes()
        .chain(canonical_replay_archive_payload(entry))
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn feed_content_reference(
    encoder: &mut CanonicalEncoder,
    reference: &rulebench_combat::ContentPackReference,
) {
    encoder.string(&reference.id);
    encoder.string(&reference.version);
    feed_content_fingerprint(encoder, &reference.fingerprint);
}

fn feed_content_fingerprint(
    encoder: &mut CanonicalEncoder,
    fingerprint: &rulebench_combat::ContentFingerprint,
) {
    encoder.string(&fingerprint.algorithm);
    encoder.string(&fingerprint.value);
}

fn feed_ruleset_metadata(
    encoder: &mut CanonicalEncoder,
    ruleset: &rulebench_combat::RulesetMetadata,
) {
    encoder.string(&ruleset.id);
    encoder.string(&ruleset.name);
    encoder.string(&ruleset.version);
    encoder.string(&ruleset.summary);
    encoder.sequence(&ruleset.modules, |encoder, module| {
        encoder.string(module.module.code());
        encoder.string(&module.version);
        match &module.configuration {
            RuleModuleConfiguration::ActionResolution(configuration) => {
                encoder.string("actionResolution");
                encoder.string(configuration.targeting_policy.code());
                encoder.sequence(
                    &configuration.supported_check_handlers,
                    |encoder, handler| {
                        encoder.string(handler.code());
                    },
                );
            }
            RuleModuleConfiguration::TurnControl(configuration) => {
                encoder.string("turnControl");
                encoder.string(configuration.turn_order_policy.code());
                encoder.string(configuration.combat_end_policy.code());
                if let CombatEndPolicy::ObjectiveSideVictory { side_id } =
                    &configuration.combat_end_policy
                {
                    encoder.string(side_id);
                }
            }
        }
    });
}

fn feed_combatant(encoder: &mut CanonicalEncoder, combatant: &Combatant) {
    encoder.string(&combatant.id);
    encoder.string(&combatant.entity_id);
    encoder.string(&combatant.name);
    encoder.string(match combatant.team {
        rulebench_combat::Team::Ally => "ally",
        rulebench_combat::Team::Enemy => "enemy",
    });
    encoder.string(&combatant.side_id);
    encoder.i32(combatant.initiative);
    encoder.position(combatant.position);
    encoder.i32(combatant.hit_points.current);
    encoder.i32(combatant.hit_points.max);
    encoder.i32(combatant.temporary_vitality);
    encoder.sequence(&combatant.class_inputs, |encoder, class| {
        encoder.string(&class.class_id);
        encoder.string(&class.version);
        encoder.u32(class.level);
    });
    encoder.sequence(&combatant.stats.base_stats, feed_named_number);
    encoder.sequence(&combatant.stats.derived_stats, feed_named_number);
    encoder.sequence(&combatant.defenses, feed_named_number);
    encoder.sequence(&combatant.resource_pools, feed_resource_pool);
    encoder.strings(&combatant.inventory_item_ids);
    encoder.strings(&combatant.equipped_item_ids);
    encoder.strings(&combatant.base_ability_ids);
    encoder.sequence(&combatant.active_modifiers, feed_active_modifier);
    encoder.strings(&combatant.conditions);
    encoder.bool(combatant.is_actor);
}

fn feed_named_number(encoder: &mut CanonicalEncoder, value: &rulebench_combat::NamedNumber) {
    encoder.string(&value.id);
    encoder.string(&value.label);
    encoder.i32(value.value);
}

fn feed_resource_pool(encoder: &mut CanonicalEncoder, pool: &ActionResourcePool) {
    encoder.string(&pool.id);
    encoder.string(pool.kind.code());
    encoder.u32(pool.maximum);
    encoder.string(pool.refresh_policy.code());
    if let ActionResourceRefreshPolicy::Turns(turns) = pool.refresh_policy {
        encoder.u32(turns);
    }
}

fn feed_stat_requirement(
    encoder: &mut CanonicalEncoder,
    requirement: &rulebench_combat::StatRequirement,
) {
    encoder.string(&requirement.stat_id);
    encoder.i32(requirement.minimum);
}

fn feed_formula(encoder: &mut CanonicalEncoder, formula: &DerivedStatFormula) {
    encoder.string(formula.code());
    match formula {
        DerivedStatFormula::Constant { value } => encoder.i32(*value),
        DerivedStatFormula::StatReference { stat_id } => encoder.string(stat_id),
        DerivedStatFormula::Sum { operands } | DerivedStatFormula::Product { operands } => {
            encoder.sequence(operands, feed_formula);
        }
        DerivedStatFormula::Difference {
            minuend,
            subtrahend,
        } => {
            feed_formula(encoder, minuend);
            feed_formula(encoder, subtrahend);
        }
    }
}

fn feed_modifier_definition(encoder: &mut CanonicalEncoder, modifier: &ModifierDefinition) {
    encoder.string(&modifier.id);
    encoder.string(&modifier.label);
    encoder.string(&modifier.summary);
    feed_modifier_tenure(encoder, modifier.default_tenure);
    encoder.string(&modifier.stacking_group);
    encoder.string(modifier.stacking_policy.code());
    feed_modifier_duration(encoder, &modifier.duration_policy);
    encoder.sequence(&modifier.stat_adjustments, |encoder, adjustment| {
        encoder.string(&adjustment.stat_id);
        encoder.string(&adjustment.stat_label);
        encoder.i32(adjustment.delta);
    });
}

fn feed_active_modifier(encoder: &mut CanonicalEncoder, modifier: &ActiveModifier) {
    encoder.string(&modifier.modifier_id);
    encoder.string(&modifier.source_id);
    encoder.string(&modifier.label);
    encoder.string(&modifier.duration);
    feed_modifier_tenure(encoder, modifier.tenure);
    encoder.string(&modifier.stacking_group);
    encoder.string(modifier.stacking_policy.code());
    feed_modifier_duration(encoder, &modifier.duration_policy);
    encoder.optional_u32(modifier.remaining_turns);
    encoder.optional_u32(modifier.remaining_rounds);
}

fn feed_modifier_tenure(encoder: &mut CanonicalEncoder, tenure: rulebench_combat::ModifierTenure) {
    encoder.string(match tenure {
        rulebench_combat::ModifierTenure::Temporary => "temporary",
        rulebench_combat::ModifierTenure::Permanent => "permanent",
    });
}

fn feed_modifier_duration(encoder: &mut CanonicalEncoder, duration: &ModifierDurationPolicy) {
    match duration {
        ModifierDurationPolicy::Permanent => encoder.string("permanent"),
        ModifierDurationPolicy::Turns(value) => {
            encoder.string("turns");
            encoder.u32(*value);
        }
        ModifierDurationPolicy::Rounds(value) => {
            encoder.string("rounds");
            encoder.u32(*value);
        }
        ModifierDurationPolicy::UntilEvent(value) => {
            encoder.string("untilEvent");
            encoder.string(value);
        }
    }
}

fn feed_action(encoder: &mut CanonicalEncoder, action: &ActionDefinition) {
    encoder.string(&action.id);
    encoder.string(&action.ruleset_id);
    encoder.string(&action.ability_id);
    encoder.string(&action.name);
    encoder.string(&action.actor_id);
    feed_targeting(encoder, &action.targeting);
    feed_check(encoder, &action.check);
    encoder.i32(action.hit.damage_bonus);
    encoder.string(&action.hit.damage_type);
    encoder.string(&action.hit.modifier_id);
    encoder.string(&action.hit.modifier_label);
    encoder.string(&action.hit.modifier_duration);
    encoder.sequence(&action.hit.operations, feed_hit_operation);
    encoder.sequence(&action.resource_costs, |encoder, cost| {
        encoder.string(&cost.resource_id);
        encoder.u32(cost.amount);
    });
    encoder.optional(action.movement.as_ref(), |encoder, movement| {
        encoder.u32(movement.allowance);
        encoder.string(match movement.topology {
            rulebench_combat::MovementTopology::OrthogonalManhattan => "orthogonalManhattan",
        });
        encoder.strings(&movement.blocking_terrain_tags);
        encoder.strings(&movement.difficult_terrain_tags);
    });
    encoder.string(&action.action_text);
    encoder.string(&action.effect_text);
}

fn feed_targeting(
    encoder: &mut CanonicalEncoder,
    targeting: &rulebench_combat::TargetingDeclaration,
) {
    encoder.string(match targeting.target_kind {
        rulebench_combat::TargetKind::Combatant => "combatant",
        rulebench_combat::TargetKind::Area => "area",
    });
    encoder.string(match targeting.selection {
        rulebench_combat::TargetSelection::Single => "single",
        rulebench_combat::TargetSelection::Multiple => "multiple",
    });
    encoder.string(match targeting.team_constraint {
        rulebench_combat::TargetTeamConstraint::Hostile => "hostile",
        rulebench_combat::TargetTeamConstraint::Ally => "ally",
        rulebench_combat::TargetTeamConstraint::Any => "any",
    });
    encoder.u32(targeting.maximum_range);
    encoder.string(match targeting.visibility_requirement {
        rulebench_combat::VisibilityRequirement::Required => "required",
        rulebench_combat::VisibilityRequirement::Ignored => "ignored",
    });
    encoder.strings(&targeting.target_ids);
    encoder.strings(&targeting.visible_target_ids);
    encoder.optional(
        targeting.operation_pipeline.as_ref(),
        |encoder, pipeline| {
            encoder.u32(pipeline.maximum_targets);
            encoder.optional(pipeline.area.as_ref(), |encoder, area| {
                encoder.string(match area.shape {
                    rulebench_combat::AreaShape::ManhattanBurst => "manhattanBurst",
                });
                encoder.u32(area.radius);
            });
            encoder.string(pipeline.roll_policy.code());
            encoder.string(match pipeline.failure_policy {
                rulebench_combat::TargetFailurePolicy::Atomic => "atomic",
            });
            encoder.string(match pipeline.target_order {
                rulebench_combat::TargetOrderPolicy::CanonicalId => "canonicalId",
            });
        },
    );
}

fn feed_check(encoder: &mut CanonicalEncoder, check: &CheckDeclaration) {
    match check {
        CheckDeclaration::Attack(value) => {
            encoder.string("attack");
            encoder.i32(value.modifier);
            encoder.string(&value.modifier_stat_id);
            encoder.string(&value.defense.id);
            encoder.string(&value.defense.label);
        }
        CheckDeclaration::SavingThrow(value) => {
            encoder.string("savingThrow");
            encoder.string(&value.save_stat_id);
            encoder.i32(value.difficulty_class);
        }
        CheckDeclaration::Contested(value) => {
            encoder.string("contested");
            encoder.string(&value.actor_stat_id);
            encoder.string(&value.target_stat_id);
        }
    }
}

fn feed_hit_operation(encoder: &mut CanonicalEncoder, operation: &HitEffectOperation) {
    encoder.string(operation.id().code());
    match operation {
        HitEffectOperation::Damage(value) => {
            encoder.i32(value.damage_bonus);
            encoder.string(&value.damage_type);
        }
        HitEffectOperation::Heal(value) => {
            encoder.i32(value.healing_bonus);
            encoder.string(&value.healing_type);
        }
        HitEffectOperation::GrantTemporaryVitality(value) => {
            encoder.i32(value.vitality_bonus);
        }
        HitEffectOperation::ApplyModifier(value) => {
            encoder.string(&value.modifier_id);
            encoder.string(&value.modifier_label);
            encoder.string(&value.modifier_duration);
        }
        HitEffectOperation::Move(value) => {
            encoder.u32(value.maximum_distance);
            encoder.string(value.movement_kind.code());
        }
        HitEffectOperation::ChangeResource(value) => {
            encoder.string(&value.resource_id);
            encoder.i32(value.delta);
        }
        HitEffectOperation::OpenReactionWindow(value) => {
            encoder.string(&value.hook_id);
            encoder.string(value.window.code());
            encoder.strings(&value.eligible_reactor_ids);
            encoder.sequence(&value.options, |encoder, option| {
                encoder.string(&option.id);
                encoder.string(&option.reactor_id);
                encoder.bool(option.opens_nested_window);
            });
            encoder.u32(value.maximum_nested_depth);
        }
    }
}

fn feed_ruleset(
    encoder: &mut CanonicalEncoder,
    ruleset: &rulebench_combat::RulesetArtifactProvenance,
) {
    encoder.string(&ruleset.ruleset_id);
    encoder.string(&ruleset.ruleset_version);
    encoder.sequence(&ruleset.module_versions, |encoder, module| {
        encoder.string(module.module.code());
        encoder.string(&module.version);
    });
    encoder.string(&ruleset.effect_operation_vocabulary_version);
}

fn feed_command(encoder: &mut CanonicalEncoder, command: &ReplayCommand) {
    encoder.string(command.code());
    match command {
        ReplayCommand::Intent(value) => {
            feed_command_copy(encoder, &value.id, &value.title, &value.summary);
            feed_intent(encoder, &value.intent);
            encoder.i32s(&value.roll_stream);
            feed_roll_mode(encoder, value.roll_mode);
        }
        ReplayCommand::Control(value) => encoder.string(value.kind.code()),
        ReplayCommand::SelectedCandidate(value) => {
            feed_command_copy(encoder, &value.id, &value.title, &value.summary);
            encoder.string(&value.action_id);
            encoder.string(&value.target_id);
            encoder.i32s(&value.roll_stream);
            feed_roll_mode(encoder, value.roll_mode);
        }
        ReplayCommand::AutomaticStep(value) => {
            feed_command_copy(encoder, &value.id, &value.title, &value.summary);
            encoder.i32s(&value.roll_stream);
            feed_policy(encoder, &value.policy);
            feed_roll_mode(encoder, value.roll_mode);
        }
        ReplayCommand::AutomaticRun(value) => {
            feed_command_copy(encoder, &value.id, &value.title, &value.summary);
            encoder.u32(value.max_steps);
            encoder.i32s(&value.roll_stream);
            feed_policy(encoder, &value.policy);
            feed_roll_mode(encoder, value.roll_mode);
        }
        ReplayCommand::Equipment(value) => {
            encoder.string(value.kind.code());
            encoder.string(&value.combatant_id);
            encoder.string(&value.item_id);
        }
        ReplayCommand::Reaction(value) => {
            encoder.string(&value.window_id);
            encoder.string(&value.reactor_id);
            encoder.string(value.response_kind.code());
            encoder.optional_string(value.option_id.as_deref());
        }
    }
}

fn feed_command_copy(encoder: &mut CanonicalEncoder, id: &str, title: &str, summary: &str) {
    encoder.string(id);
    encoder.string(title);
    encoder.string(summary);
}

fn feed_policy(
    encoder: &mut CanonicalEncoder,
    policy: &rulebench_combat::CombatAutomationPolicySpec,
) {
    encoder.string(&policy.id);
    encoder.u32(policy.version);
    encoder.string(policy.no_candidate_behavior.code());
}

fn feed_roll_mode(encoder: &mut CanonicalEncoder, mode: CommandRollMode) {
    match mode {
        CommandRollMode::Supplied => encoder.string("supplied"),
        CommandRollMode::AuthorityGenerated { seed } => {
            encoder.string("authorityGenerated");
            encoder.u64(seed);
        }
        CommandRollMode::RecordedGenerated { seed } => {
            encoder.string("recordedGenerated");
            encoder.u64(seed);
        }
    }
}

fn feed_intent(encoder: &mut CanonicalEncoder, intent: &UseActionIntent) {
    encoder.string(&intent.actor_id);
    encoder.string(&intent.action_id);
    encoder.string(&intent.target_id);
    encoder.strings(&intent.target_ids);
    encoder.optional_position(intent.target_cell);
    encoder.optional_position(intent.destination_cell);
    encoder.optional_position(intent.observed_origin);
}

fn feed_step_evidence(encoder: &mut CanonicalEncoder, evidence: &ReplayStepEvidence) {
    encoder.bool(evidence.accepted);
    encoder.string(&evidence.decision_code);
    feed_state_fingerprint(encoder, &evidence.state_before_fingerprint);
    feed_state_fingerprint(encoder, &evidence.state_after_fingerprint);
    encoder.sequence(&evidence.accepted_events, feed_event);
    encoder.sequence(&evidence.command_audit, feed_audit);
    encoder.sequence(&evidence.rolls, feed_roll);
    encoder.sequence(&evidence.trace, feed_trace);
    encoder.string(&evidence.gameplay_module_state_hash);
    encoder.strings(&evidence.gameplay_decision_receipt_hashes);
}

fn feed_state_fingerprint(encoder: &mut CanonicalEncoder, fingerprint: &StateFingerprint) {
    encoder.string(&fingerprint.algorithm);
    encoder.string(&fingerprint.value);
}

fn feed_event(encoder: &mut CanonicalEncoder, event: &DomainEvent) {
    match event {
        DomainEvent::IntentShapeAccepted {
            actor_id,
            action_id,
            target_id,
        }
        | DomainEvent::ActionUsed {
            actor_id,
            action_id,
            target_id,
        } => {
            encoder.string(match event {
                DomainEvent::IntentShapeAccepted { .. } => "intentShapeAccepted",
                _ => "actionUsed",
            });
            encoder.string(actor_id);
            encoder.string(action_id);
            encoder.string(target_id);
        }
        DomainEvent::AttackRolled {
            actor_id,
            target_id,
            total,
            defense_id,
            defense_value,
            outcome,
        } => {
            encoder.string("attackRolled");
            encoder.string(actor_id);
            encoder.string(target_id);
            encoder.i32(*total);
            encoder.string(defense_id);
            encoder.i32(*defense_value);
            encoder.string(match outcome {
                rulebench_combat::AttackOutcome::Hit => "hit",
                rulebench_combat::AttackOutcome::Miss => "miss",
            });
        }
        DomainEvent::SavingThrowResolved {
            actor_id,
            target_id,
            total,
            difficulty_class,
            outcome,
        } => {
            encoder.string("savingThrowResolved");
            encoder.string(actor_id);
            encoder.string(target_id);
            encoder.i32(*total);
            encoder.i32(*difficulty_class);
            encoder.string(match outcome {
                rulebench_combat::SavingThrowOutcome::Saved => "saved",
                rulebench_combat::SavingThrowOutcome::Failed => "failed",
            });
        }
        DomainEvent::ContestedCheckResolved {
            actor_id,
            target_id,
            actor_total,
            target_total,
            outcome,
        } => {
            encoder.string("contestedCheckResolved");
            encoder.string(actor_id);
            encoder.string(target_id);
            encoder.i32(*actor_total);
            encoder.i32(*target_total);
            encoder.string(match outcome {
                rulebench_combat::ContestedCheckOutcome::ActorWins => "actorWins",
                rulebench_combat::ContestedCheckOutcome::TargetWins => "targetWins",
            });
        }
        DomainEvent::DamageApplied {
            target_id,
            amount,
            damage_type,
        } => {
            encoder.string("damageApplied");
            encoder.string(target_id);
            encoder.i32(*amount);
            encoder.string(damage_type);
        }
        DomainEvent::HealingApplied {
            target_id,
            amount,
            healing_type,
        } => {
            encoder.string("healingApplied");
            encoder.string(target_id);
            encoder.i32(*amount);
            encoder.string(healing_type);
        }
        DomainEvent::TemporaryVitalityGranted { target_id, amount } => {
            encoder.string("temporaryVitalityGranted");
            encoder.string(target_id);
            encoder.i32(*amount);
        }
        DomainEvent::ModifierApplied {
            target_id,
            modifier_id,
            duration,
        } => {
            encoder.string("modifierApplied");
            encoder.string(target_id);
            encoder.string(modifier_id);
            encoder.string(duration);
        }
        DomainEvent::EffectMovementApplied {
            target_id,
            movement_kind,
            from,
            to,
        } => {
            encoder.string("effectMovementApplied");
            encoder.string(target_id);
            encoder.string(movement_kind.code());
            encoder.position(*from);
            encoder.position(*to);
        }
        DomainEvent::ResourceChanged {
            target_id,
            resource_id,
            delta,
            before,
            after,
        } => {
            encoder.string("resourceChanged");
            encoder.string(target_id);
            encoder.string(resource_id);
            encoder.i32(*delta);
            encoder.i32(*before);
            encoder.i32(*after);
        }
        DomainEvent::PositionChanged { actor_id, from, to } => {
            encoder.string("positionChanged");
            encoder.string(actor_id);
            encoder.position(*from);
            encoder.position(*to);
        }
        DomainEvent::MovementSpent {
            actor_id,
            amount,
            remaining,
        } => {
            encoder.string("movementSpent");
            encoder.string(actor_id);
            encoder.u32(*amount);
            encoder.u32(*remaining);
        }
    }
}

fn feed_audit(encoder: &mut CanonicalEncoder, audit: &CommandAuditEntry) {
    encoder.string(&audit.id);
    encoder.string(&audit.step_id);
    encoder.u32(audit.sequence);
    encoder.string(audit.outcome_class.code());
    encoder.string(audit.decision_kind.code());
    encoder.optional_code(audit.preflight_decision_kind.map(|value| value.code()));
    encoder.bool(audit.accepted);
    encoder.optional_code(audit.rejection.map(|value| value.code()));
    encoder.u32(audit.event_count);
    encoder.u32(audit.trace_count);
    encoder.sequence(&audit.roll_consumption, feed_roll);
    feed_state_fingerprint(encoder, &audit.state_before_fingerprint);
    feed_state_fingerprint(encoder, &audit.state_after_fingerprint);
}

fn feed_roll(encoder: &mut CanonicalEncoder, roll: &RollConsumptionEntry) {
    encoder.u32(roll.sequence);
    encoder.string(roll.request_kind.code());
    encoder.optional_i32(roll.supplied_value);
    encoder.bool(roll.consumed);
    encoder.string(&roll.reason);
}

fn feed_trace(encoder: &mut CanonicalEncoder, trace: &TraceEntry) {
    encoder.u32(trace.sequence);
    encoder.string(match trace.phase {
        TracePhase::Proposal => "proposal",
        TracePhase::Validation => "validation",
        TracePhase::Resolution => "resolution",
        TracePhase::Commit => "commit",
    });
    encoder.string(match trace.status {
        TraceStatus::Accepted => "accepted",
        TraceStatus::Rejected => "rejected",
        TraceStatus::Info => "info",
    });
    encoder.string(&trace.message);
    encoder.string(&trace.detail);
}

fn feed_randomness(encoder: &mut CanonicalEncoder, value: &ReplayCommandRandomnessProvenance) {
    encoder.u32(value.command_sequence);
    encoder.string(&value.source_id);
    match &value.source {
        ReplayRandomnessSource::Supplied => encoder.string("supplied"),
        ReplayRandomnessSource::Generated {
            seed,
            algorithm_version,
        } => {
            encoder.string("generated");
            encoder.u64(*seed);
            encoder.string(algorithm_version);
        }
    }
    encoder.i32s(&value.supplied_values);
    encoder.sequence(&value.generated_requests, |encoder, request| {
        encoder.u32(request.sequence);
        encoder.string(&request.request_id);
        encoder.string(request.request_kind.code());
        encoder.i32(request.minimum);
        encoder.i32(request.maximum);
        encoder.i32(request.value);
    });
    encoder.sequence(&value.consumption, feed_roll);
    encoder.i32s(&value.unused_values);
}

struct CanonicalEncoder {
    bytes: Vec<u8>,
}

impl CanonicalEncoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }
    fn finish(self) -> Vec<u8> {
        self.bytes
    }
    fn raw(&mut self, bytes: &[u8]) {
        self.bytes
            .extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        self.bytes.extend_from_slice(bytes);
    }
    fn string(&mut self, value: &str) {
        self.raw(value.as_bytes());
    }
    fn bool(&mut self, value: bool) {
        self.bytes.push(u8::from(value));
    }
    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn sequence<T>(&mut self, values: &[T], feed: impl Fn(&mut Self, &T)) {
        self.u64(values.len() as u64);
        for value in values {
            feed(self, value);
        }
    }
    fn strings(&mut self, values: &[String]) {
        self.sequence(values, |encoder, value| encoder.string(value));
    }
    fn i32s(&mut self, values: &[i32]) {
        self.sequence(values, |encoder, value| encoder.i32(*value));
    }
    fn optional_string(&mut self, value: Option<&str>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.string(value);
        }
    }
    fn optional<T>(&mut self, value: Option<&T>, feed: impl Fn(&mut Self, &T)) {
        self.bool(value.is_some());
        if let Some(value) = value {
            feed(self, value);
        }
    }
    fn optional_code(&mut self, value: Option<&str>) {
        self.optional_string(value);
    }
    fn optional_i32(&mut self, value: Option<i32>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.i32(value);
        }
    }
    fn optional_u32(&mut self, value: Option<u32>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.u32(value);
        }
    }
    fn position(&mut self, value: rulebench_combat::GridPosition) {
        self.u32(value.x);
        self.u32(value.y);
    }
    fn optional_position(&mut self, value: Option<rulebench_combat::GridPosition>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.position(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use rulebench_combat::{
        CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec,
        CombatSessionCandidateSelectionSpec, CombatSessionIntentCommandSpec, CommandAuditEntry,
        CommandDecisionKind, CommandOutcomeClass, EquipmentCommandSpec, GridPosition,
        HitEffectOperation, ReactionCommandSpec, ResourceChangeEffectOperation,
        RollConsumptionEntry, RollRequestKind, UseActionIntent,
    };

    use super::*;
    use crate::verification::tests::recorded_control_package;
    use crate::{
        ReplayAcceptedEvents, ReplayCommandRandomnessProvenance, ReplayNarration,
        ReplayRollEvidence, ReplayTraceEvidence,
    };

    #[test]
    fn canonical_payload_has_a_stable_golden_identity_independent_of_debug_output() {
        let entry = ReplayArchiveEntry::new(recorded_control_package(), "canonical-golden");

        assert_eq!(
            canonical_replay_archive_payload_fingerprint(&entry),
            "c61f4e498a686aec"
        );
        assert_eq!(
            entry.payload_fingerprint,
            canonical_replay_archive_payload_fingerprint(&entry)
        );
        assert!(entry.is_self_consistent());
    }

    #[test]
    fn canonical_payload_mutations_cover_archive_scenario_commands_and_evidence() {
        let entry = ReplayArchiveEntry::new(recorded_control_package(), "canonical-mutations");
        let expected = entry.payload_fingerprint.clone();

        assert_changed(&entry, &expected, |value| {
            value.metadata.completed_at.push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.package_version.push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .session
                .id
                .push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .scenario
                .metadata
                .title
                .push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .scenario
                .content_pack_set
                .as_mut()
                .expect("test package has content provenance")
                .fingerprint
                .value
                .push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.initial_session.scenario.grid.width += 1
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .scenario
                .selected_action
                .targeting
                .target_ids
                .push("second-target".to_string())
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .scenario
                .selected_action
                .resource_costs[0]
                .amount += 1
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .initial_session
                .scenario
                .selected_action
                .hit
                .operations
                .push(HitEffectOperation::ChangeResource(
                    ResourceChangeEffectOperation {
                        resource_id: "charge".to_string(),
                        delta: -1,
                    },
                ))
        });
        assert_changed(&entry, &expected, |value| {
            value.package.ruleset.ruleset_version.push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.commands[0].id.push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.commands[0]
                .expected
                .decision_code
                .push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .evidence
                .accepted_events
                .push(ReplayAcceptedEvents {
                    command_sequence: 99,
                    events: value.package.commands[0].expected.accepted_events.clone(),
                })
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .evidence
                .command_audit
                .push(CommandAuditEntry {
                    id: "audit".to_string(),
                    step_id: "step".to_string(),
                    sequence: 99,
                    outcome_class: CommandOutcomeClass::AcceptedHit,
                    decision_kind: CommandDecisionKind::AcceptedByResolver,
                    preflight_decision_kind: None,
                    accepted: true,
                    rejection: None,
                    event_count: 0,
                    trace_count: 0,
                    roll_consumption: Vec::new(),
                    state_before_fingerprint: value.package.commands[0]
                        .expected
                        .state_before_fingerprint
                        .clone(),
                    state_after_fingerprint: value.package.commands[0]
                        .expected
                        .state_after_fingerprint
                        .clone(),
                })
        });
        assert_changed(&entry, &expected, |value| {
            value.package.evidence.rolls.push(ReplayRollEvidence {
                command_sequence: 99,
                consumption: vec![RollConsumptionEntry {
                    sequence: 99,
                    request_kind: RollRequestKind::DamageRoll,
                    supplied_value: Some(4),
                    consumed: true,
                    reason: "canonical mutation".to_string(),
                }],
            })
        });
        assert_changed(&entry, &expected, |value| {
            value.package.evidence.trace.push(ReplayTraceEvidence {
                command_sequence: 99,
                entries: value.package.commands[0].expected.trace.clone(),
            })
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .evidence
                .randomness
                .push(ReplayCommandRandomnessProvenance::supplied(
                    0,
                    "canonical-mutation",
                    vec![4],
                    Vec::new(),
                ))
        });
        assert_changed(&entry, &expected, |value| {
            value
                .package
                .final_state_fingerprint
                .value
                .push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.fingerprint_kind.push_str("-changed")
        });
        assert_changed(&entry, &expected, |value| {
            value.package.narration = Some(ReplayNarration {
                title: "Canonical".to_string(),
                summary: "Presentation copy is still integrity protected.".to_string(),
                command_summaries: vec!["Started".to_string()],
            })
        });
    }

    #[test]
    fn canonical_payload_distinguishes_every_typed_command_shape_and_roll_mode() {
        let mut intent = UseActionIntent::for_targets(
            "actor",
            "action",
            vec!["target-b".to_string(), "target-a".to_string()],
        );
        intent.target_cell = Some(GridPosition { x: 2, y: 3 });
        intent.destination_cell = Some(GridPosition { x: 4, y: 5 });
        intent.observed_origin = Some(GridPosition { x: 0, y: 1 });
        let commands = vec![
            ReplayCommand::Control(rulebench_combat::CombatControlCommandSpec::explicit_start()),
            ReplayCommand::Intent(
                CombatSessionIntentCommandSpec::new(
                    "intent",
                    "Intent",
                    "Typed intent",
                    intent,
                    vec![],
                )
                .with_generated_rolls(41),
            ),
            ReplayCommand::SelectedCandidate(
                CombatSessionCandidateSelectionSpec::new(
                    "candidate",
                    "Candidate",
                    "Typed candidate",
                    "action",
                    "target",
                    vec![],
                )
                .with_generated_rolls(42),
            ),
            ReplayCommand::AutomaticStep(
                CombatSessionAutomaticStepSpec::new(
                    "automatic-step",
                    "Automatic step",
                    "Typed automatic step",
                    vec![],
                )
                .with_generated_rolls(43),
            ),
            ReplayCommand::AutomaticRun(
                CombatSessionAutomaticRunSpec::new(
                    "automatic-run",
                    "Automatic run",
                    "Typed automatic run",
                    7,
                    vec![],
                )
                .with_generated_rolls(44),
            ),
            ReplayCommand::Equipment(EquipmentCommandSpec::equip("actor", "item")),
            ReplayCommand::Reaction(ReactionCommandSpec::accept("window", "reactor", "option")),
        ];
        let base = ReplayArchiveEntry::new(recorded_control_package(), "command-shapes");
        let identities = commands
            .into_iter()
            .map(|command| {
                let mut entry = base.clone();
                entry.package.commands[0].command = command;
                canonical_replay_archive_payload_fingerprint(&entry)
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(identities.len(), 7);
    }

    fn assert_changed(
        entry: &ReplayArchiveEntry,
        expected: &str,
        mutation: impl FnOnce(&mut ReplayArchiveEntry),
    ) {
        let mut changed = entry.clone();
        mutation(&mut changed);
        assert_ne!(
            canonical_replay_archive_payload_fingerprint(&changed),
            expected
        );
    }
}
