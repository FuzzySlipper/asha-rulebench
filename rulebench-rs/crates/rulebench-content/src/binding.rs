use std::collections::BTreeSet;

use rulebench_ruleset::{
    EffectOperationId, HitEffect, HitEffectOperation, ModifierEffectOperation, OperationPipelineV2,
    ReactionHookEffectOperation, ReactionOptionDeclaration, TargetKind, TargetTeamConstraint,
    TargetingDeclaration,
};

use crate::{
    fingerprint_authored_action, validate_scenario_content_report, AuthoredActionDefinition,
    AuthoredEffectOperation, AuthoredReactionHookEffectOperation, CanonicalContentPack,
    ContentFingerprint, ContentPackReference, ContentPackSetReference, ImportedContentPack,
    ModifierDefinition, ReactionParticipantSelector, RulebenchScenario,
};

pub const AUTHORED_ACTION_BINDING_VERSION: &str = "1";
pub const AUTHORED_ACTION_CHECK_VOCABULARY_VERSION: &str = "1";
pub const AUTHORED_ACTION_REACTION_EXPANSION_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredActionBindingRequest {
    pub content_pack: ContentPackReference,
    pub action_id: String,
    pub actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredActionAbilityGrantReceipt {
    pub actor_id: String,
    pub ability_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredActionBindingReceipt {
    pub binding_version: String,
    pub content_pack_set: ContentPackSetReference,
    pub action_id: String,
    pub action_definition_fingerprint: ContentFingerprint,
    pub ability_id: String,
    pub scenario_id: String,
    pub actor_id: String,
    pub grant: AuthoredActionAbilityGrantReceipt,
    pub targeting_operation_vocabulary_version: String,
    pub check_vocabulary_version: String,
    pub effect_operation_vocabulary_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredActionBindingError {
    pub code: &'static str,
    pub reference_id: Option<String>,
    pub message: String,
    pub diagnostic_codes: Vec<String>,
}

pub fn bind_authored_action(
    mut scenario: RulebenchScenario,
    imported: &ImportedContentPack,
    request: &AuthoredActionBindingRequest,
) -> Result<RulebenchScenario, AuthoredActionBindingError> {
    if request.content_pack != imported.resolved_set.reference.root {
        return Err(rejected(
            "staleAuthoredActionPack",
            Some(request.content_pack.id.clone()),
            "The authored-action binding root does not match the exact resolved pack set.",
        ));
    }
    if scenario.authored_action_binding.is_some() {
        return Err(rejected(
            "duplicateAuthoredActionBinding",
            Some(scenario.metadata.id.clone()),
            "The scenario already has an authored-action binding receipt.",
        ));
    }
    let selected_ruleset = scenario.selected_ruleset().ok_or_else(|| {
        rejected(
            "missingAuthoredActionScenarioRuleset",
            Some(scenario.selected_ruleset_id.clone()),
            "The selected scenario ruleset does not exist.",
        )
    })?;
    if selected_ruleset.artifact_provenance() != imported.pack.ruleset {
        return Err(rejected(
            "incompatibleAuthoredActionRuleset",
            Some(imported.pack.ruleset.ruleset_id.clone()),
            "The authored content provider provenance does not match the selected scenario ruleset.",
        ));
    }

    let action = unique_action(&imported.resolved_set.packs, &request.action_id)?;
    let ability = unique_ability(&imported.resolved_set.packs, &action.ability_id)?;
    let actor_index = scenario
        .combatants
        .iter()
        .position(|combatant| combatant.id == request.actor_id)
        .ok_or_else(|| {
            rejected(
                "unknownAuthoredActionActor",
                Some(request.actor_id.clone()),
                "The authored-action binding actor is not present in the selected scenario.",
            )
        })?;

    validate_resource_pools(&scenario, actor_index, action)?;
    merge_ability(&mut scenario, ability);
    for modifier_id in referenced_modifier_ids(action) {
        let modifier = unique_modifier(&imported.resolved_set.packs, &modifier_id)?;
        merge_modifier(&mut scenario, modifier)?;
    }

    let target_ids = derive_target_ids(&scenario, actor_index, action);
    if target_ids.is_empty() && !is_cell_movement(action) {
        return Err(rejected(
            "authoredActionTargetExhausted",
            Some(action.id.clone()),
            "The authored action has no legal target candidates in the selected scenario.",
        ));
    }
    let operations = materialize_operations(&scenario, actor_index, action, &target_ids)?;
    let runtime_action = materialize_action(
        &scenario.selected_ruleset_id,
        &request.actor_id,
        action,
        target_ids,
        operations,
    );
    if scenario
        .actions
        .iter()
        .any(|existing| existing.id == runtime_action.id)
    {
        return Err(rejected(
            "authoredActionRuntimeCollision",
            Some(runtime_action.id.clone()),
            "The authored action id collides with an existing scenario action.",
        ));
    }

    let actor = &mut scenario.combatants[actor_index];
    if !actor.base_ability_ids.contains(&ability.id) {
        actor.base_ability_ids.push(ability.id.clone());
        actor.base_ability_ids.sort();
    }
    scenario.selected_ability_id = Some(ability.id.clone());
    scenario.selected_action = runtime_action.clone();
    scenario.actions.push(runtime_action);
    scenario
        .actions
        .sort_by(|left, right| left.id.cmp(&right.id));
    scenario.content_pack_set = Some(imported.resolved_set.reference.clone());
    scenario.authored_action_binding = Some(AuthoredActionBindingReceipt {
        binding_version: AUTHORED_ACTION_BINDING_VERSION.to_string(),
        content_pack_set: imported.resolved_set.reference.clone(),
        action_id: action.id.clone(),
        action_definition_fingerprint: fingerprint_authored_action(action),
        ability_id: ability.id.clone(),
        scenario_id: scenario.metadata.id.clone(),
        actor_id: request.actor_id.clone(),
        grant: AuthoredActionAbilityGrantReceipt {
            actor_id: request.actor_id.clone(),
            ability_id: ability.id.clone(),
        },
        targeting_operation_vocabulary_version: OperationPipelineV2::VOCABULARY_VERSION.to_string(),
        check_vocabulary_version: AUTHORED_ACTION_CHECK_VOCABULARY_VERSION.to_string(),
        effect_operation_vocabulary_version: EffectOperationId::VOCABULARY_VERSION.to_string(),
    });

    let report = validate_scenario_content_report(&scenario);
    if !report.accepted {
        return Err(AuthoredActionBindingError {
            code: "invalidAuthoredActionScenarioComposition",
            reference_id: Some(action.id.clone()),
            message: "The materialized authored action failed complete Rust scenario validation."
                .to_string(),
            diagnostic_codes: report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.code().to_string())
                .collect(),
        });
    }
    Ok(scenario)
}

fn unique_action<'a>(
    packs: &'a [CanonicalContentPack],
    action_id: &str,
) -> Result<&'a AuthoredActionDefinition, AuthoredActionBindingError> {
    let matches = packs
        .iter()
        .flat_map(|pack| &pack.catalogs.actions)
        .filter(|action| action.id == action_id)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [action] => Ok(*action),
        [] => Err(rejected(
            "unknownAuthoredAction",
            Some(action_id.to_string()),
            "The exact resolved pack set does not contain the requested authored action.",
        )),
        _ => Err(rejected(
            "collidedAuthoredAction",
            Some(action_id.to_string()),
            "The exact resolved pack set contains more than one authored action with this id.",
        )),
    }
}

fn unique_ability<'a>(
    packs: &'a [CanonicalContentPack],
    ability_id: &str,
) -> Result<&'a rulebench_ruleset::AbilityDefinition, AuthoredActionBindingError> {
    let matches = packs
        .iter()
        .flat_map(|pack| &pack.catalogs.abilities)
        .filter(|ability| ability.id == ability_id)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [ability] => Ok(*ability),
        [] => Err(rejected(
            "unknownAuthoredActionAbility",
            Some(ability_id.to_string()),
            "The exact resolved pack set does not contain the authored action ability.",
        )),
        _ => Err(rejected(
            "collidedAuthoredActionAbility",
            Some(ability_id.to_string()),
            "The exact resolved pack set contains more than one ability with this id.",
        )),
    }
}

fn unique_modifier<'a>(
    packs: &'a [CanonicalContentPack],
    modifier_id: &str,
) -> Result<&'a ModifierDefinition, AuthoredActionBindingError> {
    let matches = packs
        .iter()
        .flat_map(|pack| &pack.catalogs.modifiers)
        .filter(|modifier| modifier.id == modifier_id)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [modifier] => Ok(*modifier),
        [] => Err(rejected(
            "unknownAuthoredActionModifier",
            Some(modifier_id.to_string()),
            "The exact resolved pack set does not contain an applied authored modifier.",
        )),
        _ => Err(rejected(
            "collidedAuthoredActionModifier",
            Some(modifier_id.to_string()),
            "The exact resolved pack set contains more than one modifier with this id.",
        )),
    }
}

fn merge_ability(scenario: &mut RulebenchScenario, ability: &rulebench_ruleset::AbilityDefinition) {
    // The selected authored definition is authoritative for its ability id.
    // Replacing a same-id scenario placeholder keeps the materialized action
    // generic while the complete scenario validation below remains fail-closed.
    if let Some(existing) = scenario
        .abilities
        .iter_mut()
        .find(|existing| existing.id == ability.id)
    {
        *existing = ability.clone();
    } else {
        scenario.abilities.push(ability.clone());
    }
    scenario
        .abilities
        .sort_by(|left, right| left.id.cmp(&right.id));
}

fn merge_modifier(
    scenario: &mut RulebenchScenario,
    modifier: &ModifierDefinition,
) -> Result<(), AuthoredActionBindingError> {
    match scenario.modifier_by_id(&modifier.id) {
        Some(existing) if existing == modifier => Ok(()),
        Some(_) => Err(rejected(
            "authoredActionModifierCollision",
            Some(modifier.id.clone()),
            "The authored modifier conflicts with the selected scenario modifier catalog.",
        )),
        None => {
            scenario.modifiers.push(modifier.clone());
            scenario
                .modifiers
                .sort_by(|left, right| left.id.cmp(&right.id));
            Ok(())
        }
    }
}

fn validate_resource_pools(
    scenario: &RulebenchScenario,
    actor_index: usize,
    action: &AuthoredActionDefinition,
) -> Result<(), AuthoredActionBindingError> {
    let actor = &scenario.combatants[actor_index];
    for cost in &action.resource_costs {
        let pool_exists = actor
            .resource_pools
            .iter()
            .any(|pool| pool.id == cost.resource_id)
            || actor.inventory_item_ids.iter().any(|item_id| {
                scenario.item_by_id(item_id).is_some_and(|item| {
                    item.granted_resource_pools
                        .iter()
                        .any(|pool| pool.id == cost.resource_id)
                })
            })
            || actor.class_inputs.iter().any(|input| {
                scenario.class_by_id(&input.class_id).is_some_and(|class| {
                    class
                        .level_grants
                        .iter()
                        .filter(|grant| grant.level <= input.level)
                        .flat_map(|grant| &grant.granted_resource_pools)
                        .any(|pool| pool.id == cost.resource_id)
                })
            });
        if !pool_exists {
            return Err(rejected(
                "missingAuthoredActionResourcePool",
                Some(cost.resource_id.clone()),
                "The selected actor does not have a resource pool required by the authored action.",
            ));
        }
    }
    Ok(())
}

fn referenced_modifier_ids(action: &AuthoredActionDefinition) -> BTreeSet<String> {
    action
        .effects
        .iter()
        .filter_map(|operation| match operation {
            AuthoredEffectOperation::ApplyModifier(modifier) => Some(modifier.modifier_id.clone()),
            _ => None,
        })
        .collect()
}

fn derive_target_ids(
    scenario: &RulebenchScenario,
    actor_index: usize,
    action: &AuthoredActionDefinition,
) -> Vec<String> {
    let actor = &scenario.combatants[actor_index];
    let mut targets = scenario
        .combatants
        .iter()
        .filter(|target| match action.targeting.team_constraint {
            TargetTeamConstraint::Hostile => target.team != actor.team,
            TargetTeamConstraint::Ally => target.team == actor.team,
            TargetTeamConstraint::Any => true,
        })
        .filter(|target| {
            actor.position.x.abs_diff(target.position.x)
                + actor.position.y.abs_diff(target.position.y)
                <= action.targeting.maximum_range
        })
        .map(|target| target.id.clone())
        .collect::<Vec<_>>();
    targets.sort();
    targets
}

fn is_cell_movement(action: &AuthoredActionDefinition) -> bool {
    action.movement.is_some() && action.targeting.target_kind == TargetKind::Area
}

fn materialize_action(
    ruleset_id: &str,
    actor_id: &str,
    action: &AuthoredActionDefinition,
    target_ids: Vec<String>,
    operations: Vec<HitEffectOperation>,
) -> rulebench_ruleset::ActionDefinition {
    let damage = operations.iter().find_map(|operation| match operation {
        HitEffectOperation::Damage(damage) => Some(damage),
        _ => None,
    });
    let modifier = operations.iter().find_map(|operation| match operation {
        HitEffectOperation::ApplyModifier(modifier) => Some(modifier),
        _ => None,
    });
    rulebench_ruleset::ActionDefinition {
        id: action.id.clone(),
        ruleset_id: ruleset_id.to_string(),
        ability_id: action.ability_id.clone(),
        name: action.name.clone(),
        actor_id: actor_id.to_string(),
        targeting: TargetingDeclaration {
            target_kind: action.targeting.target_kind,
            selection: action.targeting.selection,
            team_constraint: action.targeting.team_constraint,
            maximum_range: action.targeting.maximum_range,
            visibility_requirement: action.targeting.visibility_requirement,
            visible_target_ids: target_ids.clone(),
            target_ids,
            operation_pipeline: action.targeting.operation_pipeline.clone(),
        },
        check: action.check.clone(),
        hit: HitEffect {
            damage_bonus: damage.map_or(0, |value| value.damage_bonus),
            damage_type: damage.map_or_else(String::new, |value| value.damage_type.clone()),
            modifier_id: modifier.map_or_else(String::new, |value| value.modifier_id.clone()),
            modifier_label: modifier.map_or_else(String::new, |value| value.modifier_label.clone()),
            modifier_duration: modifier
                .map_or_else(String::new, |value| value.modifier_duration.clone()),
            operations,
        },
        resource_costs: action.resource_costs.clone(),
        movement: action.movement.clone(),
        action_text: action.action_text.clone(),
        effect_text: action.effect_text.clone(),
    }
}

fn materialize_operations(
    scenario: &RulebenchScenario,
    actor_index: usize,
    action: &AuthoredActionDefinition,
    target_ids: &[String],
) -> Result<Vec<HitEffectOperation>, AuthoredActionBindingError> {
    action
        .effects
        .iter()
        .map(|operation| match operation {
            AuthoredEffectOperation::Damage(value) => Ok(HitEffectOperation::Damage(value.clone())),
            AuthoredEffectOperation::Heal(value) => Ok(HitEffectOperation::Heal(value.clone())),
            AuthoredEffectOperation::GrantTemporaryVitality(value) => {
                Ok(HitEffectOperation::GrantTemporaryVitality(value.clone()))
            }
            AuthoredEffectOperation::ApplyModifier(value) => {
                let modifier = scenario.modifier_by_id(&value.modifier_id).ok_or_else(|| {
                    rejected(
                        "unknownAuthoredActionModifier",
                        Some(value.modifier_id.clone()),
                        "The materialized scenario does not contain the applied modifier.",
                    )
                })?;
                Ok(HitEffectOperation::ApplyModifier(ModifierEffectOperation {
                    modifier_id: modifier.id.clone(),
                    modifier_label: modifier.label.clone(),
                    modifier_duration: modifier.duration_policy.display_label(),
                }))
            }
            AuthoredEffectOperation::Move(value) => Ok(HitEffectOperation::Move(value.clone())),
            AuthoredEffectOperation::ChangeResource(value) => {
                Ok(HitEffectOperation::ChangeResource(value.clone()))
            }
            AuthoredEffectOperation::OpenReactionWindow(value) => {
                Ok(HitEffectOperation::OpenReactionWindow(
                    materialize_reaction_hook(scenario, actor_index, target_ids, value)?,
                ))
            }
        })
        .collect()
}

fn materialize_reaction_hook(
    scenario: &RulebenchScenario,
    actor_index: usize,
    target_ids: &[String],
    hook: &AuthoredReactionHookEffectOperation,
) -> Result<ReactionHookEffectOperation, AuthoredActionBindingError> {
    let mut eligible = BTreeSet::new();
    for selector in &hook.eligible_reactors {
        eligible.extend(expand_selector(
            scenario,
            actor_index,
            target_ids,
            *selector,
        ));
    }
    if eligible.is_empty() || eligible.len() > AUTHORED_ACTION_REACTION_EXPANSION_LIMIT {
        return Err(rejected(
            "invalidAuthoredReactionExpansion",
            Some(hook.hook_id.clone()),
            "The authored reaction selector expansion is empty or exceeds the Rust limit.",
        ));
    }
    let mut options = Vec::new();
    for option in &hook.options {
        let reactors = expand_selector(scenario, actor_index, target_ids, option.reactor);
        if reactors.is_empty() {
            return Err(rejected(
                "invalidAuthoredReactionOptionExpansion",
                Some(option.id.clone()),
                "The authored reaction option selector expands to no participants.",
            ));
        }
        for reactor_id in reactors {
            options.push(ReactionOptionDeclaration {
                id: format!("{}@{}", option.id, reactor_id),
                reactor_id,
                opens_nested_window: option.opens_nested_window,
            });
        }
    }
    if options.len() > AUTHORED_ACTION_REACTION_EXPANSION_LIMIT {
        return Err(rejected(
            "authoredReactionOptionExpansionLimitExceeded",
            Some(hook.hook_id.clone()),
            "The authored reaction option expansion exceeds the Rust limit.",
        ));
    }
    options.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(ReactionHookEffectOperation {
        hook_id: hook.hook_id.clone(),
        window: hook.window,
        eligible_reactor_ids: eligible.into_iter().collect(),
        options,
        maximum_nested_depth: hook.maximum_nested_depth,
    })
}

fn expand_selector(
    scenario: &RulebenchScenario,
    actor_index: usize,
    target_ids: &[String],
    selector: ReactionParticipantSelector,
) -> Vec<String> {
    let actor = &scenario.combatants[actor_index];
    let target_teams = scenario
        .combatants
        .iter()
        .filter(|combatant| target_ids.contains(&combatant.id))
        .map(|combatant| combatant.team)
        .collect::<Vec<_>>();
    let mut values = match selector {
        ReactionParticipantSelector::DeclaredTargets => target_ids.to_vec(),
        ReactionParticipantSelector::ActorAllies => scenario
            .combatants
            .iter()
            .filter(|combatant| combatant.id != actor.id && combatant.team == actor.team)
            .map(|combatant| combatant.id.clone())
            .collect(),
        ReactionParticipantSelector::TargetAllies => scenario
            .combatants
            .iter()
            .filter(|combatant| target_teams.contains(&combatant.team))
            .map(|combatant| combatant.id.clone())
            .collect(),
        ReactionParticipantSelector::AllOtherParticipants => scenario
            .combatants
            .iter()
            .filter(|combatant| combatant.id != actor.id)
            .map(|combatant| combatant.id.clone())
            .collect(),
    };
    values.sort();
    values.dedup();
    values
}

fn rejected(
    code: &'static str,
    reference_id: Option<String>,
    message: impl Into<String>,
) -> AuthoredActionBindingError {
    AuthoredActionBindingError {
        code,
        reference_id,
        message: message.into(),
        diagnostic_codes: Vec::new(),
    }
}
