pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

mod action_resources;
mod catalog;
mod combat_flow;
mod content;
mod control;
mod effects;
mod projection;
mod scenario;
mod session;
mod stats;

pub use action_resources::*;
pub use catalog::*;
pub use combat_flow::*;
pub use content::*;
pub use control::*;
pub use effects::*;
pub use projection::*;
pub use rulebench_core::{BoundedValue, GridPosition, NamedNumber, StateFingerprint, Team};
pub use rulebench_ruleset::{
    validate_rule_modules, AbilityDefinition, AbilityDefinitionKind, ActionDefinition,
    ActionResolutionModuleConfiguration, ActionResolutionTargetingPolicy, AttackCheckDeclaration,
    CheckDeclaration, ContestedCheckDeclaration, DamageEffectOperation, DefenseReference,
    HitEffect, HitEffectOperation, ModifierEffectOperation, ModifierTenure,
    RuleModuleConfiguration, RuleModuleDeclaration, RuleModuleId, RuleModuleValidationError,
    RulesetMetadata, SavingThrowCheckDeclaration, TargetKind, TargetSelection,
    TargetTeamConstraint, TargetingDeclaration, TurnControlModuleConfiguration, TurnOrderPolicy,
    ValidatedRuleModuleDeclaration, ValidatedRuleModuleRegistry, VisibilityRequirement,
};
pub use scenario::*;
pub use session::*;
pub use stats::{StatBlock, StatDefinition, StatDefinitionKind};
