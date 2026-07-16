/// Static identity for one authored scenario or content validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMetadata {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDiagnosticSeverity {
    Error,
    Warning,
}

impl ContentDiagnosticSeverity {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDiagnosticSeverity::Error => "error",
            ContentDiagnosticSeverity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDiagnosticCode {
    InvalidContentPackSetReference,
    InvalidAuthoredActionBindingReceipt,
    EmptyRulesetId,
    DuplicateRulesetId,
    SelectedRulesetMissingFromCatalog,
    UnknownRulesetModule,
    MissingRequiredRulesetModule,
    DuplicateRulesetModule,
    IncompatibleRulesetModuleVersion,
    RulesetModuleConfigurationMismatch,
    EmptyAbilityId,
    DuplicateAbilityId,
    EmptyEntityId,
    DuplicateEntityId,
    EmptyDamageAdjustmentType,
    ConflictingDamageAdjustment,
    EmptyActionId,
    DuplicateActionId,
    InvalidActionResourceCost,
    DuplicateActionResourceCost,
    EmptyActionResourcePoolId,
    DuplicateActionResourcePoolId,
    InvalidActionResourcePoolMaximum,
    InvalidActionResourceRefreshPolicy,
    MissingActionResourcePool,
    EmptyClassId,
    DuplicateClassId,
    EmptyClassVersion,
    InvalidClassGrantLevel,
    DuplicateClassGrantLevel,
    MissingClassPrerequisiteStat,
    MissingClassGrantedModifier,
    MissingClassGrantedAbility,
    InvalidClassInputLevel,
    DuplicateCombatantClass,
    ClassVersionMismatch,
    ClassPrerequisiteNotMet,
    ClassResourcePoolConflict,
    EmptyStatDefinitionId,
    DuplicateStatDefinitionId,
    BaseStatFormulaNotAllowed,
    MissingDerivedStatFormula,
    UnknownDerivedStatReference,
    DerivedStatFormulaCycle,
    InvalidDerivedStatFormula,
    MissingCombatantBaseStat,
    DuplicateCombatantBaseStat,
    AuthoredDerivedStatValue,
    EmptyModifierId,
    DuplicateModifierId,
    EmptyModifierStackingGroup,
    InvalidModifierTurnDuration,
    InvalidModifierRoundDuration,
    EmptyModifierDurationEvent,
    ModifierTenureDurationMismatch,
    EmptyItemId,
    DuplicateItemId,
    EmptyItemEquipmentSlot,
    MissingItemRequirementStat,
    MissingItemGrantedModifier,
    MissingItemGrantedAbility,
    MissingInventoryItem,
    DuplicateInventoryItem,
    DuplicateEquippedItem,
    EquippedItemNotOwned,
    EquipmentSlotConflict,
    EquipmentRequirementNotMet,
    EquipmentResourcePoolConflict,
    MissingBaseAbility,
    SelectedAbilityMissingFromCatalog,
    SelectedActionMissingFromCatalog,
    SelectedClassMissingFromCatalog,
    SelectedItemMissingFromCatalog,
    MissingCombatantEntity,
    EmptyCombatantId,
    EmptyCombatantSide,
    MissingCombatEndObjectiveSide,
    DuplicateCombatantId,
    MissingActionAbility,
    MissingActionAbilityGrant,
    CrossRulesetActionReference,
    MissingActionActor,
    MissingActionTarget,
    UnsupportedTargetingDeclaration,
    InvalidOperationPipelineDeclaration,
    UnsupportedCheckDeclaration,
    UnsupportedEffectOperation,
    InvalidEffectOperation,
    InvalidReactionHookId,
    DuplicateReactionHook,
    InvalidReactionEligibleReactor,
    DuplicateReactionEligibleReactor,
    InvalidReactionOption,
    DuplicateReactionOption,
    InvalidReactionNestedDepth,
    VisibleTargetOutsideTargetIds,
    MissingAttackModifierStat,
    MissingSavingThrowStat,
    MissingContestedActorStat,
    MissingContestedTargetStat,
    InvalidSavingThrowDifficultyClass,
    MissingTargetDefense,
    MissingCombatantClass,
    MissingCombatantStatDefinition,
    MissingHitModifierDefinition,
    MissingModifierStatAdjustmentTarget,
    MissingActiveModifierDefinition,
    MissingEquippedItem,
}

impl ContentDiagnosticCode {
    pub const fn code(self) -> &'static str {
        match self {
            ContentDiagnosticCode::InvalidContentPackSetReference => {
                "invalidContentPackSetReference"
            }
            ContentDiagnosticCode::InvalidAuthoredActionBindingReceipt => {
                "invalidAuthoredActionBindingReceipt"
            }
            ContentDiagnosticCode::EmptyRulesetId => "emptyRulesetId",
            ContentDiagnosticCode::DuplicateRulesetId => "duplicateRulesetId",
            ContentDiagnosticCode::SelectedRulesetMissingFromCatalog => {
                "selectedRulesetMissingFromCatalog"
            }
            ContentDiagnosticCode::UnknownRulesetModule => "unknownRulesetModule",
            ContentDiagnosticCode::MissingRequiredRulesetModule => "missingRequiredRulesetModule",
            ContentDiagnosticCode::DuplicateRulesetModule => "duplicateRulesetModule",
            ContentDiagnosticCode::IncompatibleRulesetModuleVersion => {
                "incompatibleRulesetModuleVersion"
            }
            ContentDiagnosticCode::RulesetModuleConfigurationMismatch => {
                "rulesetModuleConfigurationMismatch"
            }
            ContentDiagnosticCode::EmptyAbilityId => "emptyAbilityId",
            ContentDiagnosticCode::DuplicateAbilityId => "duplicateAbilityId",
            ContentDiagnosticCode::EmptyEntityId => "emptyEntityId",
            ContentDiagnosticCode::DuplicateEntityId => "duplicateEntityId",
            ContentDiagnosticCode::EmptyDamageAdjustmentType => "emptyDamageAdjustmentType",
            ContentDiagnosticCode::ConflictingDamageAdjustment => "conflictingDamageAdjustment",
            ContentDiagnosticCode::EmptyActionId => "emptyActionId",
            ContentDiagnosticCode::DuplicateActionId => "duplicateActionId",
            ContentDiagnosticCode::InvalidActionResourceCost => "invalidActionResourceCost",
            ContentDiagnosticCode::DuplicateActionResourceCost => "duplicateActionResourceCost",
            ContentDiagnosticCode::EmptyActionResourcePoolId => "emptyActionResourcePoolId",
            ContentDiagnosticCode::DuplicateActionResourcePoolId => "duplicateActionResourcePoolId",
            ContentDiagnosticCode::InvalidActionResourcePoolMaximum => {
                "invalidActionResourcePoolMaximum"
            }
            ContentDiagnosticCode::InvalidActionResourceRefreshPolicy => {
                "invalidActionResourceRefreshPolicy"
            }
            ContentDiagnosticCode::MissingActionResourcePool => "missingActionResourcePool",
            ContentDiagnosticCode::EmptyClassId => "emptyClassId",
            ContentDiagnosticCode::DuplicateClassId => "duplicateClassId",
            ContentDiagnosticCode::EmptyClassVersion => "emptyClassVersion",
            ContentDiagnosticCode::InvalidClassGrantLevel => "invalidClassGrantLevel",
            ContentDiagnosticCode::DuplicateClassGrantLevel => "duplicateClassGrantLevel",
            ContentDiagnosticCode::MissingClassPrerequisiteStat => "missingClassPrerequisiteStat",
            ContentDiagnosticCode::MissingClassGrantedModifier => "missingClassGrantedModifier",
            ContentDiagnosticCode::MissingClassGrantedAbility => "missingClassGrantedAbility",
            ContentDiagnosticCode::InvalidClassInputLevel => "invalidClassInputLevel",
            ContentDiagnosticCode::DuplicateCombatantClass => "duplicateCombatantClass",
            ContentDiagnosticCode::ClassVersionMismatch => "classVersionMismatch",
            ContentDiagnosticCode::ClassPrerequisiteNotMet => "classPrerequisiteNotMet",
            ContentDiagnosticCode::ClassResourcePoolConflict => "classResourcePoolConflict",
            ContentDiagnosticCode::EmptyStatDefinitionId => "emptyStatDefinitionId",
            ContentDiagnosticCode::DuplicateStatDefinitionId => "duplicateStatDefinitionId",
            ContentDiagnosticCode::BaseStatFormulaNotAllowed => "baseStatFormulaNotAllowed",
            ContentDiagnosticCode::MissingDerivedStatFormula => "missingDerivedStatFormula",
            ContentDiagnosticCode::UnknownDerivedStatReference => "unknownDerivedStatReference",
            ContentDiagnosticCode::DerivedStatFormulaCycle => "derivedStatFormulaCycle",
            ContentDiagnosticCode::InvalidDerivedStatFormula => "invalidDerivedStatFormula",
            ContentDiagnosticCode::MissingCombatantBaseStat => "missingCombatantBaseStat",
            ContentDiagnosticCode::DuplicateCombatantBaseStat => "duplicateCombatantBaseStat",
            ContentDiagnosticCode::AuthoredDerivedStatValue => "authoredDerivedStatValue",
            ContentDiagnosticCode::EmptyModifierId => "emptyModifierId",
            ContentDiagnosticCode::DuplicateModifierId => "duplicateModifierId",
            ContentDiagnosticCode::EmptyModifierStackingGroup => "emptyModifierStackingGroup",
            ContentDiagnosticCode::InvalidModifierTurnDuration => "invalidModifierTurnDuration",
            ContentDiagnosticCode::InvalidModifierRoundDuration => "invalidModifierRoundDuration",
            ContentDiagnosticCode::EmptyModifierDurationEvent => "emptyModifierDurationEvent",
            ContentDiagnosticCode::ModifierTenureDurationMismatch => {
                "modifierTenureDurationMismatch"
            }
            ContentDiagnosticCode::EmptyItemId => "emptyItemId",
            ContentDiagnosticCode::DuplicateItemId => "duplicateItemId",
            ContentDiagnosticCode::EmptyItemEquipmentSlot => "emptyItemEquipmentSlot",
            ContentDiagnosticCode::MissingItemRequirementStat => "missingItemRequirementStat",
            ContentDiagnosticCode::MissingItemGrantedModifier => "missingItemGrantedModifier",
            ContentDiagnosticCode::MissingItemGrantedAbility => "missingItemGrantedAbility",
            ContentDiagnosticCode::MissingInventoryItem => "missingInventoryItem",
            ContentDiagnosticCode::DuplicateInventoryItem => "duplicateInventoryItem",
            ContentDiagnosticCode::DuplicateEquippedItem => "duplicateEquippedItem",
            ContentDiagnosticCode::EquippedItemNotOwned => "equippedItemNotOwned",
            ContentDiagnosticCode::EquipmentSlotConflict => "equipmentSlotConflict",
            ContentDiagnosticCode::EquipmentRequirementNotMet => "equipmentRequirementNotMet",
            ContentDiagnosticCode::EquipmentResourcePoolConflict => "equipmentResourcePoolConflict",
            ContentDiagnosticCode::MissingBaseAbility => "missingBaseAbility",
            ContentDiagnosticCode::SelectedAbilityMissingFromCatalog => {
                "selectedAbilityMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedActionMissingFromCatalog => {
                "selectedActionMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedClassMissingFromCatalog => {
                "selectedClassMissingFromCatalog"
            }
            ContentDiagnosticCode::SelectedItemMissingFromCatalog => {
                "selectedItemMissingFromCatalog"
            }
            ContentDiagnosticCode::MissingCombatantEntity => "missingCombatantEntity",
            ContentDiagnosticCode::EmptyCombatantId => "emptyCombatantId",
            ContentDiagnosticCode::EmptyCombatantSide => "emptyCombatantSide",
            ContentDiagnosticCode::MissingCombatEndObjectiveSide => "missingCombatEndObjectiveSide",
            ContentDiagnosticCode::DuplicateCombatantId => "duplicateCombatantId",
            ContentDiagnosticCode::MissingActionAbility => "missingActionAbility",
            ContentDiagnosticCode::MissingActionAbilityGrant => "missingActionAbilityGrant",
            ContentDiagnosticCode::CrossRulesetActionReference => "crossRulesetActionReference",
            ContentDiagnosticCode::MissingActionActor => "missingActionActor",
            ContentDiagnosticCode::MissingActionTarget => "missingActionTarget",
            ContentDiagnosticCode::UnsupportedTargetingDeclaration => {
                "unsupportedTargetingDeclaration"
            }
            ContentDiagnosticCode::InvalidOperationPipelineDeclaration => {
                "invalidOperationPipelineDeclaration"
            }
            ContentDiagnosticCode::UnsupportedCheckDeclaration => "unsupportedCheckDeclaration",
            ContentDiagnosticCode::UnsupportedEffectOperation => "unsupportedEffectOperation",
            ContentDiagnosticCode::InvalidEffectOperation => "invalidEffectOperation",
            ContentDiagnosticCode::InvalidReactionHookId => "invalidReactionHookId",
            ContentDiagnosticCode::DuplicateReactionHook => "duplicateReactionHook",
            ContentDiagnosticCode::InvalidReactionEligibleReactor => {
                "invalidReactionEligibleReactor"
            }
            ContentDiagnosticCode::DuplicateReactionEligibleReactor => {
                "duplicateReactionEligibleReactor"
            }
            ContentDiagnosticCode::InvalidReactionOption => "invalidReactionOption",
            ContentDiagnosticCode::DuplicateReactionOption => "duplicateReactionOption",
            ContentDiagnosticCode::InvalidReactionNestedDepth => "invalidReactionNestedDepth",
            ContentDiagnosticCode::VisibleTargetOutsideTargetIds => "visibleTargetOutsideTargetIds",
            ContentDiagnosticCode::MissingAttackModifierStat => "missingAttackModifierStat",
            ContentDiagnosticCode::MissingSavingThrowStat => "missingSavingThrowStat",
            ContentDiagnosticCode::MissingContestedActorStat => "missingContestedActorStat",
            ContentDiagnosticCode::MissingContestedTargetStat => "missingContestedTargetStat",
            ContentDiagnosticCode::InvalidSavingThrowDifficultyClass => {
                "invalidSavingThrowDifficultyClass"
            }
            ContentDiagnosticCode::MissingTargetDefense => "missingTargetDefense",
            ContentDiagnosticCode::MissingCombatantClass => "missingCombatantClass",
            ContentDiagnosticCode::MissingCombatantStatDefinition => {
                "missingCombatantStatDefinition"
            }
            ContentDiagnosticCode::MissingHitModifierDefinition => "missingHitModifierDefinition",
            ContentDiagnosticCode::MissingModifierStatAdjustmentTarget => {
                "missingModifierStatAdjustmentTarget"
            }
            ContentDiagnosticCode::MissingActiveModifierDefinition => {
                "missingActiveModifierDefinition"
            }
            ContentDiagnosticCode::MissingEquippedItem => "missingEquippedItem",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDiagnostic {
    pub severity: ContentDiagnosticSeverity,
    pub code: ContentDiagnosticCode,
    pub content_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationReport {
    pub accepted: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<ContentDiagnostic>,
}

impl ContentValidationReport {
    pub fn from_diagnostics(diagnostics: Vec<ContentDiagnostic>) -> Self {
        let error_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ContentDiagnosticSeverity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ContentDiagnosticSeverity::Warning)
            .count();

        Self {
            accepted: error_count == 0,
            error_count,
            warning_count,
            diagnostics,
        }
    }
}
