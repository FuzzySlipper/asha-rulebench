use rulebench_rules::{
    ActionResourceCost, ActionRollPolicy, AreaShape, AreaTargetingDeclaration,
    AttackCheckDeclaration, AuthoredActionDefinition, AuthoredEffectOperation,
    AuthoredModifierEffectOperation, AuthoredReactionHookEffectOperation,
    AuthoredReactionOptionDeclaration, AuthoredTargetingDeclaration, CheckDeclaration,
    ContestedCheckDeclaration, DamageEffectOperation, DefenseReference, HealingEffectOperation,
    ModifierDefinition, ModifierDurationPolicy, ModifierStackingPolicy, ModifierStatAdjustment,
    ModifierTenure, MovementActionDeclaration, MovementEffectOperation, MovementKind,
    MovementTopology, OperationPipelineV2, ReactionParticipantSelector, ReactionWindow,
    ResourceChangeEffectOperation, SavingThrowCheckDeclaration, TargetFailurePolicy, TargetKind,
    TargetOrderPolicy, TargetSelection, TargetTeamConstraint, TemporaryVitalityEffectOperation,
    VisibilityRequirement,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionDefinitionDto {
    pub id: String,
    pub ability_id: String,
    pub name: String,
    pub targeting: AuthoredTargetingDeclarationDto,
    pub check: AuthoredCheckDeclarationDto,
    pub effects: Vec<AuthoredEffectOperationDto>,
    #[serde(default)]
    pub resource_costs: Vec<AuthoredActionResourceCostDto>,
    #[serde(default)]
    pub movement: Option<AuthoredMovementActionDeclarationDto>,
    pub action_text: String,
    pub effect_text: String,
}

impl AuthoredActionDefinitionDto {
    pub(crate) fn to_authority(&self) -> AuthoredActionDefinition {
        AuthoredActionDefinition {
            id: self.id.clone(),
            ability_id: self.ability_id.clone(),
            name: self.name.clone(),
            targeting: self.targeting.to_authority(),
            check: self.check.to_authority(),
            effects: self
                .effects
                .iter()
                .map(AuthoredEffectOperationDto::to_authority)
                .collect(),
            resource_costs: self
                .resource_costs
                .iter()
                .map(AuthoredActionResourceCostDto::to_authority)
                .collect(),
            movement: self
                .movement
                .as_ref()
                .map(AuthoredMovementActionDeclarationDto::to_authority),
            action_text: self.action_text.clone(),
            effect_text: self.effect_text.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredTargetingDeclarationDto {
    pub target_kind: AuthoredTargetKindDto,
    pub selection: AuthoredTargetSelectionDto,
    pub team_constraint: AuthoredTargetTeamConstraintDto,
    pub maximum_range: u32,
    pub visibility_requirement: AuthoredVisibilityRequirementDto,
    #[serde(default)]
    pub operation_pipeline: Option<AuthoredOperationPipelineDto>,
}

impl AuthoredTargetingDeclarationDto {
    fn to_authority(&self) -> AuthoredTargetingDeclaration {
        AuthoredTargetingDeclaration {
            target_kind: self.target_kind.to_authority(),
            selection: self.selection.to_authority(),
            team_constraint: self.team_constraint.to_authority(),
            maximum_range: self.maximum_range,
            visibility_requirement: self.visibility_requirement.to_authority(),
            operation_pipeline: self
                .operation_pipeline
                .as_ref()
                .map(AuthoredOperationPipelineDto::to_authority),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTargetKindDto {
    Combatant,
    Area,
}

impl AuthoredTargetKindDto {
    const fn to_authority(self) -> TargetKind {
        match self {
            Self::Combatant => TargetKind::Combatant,
            Self::Area => TargetKind::Area,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTargetSelectionDto {
    Single,
    Multiple,
}

impl AuthoredTargetSelectionDto {
    const fn to_authority(self) -> TargetSelection {
        match self {
            Self::Single => TargetSelection::Single,
            Self::Multiple => TargetSelection::Multiple,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTargetTeamConstraintDto {
    Hostile,
    Ally,
    Any,
}

impl AuthoredTargetTeamConstraintDto {
    const fn to_authority(self) -> TargetTeamConstraint {
        match self {
            Self::Hostile => TargetTeamConstraint::Hostile,
            Self::Ally => TargetTeamConstraint::Ally,
            Self::Any => TargetTeamConstraint::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredVisibilityRequirementDto {
    Required,
    Ignored,
}

impl AuthoredVisibilityRequirementDto {
    const fn to_authority(self) -> VisibilityRequirement {
        match self {
            Self::Required => VisibilityRequirement::Required,
            Self::Ignored => VisibilityRequirement::Ignored,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredOperationPipelineDto {
    pub maximum_targets: u32,
    #[serde(default)]
    pub area: Option<AuthoredAreaTargetingDeclarationDto>,
    pub roll_policy: AuthoredActionRollPolicyDto,
    pub failure_policy: AuthoredTargetFailurePolicyDto,
    pub target_order: AuthoredTargetOrderPolicyDto,
}

impl AuthoredOperationPipelineDto {
    fn to_authority(&self) -> OperationPipelineV2 {
        OperationPipelineV2 {
            maximum_targets: self.maximum_targets,
            area: self
                .area
                .as_ref()
                .map(AuthoredAreaTargetingDeclarationDto::to_authority),
            roll_policy: self.roll_policy.to_authority(),
            failure_policy: self.failure_policy.to_authority(),
            target_order: self.target_order.to_authority(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredAreaTargetingDeclarationDto {
    pub shape: AuthoredAreaShapeDto,
    pub radius: u32,
}

impl AuthoredAreaTargetingDeclarationDto {
    fn to_authority(&self) -> AreaTargetingDeclaration {
        AreaTargetingDeclaration {
            shape: self.shape.to_authority(),
            radius: self.radius,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredAreaShapeDto {
    ManhattanBurst,
}

impl AuthoredAreaShapeDto {
    const fn to_authority(self) -> AreaShape {
        match self {
            Self::ManhattanBurst => AreaShape::ManhattanBurst,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredActionRollPolicyDto {
    Shared,
    PerTarget,
    NoRoll,
}

impl AuthoredActionRollPolicyDto {
    const fn to_authority(self) -> ActionRollPolicy {
        match self {
            Self::Shared => ActionRollPolicy::Shared,
            Self::PerTarget => ActionRollPolicy::PerTarget,
            Self::NoRoll => ActionRollPolicy::NoRoll,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTargetFailurePolicyDto {
    Atomic,
}

impl AuthoredTargetFailurePolicyDto {
    const fn to_authority(self) -> TargetFailurePolicy {
        match self {
            Self::Atomic => TargetFailurePolicy::Atomic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredTargetOrderPolicyDto {
    CanonicalId,
}

impl AuthoredTargetOrderPolicyDto {
    const fn to_authority(self) -> TargetOrderPolicy {
        match self {
            Self::CanonicalId => TargetOrderPolicy::CanonicalId,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AuthoredCheckDeclarationDto {
    Attack {
        modifier: i32,
        modifier_stat_id: String,
        defense: AuthoredDefenseReferenceDto,
    },
    SavingThrow {
        save_stat_id: String,
        difficulty_class: i32,
    },
    Contested {
        actor_stat_id: String,
        target_stat_id: String,
    },
}

impl AuthoredCheckDeclarationDto {
    fn to_authority(&self) -> CheckDeclaration {
        match self {
            Self::Attack {
                modifier,
                modifier_stat_id,
                defense,
            } => CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: *modifier,
                modifier_stat_id: modifier_stat_id.clone(),
                defense: defense.to_authority(),
            }),
            Self::SavingThrow {
                save_stat_id,
                difficulty_class,
            } => CheckDeclaration::SavingThrow(SavingThrowCheckDeclaration {
                save_stat_id: save_stat_id.clone(),
                difficulty_class: *difficulty_class,
            }),
            Self::Contested {
                actor_stat_id,
                target_stat_id,
            } => CheckDeclaration::Contested(ContestedCheckDeclaration {
                actor_stat_id: actor_stat_id.clone(),
                target_stat_id: target_stat_id.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredDefenseReferenceDto {
    pub id: String,
    pub label: String,
}

impl AuthoredDefenseReferenceDto {
    fn to_authority(&self) -> DefenseReference {
        DefenseReference {
            id: self.id.clone(),
            label: self.label.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredActionResourceCostDto {
    pub resource_id: String,
    pub amount: u32,
}

impl AuthoredActionResourceCostDto {
    fn to_authority(&self) -> ActionResourceCost {
        ActionResourceCost {
            resource_id: self.resource_id.clone(),
            amount: self.amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredMovementActionDeclarationDto {
    pub allowance: u32,
    pub topology: AuthoredMovementTopologyDto,
    #[serde(default)]
    pub blocking_terrain_tags: Vec<String>,
    #[serde(default)]
    pub difficult_terrain_tags: Vec<String>,
}

impl AuthoredMovementActionDeclarationDto {
    fn to_authority(&self) -> MovementActionDeclaration {
        MovementActionDeclaration {
            allowance: self.allowance,
            topology: self.topology.to_authority(),
            blocking_terrain_tags: self.blocking_terrain_tags.clone(),
            difficult_terrain_tags: self.difficult_terrain_tags.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredMovementTopologyDto {
    OrthogonalManhattan,
}

impl AuthoredMovementTopologyDto {
    const fn to_authority(self) -> MovementTopology {
        match self {
            Self::OrthogonalManhattan => MovementTopology::OrthogonalManhattan,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AuthoredEffectOperationDto {
    Damage {
        damage_bonus: i32,
        damage_type: String,
    },
    Heal {
        healing_bonus: i32,
        healing_type: String,
    },
    GrantTemporaryVitality {
        vitality_bonus: i32,
    },
    ApplyModifier {
        modifier_id: String,
    },
    Move {
        maximum_distance: u32,
        movement_kind: AuthoredMovementKindDto,
    },
    ChangeResource {
        resource_id: String,
        delta: i32,
    },
    OpenReactionWindow {
        hook_id: String,
        window: AuthoredReactionWindowDto,
        eligible_reactors: Vec<AuthoredReactionParticipantSelectorDto>,
        options: Vec<AuthoredReactionOptionDeclarationDto>,
        maximum_nested_depth: u32,
    },
}

impl AuthoredEffectOperationDto {
    fn to_authority(&self) -> AuthoredEffectOperation {
        match self {
            Self::Damage {
                damage_bonus,
                damage_type,
            } => AuthoredEffectOperation::Damage(DamageEffectOperation {
                damage_bonus: *damage_bonus,
                damage_type: damage_type.clone(),
            }),
            Self::Heal {
                healing_bonus,
                healing_type,
            } => AuthoredEffectOperation::Heal(HealingEffectOperation {
                healing_bonus: *healing_bonus,
                healing_type: healing_type.clone(),
            }),
            Self::GrantTemporaryVitality { vitality_bonus } => {
                AuthoredEffectOperation::GrantTemporaryVitality(TemporaryVitalityEffectOperation {
                    vitality_bonus: *vitality_bonus,
                })
            }
            Self::ApplyModifier { modifier_id } => {
                AuthoredEffectOperation::ApplyModifier(AuthoredModifierEffectOperation {
                    modifier_id: modifier_id.clone(),
                })
            }
            Self::Move {
                maximum_distance,
                movement_kind,
            } => AuthoredEffectOperation::Move(MovementEffectOperation {
                maximum_distance: *maximum_distance,
                movement_kind: movement_kind.to_authority(),
            }),
            Self::ChangeResource { resource_id, delta } => {
                AuthoredEffectOperation::ChangeResource(ResourceChangeEffectOperation {
                    resource_id: resource_id.clone(),
                    delta: *delta,
                })
            }
            Self::OpenReactionWindow {
                hook_id,
                window,
                eligible_reactors,
                options,
                maximum_nested_depth,
            } => AuthoredEffectOperation::OpenReactionWindow(AuthoredReactionHookEffectOperation {
                hook_id: hook_id.clone(),
                window: window.to_authority(),
                eligible_reactors: eligible_reactors
                    .iter()
                    .map(|selector| selector.to_authority())
                    .collect(),
                options: options
                    .iter()
                    .map(AuthoredReactionOptionDeclarationDto::to_authority)
                    .collect(),
                maximum_nested_depth: *maximum_nested_depth,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredMovementKindDto {
    Push,
    Pull,
    Shift,
}

impl AuthoredMovementKindDto {
    const fn to_authority(self) -> MovementKind {
        match self {
            Self::Push => MovementKind::Push,
            Self::Pull => MovementKind::Pull,
            Self::Shift => MovementKind::Shift,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredReactionWindowDto {
    BeforeEffect,
    AfterEffect,
}

impl AuthoredReactionWindowDto {
    const fn to_authority(self) -> ReactionWindow {
        match self {
            Self::BeforeEffect => ReactionWindow::BeforeEffect,
            Self::AfterEffect => ReactionWindow::AfterEffect,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredReactionParticipantSelectorDto {
    DeclaredTargets,
    ActorAllies,
    TargetAllies,
    AllOtherParticipants,
}

impl AuthoredReactionParticipantSelectorDto {
    const fn to_authority(self) -> ReactionParticipantSelector {
        match self {
            Self::DeclaredTargets => ReactionParticipantSelector::DeclaredTargets,
            Self::ActorAllies => ReactionParticipantSelector::ActorAllies,
            Self::TargetAllies => ReactionParticipantSelector::TargetAllies,
            Self::AllOtherParticipants => ReactionParticipantSelector::AllOtherParticipants,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredReactionOptionDeclarationDto {
    pub id: String,
    pub reactor: AuthoredReactionParticipantSelectorDto,
    pub opens_nested_window: bool,
}

impl AuthoredReactionOptionDeclarationDto {
    fn to_authority(&self) -> AuthoredReactionOptionDeclaration {
        AuthoredReactionOptionDeclaration {
            id: self.id.clone(),
            reactor: self.reactor.to_authority(),
            opens_nested_window: self.opens_nested_window,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredModifierDefinitionDto {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub default_tenure: AuthoredModifierTenureDto,
    pub stacking_group: String,
    pub stacking_policy: AuthoredModifierStackingPolicyDto,
    pub duration_policy: AuthoredModifierDurationPolicyDto,
    #[serde(default)]
    pub stat_adjustments: Vec<AuthoredModifierStatAdjustmentDto>,
}

impl AuthoredModifierDefinitionDto {
    pub(crate) fn to_authority(&self) -> ModifierDefinition {
        ModifierDefinition {
            id: self.id.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            default_tenure: self.default_tenure.to_authority(),
            stacking_group: self.stacking_group.clone(),
            stacking_policy: self.stacking_policy.to_authority(),
            duration_policy: self.duration_policy.to_authority(),
            stat_adjustments: self
                .stat_adjustments
                .iter()
                .map(AuthoredModifierStatAdjustmentDto::to_authority)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredModifierTenureDto {
    Temporary,
    Permanent,
}

impl AuthoredModifierTenureDto {
    const fn to_authority(self) -> ModifierTenure {
        match self {
            Self::Temporary => ModifierTenure::Temporary,
            Self::Permanent => ModifierTenure::Permanent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthoredModifierStackingPolicyDto {
    Stack,
    Replace,
    Refresh,
}

impl AuthoredModifierStackingPolicyDto {
    const fn to_authority(self) -> ModifierStackingPolicy {
        match self {
            Self::Stack => ModifierStackingPolicy::Stack,
            Self::Replace => ModifierStackingPolicy::Replace,
            Self::Refresh => ModifierStackingPolicy::Refresh,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AuthoredModifierDurationPolicyDto {
    Permanent,
    Turns { turns: u32 },
    Rounds { rounds: u32 },
    UntilEvent { event: String },
}

impl AuthoredModifierDurationPolicyDto {
    fn to_authority(&self) -> ModifierDurationPolicy {
        match self {
            Self::Permanent => ModifierDurationPolicy::Permanent,
            Self::Turns { turns } => ModifierDurationPolicy::Turns(*turns),
            Self::Rounds { rounds } => ModifierDurationPolicy::Rounds(*rounds),
            Self::UntilEvent { event } => ModifierDurationPolicy::UntilEvent(event.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredModifierStatAdjustmentDto {
    pub stat_id: String,
    pub stat_label: String,
    pub delta: i32,
}

impl AuthoredModifierStatAdjustmentDto {
    fn to_authority(&self) -> ModifierStatAdjustment {
        ModifierStatAdjustment {
            stat_id: self.stat_id.clone(),
            stat_label: self.stat_label.clone(),
            delta: self.delta,
        }
    }
}
