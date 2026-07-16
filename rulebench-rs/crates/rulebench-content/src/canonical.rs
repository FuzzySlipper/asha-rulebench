use crate::{
    AuthoredActionDefinition, AuthoredEffectOperation, CanonicalContentPack, ContentFingerprint,
    ContentPackCatalogs, ContentPackDefinition, ContentPackReference, DerivedStatFormula,
    ModifierDurationPolicy, CONTENT_PACK_SET_FINGERPRINT_ALGORITHM,
};
use rulebench_ruleset::{
    ActionResourcePool, ActionResourceRefreshPolicy, CheckDeclaration, CombatEndPolicy,
    MovementKind, RuleModuleConfiguration, RulesetArtifactProvenance,
};

pub fn canonicalize_content_pack(definition: ContentPackDefinition) -> CanonicalContentPack {
    let mut pack = CanonicalContentPack {
        canonical_version: definition.canonical_version,
        identity: definition.identity,
        title: definition.title,
        summary: definition.summary,
        tags: definition.tags,
        provenance: definition.provenance,
        ruleset: definition.ruleset,
        dependencies: definition.dependencies,
        collision_policy: definition.collision_policy,
        catalogs: definition.catalogs,
        fingerprint: ContentFingerprint {
            algorithm: definition
                .canonical_version
                .fingerprint_algorithm()
                .to_string(),
            value: String::new(),
        },
    };

    canonicalize_pack_fields(&mut pack);
    pack.fingerprint = fingerprint_canonical_pack(&pack);
    pack
}

pub fn fingerprint_content_pack_set(
    root: &ContentPackReference,
    packs: &[ContentPackReference],
) -> ContentFingerprint {
    let mut sorted_packs = packs.to_vec();
    sorted_packs.sort();
    fingerprint_pack_set_fields(root, &sorted_packs)
}

fn canonicalize_pack_fields(pack: &mut CanonicalContentPack) {
    pack.tags.sort();
    pack.tags.dedup();
    pack.dependencies.sort();
    pack.ruleset
        .module_versions
        .sort_by_key(|module| module.module.code());
    canonicalize_catalogs(&mut pack.catalogs);
}

fn canonicalize_catalogs(catalogs: &mut ContentPackCatalogs) {
    catalogs
        .rulesets
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .entities
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .abilities
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .classes
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .stat_definitions
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .modifiers
        .sort_by(|left, right| left.id.cmp(&right.id));
    catalogs.items.sort_by(|left, right| left.id.cmp(&right.id));
    catalogs
        .actions
        .sort_by(|left, right| left.id.cmp(&right.id));

    for ruleset in &mut catalogs.rulesets {
        ruleset.modules.sort_by_key(|module| module.module.code());
        for module in &mut ruleset.modules {
            if let RuleModuleConfiguration::ActionResolution(configuration) =
                &mut module.configuration
            {
                configuration
                    .supported_check_handlers
                    .sort_by_key(|handler| handler.code());
            }
        }
    }
    for entity in &mut catalogs.entities {
        entity.tags.sort();
        entity.tags.dedup();
        entity
            .damage_adjustments
            .sort_by(|left, right| left.damage_type.cmp(&right.damage_type));
    }
    for ability in &mut catalogs.abilities {
        ability.tags.sort();
        ability.tags.dedup();
    }
    for class in &mut catalogs.classes {
        class.tags.sort();
        class.tags.dedup();
        class
            .prerequisites
            .sort_by(|left, right| left.stat_id.cmp(&right.stat_id));
        class.level_grants.sort_by_key(|grant| grant.level);
        for grant in &mut class.level_grants {
            grant.granted_modifier_ids.sort();
            grant.granted_modifier_ids.dedup();
            grant.granted_ability_ids.sort();
            grant.granted_ability_ids.dedup();
            grant
                .granted_resource_pools
                .sort_by(|left, right| left.id.cmp(&right.id));
        }
    }
    for modifier in &mut catalogs.modifiers {
        modifier
            .stat_adjustments
            .sort_by(|left, right| left.stat_id.cmp(&right.stat_id));
    }
    for item in &mut catalogs.items {
        item.tags.sort();
        item.tags.dedup();
        item.requirements
            .sort_by(|left, right| left.stat_id.cmp(&right.stat_id));
        item.granted_modifier_ids.sort();
        item.granted_modifier_ids.dedup();
        item.granted_ability_ids.sort();
        item.granted_ability_ids.dedup();
        item.granted_resource_pools
            .sort_by(|left, right| left.id.cmp(&right.id));
    }
    for action in &mut catalogs.actions {
        action
            .resource_costs
            .sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
        if let Some(movement) = &mut action.movement {
            movement.blocking_terrain_tags.sort();
            movement.blocking_terrain_tags.dedup();
            movement.difficult_terrain_tags.sort();
            movement.difficult_terrain_tags.dedup();
        }
        for operation in &mut action.effects {
            if let AuthoredEffectOperation::OpenReactionWindow(hook) = operation {
                hook.eligible_reactors.sort();
                hook.eligible_reactors.dedup();
                hook.options.sort_by(|left, right| left.id.cmp(&right.id));
            }
        }
    }
}

fn fingerprint_canonical_pack(pack: &CanonicalContentPack) -> ContentFingerprint {
    let mut encoder = FingerprintEncoder::new();
    let algorithm = pack.canonical_version.fingerprint_algorithm();
    encoder.feed_str(algorithm);
    encoder.feed_str("canonicalContentPack");
    encoder.feed_str(&pack.identity.id);
    encoder.feed_str(&pack.identity.version);
    encoder.feed_str(&pack.title);
    encoder.feed_str(&pack.summary);
    encoder.feed_strings(&pack.tags);
    encoder.feed_str(pack.provenance.source_kind.code());
    encoder.feed_str(&pack.provenance.source_id);
    encoder.feed_optional_str(pack.provenance.authored_by.as_deref());
    feed_ruleset_provenance(&mut encoder, &pack.ruleset);
    encoder.feed_sequence(&pack.dependencies, feed_content_pack_reference);
    encoder.feed_str(pack.collision_policy.code());
    feed_catalogs(&mut encoder, &pack.catalogs);

    ContentFingerprint {
        algorithm: algorithm.to_string(),
        value: encoder.finish_hex(),
    }
}

fn fingerprint_pack_set_fields(
    root: &ContentPackReference,
    packs: &[ContentPackReference],
) -> ContentFingerprint {
    let mut encoder = FingerprintEncoder::new();
    encoder.feed_str(CONTENT_PACK_SET_FINGERPRINT_ALGORITHM);
    encoder.feed_str("contentPackSet");
    feed_content_pack_reference(&mut encoder, root);
    encoder.feed_sequence(packs, feed_content_pack_reference);

    ContentFingerprint {
        algorithm: CONTENT_PACK_SET_FINGERPRINT_ALGORITHM.to_string(),
        value: encoder.finish_hex(),
    }
}

fn feed_content_pack_reference(encoder: &mut FingerprintEncoder, reference: &ContentPackReference) {
    encoder.feed_str("contentPackReference");
    encoder.feed_str(&reference.id);
    encoder.feed_str(&reference.version);
    encoder.feed_str(&reference.fingerprint.algorithm);
    encoder.feed_str(&reference.fingerprint.value);
}

fn feed_ruleset_provenance(
    encoder: &mut FingerprintEncoder,
    provenance: &RulesetArtifactProvenance,
) {
    encoder.feed_str("rulesetProvenance");
    encoder.feed_str(&provenance.ruleset_id);
    encoder.feed_str(&provenance.ruleset_version);
    encoder.feed_u32(provenance.module_versions.len() as u32);
    for module in &provenance.module_versions {
        encoder.feed_str(module.module.code());
        encoder.feed_str(&module.version);
    }
    encoder.feed_str(&provenance.effect_operation_vocabulary_version);
}

fn feed_catalogs(encoder: &mut FingerprintEncoder, catalogs: &ContentPackCatalogs) {
    encoder.feed_str("rulesets");
    encoder.feed_u32(catalogs.rulesets.len() as u32);
    for ruleset in &catalogs.rulesets {
        encoder.feed_str(&ruleset.id);
        encoder.feed_str(&ruleset.name);
        encoder.feed_str(&ruleset.version);
        encoder.feed_str(&ruleset.summary);
        encoder.feed_u32(ruleset.modules.len() as u32);
        for module in &ruleset.modules {
            encoder.feed_str(module.module.code());
            encoder.feed_str(&module.version);
            match &module.configuration {
                RuleModuleConfiguration::ActionResolution(configuration) => {
                    encoder.feed_str("actionResolution");
                    encoder.feed_str(configuration.targeting_policy.code());
                    encoder.feed_u32(configuration.supported_check_handlers.len() as u32);
                    for handler in &configuration.supported_check_handlers {
                        encoder.feed_str(handler.code());
                    }
                }
                RuleModuleConfiguration::TurnControl(configuration) => {
                    encoder.feed_str("turnControl");
                    encoder.feed_str(configuration.turn_order_policy.code());
                    encoder.feed_str(configuration.combat_end_policy.code());
                    match &configuration.combat_end_policy {
                        CombatEndPolicy::ObjectiveSideVictory { side_id } => {
                            encoder.feed_bool(true);
                            encoder.feed_str(side_id);
                        }
                        CombatEndPolicy::LastSideStanding | CombatEndPolicy::ExplicitOnly => {
                            encoder.feed_bool(false);
                        }
                    }
                }
            }
        }
    }

    encoder.feed_str("entities");
    encoder.feed_u32(catalogs.entities.len() as u32);
    for entity in &catalogs.entities {
        encoder.feed_str(&entity.id);
        encoder.feed_str(&entity.name);
        encoder.feed_str(&entity.summary);
        encoder.feed_strings(&entity.tags);
        encoder.feed_u32(entity.damage_adjustments.len() as u32);
        for adjustment in &entity.damage_adjustments {
            encoder.feed_str(&adjustment.damage_type);
            encoder.feed_str(adjustment.policy.code());
        }
    }

    encoder.feed_str("abilities");
    encoder.feed_u32(catalogs.abilities.len() as u32);
    for ability in &catalogs.abilities {
        encoder.feed_str(&ability.id);
        encoder.feed_str(&ability.name);
        encoder.feed_str(ability.kind.code());
        encoder.feed_str(&ability.summary);
        encoder.feed_strings(&ability.tags);
    }

    encoder.feed_str("classes");
    encoder.feed_u32(catalogs.classes.len() as u32);
    for class in &catalogs.classes {
        encoder.feed_str(&class.id);
        encoder.feed_str(&class.name);
        encoder.feed_str(&class.version);
        encoder.feed_str(&class.summary);
        encoder.feed_strings(&class.tags);
        encoder.feed_u32(class.prerequisites.len() as u32);
        for requirement in &class.prerequisites {
            encoder.feed_str(&requirement.stat_id);
            encoder.feed_i32(requirement.minimum);
        }
        encoder.feed_u32(class.level_grants.len() as u32);
        for grant in &class.level_grants {
            encoder.feed_u32(grant.level);
            encoder.feed_strings(&grant.granted_modifier_ids);
            encoder.feed_strings(&grant.granted_ability_ids);
            encoder.feed_sequence(&grant.granted_resource_pools, feed_action_resource_pool);
        }
    }

    encoder.feed_str("statDefinitions");
    encoder.feed_u32(catalogs.stat_definitions.len() as u32);
    for definition in &catalogs.stat_definitions {
        encoder.feed_str(&definition.id);
        encoder.feed_str(&definition.label);
        encoder.feed_str(definition.kind.code());
        match &definition.formula {
            Some(formula) => {
                encoder.feed_bool(true);
                feed_derived_stat_formula(encoder, formula);
            }
            None => encoder.feed_bool(false),
        }
        encoder.feed_str(&definition.summary);
    }

    encoder.feed_str("modifiers");
    encoder.feed_u32(catalogs.modifiers.len() as u32);
    for modifier in &catalogs.modifiers {
        encoder.feed_str(&modifier.id);
        encoder.feed_str(&modifier.label);
        encoder.feed_str(&modifier.summary);
        encoder.feed_str(modifier.default_tenure.code());
        encoder.feed_str(&modifier.stacking_group);
        encoder.feed_str(modifier.stacking_policy.code());
        feed_modifier_duration(encoder, &modifier.duration_policy);
        encoder.feed_u32(modifier.stat_adjustments.len() as u32);
        for adjustment in &modifier.stat_adjustments {
            encoder.feed_str(&adjustment.stat_id);
            encoder.feed_str(&adjustment.stat_label);
            encoder.feed_i32(adjustment.delta);
        }
    }

    encoder.feed_str("items");
    encoder.feed_u32(catalogs.items.len() as u32);
    for item in &catalogs.items {
        encoder.feed_str(&item.id);
        encoder.feed_str(&item.name);
        encoder.feed_str(&item.summary);
        encoder.feed_strings(&item.tags);
        encoder.feed_str(&item.equipment_slot);
        encoder.feed_u32(item.requirements.len() as u32);
        for requirement in &item.requirements {
            encoder.feed_str(&requirement.stat_id);
            encoder.feed_i32(requirement.minimum);
        }
        encoder.feed_strings(&item.granted_modifier_ids);
        encoder.feed_strings(&item.granted_ability_ids);
        encoder.feed_sequence(&item.granted_resource_pools, feed_action_resource_pool);
    }

    encoder.feed_str("actions");
    encoder.feed_u32(catalogs.actions.len() as u32);
    for action in &catalogs.actions {
        feed_action(encoder, action);
    }
}

fn feed_action_resource_pool(encoder: &mut FingerprintEncoder, pool: &ActionResourcePool) {
    encoder.feed_str(&pool.id);
    encoder.feed_str(pool.kind.code());
    encoder.feed_u32(pool.maximum);
    encoder.feed_str(pool.refresh_policy.code());
    if let ActionResourceRefreshPolicy::Turns(turns) = pool.refresh_policy {
        encoder.feed_u32(turns);
    }
}

fn feed_derived_stat_formula(encoder: &mut FingerprintEncoder, formula: &DerivedStatFormula) {
    encoder.feed_str(formula.code());
    match formula {
        DerivedStatFormula::Constant { value } => encoder.feed_i32(*value),
        DerivedStatFormula::StatReference { stat_id } => encoder.feed_str(stat_id),
        DerivedStatFormula::Sum { operands } | DerivedStatFormula::Product { operands } => {
            encoder.feed_u32(operands.len() as u32);
            for operand in operands {
                feed_derived_stat_formula(encoder, operand);
            }
        }
        DerivedStatFormula::Difference {
            minuend,
            subtrahend,
        } => {
            feed_derived_stat_formula(encoder, minuend);
            feed_derived_stat_formula(encoder, subtrahend);
        }
    }
}

fn feed_modifier_duration(encoder: &mut FingerprintEncoder, duration: &ModifierDurationPolicy) {
    match duration {
        ModifierDurationPolicy::Permanent => encoder.feed_str("permanent"),
        ModifierDurationPolicy::Turns(turns) => {
            encoder.feed_str("turns");
            encoder.feed_u32(*turns);
        }
        ModifierDurationPolicy::Rounds(rounds) => {
            encoder.feed_str("rounds");
            encoder.feed_u32(*rounds);
        }
        ModifierDurationPolicy::UntilEvent(event) => {
            encoder.feed_str("untilEvent");
            encoder.feed_str(event);
        }
    }
}

fn feed_action(encoder: &mut FingerprintEncoder, action: &AuthoredActionDefinition) {
    encoder.feed_str(&action.id);
    encoder.feed_str(&action.ability_id);
    encoder.feed_str(&action.name);
    encoder.feed_str(match action.targeting.target_kind {
        rulebench_ruleset::TargetKind::Combatant => "combatant",
        rulebench_ruleset::TargetKind::Area => "area",
    });
    encoder.feed_str(match action.targeting.selection {
        rulebench_ruleset::TargetSelection::Single => "single",
        rulebench_ruleset::TargetSelection::Multiple => "multiple",
    });
    encoder.feed_str(match action.targeting.team_constraint {
        rulebench_ruleset::TargetTeamConstraint::Hostile => "hostile",
        rulebench_ruleset::TargetTeamConstraint::Ally => "ally",
        rulebench_ruleset::TargetTeamConstraint::Any => "any",
    });
    encoder.feed_u32(action.targeting.maximum_range);
    encoder.feed_str(match action.targeting.visibility_requirement {
        rulebench_ruleset::VisibilityRequirement::Required => "required",
        rulebench_ruleset::VisibilityRequirement::Ignored => "ignored",
    });
    match &action.targeting.operation_pipeline {
        Some(pipeline) => {
            encoder.feed_str("operationPipelineV2");
            encoder.feed_str(rulebench_ruleset::OperationPipelineV2::VOCABULARY_VERSION);
            encoder.feed_u32(pipeline.maximum_targets);
            match &pipeline.area {
                Some(area) => {
                    encoder.feed_str(match area.shape {
                        rulebench_ruleset::AreaShape::ManhattanBurst => "manhattanBurst",
                    });
                    encoder.feed_u32(area.radius);
                }
                None => encoder.feed_str("noArea"),
            }
            encoder.feed_str(match pipeline.roll_policy {
                rulebench_ruleset::ActionRollPolicy::Shared => "shared",
                rulebench_ruleset::ActionRollPolicy::PerTarget => "perTarget",
                rulebench_ruleset::ActionRollPolicy::NoRoll => "noRoll",
            });
            encoder.feed_str(match pipeline.failure_policy {
                rulebench_ruleset::TargetFailurePolicy::Atomic => "atomic",
            });
            encoder.feed_str(match pipeline.target_order {
                rulebench_ruleset::TargetOrderPolicy::CanonicalId => "canonicalId",
            });
        }
        None => encoder.feed_str("noOperationPipeline"),
    }
    feed_check(encoder, &action.check);
    encoder.feed_u32(action.effects.len() as u32);
    for operation in &action.effects {
        feed_authored_effect_operation(encoder, operation);
    }
    encoder.feed_u32(action.resource_costs.len() as u32);
    for cost in &action.resource_costs {
        encoder.feed_str(&cost.resource_id);
        encoder.feed_u32(cost.amount);
    }
    match &action.movement {
        Some(movement) => {
            encoder.feed_str("movement");
            encoder.feed_u32(movement.allowance);
            encoder.feed_str(match movement.topology {
                rulebench_ruleset::MovementTopology::OrthogonalManhattan => "orthogonalManhattan",
            });
            encoder.feed_strings(&movement.blocking_terrain_tags);
            encoder.feed_strings(&movement.difficult_terrain_tags);
        }
        None => encoder.feed_str("noMovement"),
    }
    encoder.feed_str(&action.action_text);
    encoder.feed_str(&action.effect_text);
}

fn feed_check(encoder: &mut FingerprintEncoder, check: &CheckDeclaration) {
    match check {
        CheckDeclaration::Attack(attack) => {
            encoder.feed_str("attack");
            encoder.feed_i32(attack.modifier);
            encoder.feed_str(&attack.modifier_stat_id);
            encoder.feed_str(&attack.defense.id);
            encoder.feed_str(&attack.defense.label);
        }
        CheckDeclaration::SavingThrow(save) => {
            encoder.feed_str("savingThrow");
            encoder.feed_str(&save.save_stat_id);
            encoder.feed_i32(save.difficulty_class);
        }
        CheckDeclaration::Contested(contested) => {
            encoder.feed_str("contested");
            encoder.feed_str(&contested.actor_stat_id);
            encoder.feed_str(&contested.target_stat_id);
        }
    }
}

fn feed_authored_effect_operation(
    encoder: &mut FingerprintEncoder,
    operation: &AuthoredEffectOperation,
) {
    encoder.feed_str(operation.code());
    match operation {
        AuthoredEffectOperation::Damage(damage) => {
            encoder.feed_i32(damage.damage_bonus);
            encoder.feed_str(&damage.damage_type);
        }
        AuthoredEffectOperation::Heal(healing) => {
            encoder.feed_i32(healing.healing_bonus);
            encoder.feed_str(&healing.healing_type);
        }
        AuthoredEffectOperation::GrantTemporaryVitality(vitality) => {
            encoder.feed_i32(vitality.vitality_bonus);
        }
        AuthoredEffectOperation::ApplyModifier(modifier) => {
            encoder.feed_str(&modifier.modifier_id);
        }
        AuthoredEffectOperation::Move(movement) => {
            encoder.feed_u32(movement.maximum_distance);
            encoder.feed_str(match movement.movement_kind {
                MovementKind::Push => "push",
                MovementKind::Pull => "pull",
                MovementKind::Shift => "shift",
            });
        }
        AuthoredEffectOperation::ChangeResource(change) => {
            encoder.feed_str(&change.resource_id);
            encoder.feed_i32(change.delta);
        }
        AuthoredEffectOperation::OpenReactionWindow(hook) => {
            encoder.feed_str(&hook.hook_id);
            encoder.feed_str(hook.window.code());
            encoder.feed_u32(hook.eligible_reactors.len() as u32);
            for selector in &hook.eligible_reactors {
                encoder.feed_str(selector.code());
            }
            encoder.feed_u32(hook.options.len() as u32);
            for option in &hook.options {
                encoder.feed_str(&option.id);
                encoder.feed_str(option.reactor.code());
                encoder.feed_bool(option.opens_nested_window);
            }
            encoder.feed_u32(hook.maximum_nested_depth);
        }
    }
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

struct FingerprintEncoder {
    hash: u64,
}

impl FingerprintEncoder {
    const fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    fn feed_sequence<T>(&mut self, values: &[T], mut feed: impl FnMut(&mut Self, &T)) {
        self.feed_u32(values.len() as u32);
        for value in values {
            feed(self, value);
        }
    }

    fn feed_strings(&mut self, values: &[String]) {
        self.feed_u32(values.len() as u32);
        for value in values {
            self.feed_str(value);
        }
    }

    fn feed_optional_str(&mut self, value: Option<&str>) {
        match value {
            Some(value) => {
                self.feed_bool(true);
                self.feed_str(value);
            }
            None => self.feed_bool(false),
        }
    }

    fn feed_str(&mut self, value: &str) {
        self.feed_u32(value.len() as u32);
        for byte in value.as_bytes() {
            self.feed_byte(*byte);
        }
    }

    fn feed_bool(&mut self, value: bool) {
        self.feed_byte(u8::from(value));
    }

    fn feed_i32(&mut self, value: i32) {
        for byte in value.to_le_bytes() {
            self.feed_byte(byte);
        }
    }

    fn feed_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.feed_byte(byte);
        }
    }

    fn feed_byte(&mut self, byte: u8) {
        self.hash ^= u64::from(byte);
        self.hash = self.hash.wrapping_mul(FNV_PRIME);
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthoredModifierEffectOperation, AuthoredReactionHookEffectOperation,
        AuthoredReactionOptionDeclaration, AuthoredTargetingDeclaration,
        ContentPackCanonicalVersion, ContentPackCollisionPolicy, ContentPackIdentity,
        ContentPackProvenance, ContentPackSourceKind, ModifierDefinition, ModifierStackingPolicy,
        ModifierStatAdjustment, ReactionParticipantSelector,
    };
    use rulebench_ruleset::{
        AbilityDefinition, AbilityDefinitionKind, ActionResourceCost, AttackCheckDeclaration,
        DamageEffectOperation, DefenseReference, ModifierTenure, MovementActionDeclaration,
        MovementTopology, ReactionWindow, RulesetModuleProvenance, TargetKind, TargetSelection,
        TargetTeamConstraint, VisibilityRequirement,
    };

    #[test]
    fn equivalent_set_and_catalog_order_fingerprints_identically() {
        let first = canonicalize_content_pack(test_definition(false));
        let second = canonicalize_content_pack(test_definition(true));

        assert_eq!(first.fingerprint, second.fingerprint);
        assert_eq!(
            fingerprint_content_pack_set(&first.exact_reference(), &first.dependencies),
            fingerprint_content_pack_set(&second.exact_reference(), &second.dependencies)
        );
    }

    #[test]
    fn material_field_change_changes_fingerprint() {
        let original = canonicalize_content_pack(test_definition(false));
        let mut changed_definition = test_definition(false);
        changed_definition.catalogs.abilities[0].summary = "Changed behavior text.".to_string();
        let changed = canonicalize_content_pack(changed_definition);

        assert_ne!(original.fingerprint, changed.fingerprint);
    }

    #[test]
    fn portable_action_material_fields_and_effect_order_change_v1_fingerprint() {
        let original = canonicalize_content_pack(portable_action_definition());
        assert_eq!(
            original.fingerprint.algorithm,
            ContentPackCanonicalVersion::V1.fingerprint_algorithm()
        );

        let mut changes = Vec::new();
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].name = "Changed action".to_string();
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].targeting.maximum_range += 1;
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].check = CheckDeclaration::Attack(AttackCheckDeclaration {
            modifier: 3,
            modifier_stat_id: "focus".to_string(),
            defense: DefenseReference {
                id: "guard".to_string(),
                label: "Guard".to_string(),
            },
        });
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].resource_costs[0].amount += 1;
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].effects.swap(0, 1);
        changes.push(changed);
        let mut changed = portable_action_definition();
        let hook = match &mut changed.catalogs.actions[0].effects[2] {
            AuthoredEffectOperation::OpenReactionWindow(hook) => hook,
            _ => panic!("third operation is the reaction fixture"),
        };
        hook.eligible_reactors[0] = ReactionParticipantSelector::ActorAllies;
        hook.options[0].reactor = ReactionParticipantSelector::ActorAllies;
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.actions[0].movement = Some(MovementActionDeclaration {
            allowance: 4,
            topology: MovementTopology::OrthogonalManhattan,
            blocking_terrain_tags: vec!["wall".to_string()],
            difficult_terrain_tags: vec!["mud".to_string()],
        });
        changes.push(changed);
        let mut changed = portable_action_definition();
        changed.catalogs.modifiers[0].summary = "Changed modifier".to_string();
        changes.push(changed);

        for changed in changes {
            assert_ne!(
                original.fingerprint,
                canonicalize_content_pack(changed).fingerprint
            );
        }
    }

    #[test]
    fn portable_action_unordered_sets_canonicalize_without_reordering_effects() {
        let first = canonicalize_content_pack(portable_action_definition());
        let mut reordered = portable_action_definition();
        reordered.catalogs.actions[0].resource_costs.reverse();
        let hook = match &mut reordered.catalogs.actions[0].effects[2] {
            AuthoredEffectOperation::OpenReactionWindow(hook) => hook,
            _ => panic!("third operation is the reaction fixture"),
        };
        hook.eligible_reactors.reverse();
        hook.options.reverse();
        let second = canonicalize_content_pack(reordered);

        assert_eq!(first.fingerprint, second.fingerprint);
        assert_eq!(first.catalogs.actions[0].effects[0].code(), "damage");
        assert_eq!(first.catalogs.actions[0].effects[1].code(), "applyModifier");
        assert_eq!(
            first.catalogs.actions[0].effects[2].code(),
            "openReactionWindow"
        );
    }

    fn portable_action_definition() -> ContentPackDefinition {
        let mut definition = test_definition(false);
        definition.canonical_version = ContentPackCanonicalVersion::V1;
        definition.catalogs.modifiers = vec![ModifierDefinition {
            id: "modifier.anchor".to_string(),
            label: "Anchor".to_string(),
            summary: "Portable modifier".to_string(),
            default_tenure: ModifierTenure::Temporary,
            stacking_group: "anchor".to_string(),
            stacking_policy: ModifierStackingPolicy::Refresh,
            duration_policy: ModifierDurationPolicy::Turns(1),
            stat_adjustments: vec![ModifierStatAdjustment {
                stat_id: "mobility".to_string(),
                stat_label: "Mobility".to_string(),
                delta: -1,
            }],
        }];
        definition.catalogs.actions = vec![AuthoredActionDefinition {
            id: "action.arc".to_string(),
            ability_id: "arc".to_string(),
            name: "Arc".to_string(),
            targeting: AuthoredTargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 6,
                visibility_requirement: VisibilityRequirement::Required,
                operation_pipeline: None,
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 2,
                modifier_stat_id: "focus".to_string(),
                defense: DefenseReference {
                    id: "guard".to_string(),
                    label: "Guard".to_string(),
                },
            }),
            effects: vec![
                AuthoredEffectOperation::Damage(DamageEffectOperation {
                    damage_bonus: 4,
                    damage_type: "arcane".to_string(),
                }),
                AuthoredEffectOperation::ApplyModifier(AuthoredModifierEffectOperation {
                    modifier_id: "modifier.anchor".to_string(),
                }),
                AuthoredEffectOperation::OpenReactionWindow(AuthoredReactionHookEffectOperation {
                    hook_id: "arc-response".to_string(),
                    window: ReactionWindow::AfterEffect,
                    eligible_reactors: vec![
                        ReactionParticipantSelector::TargetAllies,
                        ReactionParticipantSelector::DeclaredTargets,
                    ],
                    options: vec![
                        AuthoredReactionOptionDeclaration {
                            id: "ward".to_string(),
                            reactor: ReactionParticipantSelector::TargetAllies,
                            opens_nested_window: false,
                        },
                        AuthoredReactionOptionDeclaration {
                            id: "brace".to_string(),
                            reactor: ReactionParticipantSelector::DeclaredTargets,
                            opens_nested_window: false,
                        },
                    ],
                    maximum_nested_depth: 0,
                }),
            ],
            resource_costs: vec![
                ActionResourceCost {
                    resource_id: "spell-slot".to_string(),
                    amount: 1,
                },
                ActionResourceCost::standard_action(),
            ],
            movement: None,
            action_text: "Cast Arc.".to_string(),
            effect_text: "Damage and anchor.".to_string(),
        }];
        definition
    }

    fn test_definition(reverse_order: bool) -> ContentPackDefinition {
        let mut tags = vec!["combat".to_string(), "fixture".to_string()];
        let mut dependencies = vec![
            test_reference("base", "1", "0001"),
            test_reference("expansion", "2", "0002"),
        ];
        let mut abilities = vec![
            AbilityDefinition {
                id: "arc".to_string(),
                name: "Arc".to_string(),
                kind: AbilityDefinitionKind::Spell,
                summary: "A material ability field.".to_string(),
                tags: vec!["lightning".to_string(), "spell".to_string()],
            },
            AbilityDefinition {
                id: "guard".to_string(),
                name: "Guard".to_string(),
                kind: AbilityDefinitionKind::Ability,
                summary: "Defensive stance.".to_string(),
                tags: vec!["stance".to_string()],
            },
        ];
        if reverse_order {
            tags.reverse();
            dependencies.reverse();
            abilities.reverse();
            abilities[1].tags.reverse();
        }

        ContentPackDefinition {
            canonical_version: ContentPackCanonicalVersion::V0,
            identity: ContentPackIdentity::new("test-pack", "1"),
            title: "Test pack".to_string(),
            summary: "Fingerprint fixture.".to_string(),
            tags,
            provenance: ContentPackProvenance {
                source_kind: ContentPackSourceKind::Embedded,
                source_id: "pack-tests".to_string(),
                authored_by: Some("rulebench".to_string()),
            },
            ruleset: RulesetArtifactProvenance {
                ruleset_id: "test-rules".to_string(),
                ruleset_version: "1".to_string(),
                module_versions: vec![RulesetModuleProvenance {
                    module: rulebench_ruleset::RuleModuleId::ActionResolution,
                    version: "1".to_string(),
                }],
                effect_operation_vocabulary_version: "1".to_string(),
            },
            dependencies,
            collision_policy: ContentPackCollisionPolicy::Reject,
            catalogs: ContentPackCatalogs {
                abilities,
                ..ContentPackCatalogs::default()
            },
        }
    }

    fn test_reference(id: &str, version: &str, value: &str) -> ContentPackReference {
        ContentPackReference {
            id: id.to_string(),
            version: version.to_string(),
            fingerprint: ContentFingerprint {
                algorithm: ContentPackCanonicalVersion::V0
                    .fingerprint_algorithm()
                    .to_string(),
                value: value.to_string(),
            },
        }
    }
}
