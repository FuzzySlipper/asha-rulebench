use rpg_core::Team;
use rpg_ir::{CheckDeclaration, VisibilityRequirement};
use rulebench_combat::{
    AttackOutcome, DomainEvent, FinalCombatantState, ScenarioProjection, TargetLegality,
    TraceEntry, TracePhase, TraceStatus,
};
use rulebench_content::{Combatant, RulebenchScenario};

pub(crate) fn render_scenario_readout(
    scenario: &RulebenchScenario,
    events: &[DomainEvent],
    trace: &[TraceEntry],
    final_state: &ScenarioProjection,
    target: &TargetLegality,
    indent: &str,
) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "{indent}  id: {},\n",
        ts_string(&scenario.metadata.id)
    ));
    out.push_str(&format!(
        "{indent}  title: {},\n",
        ts_string(&scenario.metadata.title)
    ));
    out.push_str(&format!(
        "{indent}  summary: {},\n",
        ts_string(&scenario.metadata.summary)
    ));
    out.push_str(&format!(
        "{indent}  seedLabel: {},\n",
        ts_string(&scenario.metadata.seed_label)
    ));
    out.push_str(&render_grid(scenario, indent));
    out.push_str(&render_combatants(
        scenario,
        &final_state.combatants,
        indent,
    ));
    out.push_str(&render_action(scenario, indent));
    out.push_str(&format!("{indent}  selectedTarget: {{\n"));
    out.push_str(&format!(
        "{indent}    targetId: {},\n",
        ts_string(&target.target_id)
    ));
    out.push_str(&format!(
        "{indent}    legality: {},\n",
        ts_string(if target.accepted {
            "accepted"
        } else {
            "rejected"
        })
    ));
    out.push_str(&format!(
        "{indent}    reason: {},\n",
        ts_string(&target.reason)
    ));
    out.push_str(&format!("{indent}  }},\n"));
    out.push_str(&format!("{indent}  domainEvents: [\n"));
    for (index, event) in events.iter().enumerate() {
        out.push_str(&render_event((index + 1) as u32, event, indent));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&format!("{indent}  trace: [\n"));
    for entry in trace {
        out.push_str(&format!("{indent}    {{\n"));
        out.push_str(&format!("{indent}      sequence: {},\n", entry.sequence));
        out.push_str(&format!(
            "{indent}      phase: {},\n",
            ts_string(phase(entry.phase))
        ));
        out.push_str(&format!(
            "{indent}      status: {},\n",
            ts_string(status(entry.status))
        ));
        out.push_str(&format!(
            "{indent}      message: {},\n",
            ts_string(&entry.message)
        ));
        out.push_str(&format!(
            "{indent}      detail: {},\n",
            ts_string(&entry.detail)
        ));
        out.push_str(&format!("{indent}    }},\n"));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out.push_str(&render_final_state(final_state, indent));
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn render_final_state(final_state: &ScenarioProjection, indent: &str) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}  finalState: {{\n"));
    out.push_str(&format!(
        "{indent}    summary: {},\n",
        ts_string(&final_state.summary)
    ));
    out.push_str(&format!("{indent}    combatants: [\n"));
    for combatant in &final_state.combatants {
        out.push_str(&render_final_combatant(combatant, indent));
    }
    out.push_str(&format!("{indent}    ],\n"));
    out.push_str(&format!("{indent}  }},\n"));
    out
}

pub(crate) fn ts_string(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

pub(crate) fn ts_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| ts_string(value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn render_grid(scenario: &RulebenchScenario, indent: &str) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}  grid: {{\n"));
    out.push_str(&format!("{indent}    width: {},\n", scenario.grid.width));
    out.push_str(&format!("{indent}    height: {},\n", scenario.grid.height));
    out.push_str(&format!("{indent}    cells: [\n"));
    for cell in &scenario.grid.cells {
        out.push_str(&format!(
            "{indent}      {{ x: {}, y: {}, terrainTags: {} }},\n",
            cell.position.x,
            cell.position.y,
            ts_string_array(&cell.terrain_tags)
        ));
    }
    out.push_str(&format!("{indent}    ],\n"));
    out.push_str(&format!("{indent}  }},\n"));
    out
}

fn render_combatants(
    scenario: &RulebenchScenario,
    final_states: &[FinalCombatantState],
    indent: &str,
) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}  combatants: [\n"));
    for combatant in &scenario.combatants {
        let final_state = final_states
            .iter()
            .find(|state| state.id == combatant.id)
            .expect("readout final state has combatant");
        out.push_str(&render_combatant(combatant, final_state, indent));
    }
    out.push_str(&format!("{indent}  ],\n"));
    out
}

fn render_combatant(
    combatant: &Combatant,
    final_state: &FinalCombatantState,
    indent: &str,
) -> String {
    let mut out = String::from("");
    out.push_str(&format!("{indent}    {{\n"));
    out.push_str(&format!(
        "{indent}      id: {},\n",
        ts_string(&combatant.id)
    ));
    out.push_str(&format!(
        "{indent}      name: {},\n",
        ts_string(&combatant.name)
    ));
    out.push_str(&format!(
        "{indent}      team: {},\n",
        ts_string(match combatant.team {
            Team::Ally => "ally",
            Team::Enemy => "enemy",
        })
    ));
    out.push_str(&format!(
        "{indent}      sideId: {},\n",
        ts_string(&combatant.side_id)
    ));
    out.push_str(&format!(
        "{indent}      position: {{ x: {}, y: {} }},\n",
        combatant.position.x, combatant.position.y
    ));
    out.push_str(&format!(
        "{indent}      hitPoints: {{ current: {}, max: {} }},\n",
        final_state.hit_points.current, final_state.hit_points.max
    ));
    out.push_str(&format!("{indent}      defenses: [\n"));
    for defense in &combatant.defenses {
        out.push_str(&format!(
            "{indent}        {{ id: {}, label: {}, value: {} }},\n",
            ts_string(&defense.id),
            ts_string(&defense.label),
            defense.value
        ));
    }
    out.push_str(&format!("{indent}      ],\n"));
    out.push_str(&format!(
        "{indent}      conditions: {},\n",
        ts_string_array(&final_state.conditions)
    ));
    out.push_str(&format!("{indent}      isActor: {},\n", combatant.is_actor));
    out.push_str(&format!("{indent}    }},\n"));
    out
}

fn render_action(scenario: &RulebenchScenario, indent: &str) -> String {
    let action = &scenario.selected_action;
    let mut out = String::from("");
    out.push_str(&format!("{indent}  selectedAction: {{\n"));
    out.push_str(&format!("{indent}    id: {},\n", ts_string(&action.id)));
    out.push_str(&format!("{indent}    name: {},\n", ts_string(&action.name)));
    out.push_str(&format!(
        "{indent}    actorId: {},\n",
        ts_string(&action.actor_id)
    ));
    out.push_str(&format!(
        "{indent}    targetIds: {},\n",
        ts_string_array(&action.targeting.target_ids)
    ));
    out.push_str(&format!(
        "{indent}    range: {},\n",
        action.targeting.maximum_range
    ));
    out.push_str(&format!(
        "{indent}    lineOfSightRequired: {},\n",
        action.targeting.visibility_requirement == VisibilityRequirement::Required
    ));
    out.push_str(&format!(
        "{indent}    visibleTargetIds: {},\n",
        ts_string_array(&action.targeting.visible_target_ids)
    ));
    match &action.check {
        CheckDeclaration::Attack(attack) => {
            out.push_str(&format!("{indent}    attack: {{\n"));
            out.push_str(&format!("{indent}      modifier: {},\n", attack.modifier));
            out.push_str(&format!(
                "{indent}      defenseId: {},\n",
                ts_string(&attack.defense.id)
            ));
            out.push_str(&format!(
                "{indent}      defenseLabel: {},\n",
                ts_string(&attack.defense.label)
            ));
            out.push_str(&format!("{indent}    }},\n"));
            out.push_str(&format!("{indent}    savingThrow: null,\n"));
            out.push_str(&format!("{indent}    contested: null,\n"));
        }
        CheckDeclaration::SavingThrow(save) => {
            out.push_str(&format!("{indent}    attack: null,\n"));
            out.push_str(&format!("{indent}    savingThrow: {{\n"));
            out.push_str(&format!(
                "{indent}      saveStatId: {},\n",
                ts_string(&save.save_stat_id)
            ));
            out.push_str(&format!(
                "{indent}      difficultyClass: {},\n",
                save.difficulty_class
            ));
            out.push_str(&format!("{indent}    }},\n"));
            out.push_str(&format!("{indent}    contested: null,\n"));
        }
        CheckDeclaration::Contested(contested) => {
            out.push_str(&format!("{indent}    attack: null,\n"));
            out.push_str(&format!("{indent}    savingThrow: null,\n"));
            out.push_str(&format!("{indent}    contested: {{\n"));
            out.push_str(&format!(
                "{indent}      actorStatId: {},\n",
                ts_string(&contested.actor_stat_id)
            ));
            out.push_str(&format!(
                "{indent}      targetStatId: {},\n",
                ts_string(&contested.target_stat_id)
            ));
            out.push_str(&format!("{indent}    }},\n"));
        }
    }
    out.push_str(&format!("{indent}    hit: {{\n"));
    out.push_str(&format!(
        "{indent}      damageBonus: {},\n",
        action.hit.damage_bonus
    ));
    out.push_str(&format!(
        "{indent}      damageType: {},\n",
        ts_string(&action.hit.damage_type)
    ));
    out.push_str(&format!(
        "{indent}      modifierId: {},\n",
        ts_string(&action.hit.modifier_id)
    ));
    out.push_str(&format!(
        "{indent}      modifierLabel: {},\n",
        ts_string(&action.hit.modifier_label)
    ));
    out.push_str(&format!(
        "{indent}      modifierDuration: {},\n",
        ts_string(&action.hit.modifier_duration)
    ));
    out.push_str(&format!("{indent}    }},\n"));
    out.push_str(&format!(
        "{indent}    actionText: {},\n",
        ts_string(&action.action_text)
    ));
    out.push_str(&format!(
        "{indent}    effectText: {},\n",
        ts_string(&action.effect_text)
    ));
    out.push_str(&format!("{indent}  }},\n"));
    out
}

fn render_event(sequence: u32, event: &DomainEvent, indent: &str) -> String {
    match event {
        DomainEvent::ActionUsed {
            actor_id,
            target_id,
            ..
        } => event_block(
            sequence,
            "ActionUsed",
            "Adept used Hexing Bolt against Raider.",
            &[actor_id.as_str(), target_id.as_str()],
            indent,
        ),
        DomainEvent::AttackRolled {
            actor_id,
            target_id,
            total,
            defense_value,
            outcome,
            ..
        } => {
            let attack_modifier = 4;
            let attack_roll = total - attack_modifier;
            event_block(
                sequence,
                "AttackRolled",
                &format!(
                    "Attack rolled {attack_roll} + {attack_modifier} vs Nerve {defense_value}: {}.",
                    match outcome {
                        AttackOutcome::Hit => "hit",
                        AttackOutcome::Miss => "miss",
                    }
                ),
                &[actor_id.as_str(), target_id.as_str()],
                indent,
            )
        }
        DomainEvent::SavingThrowResolved {
            actor_id,
            target_id,
            total,
            difficulty_class,
            outcome,
        } => event_block(
            sequence,
            "SavingThrowResolved",
            &format!(
                "Saving throw total {total} against DC {difficulty_class}: {}.",
                match outcome {
                    rulebench_combat::SavingThrowOutcome::Saved => "saved",
                    rulebench_combat::SavingThrowOutcome::Failed => "failed",
                }
            ),
            &[actor_id.as_str(), target_id.as_str()],
            indent,
        ),
        DomainEvent::ContestedCheckResolved {
            actor_id,
            target_id,
            actor_total,
            target_total,
            outcome,
        } => event_block(
            sequence,
            "ContestedCheckResolved",
            &format!(
                "Contested totals {actor_total} versus {target_total}: {}.",
                match outcome {
                    rulebench_combat::ContestedCheckOutcome::ActorWins => "actor wins",
                    rulebench_combat::ContestedCheckOutcome::TargetWins => "target wins",
                }
            ),
            &[actor_id.as_str(), target_id.as_str()],
            indent,
        ),
        DomainEvent::DamageApplied {
            target_id, amount, ..
        } => event_block(
            sequence,
            "DamageApplied",
            &format!("Raider took {amount} psychic damage."),
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::HealingApplied {
            target_id, amount, ..
        } => event_block(
            sequence,
            "HealingApplied",
            &format!("Raider recovered {amount} vitality."),
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::TemporaryVitalityGranted {
            target_id, amount, ..
        } => event_block(
            sequence,
            "TemporaryVitalityGranted",
            &format!("Raider gained {amount} temporary vitality."),
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::ModifierApplied { target_id, .. } => event_block(
            sequence,
            "ModifierApplied",
            "Raider became rattled until end of next turn.",
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::PositionChanged { actor_id, from, to } => event_block(
            sequence,
            "PositionChanged",
            &format!(
                "{actor_id} moved from {},{} to {},{}.",
                from.x, from.y, to.x, to.y
            ),
            &[actor_id.as_str()],
            indent,
        ),
        DomainEvent::MovementSpent {
            actor_id,
            amount,
            remaining,
        } => event_block(
            sequence,
            "MovementSpent",
            &format!("{actor_id} spent {amount} movement; {remaining} remains."),
            &[actor_id.as_str()],
            indent,
        ),
        DomainEvent::EffectMovementApplied {
            target_id,
            movement_kind,
            from,
            to,
        } => event_block(
            sequence,
            "EffectMovementApplied",
            &format!(
                "{target_id} resolved {} movement from {},{} to {},{}.",
                movement_kind.code(),
                from.x,
                from.y,
                to.x,
                to.y
            ),
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::ResourceChanged {
            target_id,
            resource_id,
            delta,
            before,
            after,
        } => event_block(
            sequence,
            "ResourceChanged",
            &format!("{target_id} changed {resource_id} by {delta} from {before} to {after}."),
            &[target_id.as_str()],
            indent,
        ),
        DomainEvent::IntentShapeAccepted { .. } => String::new(),
    }
}

fn event_block(
    sequence: u32,
    event_type: &str,
    summary: &str,
    entity_ids: &[&str],
    indent: &str,
) -> String {
    let ids = entity_ids
        .iter()
        .map(|id| ts_string(id))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{indent}    {{\n{indent}      sequence: {sequence},\n{indent}      type: {},\n{indent}      summary: {},\n{indent}      entityIds: [{ids}],\n{indent}    }},\n",
        ts_string(event_type),
        ts_string(summary)
    )
}

fn render_final_combatant(combatant: &FinalCombatantState, indent: &str) -> String {
    format!(
        "{indent}      {{\n{indent}        id: {},\n{indent}        name: {},\n{indent}        hitPoints: {{ current: {}, max: {} }},\n{indent}        conditions: {},\n{indent}      }},\n",
        ts_string(&combatant.id),
        ts_string(&combatant.name),
        combatant.hit_points.current,
        combatant.hit_points.max,
        ts_string_array(&combatant.conditions)
    )
}

fn phase(phase: TracePhase) -> &'static str {
    match phase {
        TracePhase::Proposal => "proposal",
        TracePhase::Validation => "validation",
        TracePhase::Resolution => "resolution",
        TracePhase::Commit => "commit",
    }
}

fn status(status: TraceStatus) -> &'static str {
    match status {
        TraceStatus::Accepted => "accepted",
        TraceStatus::Rejected => "rejected",
        TraceStatus::Info => "info",
    }
}
