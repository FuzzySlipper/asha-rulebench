use super::{ProtocolField, ProtocolInterface};

type Field = ProtocolField;
type Interface = ProtocolInterface;

pub fn interfaces() -> &'static [ProtocolInterface] {
    &[
        Interface {
            name: "RulebenchCombatSessionHandleDto",
            fields: &[Field {
                name: "id",
                ty: "string",
            }],
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
                    name: "executedStepCount",
                    ty: "number",
                },
                Field {
                    name: "stepDecisions",
                    ty: "readonly RulebenchAutomaticStepDecisionDto[]",
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
                    name: "targetOptions",
                    ty: "readonly RulebenchCurrentActorTargetOptionDto[]",
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
                    name: "combatShouldEnd",
                    ty: "boolean",
                },
                Field {
                    name: "conditionKind",
                    ty: "RulebenchCombatEndConditionKindDto",
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
            name: "RulebenchActiveModifierDto",
            fields: &[
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
                Field {
                    name: "tenure",
                    ty: "RulebenchModifierTenureDto",
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
                    name: "turnTransitionSequence",
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
                    name: "resourceKind",
                    ty: "RulebenchActionResourceKindDto",
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
                    ty: "RulebenchAttackSpecDto",
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
    ]
}
