pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

mod action_resources;
mod combat_flow;
mod control;
mod effects;
mod projection;
mod session;

pub use action_resources::*;
pub use combat_flow::*;
pub use control::*;
pub use effects::*;
pub use projection::*;
pub use rulebench_content::*;
pub use rulebench_core::{BoundedValue, GridPosition, NamedNumber, StateFingerprint, Team};
pub use rulebench_ruleset::{
    validate_rule_modules, AbilityDefinition, AbilityDefinitionKind, ActionDefinition,
    ActionResolutionModuleConfiguration, ActionResolutionTargetingPolicy, AttackCheckDeclaration,
    CheckDeclaration, CheckHandlerKind, ContestedCheckDeclaration, DamageEffectOperation,
    DefenseReference, EffectOperationId, HealingEffectOperation, HitEffect, HitEffectOperation,
    ModifierEffectOperation, ModifierTenure, MovementEffectOperation, MovementKind,
    ReactionHookEffectOperation, ReactionWindow, ResourceChangeEffectOperation,
    RuleModuleConfiguration, RuleModuleDeclaration, RuleModuleId, RuleModuleValidationError,
    RulesetArtifactProvenance, RulesetCompatibilityError, RulesetMetadata, RulesetModuleProvenance,
    SavingThrowCheckDeclaration, TargetKind, TargetSelection, TargetTeamConstraint,
    TargetingDeclaration, TemporaryVitalityEffectOperation, TurnControlModuleConfiguration,
    TurnOrderPolicy, ValidatedRuleModuleDeclaration, ValidatedRuleModuleRegistry,
    VisibilityRequirement,
};
pub use session::*;
