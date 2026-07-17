use std::collections::BTreeSet;

use rulebench_ruleset::{
    ActionResourceCost, AttackCheckDeclaration, CheckDeclaration, DefenseReference, HitEffect,
    TargetKind, TargetSelection, TargetTeamConstraint, TargetingDeclaration, VisibilityRequirement,
};

use crate::{
    binding::materialize_authored_action_for_scenario, validate_scenario_content_report,
    AuthoredActionBindingRequest, AuthoredScenarioBindingReceipt, AuthoredScenarioControlMode,
    AuthoredScenarioDefinition, AuthoredScenarioParticipantReceipt, Combatant, ImportedContentPack,
    RulebenchScenario, ScenarioMetadata, AUTHORED_SCENARIO_BINDING_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioBindingError {
    pub code: &'static str,
    pub reference_id: Option<String>,
    pub message: String,
    pub diagnostic_codes: Vec<String>,
}

pub fn materialize_authored_scenario(
    imported: &ImportedContentPack,
    scenario_id: &str,
) -> Result<RulebenchScenario, AuthoredScenarioBindingError> {
    let definition = unique_scenario(imported, scenario_id)?;
    validate_control(definition)?;

    let mut rulesets = imported
        .resolved_set
        .packs
        .iter()
        .flat_map(|pack| pack.catalogs.rulesets.iter().cloned())
        .collect::<Vec<_>>();
    rulesets.sort_by(|left, right| left.id.cmp(&right.id));
    rulesets.dedup_by(|left, right| left.id == right.id && left == right);
    let selected_ruleset = rulesets
        .iter()
        .find(|ruleset| ruleset.id == definition.ruleset_id)
        .ok_or_else(|| {
            rejected(
                "missingAuthoredScenarioRuleset",
                Some(definition.ruleset_id.clone()),
                "The authored scenario selected ruleset is absent from its exact pack set.",
            )
        })?;
    if selected_ruleset.artifact_provenance() != imported.pack.ruleset {
        return Err(rejected(
            "incompatibleAuthoredScenarioRuleset",
            Some(definition.ruleset_id.clone()),
            "The authored scenario ruleset does not match the root pack provider provenance.",
        ));
    }

    let entities = collect_catalog(imported, |pack| &pack.catalogs.entities);
    let abilities = collect_catalog(imported, |pack| &pack.catalogs.abilities);
    let classes = collect_catalog(imported, |pack| &pack.catalogs.classes);
    let stat_definitions = collect_catalog(imported, |pack| &pack.catalogs.stat_definitions);
    let modifiers = collect_catalog(imported, |pack| &pack.catalogs.modifiers);
    let items = collect_catalog(imported, |pack| &pack.catalogs.items);
    let combatants = definition
        .participants
        .iter()
        .map(|participant| Combatant {
            id: participant.id.clone(),
            entity_id: participant.entity_id.clone(),
            name: participant.name.clone(),
            team: participant.team,
            side_id: participant.side_id.clone(),
            initiative: participant.initiative,
            position: participant.position,
            hit_points: participant.hit_points,
            temporary_vitality: participant.temporary_vitality,
            class_inputs: participant.class_inputs.clone(),
            stats: participant.stats.clone(),
            defenses: participant.defenses.clone(),
            resource_pools: participant.resource_pools.clone(),
            inventory_item_ids: participant.inventory_item_ids.clone(),
            equipped_item_ids: participant.equipped_item_ids.clone(),
            base_ability_ids: participant.base_ability_ids.clone(),
            active_modifiers: Vec::new(),
            conditions: Vec::new(),
            is_actor: participant.is_actor,
        })
        .collect::<Vec<_>>();

    let mut scenario = RulebenchScenario {
        metadata: ScenarioMetadata {
            id: definition.id.clone(),
            title: definition.title.clone(),
            summary: definition.summary.clone(),
            seed_label: definition.seed_label.clone(),
        },
        content_pack_set: Some(imported.resolved_set.reference.clone()),
        authored_action_binding: None,
        authored_scenario_binding: None,
        rulesets,
        selected_ruleset_id: definition.ruleset_id.clone(),
        grid: definition.grid.clone(),
        combatants,
        entities,
        abilities,
        selected_ability_id: None,
        classes,
        selected_class_id: None,
        stat_definitions,
        modifiers,
        items,
        selected_item_id: None,
        actions: Vec::new(),
        selected_action: placeholder_action(&definition.ruleset_id),
    };

    let mut runtime_action_ids = BTreeSet::new();
    for participant in &definition.participants {
        for grant in &participant.action_grants {
            if !runtime_action_ids.insert(grant.runtime_action_id.clone()) {
                return Err(rejected(
                    "duplicateAuthoredScenarioRuntimeAction",
                    Some(grant.runtime_action_id.clone()),
                    "Scenario-local runtime action ids must be unique.",
                ));
            }
            scenario = materialize_authored_action_for_scenario(
                scenario,
                imported,
                &AuthoredActionBindingRequest {
                    content_pack: imported.resolved_set.reference.root.clone(),
                    action_id: grant.action_id.clone(),
                    actor_id: participant.id.clone(),
                },
                &grant.runtime_action_id,
                &participant.visible_target_ids,
            )
            .map_err(|error| AuthoredScenarioBindingError {
                code: error.code,
                reference_id: error.reference_id,
                message: error.message,
                diagnostic_codes: error.diagnostic_codes,
            })?;
            scenario.authored_action_binding = None;
        }
    }

    let selected_action = scenario
        .actions
        .iter()
        .find(|action| action.id == definition.selected_action_id)
        .cloned()
        .ok_or_else(|| {
            rejected(
            "missingAuthoredScenarioSelectedAction",
            Some(definition.selected_action_id.clone()),
            "The authored scenario selected action is not one of its materialized action grants.",
        )
        })?;
    scenario.selected_ability_id = Some(selected_action.ability_id.clone());
    scenario.selected_action = selected_action;
    scenario.authored_scenario_binding = Some(AuthoredScenarioBindingReceipt {
        binding_version: AUTHORED_SCENARIO_BINDING_VERSION,
        content_pack_set: imported.resolved_set.reference.clone(),
        scenario_id: definition.id.clone(),
        participants: definition
            .participants
            .iter()
            .map(|participant| AuthoredScenarioParticipantReceipt {
                participant_id: participant.id.clone(),
                archetypes: participant.class_inputs.clone(),
                loadout_item_ids: participant.equipped_item_ids.clone(),
                action_grants: participant.action_grants.clone(),
            })
            .collect(),
        control: definition.control.clone(),
    });

    let report = validate_scenario_content_report(&scenario);
    if !report.accepted {
        return Err(AuthoredScenarioBindingError {
            code: "invalidAuthoredScenarioComposition",
            reference_id: Some(definition.id.clone()),
            message: "The authored scenario failed complete Rust content validation.".to_string(),
            diagnostic_codes: report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.code().to_string())
                .collect(),
        });
    }
    Ok(scenario)
}

pub fn authored_scenario_definition<'a>(
    imported: &'a ImportedContentPack,
    scenario_id: &str,
) -> Option<&'a AuthoredScenarioDefinition> {
    imported
        .resolved_set
        .packs
        .iter()
        .flat_map(|pack| &pack.catalogs.scenarios)
        .find(|scenario| scenario.id == scenario_id)
}

fn unique_scenario<'a>(
    imported: &'a ImportedContentPack,
    scenario_id: &str,
) -> Result<&'a AuthoredScenarioDefinition, AuthoredScenarioBindingError> {
    let matches = imported
        .resolved_set
        .packs
        .iter()
        .flat_map(|pack| &pack.catalogs.scenarios)
        .filter(|scenario| scenario.id == scenario_id)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [scenario] => Ok(*scenario),
        [] => Err(rejected(
            "unknownAuthoredScenario",
            Some(scenario_id.to_string()),
            "The exact content pack set does not contain the requested authored scenario.",
        )),
        _ => Err(rejected(
            "collidedAuthoredScenario",
            Some(scenario_id.to_string()),
            "The exact content pack set contains multiple authored scenarios with this id.",
        )),
    }
}

fn validate_control(
    definition: &AuthoredScenarioDefinition,
) -> Result<(), AuthoredScenarioBindingError> {
    let valid = match definition.control.mode {
        AuthoredScenarioControlMode::Manual => {
            definition.control.automation_policy_id.is_none()
                && definition.control.automation_policy_version.is_none()
        }
        AuthoredScenarioControlMode::Automatic => {
            definition
                .control
                .automation_policy_id
                .as_ref()
                .is_some_and(|id| !id.is_empty())
                && definition
                    .control
                    .automation_policy_version
                    .is_some_and(|version| version > 0)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(rejected(
            "invalidAuthoredScenarioControl",
            Some(definition.id.clone()),
            "Manual control must omit automation provenance and automatic control must select an exact policy id and version.",
        ))
    }
}

fn collect_catalog<T: Clone>(
    imported: &ImportedContentPack,
    catalog: impl Fn(&crate::CanonicalContentPack) -> &Vec<T>,
) -> Vec<T> {
    imported
        .resolved_set
        .packs
        .iter()
        .flat_map(|pack| catalog(pack).iter().cloned())
        .collect()
}

fn placeholder_action(ruleset_id: &str) -> rulebench_ruleset::ActionDefinition {
    rulebench_ruleset::ActionDefinition {
        id: "authored-scenario-unbound".to_string(),
        ruleset_id: ruleset_id.to_string(),
        ability_id: "authored-scenario-unbound".to_string(),
        name: "Unbound authored scenario action".to_string(),
        actor_id: "authored-scenario-unbound".to_string(),
        targeting: TargetingDeclaration {
            target_kind: TargetKind::Combatant,
            selection: TargetSelection::Single,
            team_constraint: TargetTeamConstraint::Hostile,
            maximum_range: 0,
            visibility_requirement: VisibilityRequirement::Ignored,
            target_ids: Vec::new(),
            visible_target_ids: Vec::new(),
            operation_pipeline: None,
        },
        check: CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 0,
            modifier_stat_id: "unbound".to_string(),
            defense: DefenseReference {
                id: "unbound".to_string(),
                label: "Unbound".to_string(),
            },
        }),
        hit: HitEffect {
            damage_bonus: 0,
            damage_type: "unbound".to_string(),
            modifier_id: String::new(),
            modifier_label: String::new(),
            modifier_duration: String::new(),
            operations: Vec::new(),
        },
        resource_costs: vec![ActionResourceCost::standard_action()],
        movement: None,
        action_text: "Unbound.".to_string(),
        effect_text: "Unbound.".to_string(),
    }
}

fn rejected(
    code: &'static str,
    reference_id: Option<String>,
    message: impl Into<String>,
) -> AuthoredScenarioBindingError {
    AuthoredScenarioBindingError {
        code,
        reference_id,
        message: message.into(),
        diagnostic_codes: Vec::new(),
    }
}
