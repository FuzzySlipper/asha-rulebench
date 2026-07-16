use super::{ProtocolField, ProtocolInterface};

type Field = ProtocolField;
type Interface = ProtocolInterface;

pub fn interfaces() -> &'static [ProtocolInterface] {
    &[
        Interface {
            name: "RulebenchCapabilitySupportDto",
            fields: &[
                Field {
                    name: "declared",
                    ty: "boolean",
                },
                Field {
                    name: "validationSupported",
                    ty: "boolean",
                },
                Field {
                    name: "runtimeExecutable",
                    ty: "boolean",
                },
                Field {
                    name: "protocolExposed",
                    ty: "boolean",
                },
                Field {
                    name: "liveHostExposed",
                    ty: "boolean",
                },
                Field {
                    name: "uiExposed",
                    ty: "boolean",
                },
                Field {
                    name: "regressionCovered",
                    ty: "boolean",
                },
                Field {
                    name: "durableAcrossRestart",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchCapabilityEntryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "kind",
                    ty: "RulebenchCapabilityKindDto",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "support",
                    ty: "RulebenchCapabilitySupportDto",
                },
                Field {
                    name: "evidence",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCapabilityIdentityDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchRulesetProviderDto",
            fields: &[
                Field {
                    name: "provider",
                    ty: "RulebenchCapabilityIdentityDto",
                },
                Field {
                    name: "ruleset",
                    ty: "RulebenchCapabilityIdentityDto",
                },
                Field {
                    name: "operationVocabularyVersion",
                    ty: "string",
                },
                Field {
                    name: "effectOperationVocabularyVersion",
                    ty: "string",
                },
                Field {
                    name: "capabilities",
                    ty: "readonly RulebenchCapabilityIdentityDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchHostCapabilityProfileDto",
            fields: &[
                Field {
                    name: "adapterId",
                    ty: "string",
                },
                Field {
                    name: "storageMode",
                    ty: "string",
                },
                Field {
                    name: "contentStorageAdapter",
                    ty: "string",
                },
                Field {
                    name: "replayStorageAdapter",
                    ty: "string",
                },
                Field {
                    name: "replayRecoveryMode",
                    ty: "string",
                },
                Field {
                    name: "sessionRecoveryMode",
                    ty: "string",
                },
                Field {
                    name: "authorityViewerMode",
                    ty: "'liveAuthorityReadback' | 'none'",
                },
            ],
        },
        Interface {
            name: "RulebenchCapabilityManifestDto",
            fields: &[
                Field {
                    name: "manifestId",
                    ty: "string",
                },
                Field {
                    name: "manifestVersion",
                    ty: "number",
                },
                Field {
                    name: "generatedArtifactSchema",
                    ty: "string",
                },
                Field {
                    name: "governedAshaRevision",
                    ty: "string",
                },
                Field {
                    name: "operationVocabularyVersion",
                    ty: "string",
                },
                Field {
                    name: "effectVocabularyVersion",
                    ty: "string",
                },
                Field {
                    name: "protocolId",
                    ty: "string",
                },
                Field {
                    name: "protocolVersion",
                    ty: "number",
                },
                Field {
                    name: "host",
                    ty: "RulebenchHostCapabilityProfileDto",
                },
                Field {
                    name: "providers",
                    ty: "readonly RulebenchRulesetProviderDto[]",
                },
                Field {
                    name: "rulesets",
                    ty: "readonly RulebenchCapabilityIdentityDto[]",
                },
                Field {
                    name: "packages",
                    ty: "readonly RulebenchCapabilityIdentityDto[]",
                },
                Field {
                    name: "scenarios",
                    ty: "readonly RulebenchCapabilityIdentityDto[]",
                },
                Field {
                    name: "capabilities",
                    ty: "readonly RulebenchCapabilityEntryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchProtocolRequestContextDto",
            fields: &[Field {
                name: "protocolVersion",
                ty: "number",
            }],
        },
        Interface {
            name: "RulebenchProtocolHandshakeDto",
            fields: &[
                Field {
                    name: "protocolId",
                    ty: "string",
                },
                Field {
                    name: "protocolVersion",
                    ty: "number",
                },
                Field {
                    name: "authoritySurface",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchScenarioOptionDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "contentPackId",
                    ty: "string | null",
                },
                Field {
                    name: "contentPackVersion",
                    ty: "string | null",
                },
                Field {
                    name: "participants",
                    ty: "readonly RulebenchScenarioParticipantOptionDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchScenarioParticipantOptionDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "sideId",
                    ty: "string",
                },
                Field {
                    name: "initiative",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionCreateRequestDto",
            fields: &[
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "scenarioId",
                    ty: "string",
                },
                Field {
                    name: "participantOrder",
                    ty: "readonly string[]",
                },
                Field {
                    name: "contentPack?",
                    ty: "RulebenchContentPackReferenceDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionIntentCommandDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "intent",
                    ty: "RulebenchUseActionIntentDto",
                },
                Field {
                    name: "rollStream",
                    ty: "readonly number[]",
                },
                Field {
                    name: "rollMode?",
                    ty: "\"supplied\" | \"authorityGenerated\"",
                },
                Field {
                    name: "generatedSeed?",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatControlCommandDto",
            fields: &[Field {
                name: "kind",
                ty: "RulebenchCombatControlCommandKindDto",
            }],
        },
        Interface {
            name: "RulebenchCombatSessionHandleDto",
            fields: &[Field {
                name: "id",
                ty: "string",
            }],
        },
        Interface {
            name: "RulebenchRulesetDefinitionDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "modules",
                    ty: "readonly RulebenchRuleModuleDeclarationDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchRuleModuleDeclarationDto",
            fields: &[
                Field {
                    name: "module",
                    ty: "RulebenchRuleModuleIdDto",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "configuration",
                    ty: "RulebenchRuleModuleConfigurationDto",
                },
            ],
        },
        Interface {
            name: "RulebenchActionResolutionModuleConfigurationDto",
            fields: &[
                Field {
                    name: "module",
                    ty: "'actionResolution'",
                },
                Field {
                    name: "targetingPolicy",
                    ty: "RulebenchActionResolutionTargetingPolicyDto",
                },
                Field {
                    name: "supportedCheckHandlers",
                    ty: "readonly RulebenchCheckHandlerKindDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchTurnControlModuleConfigurationDto",
            fields: &[
                Field {
                    name: "module",
                    ty: "'turnControl'",
                },
                Field {
                    name: "turnOrderPolicy",
                    ty: "RulebenchTurnOrderPolicyDto",
                },
                Field {
                    name: "combatEndPolicy",
                    ty: "RulebenchCombatEndPolicyKindDto",
                },
                Field {
                    name: "objectiveSide",
                    ty: "RulebenchCombatSideIdDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchRulesetCatalogDto",
            fields: &[
                Field {
                    name: "selectedRulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesets",
                    ty: "readonly RulebenchRulesetSummaryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchRulesetSummaryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionCatalogDto",
            fields: &[
                Field {
                    name: "summaries",
                    ty: "readonly RulebenchCombatSessionSummaryDto[]",
                },
                Field {
                    name: "readouts",
                    ty: "readonly RulebenchCombatSessionStepReadoutDto[]",
                },
                Field {
                    name: "controlHistoryReadouts",
                    ty: "readonly RulebenchCombatControlHistoryReadoutDto[]",
                },
                Field {
                    name: "scriptReadouts",
                    ty: "readonly RulebenchCombatScriptReadoutDto[]",
                },
                Field {
                    name: "automaticRunReadouts",
                    ty: "readonly RulebenchAutomaticRunReadoutDto[]",
                },
                Field {
                    name: "automaticRunReplayReadouts",
                    ty: "readonly RulebenchAutomaticRunReplayReadoutDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchStateFingerprintDto",
            fields: &[
                Field {
                    name: "algorithm",
                    ty: "string",
                },
                Field {
                    name: "value",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatControlHistoryReadoutDto",
            fields: &[
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "history",
                    ty: "readonly RulebenchCombatControlHistoryEntryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatControlHistoryEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "commandKind",
                    ty: "RulebenchCombatControlCommandKindDto",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchCombatControlDecisionKindDto",
                },
                Field {
                    name: "previousLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "nextLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "previousRoundNumber",
                    ty: "number",
                },
                Field {
                    name: "previousTurnIndex",
                    ty: "number",
                },
                Field {
                    name: "previousActorId",
                    ty: "string | null",
                },
                Field {
                    name: "nextRoundNumber",
                    ty: "number",
                },
                Field {
                    name: "nextTurnIndex",
                    ty: "number",
                },
                Field {
                    name: "nextActorId",
                    ty: "string | null",
                },
                Field {
                    name: "lifecycleTransitionSequence",
                    ty: "number | null",
                },
                Field {
                    name: "turnTransitionSequence",
                    ty: "number | null",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "stateAfterFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionSummaryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "seedLabel",
                    ty: "string",
                },
                Field {
                    name: "steps",
                    ty: "readonly RulebenchCombatSessionStepSummaryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatScriptReadoutDto",
            fields: &[
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "scriptId",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "steps",
                    ty: "readonly RulebenchCombatScriptStepReadoutDto[]",
                },
                Field {
                    name: "finalLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "finalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "finalState",
                    ty: "RulebenchFinalStateDto",
                },
                Field {
                    name: "finalTurnOrder",
                    ty: "RulebenchCombatTurnOrderDto",
                },
                Field {
                    name: "finalActionResourceLedger",
                    ty: "RulebenchActionResourceLedgerDto",
                },
                Field {
                    name: "finalEquipmentLedger",
                    ty: "RulebenchEquipmentLedgerDto",
                },
                Field {
                    name: "finalClassBuildLedger",
                    ty: "RulebenchClassBuildLedgerDto",
                },
                Field {
                    name: "currentTurnActionUsage",
                    ty: "RulebenchActionUsageSummaryDto",
                },
                Field {
                    name: "finalCurrentActorOptions",
                    ty: "RulebenchCurrentActorOptionSummaryDto",
                },
                Field {
                    name: "finalCombatantVitality",
                    ty: "RulebenchCombatantVitalitySummaryDto",
                },
                Field {
                    name: "finalCombatEndCondition",
                    ty: "RulebenchCombatEndConditionDto",
                },
                Field {
                    name: "finalization",
                    ty: "RulebenchCombatFinalizationDto | null",
                },
                Field {
                    name: "lifecycleTransitionLog",
                    ty: "readonly RulebenchLifecycleTransitionEntryDto[]",
                },
                Field {
                    name: "turnTransitionLog",
                    ty: "readonly RulebenchTurnTransitionEntryDto[]",
                },
                Field {
                    name: "commandAuditLog",
                    ty: "readonly RulebenchCommandAuditEntryDto[]",
                },
                Field {
                    name: "actionUsageLog",
                    ty: "readonly RulebenchActionUsageEntryDto[]",
                },
                Field {
                    name: "actionResourceTransitionLog",
                    ty: "readonly RulebenchActionResourceTransitionEntryDto[]",
                },
                Field {
                    name: "equipmentTransitionLog",
                    ty: "readonly RulebenchEquipmentTransitionEntryDto[]",
                },
                Field {
                    name: "currentReactionWindow",
                    ty: "RulebenchReactionWindowDto | null",
                },
                Field {
                    name: "reactionWindowLifecycleLog",
                    ty: "readonly RulebenchReactionWindowLifecycleEntryDto[]",
                },
                Field {
                    name: "reactionAuditLog",
                    ty: "readonly RulebenchReactionAuditEntryDto[]",
                },
                Field {
                    name: "modifierDurationExpirationLog",
                    ty: "readonly RulebenchModifierDurationExpirationEntryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatTurnOrderDto",
            fields: &[
                Field {
                    name: "roundNumber",
                    ty: "number",
                },
                Field {
                    name: "currentTurnIndex",
                    ty: "number",
                },
                Field {
                    name: "participantOrder",
                    ty: "readonly string[]",
                },
                Field {
                    name: "currentActorId",
                    ty: "string | null",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionOptionDto",
            fields: &[
                Field {
                    name: "optionId",
                    ty: "string",
                },
                Field {
                    name: "reactorId",
                    ty: "string",
                },
                Field {
                    name: "opensNestedWindow",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionResponseEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "reactorId",
                    ty: "string",
                },
                Field {
                    name: "responseKind",
                    ty: "RulebenchReactionResponseKindDto",
                },
                Field {
                    name: "optionId",
                    ty: "string | null",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionWindowDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "hookId",
                    ty: "string",
                },
                Field {
                    name: "timing",
                    ty: "RulebenchReactionWindowTimingDto",
                },
                Field {
                    name: "depth",
                    ty: "number",
                },
                Field {
                    name: "maximumNestedDepth",
                    ty: "number",
                },
                Field {
                    name: "parentWindowId",
                    ty: "string | null",
                },
                Field {
                    name: "triggerStepId",
                    ty: "string",
                },
                Field {
                    name: "triggerActionId",
                    ty: "string",
                },
                Field {
                    name: "eligibleReactorIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "currentReactorId",
                    ty: "string | null",
                },
                Field {
                    name: "options",
                    ty: "readonly RulebenchReactionOptionDto[]",
                },
                Field {
                    name: "responses",
                    ty: "readonly RulebenchReactionResponseEntryDto[]",
                },
                Field {
                    name: "status",
                    ty: "RulebenchReactionWindowStatusDto",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionCommandSpecDto",
            fields: &[
                Field {
                    name: "windowId",
                    ty: "string",
                },
                Field {
                    name: "reactorId",
                    ty: "string",
                },
                Field {
                    name: "responseKind",
                    ty: "RulebenchReactionResponseKindDto",
                },
                Field {
                    name: "optionId",
                    ty: "string | null",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionCommandReadoutDto",
            fields: &[
                Field {
                    name: "command",
                    ty: "RulebenchReactionCommandSpecDto",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchReactionDecisionKindDto",
                },
                Field {
                    name: "previousWindow",
                    ty: "RulebenchReactionWindowDto | null",
                },
                Field {
                    name: "nextWindow",
                    ty: "RulebenchReactionWindowDto | null",
                },
                Field {
                    name: "openedNestedWindow",
                    ty: "RulebenchReactionWindowDto | null",
                },
                Field {
                    name: "resumedPendingResolution",
                    ty: "boolean",
                },
                Field {
                    name: "trace",
                    ty: "readonly RulebenchLiveTraceEntryDto[]",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionWindowLifecycleEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "lifecycleKind",
                    ty: "RulebenchReactionWindowLifecycleKindDto",
                },
                Field {
                    name: "windowId",
                    ty: "string",
                },
                Field {
                    name: "parentWindowId",
                    ty: "string | null",
                },
                Field {
                    name: "depth",
                    ty: "number",
                },
                Field {
                    name: "reactorId",
                    ty: "string | null",
                },
                Field {
                    name: "optionId",
                    ty: "string | null",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReactionAuditEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "windowId",
                    ty: "string",
                },
                Field {
                    name: "reactorId",
                    ty: "string",
                },
                Field {
                    name: "responseKind",
                    ty: "RulebenchReactionResponseKindDto",
                },
                Field {
                    name: "optionId",
                    ty: "string | null",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchReactionDecisionKindDto",
                },
                Field {
                    name: "trace",
                    ty: "readonly RulebenchLiveTraceEntryDto[]",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatAutomationPolicySpecDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "number",
                },
                Field {
                    name: "noCandidateBehavior",
                    ty: "RulebenchCombatAutomationNoCandidateBehaviorDto",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatAutomationPolicyValidationReadoutDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "code",
                    ty: "RulebenchCombatAutomationPolicyValidationCodeDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatAutomationCandidateEvidenceDto",
            fields: &[
                Field {
                    name: "index",
                    ty: "number",
                },
                Field {
                    name: "actionId",
                    ty: "string",
                },
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "targetSideId",
                    ty: "string",
                },
                Field {
                    name: "targetCurrentHitPoints",
                    ty: "number",
                },
                Field {
                    name: "targetMaxHitPoints",
                    ty: "number",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchCommandPreflightDecisionKindDto",
                },
                Field {
                    name: "policyScore",
                    ty: "number",
                },
                Field {
                    name: "policyReason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatAutomationPolicyDecisionEvidenceDto",
            fields: &[
                Field {
                    name: "policy",
                    ty: "RulebenchCombatAutomationPolicySpecDto",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "operationKind",
                    ty: "RulebenchAutomaticStepOperationKindDto | null",
                },
                Field {
                    name: "selectedActionId",
                    ty: "string | null",
                },
                Field {
                    name: "selectedTargetId",
                    ty: "string | null",
                },
                Field {
                    name: "selectedCandidateIndex",
                    ty: "number | null",
                },
                Field {
                    name: "candidateCount",
                    ty: "number",
                },
                Field {
                    name: "acceptedCandidateCount",
                    ty: "number",
                },
                Field {
                    name: "candidates",
                    ty: "readonly RulebenchCombatAutomationCandidateEvidenceDto[]",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomaticStepSpecDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "rollStream",
                    ty: "readonly number[]",
                },
                Field {
                    name: "policy",
                    ty: "RulebenchCombatAutomationPolicySpecDto",
                },
                Field {
                    name: "rollMode?",
                    ty: "\"supplied\" | \"authorityGenerated\"",
                },
                Field {
                    name: "generatedSeed?",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomaticRunSpecDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "maxSteps",
                    ty: "number",
                },
                Field {
                    name: "rollStream",
                    ty: "readonly number[]",
                },
                Field {
                    name: "policy",
                    ty: "RulebenchCombatAutomationPolicySpecDto",
                },
                Field {
                    name: "rollMode?",
                    ty: "\"supplied\" | \"authorityGenerated\"",
                },
                Field {
                    name: "generatedSeed?",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomaticRunReadoutDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchAutomaticRunDecisionKindDto",
                },
                Field {
                    name: "maxSteps",
                    ty: "number",
                },
                Field {
                    name: "policy",
                    ty: "RulebenchCombatAutomationPolicySpecDto",
                },
                Field {
                    name: "executedStepCount",
                    ty: "number",
                },
                Field {
                    name: "stepDecisions",
                    ty: "readonly RulebenchAutomaticStepDecisionDto[]",
                },
                Field {
                    name: "policyDecisions",
                    ty: "readonly RulebenchCombatAutomationPolicyDecisionEvidenceDto[]",
                },
                Field {
                    name: "finalLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "finalState",
                    ty: "RulebenchFinalStateDto",
                },
                Field {
                    name: "finalActionResourceLedger",
                    ty: "RulebenchActionResourceLedgerDto",
                },
                Field {
                    name: "finalEquipmentLedger",
                    ty: "RulebenchEquipmentLedgerDto",
                },
                Field {
                    name: "finalClassBuildLedger",
                    ty: "RulebenchClassBuildLedgerDto",
                },
                Field {
                    name: "finalCurrentActorOptions",
                    ty: "RulebenchCurrentActorOptionSummaryDto",
                },
                Field {
                    name: "finalCombatantVitality",
                    ty: "RulebenchCombatantVitalitySummaryDto",
                },
                Field {
                    name: "finalCombatEndCondition",
                    ty: "RulebenchCombatEndConditionDto",
                },
                Field {
                    name: "finalization",
                    ty: "RulebenchCombatFinalizationDto | null",
                },
                Field {
                    name: "combatLog",
                    ty: "readonly RulebenchCombatLogEntryDto[]",
                },
                Field {
                    name: "commandAuditLog",
                    ty: "readonly RulebenchCommandAuditEntryDto[]",
                },
                Field {
                    name: "lifecycleTransitionLog",
                    ty: "readonly RulebenchLifecycleTransitionEntryDto[]",
                },
                Field {
                    name: "turnTransitionLog",
                    ty: "readonly RulebenchTurnTransitionEntryDto[]",
                },
                Field {
                    name: "actionUsageLog",
                    ty: "readonly RulebenchActionUsageEntryDto[]",
                },
                Field {
                    name: "actionResourceTransitionLog",
                    ty: "readonly RulebenchActionResourceTransitionEntryDto[]",
                },
                Field {
                    name: "equipmentTransitionLog",
                    ty: "readonly RulebenchEquipmentTransitionEntryDto[]",
                },
                Field {
                    name: "currentReactionWindow",
                    ty: "RulebenchReactionWindowDto | null",
                },
                Field {
                    name: "reactionWindowLifecycleLog",
                    ty: "readonly RulebenchReactionWindowLifecycleEntryDto[]",
                },
                Field {
                    name: "reactionAuditLog",
                    ty: "readonly RulebenchReactionAuditEntryDto[]",
                },
                Field {
                    name: "modifierDurationExpirationLog",
                    ty: "readonly RulebenchModifierDurationExpirationEntryDto[]",
                },
                Field {
                    name: "combatLogEntryCount",
                    ty: "number",
                },
                Field {
                    name: "auditEntryCount",
                    ty: "number",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomaticRunReplayReadoutDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchAutomaticRunReplayDecisionKindDto",
                },
                Field {
                    name: "expectedFinalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "actualFinalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "finalStateFingerprintMatches",
                    ty: "boolean",
                },
                Field {
                    name: "finalizationMatches",
                    ty: "boolean",
                },
                Field {
                    name: "expectedRunDecisionKind",
                    ty: "RulebenchAutomaticRunDecisionKindDto",
                },
                Field {
                    name: "actualRunDecisionKind",
                    ty: "RulebenchAutomaticRunDecisionKindDto",
                },
                Field {
                    name: "runDecisionKindMatches",
                    ty: "boolean",
                },
                Field {
                    name: "expectedExecutedStepCount",
                    ty: "number",
                },
                Field {
                    name: "actualExecutedStepCount",
                    ty: "number",
                },
                Field {
                    name: "executedStepCountMatches",
                    ty: "boolean",
                },
                Field {
                    name: "policyDecisionsMatch",
                    ty: "boolean",
                },
                Field {
                    name: "actionResourceTransitionLogMatches",
                    ty: "boolean",
                },
                Field {
                    name: "equipmentLedgerMatches",
                    ty: "boolean",
                },
                Field {
                    name: "classBuildLedgerMatches",
                    ty: "boolean",
                },
                Field {
                    name: "equipmentTransitionLogMatches",
                    ty: "boolean",
                },
                Field {
                    name: "reactionWindowLifecycleLogMatches",
                    ty: "boolean",
                },
                Field {
                    name: "reactionAuditLogMatches",
                    ty: "boolean",
                },
                Field {
                    name: "modifierDurationExpirationLogMatches",
                    ty: "boolean",
                },
                Field {
                    name: "replayedRun",
                    ty: "RulebenchAutomaticRunReadoutDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomaticStepDecisionDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchAutomaticStepDecisionKindDto",
                },
                Field {
                    name: "operationKind",
                    ty: "RulebenchAutomaticStepOperationKindDto | null",
                },
                Field {
                    name: "policyValidation",
                    ty: "RulebenchCombatAutomationPolicyValidationReadoutDto",
                },
                Field {
                    name: "policyDecision",
                    ty: "RulebenchCombatAutomationPolicyDecisionEvidenceDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchLifecycleTransitionEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "trigger",
                    ty: "RulebenchLifecycleTransitionTriggerDto",
                },
                Field {
                    name: "stepIndex",
                    ty: "number",
                },
                Field {
                    name: "previousLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "nextLifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "startedAtStep",
                    ty: "number | null",
                },
                Field {
                    name: "endedAtStep",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchTurnTransitionEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "previousRoundNumber",
                    ty: "number",
                },
                Field {
                    name: "previousTurnIndex",
                    ty: "number",
                },
                Field {
                    name: "previousActorId",
                    ty: "string | null",
                },
                Field {
                    name: "nextRoundNumber",
                    ty: "number",
                },
                Field {
                    name: "nextTurnIndex",
                    ty: "number",
                },
                Field {
                    name: "nextActorId",
                    ty: "string | null",
                },
                Field {
                    name: "wrappedRound",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchActionUsageSummaryDto",
            fields: &[
                Field {
                    name: "roundNumber",
                    ty: "number",
                },
                Field {
                    name: "turnIndex",
                    ty: "number",
                },
                Field {
                    name: "currentActorId",
                    ty: "string | null",
                },
                Field {
                    name: "usedActionCount",
                    ty: "number",
                },
                Field {
                    name: "usedActionIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "usedAbilityIds",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCurrentActorOptionSummaryDto",
            fields: &[
                Field {
                    name: "roundNumber",
                    ty: "number",
                },
                Field {
                    name: "turnIndex",
                    ty: "number",
                },
                Field {
                    name: "lifecyclePhase",
                    ty: "RulebenchCombatLifecyclePhaseDto",
                },
                Field {
                    name: "currentActorId",
                    ty: "string | null",
                },
                Field {
                    name: "currentActorDefeated",
                    ty: "boolean",
                },
                Field {
                    name: "available",
                    ty: "boolean",
                },
                Field {
                    name: "unavailableReason",
                    ty: "RulebenchCurrentActorOptionsUnavailableReasonDto | null",
                },
                Field {
                    name: "actions",
                    ty: "readonly RulebenchCurrentActorActionOptionDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCurrentActorActionOptionDto",
            fields: &[
                Field {
                    name: "actionId",
                    ty: "string",
                },
                Field {
                    name: "abilityId",
                    ty: "string",
                },
                Field {
                    name: "actionName",
                    ty: "string",
                },
                Field {
                    name: "available",
                    ty: "boolean",
                },
                Field {
                    name: "unavailableReason",
                    ty: "string | null",
                },
                Field {
                    name: "resourceCosts",
                    ty: "readonly RulebenchActionResourceCostDto[]",
                },
                Field {
                    name: "resourceStates",
                    ty: "readonly RulebenchActionResourceStateDto[]",
                },
                Field {
                    name: "targetOptions",
                    ty: "readonly RulebenchCurrentActorTargetOptionDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchActionResourceCostDto",
            fields: &[
                Field {
                    name: "resourceId",
                    ty: "string",
                },
                Field {
                    name: "amount",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchCurrentActorTargetOptionDto",
            fields: &[
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "targetName",
                    ty: "string",
                },
                Field {
                    name: "currentHitPoints",
                    ty: "number",
                },
                Field {
                    name: "maxHitPoints",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatantVitalitySummaryDto",
            fields: &[
                Field {
                    name: "combatants",
                    ty: "readonly RulebenchCombatantVitalityEntryDto[]",
                },
                Field {
                    name: "activeCombatantIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "defeatedCombatantIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "activeCount",
                    ty: "number",
                },
                Field {
                    name: "defeatedCount",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatantVitalityEntryDto",
            fields: &[
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "currentHitPoints",
                    ty: "number",
                },
                Field {
                    name: "maxHitPoints",
                    ty: "number",
                },
                Field {
                    name: "defeated",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatEndConditionDto",
            fields: &[
                Field {
                    name: "policy",
                    ty: "RulebenchCombatEndPolicyDto",
                },
                Field {
                    name: "combatShouldEnd",
                    ty: "boolean",
                },
                Field {
                    name: "conditionKind",
                    ty: "RulebenchCombatEndConditionKindDto",
                },
                Field {
                    name: "outcomeKind",
                    ty: "RulebenchCombatOutcomeKindDto",
                },
                Field {
                    name: "activeSides",
                    ty: "readonly RulebenchCombatSideIdDto[]",
                },
                Field {
                    name: "defeatedSides",
                    ty: "readonly RulebenchCombatSideIdDto[]",
                },
                Field {
                    name: "winningSides",
                    ty: "readonly RulebenchCombatSideIdDto[]",
                },
                Field {
                    name: "activeAllyCount",
                    ty: "number",
                },
                Field {
                    name: "activeEnemyCount",
                    ty: "number",
                },
                Field {
                    name: "defeatedAllyCount",
                    ty: "number",
                },
                Field {
                    name: "defeatedEnemyCount",
                    ty: "number",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatFinalizationDto",
            fields: &[
                Field {
                    name: "trigger",
                    ty: "RulebenchLifecycleTransitionTriggerDto",
                },
                Field {
                    name: "finalizedAtStep",
                    ty: "number",
                },
                Field {
                    name: "endCondition",
                    ty: "RulebenchCombatEndConditionDto",
                },
                Field {
                    name: "outcomeKind",
                    ty: "RulebenchCombatOutcomeKindDto",
                },
                Field {
                    name: "winningSides",
                    ty: "readonly RulebenchCombatSideIdDto[]",
                },
                Field {
                    name: "remainingSides",
                    ty: "readonly RulebenchCombatSideIdDto[]",
                },
                Field {
                    name: "finalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "combatLogEntryCount",
                    ty: "number",
                },
                Field {
                    name: "commandAuditEntryCount",
                    ty: "number",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchActiveModifierDto",
            fields: &[
                Field {
                    name: "modifierId",
                    ty: "string",
                },
                Field {
                    name: "sourceId",
                    ty: "string",
                },
                Field {
                    name: "label",
                    ty: "string",
                },
                Field {
                    name: "duration",
                    ty: "string",
                },
                Field {
                    name: "tenure",
                    ty: "RulebenchModifierTenureDto",
                },
                Field {
                    name: "stackingGroup",
                    ty: "string",
                },
                Field {
                    name: "stackingPolicy",
                    ty: "RulebenchModifierStackingPolicyDto",
                },
                Field {
                    name: "durationPolicy",
                    ty: "RulebenchModifierDurationPolicyDto",
                },
                Field {
                    name: "remainingTurns",
                    ty: "number | null",
                },
                Field {
                    name: "remainingRounds",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchModifierDurationExpirationEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "modifierId",
                    ty: "string",
                },
                Field {
                    name: "previousModifier",
                    ty: "RulebenchActiveModifierDto",
                },
                Field {
                    name: "nextModifier",
                    ty: "RulebenchActiveModifierDto | null",
                },
                Field {
                    name: "trigger",
                    ty: "RulebenchModifierDurationTransitionTriggerDto",
                },
                Field {
                    name: "turnTransitionSequence",
                    ty: "number | null",
                },
                Field {
                    name: "roundNumber",
                    ty: "number | null",
                },
                Field {
                    name: "turnIndex",
                    ty: "number | null",
                },
                Field {
                    name: "currentActorId",
                    ty: "string | null",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCommandAuditEntryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "stepId",
                    ty: "string",
                },
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchCommandOutcomeClassDto",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchCommandDecisionKindDto",
                },
                Field {
                    name: "preflightDecisionKind",
                    ty: "RulebenchCommandPreflightDecisionKindDto | null",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "rejection",
                    ty: "RulebenchRejectionCodeDto | null",
                },
                Field {
                    name: "eventCount",
                    ty: "number",
                },
                Field {
                    name: "traceCount",
                    ty: "number",
                },
                Field {
                    name: "rollConsumption",
                    ty: "readonly RulebenchRollConsumptionEntryDto[]",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "stateAfterFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
            ],
        },
        Interface {
            name: "RulebenchRollConsumptionEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "requestKind",
                    ty: "RulebenchRollRequestKindDto",
                },
                Field {
                    name: "suppliedValue",
                    ty: "number | null",
                },
                Field {
                    name: "consumed",
                    ty: "boolean",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchActionUsageEntryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "stepId",
                    ty: "string",
                },
                Field {
                    name: "stepIndex",
                    ty: "number",
                },
                Field {
                    name: "roundNumber",
                    ty: "number",
                },
                Field {
                    name: "turnIndex",
                    ty: "number",
                },
                Field {
                    name: "actorId",
                    ty: "string",
                },
                Field {
                    name: "actionId",
                    ty: "string",
                },
                Field {
                    name: "abilityId",
                    ty: "string",
                },
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchCommandOutcomeClassDto",
                },
            ],
        },
        Interface {
            name: "RulebenchActionResourceTransitionEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "transitionKind",
                    ty: "RulebenchActionResourceTransitionKindDto",
                },
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "resourceId",
                    ty: "string",
                },
                Field {
                    name: "resourceKind",
                    ty: "RulebenchActionResourceKindDto",
                },
                Field {
                    name: "amount",
                    ty: "number",
                },
                Field {
                    name: "previousResource",
                    ty: "RulebenchActionResourceStateDto",
                },
                Field {
                    name: "nextResource",
                    ty: "RulebenchActionResourceStateDto",
                },
                Field {
                    name: "commandStepId",
                    ty: "string | null",
                },
                Field {
                    name: "commandStepIndex",
                    ty: "number | null",
                },
                Field {
                    name: "turnTransitionSequence",
                    ty: "number | null",
                },
                Field {
                    name: "roundNumber",
                    ty: "number | null",
                },
                Field {
                    name: "turnIndex",
                    ty: "number | null",
                },
                Field {
                    name: "currentActorId",
                    ty: "string | null",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatScriptStepReadoutDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "commandKind",
                    ty: "RulebenchCombatScriptCommandKindDto",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchCombatScriptDecisionKindDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "stateAfterFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "runtimeStepId",
                    ty: "string | null",
                },
                Field {
                    name: "commandAuditSequence",
                    ty: "number | null",
                },
                Field {
                    name: "controlHistorySequence",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionStepSummaryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "index",
                    ty: "number",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchCommandOutcomeClassDto",
                },
                Field {
                    name: "logIndex",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchCommandAttemptDto",
            fields: &[
                Field {
                    name: "stepId",
                    ty: "string",
                },
                Field {
                    name: "stepIndex",
                    ty: "number",
                },
                Field {
                    name: "actorId",
                    ty: "string",
                },
                Field {
                    name: "actionId",
                    ty: "string",
                },
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "rollStream",
                    ty: "readonly number[]",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchCommandOutcomeClassDto",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatLogEntryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "stepId",
                    ty: "string",
                },
                Field {
                    name: "logIndex",
                    ty: "number",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchCommandOutcomeClassDto",
                },
                Field {
                    name: "eventTypes",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchActionResourceLedgerDto",
            fields: &[Field {
                name: "combatants",
                ty: "readonly RulebenchCombatantActionResourceReadoutDto[]",
            }],
        },
        Interface {
            name: "RulebenchEquipmentLedgerDto",
            fields: &[Field {
                name: "combatants",
                ty: "readonly RulebenchCombatantEquipmentReadoutDto[]",
            }],
        },
        Interface {
            name: "RulebenchClassBuildLedgerDto",
            fields: &[Field {
                name: "combatants",
                ty: "readonly RulebenchCombatantClassBuildReadoutDto[]",
            }],
        },
        Interface {
            name: "RulebenchCombatantClassBuildReadoutDto",
            fields: &[
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "classInputs",
                    ty: "readonly RulebenchClassBuildInputReadoutDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchClassBuildInputReadoutDto",
            fields: &[
                Field {
                    name: "classId",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "level",
                    ty: "number",
                },
                Field {
                    name: "appliedGrantLevels",
                    ty: "readonly number[]",
                },
                Field {
                    name: "sourceIds",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchEquipmentCommandSpecDto",
            fields: &[
                Field {
                    name: "kind",
                    ty: "RulebenchEquipmentCommandKindDto",
                },
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "itemId",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchEquipmentCommandReadoutDto",
            fields: &[
                Field {
                    name: "command",
                    ty: "RulebenchEquipmentCommandSpecDto",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchEquipmentDecisionKindDto",
                },
                Field {
                    name: "previousEquipment",
                    ty: "RulebenchCombatantEquipmentReadoutDto | null",
                },
                Field {
                    name: "nextEquipment",
                    ty: "RulebenchCombatantEquipmentReadoutDto | null",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatantEquipmentReadoutDto",
            fields: &[
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "inventoryItemIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "equippedItemIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "availableAbilityIds",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchEquipmentTransitionEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "transitionKind",
                    ty: "RulebenchEquipmentTransitionKindDto",
                },
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "itemId",
                    ty: "string",
                },
                Field {
                    name: "equipmentSlot",
                    ty: "string",
                },
                Field {
                    name: "grantedModifierIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "grantedAbilityIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "grantedResourceIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "previousEquipment",
                    ty: "RulebenchCombatantEquipmentReadoutDto",
                },
                Field {
                    name: "nextEquipment",
                    ty: "RulebenchCombatantEquipmentReadoutDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatantActionResourceReadoutDto",
            fields: &[
                Field {
                    name: "combatantId",
                    ty: "string",
                },
                Field {
                    name: "resources",
                    ty: "readonly RulebenchActionResourceStateDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchActionResourceStateDto",
            fields: &[
                Field {
                    name: "resourceId",
                    ty: "string",
                },
                Field {
                    name: "sourceId",
                    ty: "string",
                },
                Field {
                    name: "kind",
                    ty: "RulebenchActionResourceKindDto",
                },
                Field {
                    name: "current",
                    ty: "number",
                },
                Field {
                    name: "max",
                    ty: "number",
                },
                Field {
                    name: "available",
                    ty: "boolean",
                },
                Field {
                    name: "refreshPolicy",
                    ty: "RulebenchActionResourceRefreshPolicyDto",
                },
                Field {
                    name: "remainingRefreshTurns",
                    ty: "number | null",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatSessionStepReadoutDto",
            fields: &[
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "step",
                    ty: "RulebenchCombatSessionStepSummaryDto",
                },
                Field {
                    name: "command",
                    ty: "RulebenchCommandAttemptDto",
                },
                Field {
                    name: "scenarioReadout",
                    ty: "RulebenchScenarioReadoutDto",
                },
                Field {
                    name: "combatLog",
                    ty: "readonly RulebenchCombatLogEntryDto[]",
                },
                Field {
                    name: "actionResourceLedger",
                    ty: "RulebenchActionResourceLedgerDto",
                },
                Field {
                    name: "stateBefore",
                    ty: "RulebenchFinalStateDto",
                },
                Field {
                    name: "stateAfter",
                    ty: "RulebenchFinalStateDto",
                },
            ],
        },
        Interface {
            name: "RulebenchScenarioCatalogDto",
            fields: &[
                Field {
                    name: "summaries",
                    ty: "readonly RulebenchScenarioCatalogSummaryDto[]",
                },
                Field {
                    name: "readouts",
                    ty: "readonly RulebenchScenarioReadoutDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentValidationCatalogDto",
            fields: &[Field {
                name: "reports",
                ty: "readonly RulebenchContentValidationReadoutDto[]",
            }],
        },
        Interface {
            name: "RulebenchContentImportCatalogDto",
            fields: &[Field {
                name: "examples",
                ty: "readonly RulebenchContentImportReadoutDto[]",
            }],
        },
        Interface {
            name: "RulebenchContentImportReadoutDto",
            fields: &[
                Field {
                    name: "exampleId",
                    ty: "string",
                },
                Field {
                    name: "pack",
                    ty: "RulebenchContentPackIdentityDto",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "errorCount",
                    ty: "number",
                },
                Field {
                    name: "warningCount",
                    ty: "number",
                },
                Field {
                    name: "diagnostics",
                    ty: "readonly RulebenchContentImportDiagnosticDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentPackIdentityDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "fingerprint",
                    ty: "RulebenchContentFingerprintDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchContentFingerprintDto",
            fields: &[
                Field {
                    name: "algorithm",
                    ty: "string",
                },
                Field {
                    name: "value",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredContentPackDocumentDto",
            fields: &[
                Field {
                    name: "format",
                    ty: "string",
                },
                Field {
                    name: "formatVersion",
                    ty: "number",
                },
                Field {
                    name: "pack",
                    ty: "RulebenchAuthoredContentPackDto",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredContentPackDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "tags",
                    ty: "readonly string[]",
                },
                Field {
                    name: "provenance",
                    ty: "RulebenchAuthoredContentProvenanceDto",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "dependencies",
                    ty: "readonly RulebenchContentPackReferenceDto[]",
                },
                Field {
                    name: "catalogs",
                    ty: "RulebenchAuthoredContentCatalogsDto",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredContentProvenanceDto",
            fields: &[
                Field {
                    name: "sourceKind",
                    ty: "RulebenchAuthoredContentSourceKindDto",
                },
                Field {
                    name: "sourceId",
                    ty: "string",
                },
                Field {
                    name: "authoredBy",
                    ty: "string | null",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredContentCatalogsDto",
            fields: &[
                Field {
                    name: "rulesets",
                    ty: "readonly RulebenchRulesetDefinitionDto[]",
                },
                Field {
                    name: "entities",
                    ty: "readonly RulebenchAuthoredEntityDefinitionDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredEntityDefinitionDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "tags",
                    ty: "readonly string[]",
                },
                Field {
                    name: "damageAdjustments",
                    ty: "readonly RulebenchAuthoredDamageAdjustmentDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchAuthoredDamageAdjustmentDto",
            fields: &[
                Field {
                    name: "damageType",
                    ty: "string",
                },
                Field {
                    name: "policy",
                    ty: "RulebenchAuthoredDamageAdjustmentPolicyDto",
                },
            ],
        },
        Interface {
            name: "RulebenchContentPackReferenceDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "string",
                },
                Field {
                    name: "fingerprint",
                    ty: "RulebenchContentFingerprintDto",
                },
            ],
        },
        Interface {
            name: "RulebenchContentImportRequestDto",
            fields: &[
                Field {
                    name: "authoredPayload",
                    ty: "string",
                },
                Field {
                    name: "replacementPolicy",
                    ty: "RulebenchContentReplacementPolicyDto",
                },
            ],
        },
        Interface {
            name: "RulebenchContentPayloadRequestDto",
            fields: &[Field {
                name: "authoredPayload",
                ty: "string",
            }],
        },
        Interface {
            name: "RulebenchContentReferenceRequestDto",
            fields: &[Field {
                name: "reference",
                ty: "RulebenchContentPackReferenceDto",
            }],
        },
        Interface {
            name: "RulebenchContentDefinitionSummaryDto",
            fields: &[
                Field {
                    name: "kind",
                    ty: "RulebenchContentDefinitionKindDto",
                },
                Field {
                    name: "id",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchStoredContentPackSummaryDto",
            fields: &[
                Field {
                    name: "reference",
                    ty: "RulebenchContentPackReferenceDto",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "sourceKind",
                    ty: "string",
                },
                Field {
                    name: "sourceId",
                    ty: "string",
                },
                Field {
                    name: "authoredBy",
                    ty: "string | null",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "dependencies",
                    ty: "readonly RulebenchContentPackReferenceDto[]",
                },
                Field {
                    name: "definitions",
                    ty: "readonly RulebenchContentDefinitionSummaryDto[]",
                },
                Field {
                    name: "active",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchContentPackReviewDto",
            fields: &[
                Field {
                    name: "pack",
                    ty: "RulebenchStoredContentPackSummaryDto",
                },
                Field {
                    name: "authoredPayload",
                    ty: "string",
                },
                Field {
                    name: "diagnostics",
                    ty: "readonly RulebenchContentImportDiagnosticDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentDefinitionChangeDto",
            fields: &[
                Field {
                    name: "kind",
                    ty: "RulebenchContentDefinitionKindDto",
                },
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "change",
                    ty: "'added' | 'removed' | 'changed'",
                },
            ],
        },
        Interface {
            name: "RulebenchContentPackDiffDto",
            fields: &[
                Field {
                    name: "before",
                    ty: "RulebenchContentPackReferenceDto",
                },
                Field {
                    name: "after",
                    ty: "RulebenchContentPackReferenceDto",
                },
                Field {
                    name: "changed",
                    ty: "boolean",
                },
                Field {
                    name: "fingerprintChanged",
                    ty: "boolean",
                },
                Field {
                    name: "rulesetCompatibilityChanged",
                    ty: "boolean",
                },
                Field {
                    name: "dependencySetChanged",
                    ty: "boolean",
                },
                Field {
                    name: "metadataChanges",
                    ty: "readonly string[]",
                },
                Field {
                    name: "definitionChanges",
                    ty: "readonly RulebenchContentDefinitionChangeDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentImportOutcomeDto",
            fields: &[
                Field {
                    name: "review",
                    ty: "RulebenchContentPackReviewDto",
                },
                Field {
                    name: "replaced",
                    ty: "RulebenchContentPackReferenceDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchContentImportAttemptDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "pack",
                    ty: "RulebenchContentPackIdentityDto",
                },
                Field {
                    name: "outcome",
                    ty: "RulebenchContentImportOutcomeDto | null",
                },
                Field {
                    name: "diagnostics",
                    ty: "readonly RulebenchContentImportDiagnosticDto[]",
                },
                Field {
                    name: "errorCode",
                    ty: "string | null",
                },
                Field {
                    name: "errorMessage",
                    ty: "string | null",
                },
            ],
        },
        Interface {
            name: "RulebenchContentAuditEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "operation",
                    ty: "string",
                },
                Field {
                    name: "reference",
                    ty: "RulebenchContentPackReferenceDto",
                },
                Field {
                    name: "detail",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchContentWorkspaceDto",
            fields: &[
                Field {
                    name: "packs",
                    ty: "readonly RulebenchStoredContentPackSummaryDto[]",
                },
                Field {
                    name: "audit",
                    ty: "readonly RulebenchContentAuditEntryDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentImportDiagnosticDto",
            fields: &[
                Field {
                    name: "severity",
                    ty: "RulebenchContentImportDiagnosticSeverityDto",
                },
                Field {
                    name: "code",
                    ty: "string",
                },
                Field {
                    name: "path",
                    ty: "string",
                },
                Field {
                    name: "referenceId",
                    ty: "string | null",
                },
                Field {
                    name: "definitionKind",
                    ty: "RulebenchContentDefinitionKindDto | null",
                },
                Field {
                    name: "message",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchContentValidationReadoutDto",
            fields: &[
                Field {
                    name: "scenarioId",
                    ty: "string",
                },
                Field {
                    name: "scenarioTitle",
                    ty: "string",
                },
                Field {
                    name: "report",
                    ty: "RulebenchContentValidationReportDto",
                },
            ],
        },
        Interface {
            name: "RulebenchContentValidationReportDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "errorCount",
                    ty: "number",
                },
                Field {
                    name: "warningCount",
                    ty: "number",
                },
                Field {
                    name: "diagnostics",
                    ty: "readonly RulebenchContentDiagnosticDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchContentDiagnosticDto",
            fields: &[
                Field {
                    name: "severity",
                    ty: "RulebenchContentDiagnosticSeverityDto",
                },
                Field {
                    name: "code",
                    ty: "string",
                },
                Field {
                    name: "contentId",
                    ty: "string | null",
                },
                Field {
                    name: "message",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchScenarioCatalogSummaryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "seedLabel",
                    ty: "string",
                },
                Field {
                    name: "outcomeClass",
                    ty: "RulebenchScenarioOutcomeClassDto",
                },
            ],
        },
        Interface {
            name: "RulebenchUseActionIntentDto",
            fields: &[
                Field {
                    name: "actorId",
                    ty: "string",
                },
                Field {
                    name: "actionId",
                    ty: "string",
                },
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "targetIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "targetCell",
                    ty: "RulebenchLiveGridPositionDto | null",
                },
                Field {
                    name: "destinationCell",
                    ty: "RulebenchLiveGridPositionDto | null",
                },
                Field {
                    name: "observedOrigin",
                    ty: "RulebenchLiveGridPositionDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchResolutionReceiptDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "authoritySurface",
                    ty: "string",
                },
                Field {
                    name: "intent",
                    ty: "RulebenchUseActionIntentDto",
                },
                Field {
                    name: "rejection",
                    ty: "RulebenchRejectionCodeDto | null",
                },
                Field {
                    name: "selectedTarget",
                    ty: "RulebenchSelectedTargetDto | null",
                },
                Field {
                    name: "attackRoll",
                    ty: "RulebenchAttackRollDto | null",
                },
                Field {
                    name: "damage",
                    ty: "RulebenchDamageOutcomeDto | null",
                },
                Field {
                    name: "modifier",
                    ty: "RulebenchModifierOutcomeDto | null",
                },
                Field {
                    name: "domainEvents",
                    ty: "readonly RulebenchDomainEventDto[]",
                },
                Field {
                    name: "trace",
                    ty: "readonly RulebenchTraceEntryDto[]",
                },
                Field {
                    name: "finalState",
                    ty: "RulebenchFinalStateDto | null",
                },
            ],
        },
        Interface {
            name: "RulebenchScenarioReadoutDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "seedLabel",
                    ty: "string",
                },
                Field {
                    name: "grid",
                    ty: "RulebenchGridDto",
                },
                Field {
                    name: "combatants",
                    ty: "readonly RulebenchCombatantDto[]",
                },
                Field {
                    name: "selectedAction",
                    ty: "RulebenchSelectedActionDto",
                },
                Field {
                    name: "selectedTarget",
                    ty: "RulebenchSelectedTargetDto",
                },
                Field {
                    name: "domainEvents",
                    ty: "readonly RulebenchDomainEventDto[]",
                },
                Field {
                    name: "trace",
                    ty: "readonly RulebenchTraceEntryDto[]",
                },
                Field {
                    name: "finalState",
                    ty: "RulebenchFinalStateDto",
                },
            ],
        },
        Interface {
            name: "RulebenchGridDto",
            fields: &[
                Field {
                    name: "width",
                    ty: "number",
                },
                Field {
                    name: "height",
                    ty: "number",
                },
                Field {
                    name: "cells",
                    ty: "readonly RulebenchGridCellDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchGridCellDto",
            fields: &[
                Field {
                    name: "x",
                    ty: "number",
                },
                Field {
                    name: "y",
                    ty: "number",
                },
                Field {
                    name: "terrainTags",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchCombatantDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "team",
                    ty: "RulebenchTeamDto",
                },
                Field {
                    name: "sideId",
                    ty: "RulebenchCombatSideIdDto",
                },
                Field {
                    name: "position",
                    ty: "RulebenchGridPositionDto",
                },
                Field {
                    name: "hitPoints",
                    ty: "RulebenchBoundedValueDto",
                },
                Field {
                    name: "defenses",
                    ty: "readonly RulebenchNamedNumberDto[]",
                },
                Field {
                    name: "conditions",
                    ty: "readonly string[]",
                },
                Field {
                    name: "isActor",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchGridPositionDto",
            fields: &[
                Field {
                    name: "x",
                    ty: "number",
                },
                Field {
                    name: "y",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchBoundedValueDto",
            fields: &[
                Field {
                    name: "current",
                    ty: "number",
                },
                Field {
                    name: "max",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchNamedNumberDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "label",
                    ty: "string",
                },
                Field {
                    name: "value",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchSelectedActionDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "actorId",
                    ty: "string",
                },
                Field {
                    name: "targetIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "range",
                    ty: "number",
                },
                Field {
                    name: "lineOfSightRequired",
                    ty: "boolean",
                },
                Field {
                    name: "visibleTargetIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "attack",
                    ty: "RulebenchAttackSpecDto | null",
                },
                Field {
                    name: "savingThrow",
                    ty: "RulebenchSavingThrowSpecDto | null",
                },
                Field {
                    name: "contested",
                    ty: "RulebenchContestedCheckSpecDto | null",
                },
                Field {
                    name: "hit",
                    ty: "RulebenchHitEffectDto",
                },
                Field {
                    name: "actionText",
                    ty: "string",
                },
                Field {
                    name: "effectText",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAttackSpecDto",
            fields: &[
                Field {
                    name: "modifier",
                    ty: "number",
                },
                Field {
                    name: "defenseId",
                    ty: "string",
                },
                Field {
                    name: "defenseLabel",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchSavingThrowSpecDto",
            fields: &[
                Field {
                    name: "saveStatId",
                    ty: "string",
                },
                Field {
                    name: "difficultyClass",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchContestedCheckSpecDto",
            fields: &[
                Field {
                    name: "actorStatId",
                    ty: "string",
                },
                Field {
                    name: "targetStatId",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchHitEffectDto",
            fields: &[
                Field {
                    name: "damageBonus",
                    ty: "number",
                },
                Field {
                    name: "damageType",
                    ty: "string",
                },
                Field {
                    name: "modifierId",
                    ty: "string",
                },
                Field {
                    name: "modifierLabel",
                    ty: "string",
                },
                Field {
                    name: "modifierDuration",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchSelectedTargetDto",
            fields: &[
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "legality",
                    ty: "RulebenchLegalityDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAttackRollDto",
            fields: &[
                Field {
                    name: "roll",
                    ty: "number",
                },
                Field {
                    name: "modifier",
                    ty: "number",
                },
                Field {
                    name: "total",
                    ty: "number",
                },
                Field {
                    name: "defenseId",
                    ty: "string",
                },
                Field {
                    name: "defenseValue",
                    ty: "number",
                },
                Field {
                    name: "outcome",
                    ty: "RulebenchAttackOutcomeDto",
                },
            ],
        },
        Interface {
            name: "RulebenchDamageOutcomeDto",
            fields: &[
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "damageType",
                    ty: "string",
                },
                Field {
                    name: "amount",
                    ty: "number",
                },
                Field {
                    name: "before",
                    ty: "RulebenchBoundedValueDto",
                },
                Field {
                    name: "after",
                    ty: "RulebenchBoundedValueDto",
                },
            ],
        },
        Interface {
            name: "RulebenchModifierOutcomeDto",
            fields: &[
                Field {
                    name: "targetId",
                    ty: "string",
                },
                Field {
                    name: "modifierId",
                    ty: "string",
                },
                Field {
                    name: "label",
                    ty: "string",
                },
                Field {
                    name: "duration",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchDomainEventDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "type",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "entityIds",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchTraceEntryDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "phase",
                    ty: "RulebenchTracePhaseDto",
                },
                Field {
                    name: "status",
                    ty: "RulebenchTraceStatusDto",
                },
                Field {
                    name: "message",
                    ty: "string",
                },
                Field {
                    name: "detail",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchFinalStateDto",
            fields: &[
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "combatants",
                    ty: "readonly RulebenchFinalCombatantStateDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchFinalCombatantStateDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "name",
                    ty: "string",
                },
                Field {
                    name: "hitPoints",
                    ty: "RulebenchBoundedValueDto",
                },
                Field {
                    name: "conditions",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayArchiveMetadataDto",
            fields: &[
                Field {
                    name: "packageId",
                    ty: "string",
                },
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "scenarioId",
                    ty: "string",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "completedAt",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayPackageReviewDto",
            fields: &[
                Field {
                    name: "packageVersion",
                    ty: "string",
                },
                Field {
                    name: "packageId",
                    ty: "string",
                },
                Field {
                    name: "sessionId",
                    ty: "string",
                },
                Field {
                    name: "scenarioId",
                    ty: "string",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "contentPackRoot",
                    ty: "RulebenchContentPackReferenceDto | null",
                },
                Field {
                    name: "contentPackSetFingerprint",
                    ty: "RulebenchContentFingerprintDto | null",
                },
                Field {
                    name: "contentPackReferences",
                    ty: "readonly RulebenchContentPackReferenceDto[]",
                },
                Field {
                    name: "commandCount",
                    ty: "number",
                },
                Field {
                    name: "finalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "fingerprintKind",
                    ty: "string",
                },
                Field {
                    name: "narrationTitle",
                    ty: "string | null",
                },
                Field {
                    name: "narrationSummary",
                    ty: "string | null",
                },
                Field {
                    name: "commands",
                    ty: "readonly RulebenchReplayCommandReviewDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayCommandReviewDto",
            fields: &[
                Field {
                    name: "sequence",
                    ty: "number",
                },
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "commandKind",
                    ty: "string",
                },
                Field {
                    name: "suppliedRollStream",
                    ty: "readonly number[]",
                },
                Field {
                    name: "narrationSummary",
                    ty: "string | null",
                },
                Field {
                    name: "expected",
                    ty: "RulebenchReplayStepEvidenceDto",
                },
                Field {
                    name: "actual",
                    ty: "RulebenchReplayStepEvidenceDto",
                },
                Field {
                    name: "snapshot",
                    ty: "RulebenchLiveSessionSnapshotDto",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayStepEvidenceDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionCode",
                    ty: "string",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "stateAfterFingerprint",
                    ty: "RulebenchStateFingerprintDto",
                },
                Field {
                    name: "acceptedEvents",
                    ty: "readonly RulebenchLiveDomainEventDto[]",
                },
                Field {
                    name: "commandAudit",
                    ty: "readonly RulebenchLiveAuditEntryDto[]",
                },
                Field {
                    name: "rolls",
                    ty: "readonly RulebenchLiveRollEvidenceDto[]",
                },
                Field {
                    name: "trace",
                    ty: "readonly RulebenchLiveTraceEntryDto[]",
                },
                Field {
                    name: "gameplayModuleStateHash",
                    ty: "string",
                },
                Field {
                    name: "gameplayDecisionReceiptHashes",
                    ty: "readonly string[]",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayMismatchDto",
            fields: &[
                Field {
                    name: "commandSequence",
                    ty: "number | null",
                },
                Field {
                    name: "commandId",
                    ty: "string | null",
                },
                Field {
                    name: "dimension",
                    ty: "RulebenchReplayMismatchDimensionDto",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayVerificationReadoutDto",
            fields: &[
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "decisionKind",
                    ty: "RulebenchReplayVerificationDecisionKindDto",
                },
                Field {
                    name: "verifiedStepCount",
                    ty: "number",
                },
                Field {
                    name: "mismatch",
                    ty: "RulebenchReplayMismatchDto | null",
                },
                Field {
                    name: "finalStateFingerprint",
                    ty: "RulebenchStateFingerprintDto | null",
                },
                Field {
                    name: "finalized",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayComparisonDifferenceDto",
            fields: &[
                Field {
                    name: "code",
                    ty: "RulebenchReplayComparisonDifferenceCodeDto",
                },
                Field {
                    name: "path",
                    ty: "string",
                },
                Field {
                    name: "commandSequence",
                    ty: "number | null",
                },
                Field {
                    name: "commandId",
                    ty: "string | null",
                },
                Field {
                    name: "expectedSummary",
                    ty: "string",
                },
                Field {
                    name: "actualSummary",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayComparisonReadoutDto",
            fields: &[
                Field {
                    name: "matches",
                    ty: "boolean",
                },
                Field {
                    name: "expectedPackageId",
                    ty: "string",
                },
                Field {
                    name: "actualPackageId",
                    ty: "string",
                },
                Field {
                    name: "comparedCommandCount",
                    ty: "number",
                },
                Field {
                    name: "firstDifference",
                    ty: "RulebenchReplayComparisonDifferenceDto | null",
                },
                Field {
                    name: "differences",
                    ty: "readonly RulebenchReplayComparisonDifferenceDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayComparisonRequestDto",
            fields: &[
                Field {
                    name: "expectedPackageId",
                    ty: "string",
                },
                Field {
                    name: "actualPackageId",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchReplayArchiveErrorDto",
            fields: &[
                Field {
                    name: "kind",
                    ty: "RulebenchReplayArchiveErrorKindDto",
                },
                Field {
                    name: "code",
                    ty: "string",
                },
                Field {
                    name: "message",
                    ty: "string",
                },
                Field {
                    name: "retryable",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchPolicyRulesetCompatibilityDto",
            fields: &[
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "compatible",
                    ty: "boolean",
                },
                Field {
                    name: "code",
                    ty: "string",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchAutomationPolicyCatalogEntryDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "version",
                    ty: "number",
                },
                Field {
                    name: "title",
                    ty: "string",
                },
                Field {
                    name: "summary",
                    ty: "string",
                },
                Field {
                    name: "selector",
                    ty: "string",
                },
                Field {
                    name: "requirement",
                    ty: "string",
                },
                Field {
                    name: "compatibility",
                    ty: "readonly RulebenchPolicyRulesetCompatibilityDto[]",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentMatrixRequestDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "scenarioIds",
                    ty: "readonly string[]",
                },
                Field {
                    name: "policies",
                    ty: "readonly RulebenchCombatAutomationPolicySpecDto[]",
                },
                Field {
                    name: "seeds",
                    ty: "readonly number[]",
                },
                Field {
                    name: "maxSteps",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentDecisionEvidenceDto",
            fields: &[
                Field {
                    name: "index",
                    ty: "number",
                },
                Field {
                    name: "stateBeforeFingerprint",
                    ty: "string",
                },
                Field {
                    name: "operationKind",
                    ty: "string | null",
                },
                Field {
                    name: "selectedActionId",
                    ty: "string | null",
                },
                Field {
                    name: "selectedTargetId",
                    ty: "string | null",
                },
                Field {
                    name: "selectedCandidateIndex",
                    ty: "number | null",
                },
                Field {
                    name: "candidateCount",
                    ty: "number",
                },
                Field {
                    name: "acceptedCandidateCount",
                    ty: "number",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentMetricsDto",
            fields: &[
                Field {
                    name: "executedStepCount",
                    ty: "number",
                },
                Field {
                    name: "acceptedCommandCount",
                    ty: "number",
                },
                Field {
                    name: "initialTotalHitPoints",
                    ty: "number",
                },
                Field {
                    name: "finalTotalHitPoints",
                    ty: "number",
                },
                Field {
                    name: "observedHitPointDelta",
                    ty: "number",
                },
                Field {
                    name: "auditEntryCount",
                    ty: "number",
                },
                Field {
                    name: "combatLogEntryCount",
                    ty: "number",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentTrialReadoutDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "scenarioId",
                    ty: "string",
                },
                Field {
                    name: "rulesetId",
                    ty: "string",
                },
                Field {
                    name: "rulesetVersion",
                    ty: "string",
                },
                Field {
                    name: "contentPackId",
                    ty: "string | null",
                },
                Field {
                    name: "contentPackVersion",
                    ty: "string | null",
                },
                Field {
                    name: "policyId",
                    ty: "string",
                },
                Field {
                    name: "policyVersion",
                    ty: "number",
                },
                Field {
                    name: "policyNoCandidateBehavior",
                    ty: "string",
                },
                Field {
                    name: "seed",
                    ty: "number",
                },
                Field {
                    name: "maxSteps",
                    ty: "number",
                },
                Field {
                    name: "accepted",
                    ty: "boolean",
                },
                Field {
                    name: "stopReason",
                    ty: "string",
                },
                Field {
                    name: "finalizationOutcome",
                    ty: "string | null",
                },
                Field {
                    name: "initialStateFingerprint",
                    ty: "string",
                },
                Field {
                    name: "finalStateFingerprint",
                    ty: "string",
                },
                Field {
                    name: "materializedRolls",
                    ty: "readonly number[]",
                },
                Field {
                    name: "decisions",
                    ty: "readonly RulebenchExperimentDecisionEvidenceDto[]",
                },
                Field {
                    name: "metrics",
                    ty: "RulebenchExperimentMetricsDto",
                },
                Field {
                    name: "replayPackageId",
                    ty: "string",
                },
                Field {
                    name: "replayVerified",
                    ty: "boolean",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentReadoutDto",
            fields: &[
                Field {
                    name: "id",
                    ty: "string",
                },
                Field {
                    name: "status",
                    ty: "string",
                },
                Field {
                    name: "plannedTrialCount",
                    ty: "number",
                },
                Field {
                    name: "completedTrialCount",
                    ty: "number",
                },
                Field {
                    name: "maxStepsPerTrial",
                    ty: "number",
                },
                Field {
                    name: "trials",
                    ty: "readonly RulebenchExperimentTrialReadoutDto[]",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentComparisonRequestDto",
            fields: &[
                Field {
                    name: "expectedExperimentId",
                    ty: "string",
                },
                Field {
                    name: "expectedTrialId",
                    ty: "string",
                },
                Field {
                    name: "actualExperimentId",
                    ty: "string",
                },
                Field {
                    name: "actualTrialId",
                    ty: "string",
                },
            ],
        },
        Interface {
            name: "RulebenchExperimentComparisonReadoutDto",
            fields: &[
                Field {
                    name: "identical",
                    ty: "boolean",
                },
                Field {
                    name: "firstDivergenceIndex",
                    ty: "number | null",
                },
                Field {
                    name: "expectedTrialId",
                    ty: "string",
                },
                Field {
                    name: "actualTrialId",
                    ty: "string",
                },
                Field {
                    name: "expectedEvidence",
                    ty: "RulebenchExperimentDecisionEvidenceDto | null",
                },
                Field {
                    name: "actualEvidence",
                    ty: "RulebenchExperimentDecisionEvidenceDto | null",
                },
                Field {
                    name: "reason",
                    ty: "string",
                },
            ],
        },
    ]
}
