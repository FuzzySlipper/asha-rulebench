use super::{ProtocolField, ProtocolInterface};
use std::sync::OnceLock;

type Field = ProtocolField;
type Interface = ProtocolInterface;

pub fn live_interfaces() -> &'static [ProtocolInterface] {
    static INTERFACES: OnceLock<Vec<ProtocolInterface>> = OnceLock::new();
    INTERFACES.get_or_init(|| {
        vec![
            Interface {
                name: "RulebenchLiveTransportErrorDto",
                fields: fields(vec![
                    field("kind", "string"),
                    field("code", "string"),
                    field("message", "string"),
                    field("retryable", "boolean"),
                ]),
            },
            Interface {
                name: "RulebenchLiveStateFingerprintDto",
                fields: fields(vec![field("algorithm", "string"), field("value", "string")]),
            },
            Interface {
                name: "RulebenchLiveGridPositionDto",
                fields: fields(vec![field("x", "number"), field("y", "number")]),
            },
            Interface {
                name: "RulebenchLiveBoardCellDto",
                fields: fields(vec![
                    field("position", "RulebenchLiveGridPositionDto"),
                    field("terrainTags", "readonly string[]"),
                    field("blocksMovement", "boolean"),
                    field("occupantIds", "readonly string[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveBoardDto",
                fields: fields(vec![
                    field("id", "string"),
                    field("width", "number"),
                    field("height", "number"),
                    field("cells", "readonly RulebenchLiveBoardCellDto[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveParticipantDto",
                fields: fields(vec![
                    field("id", "string"),
                    field("name", "string"),
                    field("currentHitPoints", "number"),
                    field("maxHitPoints", "number"),
                    field("temporaryVitality", "number"),
                    field("defeated", "boolean"),
                    field("conditions", "readonly string[]"),
                    field("position", "RulebenchLiveGridPositionDto"),
                    field("movementRemaining", "number"),
                    field("movementMaximum", "number"),
                ]),
            },
            Interface {
                name: "RulebenchLiveActionResourceCostDto",
                fields: fields(vec![
                    field("resourceId", "string"),
                    field("amount", "number"),
                ]),
            },
            Interface {
                name: "RulebenchLiveActionResourceStateDto",
                fields: fields(vec![
                    field("resourceId", "string"),
                    field("sourceId", "string"),
                    field("kind", "string"),
                    field("current", "number"),
                    field("max", "number"),
                    field("available", "boolean"),
                    field("refreshPolicy", "string"),
                    field("refreshTurns", "number | null"),
                    field("remainingRefreshTurns", "number | null"),
                ]),
            },
            Interface {
                name: "RulebenchLiveTargetOptionDto",
                fields: fields(vec![
                    field("targetId", "string"),
                    field("targetName", "string"),
                    field("currentHitPoints", "number"),
                    field("maxHitPoints", "number"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCellOptionDto",
                fields: fields(vec![
                    field("position", "RulebenchLiveGridPositionDto"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveActionOptionDto",
                fields: fields(vec![
                    field("actionId", "string"),
                    field("abilityId", "string"),
                    field("actionName", "string"),
                    field("available", "boolean"),
                    field("unavailableReason", "string | null"),
                    field(
                        "resourceCosts",
                        "readonly RulebenchLiveActionResourceCostDto[]",
                    ),
                    field(
                        "resourceStates",
                        "readonly RulebenchLiveActionResourceStateDto[]",
                    ),
                    field("targetMode", "'self' | 'entity' | 'cell'"),
                    field("targets", "readonly RulebenchLiveTargetOptionDto[]"),
                    field("destinations", "readonly RulebenchLiveCellOptionDto[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCurrentActorOptionsDto",
                fields: fields(vec![
                    field("roundNumber", "number"),
                    field("turnIndex", "number"),
                    field("lifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("currentActorId", "string | null"),
                    field("currentActorDefeated", "boolean"),
                    field("available", "boolean"),
                    field("unavailableReason", "string | null"),
                    field("actions", "readonly RulebenchLiveActionOptionDto[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCombatLogEntryDto",
                fields: fields(vec![
                    field("id", "string"),
                    field("stepId", "string"),
                    field("logIndex", "number"),
                    field("title", "string"),
                    field("summary", "string"),
                    field("outcomeClass", "RulebenchCommandOutcomeClassDto"),
                    field("eventTypes", "readonly string[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveAuditEntryDto",
                fields: fields(vec![
                    field("id", "string"),
                    field("stepId", "string"),
                    field("sequence", "number"),
                    field("outcomeClass", "RulebenchCommandOutcomeClassDto"),
                    field("decisionKind", "RulebenchCommandDecisionKindDto"),
                    field(
                        "preflightDecisionKind",
                        "RulebenchCommandPreflightDecisionKindDto | null",
                    ),
                    field("accepted", "boolean"),
                    field("rejectionCode", "RulebenchRejectionCodeDto | null"),
                    field("eventCount", "number"),
                    field("traceCount", "number"),
                    field("stateBeforeFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("stateAfterFingerprint", "RulebenchLiveStateFingerprintDto"),
                ]),
            },
            Interface {
                name: "RulebenchLiveGeneratedRollDto",
                fields: fields(vec![
                    field("sequence", "number"),
                    field("commandId", "string"),
                    field("requestKind", "RulebenchRollRequestKindDto"),
                    field("dieExpression", "string"),
                    field("value", "number"),
                    field("sourceMode", "\"authorityGenerated\""),
                ]),
            },
            Interface {
                name: "RulebenchLiveCombatEndDto",
                fields: fields(vec![
                    field("shouldEnd", "boolean"),
                    field("conditionKind", "RulebenchCombatEndConditionKindDto"),
                    field("outcomeKind", "RulebenchCombatOutcomeKindDto"),
                    field("activeSides", "readonly RulebenchCombatSideIdDto[]"),
                    field("defeatedSides", "readonly RulebenchCombatSideIdDto[]"),
                    field("winningSides", "readonly RulebenchCombatSideIdDto[]"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveFinalizationDto",
                fields: fields(vec![
                    field("trigger", "RulebenchLifecycleTransitionTriggerDto"),
                    field("finalizedAtStep", "number"),
                    field("outcomeKind", "RulebenchCombatOutcomeKindDto"),
                    field("winningSides", "readonly RulebenchCombatSideIdDto[]"),
                    field("remainingSides", "readonly RulebenchCombatSideIdDto[]"),
                    field("finalStateFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveSessionSnapshotDto",
                fields: fields(vec![
                    field("sessionId", "string"),
                    field("nextStepIndex", "number"),
                    field("lifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("startedAtStep", "number | null"),
                    field("endedAtStep", "number | null"),
                    field("roundNumber", "number"),
                    field("turnIndex", "number"),
                    field("participantOrder", "readonly string[]"),
                    field("currentActorId", "string | null"),
                    field("participants", "readonly RulebenchLiveParticipantDto[]"),
                    field("board", "RulebenchLiveBoardDto"),
                    field("options", "RulebenchLiveCurrentActorOptionsDto"),
                    field("combatEnd", "RulebenchLiveCombatEndDto"),
                    field("finalization", "RulebenchLiveFinalizationDto | null"),
                    field("combatLog", "readonly RulebenchLiveCombatLogEntryDto[]"),
                    field("auditLog", "readonly RulebenchLiveAuditEntryDto[]"),
                    field("stateFingerprint", "RulebenchLiveStateFingerprintDto"),
                ]),
            },
            Interface {
                name: "RulebenchLiveRollEvidenceDto",
                fields: fields(vec![
                    field("sequence", "number"),
                    field("requestKind", "string"),
                    field("suppliedValue", "number | null"),
                    field("consumed", "boolean"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveTraceEntryDto",
                fields: fields(vec![
                    field("sequence", "number"),
                    field("phase", "RulebenchTracePhaseDto"),
                    field("status", "RulebenchTraceStatusDto"),
                    field("message", "string"),
                    field("detail", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveDomainEventDto",
                fields: fields(vec![field("kind", "string"), field("summary", "string")]),
            },
            Interface {
                name: "RulebenchLivePreflightDto",
                fields: fields(vec![
                    field("intent", "RulebenchUseActionIntentDto"),
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchCommandPreflightDecisionKindDto"),
                    field("rejectionCode", "RulebenchRejectionCodeDto | null"),
                    field("currentActorId", "string | null"),
                    field("targetId", "string | null"),
                    field("targetAccepted", "boolean | null"),
                    field("targetReason", "string | null"),
                    field(
                        "resourceCosts",
                        "readonly RulebenchLiveActionResourceCostDto[]",
                    ),
                    field(
                        "actionResource",
                        "RulebenchLiveActionResourceStateDto | null",
                    ),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCandidateDto",
                fields: fields(vec![
                    field("intent", "RulebenchUseActionIntentDto"),
                    field("abilityId", "string"),
                    field("targetName", "string"),
                    field("targetCurrentHitPoints", "number"),
                    field("targetMaxHitPoints", "number"),
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchCommandPreflightDecisionKindDto"),
                    field("rejectionCode", "RulebenchRejectionCodeDto | null"),
                    field("reason", "string"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCandidateSummaryDto",
                fields: fields(vec![
                    field("roundNumber", "number"),
                    field("turnIndex", "number"),
                    field("lifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("currentActorId", "string | null"),
                    field("available", "boolean"),
                    field("unavailableReason", "string | null"),
                    field("candidates", "readonly RulebenchLiveCandidateDto[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCommandStepDto",
                fields: fields(vec![
                    field("sessionId", "string"),
                    field("stepId", "string"),
                    field("stepIndex", "number"),
                    field("title", "string"),
                    field("summary", "string"),
                    field("outcomeClass", "RulebenchCommandOutcomeClassDto"),
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchCommandDecisionKindDto"),
                    field("rejectionCode", "RulebenchRejectionCodeDto | null"),
                    field("intent", "RulebenchUseActionIntentDto"),
                    field("rolls", "readonly RulebenchLiveRollEvidenceDto[]"),
                    field("events", "readonly RulebenchLiveDomainEventDto[]"),
                    field("trace", "readonly RulebenchLiveTraceEntryDto[]"),
                    field("stateBeforeFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("stateAfterFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("rollMode", "\"supplied\" | \"authorityGenerated\""),
                    field("generatedRolls", "readonly RulebenchLiveGeneratedRollDto[]"),
                ]),
            },
            Interface {
                name: "RulebenchLiveCommandExecutionDto",
                fields: fields(vec![
                    field("step", "RulebenchLiveCommandStepDto"),
                    field("snapshot", "RulebenchLiveSessionSnapshotDto"),
                ]),
            },
            Interface {
                name: "RulebenchLiveControlExecutionDto",
                fields: fields(vec![
                    field("commandKind", "RulebenchCombatControlCommandKindDto"),
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchCombatControlDecisionKindDto"),
                    field("previousLifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("nextLifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("stateBeforeFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("stateAfterFingerprint", "RulebenchLiveStateFingerprintDto"),
                    field("reason", "string"),
                    field("snapshot", "RulebenchLiveSessionSnapshotDto"),
                ]),
            },
            Interface {
                name: "RulebenchLiveAutomaticStepDto",
                fields: fields(vec![
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchAutomaticStepDecisionKindDto"),
                    field(
                        "operationKind",
                        "RulebenchAutomaticStepOperationKindDto | null",
                    ),
                    field("lifecyclePhase", "RulebenchCombatLifecyclePhaseDto"),
                    field("currentActorId", "string | null"),
                    field("policyId", "string"),
                    field("policyVersion", "number"),
                    field("selectedActionId", "string | null"),
                    field("selectedTargetId", "string | null"),
                    field("candidateCount", "number"),
                    field("acceptedCandidateCount", "number"),
                    field("submittedStep", "RulebenchLiveCommandStepDto | null"),
                    field("reason", "string"),
                    field("snapshot", "RulebenchLiveSessionSnapshotDto | null"),
                ]),
            },
            Interface {
                name: "RulebenchLiveAutomaticRunDto",
                fields: fields(vec![
                    field("id", "string"),
                    field("title", "string"),
                    field("summary", "string"),
                    field("accepted", "boolean"),
                    field("decisionKind", "RulebenchAutomaticRunDecisionKindDto"),
                    field("maxSteps", "number"),
                    field("executedStepCount", "number"),
                    field("policyId", "string"),
                    field("policyVersion", "number"),
                    field("steps", "readonly RulebenchLiveAutomaticStepDto[]"),
                    field("finalSnapshot", "RulebenchLiveSessionSnapshotDto"),
                    field("reason", "string"),
                ]),
            },
        ]
    })
}

const fn field(name: &'static str, ty: &'static str) -> ProtocolField {
    Field { name, ty }
}

fn fields(values: Vec<ProtocolField>) -> &'static [ProtocolField] {
    Box::leak(values.into_boxed_slice())
}
