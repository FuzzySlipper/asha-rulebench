use std::collections::BTreeSet;

use super::{
    AuthoredContentPack, ContentImportDiagnostic, ContentImportDiagnosticCode,
    ContentImportDiagnosticSeverity, ContentImportLimits, ContentImportReport,
};
use crate::{
    AuthoredEffectOperation, AuthoredScenarioBindingError, ModifierDurationPolicy,
    ResolvedContentPackSet, CONTENT_PACK_FINGERPRINT_ALGORITHM_V1,
    CONTENT_PACK_FINGERPRINT_ALGORITHM_V2,
};
use crate::{ContentDefinitionKind, ContentPackDiagnostic, CONTENT_PACK_FINGERPRINT_ALGORITHM};
use rulebench_ruleset::{
    CheckDeclaration, EffectOperationId, ModifierTenure, OperationPipelineV2,
    RulesetProviderCatalog, RulesetProviderDescriptor, TargetKind, TargetSelection,
    TargetingOperationId,
};

const CHECK_CAPABILITY_VERSION: &str = "1";

pub(super) fn validate_authored_pack(
    authored: &AuthoredContentPack,
    limits: ContentImportLimits,
) -> Vec<ContentImportDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_required_string(&mut diagnostics, "identity.id", &authored.identity.id);
    validate_required_string(
        &mut diagnostics,
        "identity.version",
        &authored.identity.version,
    );
    validate_duplicate_tags(&mut diagnostics, &authored.tags);
    validate_required_string(&mut diagnostics, "title", &authored.title);
    validate_required_string(&mut diagnostics, "summary", &authored.summary);
    validate_required_string(
        &mut diagnostics,
        "provenance.sourceId",
        &authored.provenance.source_id,
    );
    validate_required_string(
        &mut diagnostics,
        "ruleset.rulesetId",
        &authored.ruleset.ruleset_id,
    );
    validate_required_string(
        &mut diagnostics,
        "ruleset.rulesetVersion",
        &authored.ruleset.ruleset_version,
    );

    for (path, value) in pack_strings(authored) {
        if value.len() > limits.maximum_string_bytes {
            let message = format!(
                "Content field {path} contains {} bytes; the limit is {}.",
                value.len(),
                limits.maximum_string_bytes
            );
            diagnostics.push(ContentImportDiagnostic {
                severity: ContentImportDiagnosticSeverity::Error,
                code: ContentImportDiagnosticCode::LimitExceeded,
                path,
                definition_kind: None,
                definition_id: None,
                message,
            });
        }
    }

    if authored.dependencies.len() > limits.maximum_dependencies {
        diagnostics.push(limit_diagnostic(
            "dependencies",
            authored.dependencies.len(),
            limits.maximum_dependencies,
        ));
    }

    let catalogs = catalog_identities(authored);
    let total_definitions = catalogs.iter().map(|(_, ids)| ids.len()).sum::<usize>();
    if total_definitions > limits.maximum_total_definitions {
        diagnostics.push(limit_diagnostic(
            "catalogs",
            total_definitions,
            limits.maximum_total_definitions,
        ));
    }
    for (kind, ids) in catalogs {
        validate_catalog(&mut diagnostics, kind, &ids, limits);
    }
    validate_ability_definitions(&mut diagnostics, authored);
    validate_modifier_definitions(&mut diagnostics, authored);
    validate_action_definitions(&mut diagnostics, authored, limits);

    for (index, dependency) in authored.dependencies.iter().enumerate() {
        validate_fingerprint(
            &mut diagnostics,
            &format!("dependencies[{index}].fingerprint"),
            &dependency.fingerprint.algorithm,
            &dependency.fingerprint.value,
        );
    }
    diagnostics
}

pub(super) fn scenario_materialization_diagnostic(
    scenario_id: &str,
    error: AuthoredScenarioBindingError,
) -> ContentImportDiagnostic {
    let initial_state = error.diagnostic_codes.iter().any(|code| {
        matches!(
            code.as_str(),
            "invalidScenarioGridDimensions"
                | "duplicateScenarioGridCell"
                | "scenarioGridCellOutOfBounds"
                | "combatantPlacementOutOfBounds"
                | "combatantPlacementOccupied"
                | "combatantPlacementBlocked"
                | "invalidCombatantVitality"
                | "invalidActionResourcePoolMaximum"
                | "invalidActionResourcePoolInitial"
                | "invalidActionResourceRefreshPolicy"
        )
    });
    ContentImportDiagnostic {
        severity: ContentImportDiagnosticSeverity::Error,
        code: if initial_state {
            ContentImportDiagnosticCode::InvalidScenarioInitialState
        } else {
            ContentImportDiagnosticCode::InvalidScenarioDeclaration
        },
        path: format!("catalogs.scenarios[{scenario_id}]"),
        definition_kind: Some(ContentDefinitionKind::Scenario),
        definition_id: Some(scenario_id.to_string()),
        message: format!(
            "Authored scenario materialization failed with {}: {} Diagnostics: {}.",
            error.code,
            error.message,
            error.diagnostic_codes.join(", ")
        ),
    }
}

fn validate_ability_definitions(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    authored: &AuthoredContentPack,
) {
    for (index, ability) in authored.catalogs.abilities.iter().enumerate() {
        for (field, value) in [("name", &ability.name), ("summary", &ability.summary)] {
            if value.is_empty() {
                diagnostics.push(ContentImportDiagnostic {
                    severity: ContentImportDiagnosticSeverity::Error,
                    code: ContentImportDiagnosticCode::EmptyField,
                    path: format!("catalogs.abilities[{index}].{field}"),
                    definition_kind: Some(ContentDefinitionKind::Ability),
                    definition_id: Some(ability.id.clone()),
                    message: format!(
                        "Authored ability {} field {field} must not be empty.",
                        ability.id
                    ),
                });
            }
        }
    }
}

fn validate_modifier_definitions(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    authored: &AuthoredContentPack,
) {
    for (index, modifier) in authored.catalogs.modifiers.iter().enumerate() {
        let base = format!("catalogs.modifiers[{index}]");
        for (field, value) in [
            ("label", &modifier.label),
            ("summary", &modifier.summary),
            ("stackingGroup", &modifier.stacking_group),
        ] {
            if value.is_empty() {
                push_definition_error(
                    diagnostics,
                    ContentImportDiagnosticCode::InvalidModifierDeclaration,
                    format!("{base}.{field}"),
                    ContentDefinitionKind::Modifier,
                    &modifier.id,
                    format!(
                        "Authored modifier {} field {field} must not be empty.",
                        modifier.id
                    ),
                );
            }
        }
        let duration_valid = match (&modifier.default_tenure, &modifier.duration_policy) {
            (ModifierTenure::Permanent, ModifierDurationPolicy::Permanent) => true,
            (ModifierTenure::Temporary, ModifierDurationPolicy::Turns(value))
            | (ModifierTenure::Temporary, ModifierDurationPolicy::Rounds(value)) => *value > 0,
            (ModifierTenure::Temporary, ModifierDurationPolicy::UntilEvent(event)) => {
                !event.is_empty()
            }
            _ => false,
        };
        if !duration_valid {
            push_definition_error(
                diagnostics,
                ContentImportDiagnosticCode::InvalidModifierDeclaration,
                format!("{base}.durationPolicy"),
                ContentDefinitionKind::Modifier,
                &modifier.id,
                format!(
                    "Authored modifier {} has an incompatible or empty tenure/duration policy.",
                    modifier.id
                ),
            );
        }
        let mut stat_ids = BTreeSet::new();
        for (adjustment_index, adjustment) in modifier.stat_adjustments.iter().enumerate() {
            if adjustment.stat_id.is_empty()
                || adjustment.stat_label.is_empty()
                || adjustment.delta == 0
                || !stat_ids.insert(adjustment.stat_id.clone())
            {
                push_definition_error(
                    diagnostics,
                    ContentImportDiagnosticCode::InvalidModifierDeclaration,
                    format!("{base}.statAdjustments[{adjustment_index}]"),
                    ContentDefinitionKind::Modifier,
                    &modifier.id,
                    format!(
                        "Authored modifier {} has an empty or duplicate stat adjustment.",
                        modifier.id
                    ),
                );
            }
        }
    }
}

fn validate_action_definitions(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    authored: &AuthoredContentPack,
    limits: ContentImportLimits,
) {
    let selected_ruleset = authored
        .catalogs
        .rulesets
        .iter()
        .find(|ruleset| ruleset.id == authored.ruleset.ruleset_id);
    for (index, action) in authored.catalogs.actions.iter().enumerate() {
        let base = format!("catalogs.actions[{index}]");
        for (field, value) in [
            ("abilityId", &action.ability_id),
            ("name", &action.name),
            ("actionText", &action.action_text),
            ("effectText", &action.effect_text),
        ] {
            if value.is_empty() {
                push_action_error(
                    diagnostics,
                    ContentImportDiagnosticCode::InvalidActionDeclaration,
                    format!("{base}.{field}"),
                    &action.id,
                    format!(
                        "Authored action {} field {field} must not be empty.",
                        action.id
                    ),
                );
            }
        }

        if action.effects.len() > limits.maximum_operations_per_action {
            diagnostics.push(limit_diagnostic(
                &format!("{base}.effects"),
                action.effects.len(),
                limits.maximum_operations_per_action,
            ));
        }
        if action.effects.is_empty() && action.movement.is_none() {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::InvalidActionDeclaration,
                format!("{base}.effects"),
                &action.id,
                format!(
                    "Authored action {} must declare an effect or movement.",
                    action.id
                ),
            );
        }

        validate_targeting(diagnostics, action, &base);
        validate_check(diagnostics, action, &base, selected_ruleset);
        validate_costs(diagnostics, action, &base);
        validate_effects(diagnostics, action, &base, limits);
    }
}

fn validate_targeting(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    base: &str,
) {
    let supported_single = action.movement.is_none()
        && action.targeting.target_kind == TargetKind::Combatant
        && action.targeting.selection == TargetSelection::Single
        && action.targeting.operation_pipeline.is_none();
    let supported_movement = action.movement.is_some()
        && action.targeting.target_kind == TargetKind::Area
        && action.targeting.selection == TargetSelection::Single
        && action.targeting.operation_pipeline.is_none();
    let supported_pipeline = action.movement.is_none()
        && action
            .targeting
            .operation_pipeline
            .as_ref()
            .is_some_and(|pipeline| {
                let bounded = (1..=OperationPipelineV2::MAXIMUM_TARGET_LIMIT)
                    .contains(&pipeline.maximum_targets);
                let shape = match (
                    action.targeting.target_kind,
                    action.targeting.selection,
                    &pipeline.area,
                ) {
                    (TargetKind::Combatant, TargetSelection::Multiple, None) => true,
                    (TargetKind::Area, TargetSelection::Multiple, Some(area)) => {
                        (1..=OperationPipelineV2::MAXIMUM_AREA_RADIUS).contains(&area.radius)
                    }
                    _ => false,
                };
                bounded && shape
            });
    if !supported_single && !supported_movement && !supported_pipeline {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::InvalidActionDeclaration,
            format!("{base}.targeting"),
            &action.id,
            format!(
                "Authored action {} declares unsupported targeting.",
                action.id
            ),
        );
    }
    if let Some(movement) = &action.movement {
        if movement.allowance == 0 {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::InvalidActionDeclaration,
                format!("{base}.movement.allowance"),
                &action.id,
                format!("Authored action {} has zero movement allowance.", action.id),
            );
        }
    }
}

fn validate_check(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    base: &str,
    selected_ruleset: Option<&rulebench_ruleset::RulesetMetadata>,
) {
    let fields_valid = match &action.check {
        CheckDeclaration::Attack(attack) => {
            !attack.modifier_stat_id.is_empty()
                && !attack.defense.id.is_empty()
                && !attack.defense.label.is_empty()
        }
        CheckDeclaration::SavingThrow(save) => {
            !save.save_stat_id.is_empty() && save.difficulty_class >= 0
        }
        CheckDeclaration::Contested(contested) => {
            !contested.actor_stat_id.is_empty() && !contested.target_stat_id.is_empty()
        }
    };
    if !fields_valid {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::InvalidActionDeclaration,
            format!("{base}.check"),
            &action.id,
            format!(
                "Authored action {} has an incomplete check declaration.",
                action.id
            ),
        );
    }
    let supported = selected_ruleset
        .and_then(|ruleset| ruleset.validate_modules().ok())
        .is_some_and(|registry| registry.action_resolution().supports_check(&action.check));
    if selected_ruleset.is_some() && !supported {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::UnsupportedActionCheck,
            format!("{base}.check"),
            &action.id,
            format!(
                "Authored action {} check is unavailable in the selected ruleset.",
                action.id
            ),
        );
    }
}

fn validate_costs(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    base: &str,
) {
    let mut resource_ids = BTreeSet::new();
    for (index, cost) in action.resource_costs.iter().enumerate() {
        if cost.resource_id.is_empty() || cost.amount == 0 {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::InvalidActionDeclaration,
                format!("{base}.resourceCosts[{index}]"),
                &action.id,
                format!(
                    "Authored action {} has an invalid resource cost.",
                    action.id
                ),
            );
        } else if !resource_ids.insert(cost.resource_id.clone()) {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::DuplicateActionResourceCost,
                format!("{base}.resourceCosts[{index}].resourceId"),
                &action.id,
                format!(
                    "Authored action {} repeats resource cost {}.",
                    action.id, cost.resource_id
                ),
            );
        }
    }
}

fn validate_effects(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    base: &str,
    limits: ContentImportLimits,
) {
    let mut reaction_count = 0;
    for (index, effect) in action.effects.iter().enumerate() {
        let path = format!("{base}.effects[{index}]");
        let valid = match effect {
            AuthoredEffectOperation::Damage(value) => !value.damage_type.is_empty(),
            AuthoredEffectOperation::Heal(value) => !value.healing_type.is_empty(),
            AuthoredEffectOperation::GrantTemporaryVitality(_) => true,
            AuthoredEffectOperation::ApplyModifier(value) => !value.modifier_id.is_empty(),
            AuthoredEffectOperation::Move(value) => {
                let has_pipeline = action.targeting.operation_pipeline.is_some();
                let distance_valid = value.maximum_distance > 0
                    && value.maximum_distance <= OperationPipelineV2::MAXIMUM_AREA_RADIUS
                    && (!matches!(value.movement_kind, rulebench_ruleset::MovementKind::Shift)
                        || value.maximum_distance == 1);
                has_pipeline && distance_valid
            }
            AuthoredEffectOperation::ChangeResource(value) => {
                action.targeting.operation_pipeline.is_some()
                    && !value.resource_id.is_empty()
                    && value.delta != 0
            }
            AuthoredEffectOperation::OpenReactionWindow(hook) => {
                reaction_count += 1;
                validate_reaction(diagnostics, action, hook, &path, limits);
                true
            }
        };
        if !valid {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::InvalidActionDeclaration,
                path,
                &action.id,
                format!(
                    "Authored action {} has an invalid effect operation.",
                    action.id
                ),
            );
        }
    }
    if reaction_count > 1 {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::InvalidReactionDeclaration,
            format!("{base}.effects"),
            &action.id,
            format!(
                "Authored action {} declares more than one reaction hook.",
                action.id
            ),
        );
    }
    validate_effect_execution_profile(diagnostics, action, base);
}

fn validate_effect_execution_profile(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    base: &str,
) {
    if action.movement.is_some() {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::UnsupportedActionEffect,
            format!("{base}.movement"),
            &action.id,
            format!(
                "Authored action {} declares top-level movement, whose current Rust resolver does not execute the authored targeting, check, or effect program.",
                action.id
            ),
        );
        return;
    }
    if action.effects.is_empty() {
        return;
    }

    let mut counts = [0_usize; 7];
    let mut previous_rank = 0_usize;
    for (index, effect) in action.effects.iter().enumerate() {
        let rank = match effect {
            AuthoredEffectOperation::Damage(_) => 0,
            AuthoredEffectOperation::Heal(_) => 1,
            AuthoredEffectOperation::GrantTemporaryVitality(_) => 2,
            AuthoredEffectOperation::ApplyModifier(_) => 3,
            AuthoredEffectOperation::Move(_) => 4,
            AuthoredEffectOperation::ChangeResource(_) => 5,
            AuthoredEffectOperation::OpenReactionWindow(_) => 6,
        };
        counts[rank] += 1;
        let repeated_non_sequential_operation = counts[rank] > 1 && rank != 5;
        let out_of_execution_order = index > 0 && rank < previous_rank;
        if repeated_non_sequential_operation || out_of_execution_order {
            push_action_error(
                diagnostics,
                ContentImportDiagnosticCode::UnsupportedActionEffect,
                format!("{base}.effects[{index}]"),
                &action.id,
                format!(
                    "Authored action {} declares an effect sequence the current Rust resolver cannot execute exactly in authored order.",
                    action.id
                ),
            );
        }
        previous_rank = rank;
    }

    if counts[0] != 1 {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::UnsupportedActionEffect,
            format!("{base}.effects"),
            &action.id,
            format!(
                "Authored action {} must declare exactly one leading damage effect until the Rust resolver supports damage-free or repeated-damage programs.",
                action.id
            ),
        );
    }
}

fn validate_reaction(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    action: &crate::AuthoredActionDefinition,
    hook: &crate::AuthoredReactionHookEffectOperation,
    path: &str,
    limits: ContentImportLimits,
) {
    let mut valid = !hook.hook_id.is_empty()
        && !hook.eligible_reactors.is_empty()
        && !hook.options.is_empty()
        && hook.maximum_nested_depth <= 2;
    if hook.eligible_reactors.len() > limits.maximum_reaction_selectors {
        diagnostics.push(limit_diagnostic(
            &format!("{path}.eligibleReactors"),
            hook.eligible_reactors.len(),
            limits.maximum_reaction_selectors,
        ));
    }
    if hook.options.len() > limits.maximum_reaction_options {
        diagnostics.push(limit_diagnostic(
            &format!("{path}.options"),
            hook.options.len(),
            limits.maximum_reaction_options,
        ));
    }
    let eligible = hook
        .eligible_reactors
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut option_ids = BTreeSet::new();
    for option in &hook.options {
        if option.id.is_empty()
            || !eligible.contains(&option.reactor)
            || !option_ids.insert(option.id.clone())
            || (option.opens_nested_window && hook.maximum_nested_depth == 0)
        {
            valid = false;
        }
    }
    if !valid {
        push_action_error(
            diagnostics,
            ContentImportDiagnosticCode::InvalidReactionDeclaration,
            path.to_string(),
            &action.id,
            format!(
                "Authored action {} has an invalid reaction declaration.",
                action.id
            ),
        );
    }
}

pub(super) fn validate_resolved_action_references(
    resolved: &ResolvedContentPackSet,
) -> Vec<ContentImportDiagnostic> {
    let ability_ids = resolved
        .packs
        .iter()
        .flat_map(|pack| {
            pack.catalogs
                .abilities
                .iter()
                .map(|value| value.id.as_str())
        })
        .collect::<BTreeSet<_>>();
    let modifier_ids = resolved
        .packs
        .iter()
        .flat_map(|pack| {
            pack.catalogs
                .modifiers
                .iter()
                .map(|value| value.id.as_str())
        })
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    for pack in &resolved.packs {
        for (index, action) in pack.catalogs.actions.iter().enumerate() {
            let base = format!(
                "resolvedPacks[{}@{}].catalogs.actions[{index}]",
                pack.identity.id, pack.identity.version
            );
            if !ability_ids.contains(action.ability_id.as_str()) {
                push_action_error(
                    &mut diagnostics,
                    ContentImportDiagnosticCode::MissingActionAbility,
                    format!("{base}.abilityId"),
                    &action.id,
                    format!(
                        "Authored action {} references unavailable ability {}.",
                        action.id, action.ability_id
                    ),
                );
            }
            for (effect_index, effect) in action.effects.iter().enumerate() {
                if let AuthoredEffectOperation::ApplyModifier(modifier) = effect {
                    if !modifier_ids.contains(modifier.modifier_id.as_str()) {
                        push_action_error(
                            &mut diagnostics,
                            ContentImportDiagnosticCode::MissingActionModifier,
                            format!("{base}.effects[{effect_index}].modifierId"),
                            &action.id,
                            format!(
                                "Authored action {} references unavailable modifier {}.",
                                action.id, modifier.modifier_id
                            ),
                        );
                    }
                }
            }
        }
    }
    diagnostics
}

pub(super) fn validate_resolved_action_compatibility(
    resolved: &ResolvedContentPackSet,
    provider_catalog: Option<&RulesetProviderCatalog>,
) -> Vec<ContentImportDiagnostic> {
    let mut diagnostics = Vec::new();
    for pack in &resolved.packs {
        for (index, action) in pack.catalogs.actions.iter().enumerate() {
            let base = format!(
                "resolvedPacks[{}@{}].catalogs.actions[{index}]",
                pack.identity.id, pack.identity.version
            );
            let Some(provider) = provider_catalog.and_then(|catalog| {
                catalog
                    .select_ruleset(&pack.ruleset.ruleset_id, &pack.ruleset.ruleset_version)
                    .ok()
            }) else {
                push_action_error(
                    &mut diagnostics,
                    ContentImportDiagnosticCode::UnavailableActionRulesetProvider,
                    format!("{base}.ruleset"),
                    &action.id,
                    format!(
                        "Authored action {} requires unavailable compiled ruleset provider {}@{}.",
                        action.id, pack.ruleset.ruleset_id, pack.ruleset.ruleset_version
                    ),
                );
                continue;
            };

            let declared_ruleset = resolved.packs.iter().find_map(|candidate| {
                candidate.catalogs.rulesets.iter().find(|ruleset| {
                    ruleset.id == pack.ruleset.ruleset_id
                        && ruleset.version == pack.ruleset.ruleset_version
                })
            });
            if !declared_ruleset.is_some_and(|ruleset| ruleset.modules == provider.ruleset.modules)
            {
                push_action_error(
                    &mut diagnostics,
                    ContentImportDiagnosticCode::IncompatibleActionRulesetProvider,
                    format!("{base}.ruleset"),
                    &action.id,
                    format!(
                        "Authored action {} ruleset modules do not match compiled provider {}@{}.",
                        action.id, provider.provider_id, provider.provider_version
                    ),
                );
                continue;
            }

            let (check_id, check_version) = check_capability(&action.check);
            if !provider_supports(provider, check_id, check_version) {
                push_missing_provider_capability(
                    &mut diagnostics,
                    ContentImportDiagnosticCode::UnsupportedActionCheck,
                    format!("{base}.check"),
                    &action.id,
                    provider,
                    check_id,
                    check_version,
                );
            }

            let targeting = targeting_capability(action);
            let targeting_id = format!("targeting.{}", targeting.code());
            if !provider_supports(
                provider,
                &targeting_id,
                OperationPipelineV2::VOCABULARY_VERSION,
            ) {
                push_missing_provider_capability(
                    &mut diagnostics,
                    ContentImportDiagnosticCode::UnsupportedActionTargeting,
                    format!("{base}.targeting"),
                    &action.id,
                    provider,
                    &targeting_id,
                    OperationPipelineV2::VOCABULARY_VERSION,
                );
            }

            for (effect_index, effect) in action.effects.iter().enumerate() {
                let capability_id = format!("operation.{}", effect.code());
                if !provider_supports(
                    provider,
                    &capability_id,
                    EffectOperationId::VOCABULARY_VERSION,
                ) {
                    push_missing_provider_capability(
                        &mut diagnostics,
                        ContentImportDiagnosticCode::UnsupportedActionEffect,
                        format!("{base}.effects[{effect_index}]"),
                        &action.id,
                        provider,
                        &capability_id,
                        EffectOperationId::VOCABULARY_VERSION,
                    );
                }
            }
        }
    }
    diagnostics
}

fn check_capability(check: &CheckDeclaration) -> (&'static str, &'static str) {
    let id = match check {
        CheckDeclaration::Attack(_) => "check.attackVsDefense",
        CheckDeclaration::SavingThrow(_) => "check.savingThrow",
        CheckDeclaration::Contested(_) => "check.contested",
    };
    (id, CHECK_CAPABILITY_VERSION)
}

fn targeting_capability(action: &crate::AuthoredActionDefinition) -> TargetingOperationId {
    if action.movement.is_some() {
        return TargetingOperationId::CellMovement;
    }
    match (
        action.targeting.target_kind,
        action.targeting.selection,
        action.targeting.operation_pipeline.as_ref(),
    ) {
        (TargetKind::Combatant, TargetSelection::Single, _) => {
            TargetingOperationId::SingleCombatant
        }
        (TargetKind::Combatant, TargetSelection::Multiple, _) => {
            TargetingOperationId::MultipleCombatants
        }
        (TargetKind::Area, _, Some(_)) => TargetingOperationId::ManhattanBurstArea,
        (TargetKind::Area, _, None) => TargetingOperationId::CellMovement,
    }
}

fn provider_supports(
    provider: &RulesetProviderDescriptor,
    capability_id: &str,
    capability_version: &str,
) -> bool {
    provider
        .capability(capability_id)
        .is_some_and(|capability| capability.version == capability_version)
}

fn push_missing_provider_capability(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    code: ContentImportDiagnosticCode,
    path: String,
    action_id: &str,
    provider: &RulesetProviderDescriptor,
    capability_id: &str,
    capability_version: &str,
) {
    push_action_error(
        diagnostics,
        code,
        path,
        action_id,
        format!(
            "Authored action {action_id} requires provider capability {capability_id}@{capability_version}, which {}@{} does not expose.",
            provider.provider_id, provider.provider_version
        ),
    );
}

fn push_action_error(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    code: ContentImportDiagnosticCode,
    path: String,
    action_id: &str,
    message: String,
) {
    push_definition_error(
        diagnostics,
        code,
        path,
        ContentDefinitionKind::Action,
        action_id,
        message,
    );
}

fn push_definition_error(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    code: ContentImportDiagnosticCode,
    path: String,
    kind: ContentDefinitionKind,
    definition_id: &str,
    message: String,
) {
    diagnostics.push(ContentImportDiagnostic {
        severity: ContentImportDiagnosticSeverity::Error,
        code,
        path,
        definition_kind: Some(kind),
        definition_id: Some(definition_id.to_string()),
        message,
    });
}

fn validate_catalog(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    kind: ContentDefinitionKind,
    ids: &[String],
    limits: ContentImportLimits,
) {
    if ids.len() > limits.maximum_definitions_per_catalog {
        diagnostics.push(ContentImportDiagnostic {
            severity: ContentImportDiagnosticSeverity::Error,
            code: ContentImportDiagnosticCode::LimitExceeded,
            path: format!("catalogs.{}", kind.code()),
            definition_kind: Some(kind),
            definition_id: None,
            message: format!(
                "Content {} catalog contains {} definitions; the limit is {}.",
                kind.code(),
                ids.len(),
                limits.maximum_definitions_per_catalog
            ),
        });
    }
    validate_duplicate_ids(diagnostics, kind, ids);
    for id in ids {
        if id.is_empty() {
            diagnostics.push(ContentImportDiagnostic {
                severity: ContentImportDiagnosticSeverity::Error,
                code: ContentImportDiagnosticCode::EmptyField,
                path: format!("catalogs.{}[].id", kind.code()),
                definition_kind: Some(kind),
                definition_id: None,
                message: format!("Content {} id must not be empty.", kind.code()),
            });
        }
    }
}

fn validate_required_string(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    path: &str,
    value: &str,
) {
    if value.is_empty() {
        diagnostics.push(ContentImportDiagnostic {
            severity: ContentImportDiagnosticSeverity::Error,
            code: ContentImportDiagnosticCode::EmptyField,
            path: path.to_string(),
            definition_kind: None,
            definition_id: None,
            message: format!("Content import field {path} must not be empty."),
        });
    }
}

fn validate_fingerprint(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    path: &str,
    algorithm: &str,
    value: &str,
) {
    let valid_value = value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !matches!(
        algorithm,
        CONTENT_PACK_FINGERPRINT_ALGORITHM
            | CONTENT_PACK_FINGERPRINT_ALGORITHM_V1
            | CONTENT_PACK_FINGERPRINT_ALGORITHM_V2
    ) || !valid_value
    {
        diagnostics.push(ContentImportDiagnostic {
            severity: ContentImportDiagnosticSeverity::Error,
            code: ContentImportDiagnosticCode::InvalidFingerprint,
            path: path.to_string(),
            definition_kind: None,
            definition_id: None,
            message: format!(
                "Content fingerprint at {path} must use a supported content-pack fingerprint algorithm with 16 lowercase hexadecimal characters."
            ),
        });
    }
}

fn limit_diagnostic(path: &str, actual: usize, maximum: usize) -> ContentImportDiagnostic {
    ContentImportDiagnostic {
        severity: ContentImportDiagnosticSeverity::Error,
        code: ContentImportDiagnosticCode::LimitExceeded,
        path: path.to_string(),
        definition_kind: None,
        definition_id: None,
        message: format!("Content field {path} contains {actual} entries; the limit is {maximum}."),
    }
}

fn validate_duplicate_ids(
    diagnostics: &mut Vec<ContentImportDiagnostic>,
    kind: ContentDefinitionKind,
    ids: &[String],
) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            diagnostics.push(ContentImportDiagnostic {
                severity: ContentImportDiagnosticSeverity::Error,
                code: ContentImportDiagnosticCode::DuplicateDefinition,
                path: format!("catalogs.{}", kind.code()),
                definition_kind: Some(kind),
                definition_id: Some(id.clone()),
                message: format!(
                    "Content {} id {id} is declared more than once.",
                    kind.code()
                ),
            });
        }
    }
}

fn validate_duplicate_tags(diagnostics: &mut Vec<ContentImportDiagnostic>, tags: &[String]) {
    let mut seen = BTreeSet::new();
    for tag in tags {
        if !seen.insert(tag) {
            diagnostics.push(ContentImportDiagnostic {
                severity: ContentImportDiagnosticSeverity::Warning,
                code: ContentImportDiagnosticCode::DuplicateTagCanonicalized,
                path: "tags".to_string(),
                definition_kind: None,
                definition_id: Some(tag.clone()),
                message: format!("Duplicate content tag {tag} was canonicalized to one entry."),
            });
        }
    }
}

fn pack_strings(authored: &AuthoredContentPack) -> Vec<(String, &str)> {
    let mut strings = vec![
        ("identity.id".to_string(), authored.identity.id.as_str()),
        (
            "identity.version".to_string(),
            authored.identity.version.as_str(),
        ),
        ("title".to_string(), authored.title.as_str()),
        ("summary".to_string(), authored.summary.as_str()),
        (
            "provenance.sourceId".to_string(),
            authored.provenance.source_id.as_str(),
        ),
        (
            "ruleset.rulesetId".to_string(),
            authored.ruleset.ruleset_id.as_str(),
        ),
        (
            "ruleset.rulesetVersion".to_string(),
            authored.ruleset.ruleset_version.as_str(),
        ),
    ];
    if let Some(authored_by) = authored.provenance.authored_by.as_deref() {
        strings.push(("provenance.authoredBy".to_string(), authored_by));
    }
    for (index, ability) in authored.catalogs.abilities.iter().enumerate() {
        strings.extend([
            (
                format!("catalogs.abilities[{index}].id"),
                ability.id.as_str(),
            ),
            (
                format!("catalogs.abilities[{index}].name"),
                ability.name.as_str(),
            ),
            (
                format!("catalogs.abilities[{index}].summary"),
                ability.summary.as_str(),
            ),
        ]);
        for (tag_index, tag) in ability.tags.iter().enumerate() {
            strings.push((
                format!("catalogs.abilities[{index}].tags[{tag_index}]"),
                tag.as_str(),
            ));
        }
    }
    for (index, class) in authored.catalogs.classes.iter().enumerate() {
        strings.extend([
            (format!("catalogs.classes[{index}].id"), class.id.as_str()),
            (
                format!("catalogs.classes[{index}].name"),
                class.name.as_str(),
            ),
            (
                format!("catalogs.classes[{index}].version"),
                class.version.as_str(),
            ),
            (
                format!("catalogs.classes[{index}].summary"),
                class.summary.as_str(),
            ),
        ]);
        for (tag_index, tag) in class.tags.iter().enumerate() {
            strings.push((format!("catalogs.classes[{index}].tags[{tag_index}]"), tag));
        }
        for (grant_index, grant) in class.level_grants.iter().enumerate() {
            for (ability_index, ability_id) in grant.granted_ability_ids.iter().enumerate() {
                strings.push((
                    format!("catalogs.classes[{index}].levelGrants[{grant_index}].grantedAbilityIds[{ability_index}]"),
                    ability_id,
                ));
            }
            for (modifier_index, modifier_id) in grant.granted_modifier_ids.iter().enumerate() {
                strings.push((
                    format!("catalogs.classes[{index}].levelGrants[{grant_index}].grantedModifierIds[{modifier_index}]"),
                    modifier_id,
                ));
            }
        }
    }
    for (index, definition) in authored.catalogs.stat_definitions.iter().enumerate() {
        strings.extend([
            (
                format!("catalogs.statDefinitions[{index}].id"),
                definition.id.as_str(),
            ),
            (
                format!("catalogs.statDefinitions[{index}].label"),
                definition.label.as_str(),
            ),
            (
                format!("catalogs.statDefinitions[{index}].summary"),
                definition.summary.as_str(),
            ),
        ]);
    }
    for (index, item) in authored.catalogs.items.iter().enumerate() {
        strings.extend([
            (format!("catalogs.items[{index}].id"), item.id.as_str()),
            (format!("catalogs.items[{index}].name"), item.name.as_str()),
            (
                format!("catalogs.items[{index}].summary"),
                item.summary.as_str(),
            ),
            (
                format!("catalogs.items[{index}].equipmentSlot"),
                item.equipment_slot.as_str(),
            ),
        ]);
        for (tag_index, tag) in item.tags.iter().enumerate() {
            strings.push((format!("catalogs.items[{index}].tags[{tag_index}]"), tag));
        }
    }
    for (index, modifier) in authored.catalogs.modifiers.iter().enumerate() {
        strings.extend([
            (
                format!("catalogs.modifiers[{index}].id"),
                modifier.id.as_str(),
            ),
            (
                format!("catalogs.modifiers[{index}].label"),
                modifier.label.as_str(),
            ),
            (
                format!("catalogs.modifiers[{index}].summary"),
                modifier.summary.as_str(),
            ),
            (
                format!("catalogs.modifiers[{index}].stackingGroup"),
                modifier.stacking_group.as_str(),
            ),
        ]);
        if let ModifierDurationPolicy::UntilEvent(event) = &modifier.duration_policy {
            strings.push((
                format!("catalogs.modifiers[{index}].durationPolicy.event"),
                event.as_str(),
            ));
        }
        for (adjustment_index, adjustment) in modifier.stat_adjustments.iter().enumerate() {
            strings.extend([
                (
                    format!(
                        "catalogs.modifiers[{index}].statAdjustments[{adjustment_index}].statId"
                    ),
                    adjustment.stat_id.as_str(),
                ),
                (
                    format!(
                        "catalogs.modifiers[{index}].statAdjustments[{adjustment_index}].statLabel"
                    ),
                    adjustment.stat_label.as_str(),
                ),
            ]);
        }
    }
    for (index, action) in authored.catalogs.actions.iter().enumerate() {
        strings.extend([
            (format!("catalogs.actions[{index}].id"), action.id.as_str()),
            (
                format!("catalogs.actions[{index}].abilityId"),
                action.ability_id.as_str(),
            ),
            (
                format!("catalogs.actions[{index}].name"),
                action.name.as_str(),
            ),
            (
                format!("catalogs.actions[{index}].actionText"),
                action.action_text.as_str(),
            ),
            (
                format!("catalogs.actions[{index}].effectText"),
                action.effect_text.as_str(),
            ),
        ]);
        for (cost_index, cost) in action.resource_costs.iter().enumerate() {
            strings.push((
                format!("catalogs.actions[{index}].resourceCosts[{cost_index}].resourceId"),
                cost.resource_id.as_str(),
            ));
        }
        match &action.check {
            CheckDeclaration::Attack(attack) => {
                strings.extend([
                    (
                        format!("catalogs.actions[{index}].check.modifierStatId"),
                        attack.modifier_stat_id.as_str(),
                    ),
                    (
                        format!("catalogs.actions[{index}].check.defense.id"),
                        attack.defense.id.as_str(),
                    ),
                    (
                        format!("catalogs.actions[{index}].check.defense.label"),
                        attack.defense.label.as_str(),
                    ),
                ]);
            }
            CheckDeclaration::SavingThrow(save) => strings.push((
                format!("catalogs.actions[{index}].check.saveStatId"),
                save.save_stat_id.as_str(),
            )),
            CheckDeclaration::Contested(contested) => strings.extend([
                (
                    format!("catalogs.actions[{index}].check.actorStatId"),
                    contested.actor_stat_id.as_str(),
                ),
                (
                    format!("catalogs.actions[{index}].check.targetStatId"),
                    contested.target_stat_id.as_str(),
                ),
            ]),
        }
        if let Some(movement) = &action.movement {
            for (tag_index, tag) in movement.blocking_terrain_tags.iter().enumerate() {
                strings.push((
                    format!("catalogs.actions[{index}].movement.blockingTerrainTags[{tag_index}]"),
                    tag.as_str(),
                ));
            }
            for (tag_index, tag) in movement.difficult_terrain_tags.iter().enumerate() {
                strings.push((
                    format!("catalogs.actions[{index}].movement.difficultTerrainTags[{tag_index}]"),
                    tag.as_str(),
                ));
            }
        }
        collect_action_operation_strings(&mut strings, index, action);
    }
    for (index, scenario) in authored.catalogs.scenarios.iter().enumerate() {
        strings.extend([
            (
                format!("catalogs.scenarios[{index}].id"),
                scenario.id.as_str(),
            ),
            (
                format!("catalogs.scenarios[{index}].title"),
                scenario.title.as_str(),
            ),
            (
                format!("catalogs.scenarios[{index}].summary"),
                scenario.summary.as_str(),
            ),
            (
                format!("catalogs.scenarios[{index}].seedLabel"),
                scenario.seed_label.as_str(),
            ),
            (
                format!("catalogs.scenarios[{index}].rulesetId"),
                scenario.ruleset_id.as_str(),
            ),
            (
                format!("catalogs.scenarios[{index}].selectedActionId"),
                scenario.selected_action_id.as_str(),
            ),
        ]);
        if let Some(policy_id) = scenario.control.automation_policy_id.as_deref() {
            strings.push((
                format!("catalogs.scenarios[{index}].control.automationPolicyId"),
                policy_id,
            ));
        }
        for (participant_index, participant) in scenario.participants.iter().enumerate() {
            strings.extend([
                (
                    format!("catalogs.scenarios[{index}].participants[{participant_index}].id"),
                    participant.id.as_str(),
                ),
                (
                    format!(
                        "catalogs.scenarios[{index}].participants[{participant_index}].entityId"
                    ),
                    participant.entity_id.as_str(),
                ),
                (
                    format!("catalogs.scenarios[{index}].participants[{participant_index}].name"),
                    participant.name.as_str(),
                ),
                (
                    format!("catalogs.scenarios[{index}].participants[{participant_index}].sideId"),
                    participant.side_id.as_str(),
                ),
            ]);
            for (grant_index, grant) in participant.action_grants.iter().enumerate() {
                strings.extend([
                    (format!("catalogs.scenarios[{index}].participants[{participant_index}].actionGrants[{grant_index}].actionId"), grant.action_id.as_str()),
                    (format!("catalogs.scenarios[{index}].participants[{participant_index}].actionGrants[{grant_index}].runtimeActionId"), grant.runtime_action_id.as_str()),
                ]);
            }
        }
    }
    strings
}

fn collect_action_operation_strings<'a>(
    strings: &mut Vec<(String, &'a str)>,
    action_index: usize,
    action: &'a crate::AuthoredActionDefinition,
) {
    for (effect_index, effect) in action.effects.iter().enumerate() {
        let base = format!("catalogs.actions[{action_index}].effects[{effect_index}]");
        match effect {
            AuthoredEffectOperation::Damage(value) => {
                strings.push((format!("{base}.damageType"), value.damage_type.as_str()));
            }
            AuthoredEffectOperation::Heal(value) => {
                strings.push((format!("{base}.healingType"), value.healing_type.as_str()));
            }
            AuthoredEffectOperation::ApplyModifier(value) => {
                strings.push((format!("{base}.modifierId"), value.modifier_id.as_str()));
            }
            AuthoredEffectOperation::ChangeResource(value) => {
                strings.push((format!("{base}.resourceId"), value.resource_id.as_str()));
            }
            AuthoredEffectOperation::OpenReactionWindow(hook) => {
                strings.push((format!("{base}.hookId"), hook.hook_id.as_str()));
                for (option_index, option) in hook.options.iter().enumerate() {
                    strings.push((
                        format!("{base}.options[{option_index}].id"),
                        option.id.as_str(),
                    ));
                }
            }
            AuthoredEffectOperation::GrantTemporaryVitality(_)
            | AuthoredEffectOperation::Move(_) => {}
        }
    }
}

fn catalog_identities(authored: &AuthoredContentPack) -> Vec<(ContentDefinitionKind, Vec<String>)> {
    vec![
        (
            ContentDefinitionKind::Ruleset,
            ids(&authored.catalogs.rulesets, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Entity,
            ids(&authored.catalogs.entities, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Ability,
            ids(&authored.catalogs.abilities, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Class,
            ids(&authored.catalogs.classes, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Stat,
            ids(&authored.catalogs.stat_definitions, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Modifier,
            ids(&authored.catalogs.modifiers, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Item,
            ids(&authored.catalogs.items, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Action,
            ids(&authored.catalogs.actions, |value| &value.id),
        ),
        (
            ContentDefinitionKind::Scenario,
            ids(&authored.catalogs.scenarios, |value| &value.id),
        ),
    ]
}

fn ids<T>(values: &[T], id: impl Fn(&T) -> &String) -> Vec<String> {
    values.iter().map(|value| id(value).clone()).collect()
}

pub(super) fn import_pack_diagnostic(diagnostic: ContentPackDiagnostic) -> ContentImportDiagnostic {
    let path = diagnostic
        .definition_kind
        .map(|kind| format!("catalogs.{}", kind.code()))
        .unwrap_or_else(|| "pack".to_string());
    ContentImportDiagnostic {
        severity: ContentImportDiagnosticSeverity::Error,
        code: ContentImportDiagnosticCode::PackValidation(diagnostic.code),
        path,
        definition_kind: diagnostic.definition_kind,
        definition_id: diagnostic.reference_id,
        message: diagnostic.message,
    }
}

pub(super) fn sort_diagnostics(diagnostics: &mut [ContentImportDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            &left.path,
            left.severity,
            left.code,
            left.definition_kind,
            &left.definition_id,
            &left.message,
        )
            .cmp(&(
                &right.path,
                right.severity,
                right.code,
                right.definition_kind,
                &right.definition_id,
                &right.message,
            ))
    });
}

pub(super) fn rejected(diagnostics: Vec<ContentImportDiagnostic>) -> ContentImportReport {
    ContentImportReport {
        accepted: false,
        diagnostics,
    }
}
