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
    EmptyActionId,
    DuplicateActionId,
    EmptyClassId,
    DuplicateClassId,
    EmptyStatDefinitionId,
    DuplicateStatDefinitionId,
    EmptyModifierId,
    DuplicateModifierId,
    EmptyItemId,
    DuplicateItemId,
    SelectedAbilityMissingFromCatalog,
    SelectedActionMissingFromCatalog,
    SelectedClassMissingFromCatalog,
    SelectedItemMissingFromCatalog,
    MissingCombatantEntity,
    MissingActionAbility,
    MissingActionActor,
    MissingActionTarget,
    UnsupportedTargetingDeclaration,
    UnsupportedCheckDeclaration,
    UnsupportedEffectOperation,
    VisibleTargetOutsideTargetIds,
    MissingAttackModifierStat,
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
            ContentDiagnosticCode::EmptyActionId => "emptyActionId",
            ContentDiagnosticCode::DuplicateActionId => "duplicateActionId",
            ContentDiagnosticCode::EmptyClassId => "emptyClassId",
            ContentDiagnosticCode::DuplicateClassId => "duplicateClassId",
            ContentDiagnosticCode::EmptyStatDefinitionId => "emptyStatDefinitionId",
            ContentDiagnosticCode::DuplicateStatDefinitionId => "duplicateStatDefinitionId",
            ContentDiagnosticCode::EmptyModifierId => "emptyModifierId",
            ContentDiagnosticCode::DuplicateModifierId => "duplicateModifierId",
            ContentDiagnosticCode::EmptyItemId => "emptyItemId",
            ContentDiagnosticCode::DuplicateItemId => "duplicateItemId",
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
            ContentDiagnosticCode::MissingActionAbility => "missingActionAbility",
            ContentDiagnosticCode::MissingActionActor => "missingActionActor",
            ContentDiagnosticCode::MissingActionTarget => "missingActionTarget",
            ContentDiagnosticCode::UnsupportedTargetingDeclaration => {
                "unsupportedTargetingDeclaration"
            }
            ContentDiagnosticCode::UnsupportedCheckDeclaration => "unsupportedCheckDeclaration",
            ContentDiagnosticCode::UnsupportedEffectOperation => "unsupportedEffectOperation",
            ContentDiagnosticCode::VisibleTargetOutsideTargetIds => "visibleTargetOutsideTargetIds",
            ContentDiagnosticCode::MissingAttackModifierStat => "missingAttackModifierStat",
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
