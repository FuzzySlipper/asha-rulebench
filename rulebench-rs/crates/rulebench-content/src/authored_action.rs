use rpg_ir::{
    ActionResourceCost, CheckDeclaration, DamageEffectOperation, HealingEffectOperation,
    MovementActionDeclaration, MovementEffectOperation, OperationPipelineV2, ReactionWindow,
    ResourceChangeEffectOperation, TargetKind, TargetSelection, TargetTeamConstraint,
    TemporaryVitalityEffectOperation, VisibilityRequirement,
};

/// Reusable action content stored in a canonical content pack.
///
/// Scenario-bound actor, target, visibility, and reaction participant ids are
/// deliberately absent. A later Rust binding step materializes those values
/// into the runtime `ActionDefinition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredActionDefinition {
    pub id: String,
    pub ability_id: String,
    pub name: String,
    pub targeting: AuthoredTargetingDeclaration,
    pub check: CheckDeclaration,
    pub effects: Vec<AuthoredEffectOperation>,
    pub resource_costs: Vec<ActionResourceCost>,
    pub movement: Option<MovementActionDeclaration>,
    pub action_text: String,
    pub effect_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredTargetingDeclaration {
    pub target_kind: TargetKind,
    pub selection: TargetSelection,
    pub team_constraint: TargetTeamConstraint,
    pub maximum_range: u32,
    pub visibility_requirement: VisibilityRequirement,
    pub operation_pipeline: Option<OperationPipelineV2>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredEffectOperation {
    Damage(DamageEffectOperation),
    Heal(HealingEffectOperation),
    GrantTemporaryVitality(TemporaryVitalityEffectOperation),
    ApplyModifier(AuthoredModifierEffectOperation),
    Move(MovementEffectOperation),
    ChangeResource(ResourceChangeEffectOperation),
    OpenReactionWindow(AuthoredReactionHookEffectOperation),
}

impl AuthoredEffectOperation {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Damage(_) => "damage",
            Self::Heal(_) => "heal",
            Self::GrantTemporaryVitality(_) => "grantTemporaryVitality",
            Self::ApplyModifier(_) => "applyModifier",
            Self::Move(_) => "move",
            Self::ChangeResource(_) => "changeResource",
            Self::OpenReactionWindow(_) => "openReactionWindow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredModifierEffectOperation {
    pub modifier_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredReactionHookEffectOperation {
    pub hook_id: String,
    pub window: ReactionWindow,
    pub eligible_reactors: Vec<ReactionParticipantSelector>,
    pub options: Vec<AuthoredReactionOptionDeclaration>,
    pub maximum_nested_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredReactionOptionDeclaration {
    pub id: String,
    pub reactor: ReactionParticipantSelector,
    pub opens_nested_window: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReactionParticipantSelector {
    DeclaredTargets,
    ActorAllies,
    TargetAllies,
    AllOtherParticipants,
}

impl ReactionParticipantSelector {
    pub const ALL: &'static [Self] = &[
        Self::DeclaredTargets,
        Self::ActorAllies,
        Self::TargetAllies,
        Self::AllOtherParticipants,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::DeclaredTargets => "declaredTargets",
            Self::ActorAllies => "actorAllies",
            Self::TargetAllies => "targetAllies",
            Self::AllOtherParticipants => "allOtherParticipants",
        }
    }
}
