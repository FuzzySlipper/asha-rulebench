import type {
  RulebenchLiveAutomaticRunDto,
  RulebenchLiveAutomaticStepDto,
  RulebenchLiveCandidateSummaryDto,
  RulebenchLiveCommandExecutionDto,
  RulebenchLiveCurrentActorOptionsDto,
  RulebenchLiveGameplayFabricDto,
  RulebenchLivePreflightDto,
  RulebenchLiveReactionExecutionDto,
  RulebenchLiveSessionSnapshotDto,
} from "@asha-rulebench/protocol";

export interface RulebenchLiveAutomaticStepView {
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly operationLabel: string | null;
  readonly policyLabel: string;
  readonly selectedActionId: string | null;
  readonly selectedTargetId: string | null;
  readonly candidateCount: number;
  readonly acceptedCandidateCount: number;
  readonly reason: string;
}

export interface RulebenchLiveAutomaticRunView {
  readonly id: string;
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly policyLabel: string;
  readonly maxSteps: number;
  readonly executedStepCount: number;
  readonly reason: string;
  readonly steps: readonly RulebenchLiveAutomaticStepView[];
  readonly finalLifecycleLabel: string;
  readonly finalFingerprintLabel: string;
}

export interface RulebenchLiveSessionView {
  readonly sessionId: string;
  readonly lifecycleLabel: string;
  readonly roundLabel: string;
  readonly turnLabel: string;
  readonly currentActorId: string | null;
  readonly fingerprintLabel: string;
  readonly actionResourceFingerprintLabel: string;
  readonly participantOrderIds: readonly string[];
  readonly participants: readonly RulebenchLiveParticipantView[];
  readonly board: RulebenchLiveBoardView;
  readonly options: RulebenchLiveOptionsView;
  readonly combatEndLabel: string;
  readonly finalizationLabel: string | null;
  readonly combatLog: readonly RulebenchLiveLogEntryView[];
  readonly auditLog: readonly RulebenchLiveAuditEntryView[];
  readonly gameplayFabric: RulebenchLiveGameplayFabricDto;
  readonly reactionWindow: RulebenchLiveReactionWindowView | null;
  readonly reactionLifecycleLabels: readonly string[];
  readonly reactionAuditLabels: readonly string[];
}

export interface RulebenchLiveReactionWindowView {
  readonly id: string;
  readonly hookId: string;
  readonly timingLabel: string;
  readonly depthLabel: string;
  readonly currentReactorId: string | null;
  readonly options: readonly RulebenchLiveReactionOptionView[];
  readonly responseLabels: readonly string[];
}

export interface RulebenchLiveReactionOptionView {
  readonly id: string;
  readonly reactorId: string;
  readonly opensNestedWindow: boolean;
  readonly label: string;
}

export interface RulebenchLiveReactionExecutionView {
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly resumedPendingResolution: boolean;
  readonly reason: string;
  readonly traceLabels: readonly string[];
}

export interface RulebenchLiveParticipantView {
  readonly id: string;
  readonly name: string;
  readonly hitPointLabel: string;
  readonly temporaryVitalityLabel: string | null;
  readonly statusLabel: string;
  readonly conditionLabels: readonly string[];
  readonly position: Readonly<{ x: number; y: number }>;
  readonly coordinateLabel: string;
  readonly movementLabel: string;
}

export interface RulebenchLiveBoardView {
  readonly id: string;
  readonly width: number;
  readonly height: number;
  readonly cells: readonly RulebenchLiveBoardCellView[];
}

export interface RulebenchLiveBoardCellView {
  readonly x: number;
  readonly y: number;
  readonly terrainLabels: readonly string[];
  readonly blocksMovement: boolean;
  readonly occupantIds: readonly string[];
  readonly coordinateLabel: string;
  readonly occupied: boolean;
}

export interface RulebenchLiveOptionsView {
  readonly available: boolean;
  readonly unavailableReason: string | null;
  readonly currentActorId: string | null;
  readonly actions: readonly RulebenchLiveActionOptionView[];
}

export interface RulebenchLiveActionOptionView {
  readonly actionId: string;
  readonly abilityId: string;
  readonly name: string;
  readonly checkKind: "attackVsDefense" | "savingThrow" | "contested";
  readonly available: boolean;
  readonly unavailableReason: string | null;
  readonly resourceCostLabels: readonly string[];
  readonly resourceLabels: readonly string[];
  readonly targetMode: "self" | "entity" | "cell";
  readonly targets: readonly RulebenchLiveTargetOptionView[];
  readonly targetSets: readonly RulebenchLiveTargetSetOptionView[];
  readonly destinations: readonly RulebenchLiveCellOptionView[];
}

export interface RulebenchLiveTargetSetOptionView {
  readonly id: string;
  readonly targetIds: readonly string[];
  readonly targetCell: Readonly<{ x: number; y: number }> | null;
  readonly rollPolicyLabel: string;
  readonly reason: string;
}

export interface RulebenchLiveTargetOptionView {
  readonly id: string;
  readonly name: string;
  readonly hitPointLabel: string;
  readonly reason: string;
}

export interface RulebenchLiveCellOptionView {
  readonly x: number;
  readonly y: number;
  readonly reason: string;
}

export interface RulebenchLiveLogEntryView {
  readonly id: string;
  readonly stepId: string;
  readonly sequenceLabel: string;
  readonly title: string;
  readonly summary: string;
  readonly outcomeLabel: string;
  readonly eventTypeLabels: readonly string[];
}

export interface RulebenchLiveAuditEntryView {
  readonly id: string;
  readonly stepId: string;
  readonly sequenceLabel: string;
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly rejectionLabel: string | null;
  readonly eventCount: number;
  readonly traceCount: number;
  readonly stateChanged: boolean;
}

export interface RulebenchLiveCandidateView {
  readonly actorId: string;
  readonly actionId: string;
  readonly targetId: string;
  readonly targetName: string;
  readonly targetHitPointLabel: string;
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly rejectionLabel: string | null;
  readonly reason: string;
}

export interface RulebenchLiveCandidateSummaryView {
  readonly available: boolean;
  readonly unavailableReason: string | null;
  readonly currentActorId: string | null;
  readonly candidates: readonly RulebenchLiveCandidateView[];
}

export interface RulebenchLivePreflightView {
  readonly accepted: boolean;
  readonly actorId: string;
  readonly actionId: string;
  readonly targetId: string;
  readonly decisionLabel: string;
  readonly rejectionLabel: string | null;
  readonly targetAccepted: boolean | null;
  readonly targetReason: string | null;
  readonly reason: string;
}

export interface RulebenchLiveCommandExecutionView {
  readonly accepted: boolean;
  readonly stepId: string;
  readonly title: string;
  readonly summary: string;
  readonly decisionLabel: string;
  readonly rejectionLabel: string | null;
  readonly eventLabels: readonly string[];
  readonly traceLabels: readonly string[];
  readonly targetResults: readonly RulebenchLiveTargetResolutionView[];
  readonly stateChanged: boolean;
  readonly rollModeLabel: string;
  readonly generatedRolls: readonly RulebenchLiveGeneratedRollView[];
}

export interface RulebenchLiveTargetResolutionView {
  readonly targetId: string;
  readonly accepted: boolean;
  readonly outcomeLabel: string;
  readonly damageLabel: string | null;
  readonly movementLabel: string | null;
  readonly resourceLabels: readonly string[];
  readonly reason: string;
}

export interface RulebenchLiveGeneratedRollView {
  readonly sequenceLabel: string;
  readonly purposeLabel: string;
  readonly dieExpression: string;
  readonly valueLabel: string;
  readonly sourceLabel: string;
}

export function projectLiveSessionSnapshot(
  snapshot: RulebenchLiveSessionSnapshotDto,
): RulebenchLiveSessionView {
  return {
    sessionId: snapshot.sessionId,
    lifecycleLabel: labelCode(snapshot.lifecyclePhase),
    roundLabel: String(snapshot.roundNumber),
    turnLabel: String(snapshot.turnIndex + 1),
    currentActorId: snapshot.currentActorId,
    fingerprintLabel: `${snapshot.stateFingerprint.algorithm}:${snapshot.stateFingerprint.value}`,
    actionResourceFingerprintLabel: `${snapshot.actionResourceFingerprint.algorithm}:${snapshot.actionResourceFingerprint.value}`,
    participantOrderIds: snapshot.participantOrder,
    participants: snapshot.participants.map((participant) => ({
      id: participant.id,
      name: participant.name,
      hitPointLabel: `${participant.currentHitPoints}/${participant.maxHitPoints} HP`,
      temporaryVitalityLabel:
        participant.temporaryVitality === 0
          ? null
          : `${participant.temporaryVitality} temporary vitality`,
      statusLabel: participant.defeated ? "Defeated" : "Active",
      conditionLabels: participant.conditions,
      position: participant.position,
      coordinateLabel: `${participant.position.x},${participant.position.y}`,
      movementLabel: `${participant.movementRemaining}/${participant.movementMaximum}`,
    })),
    board: {
      id: snapshot.board.id,
      width: snapshot.board.width,
      height: snapshot.board.height,
      cells: snapshot.board.cells.map((cell) => ({
        x: cell.position.x,
        y: cell.position.y,
        terrainLabels: cell.terrainTags,
        blocksMovement: cell.blocksMovement,
        occupantIds: cell.occupantIds,
        coordinateLabel: `${cell.position.x},${cell.position.y}`,
        occupied: cell.occupantIds.length > 0,
      })),
    },
    options: projectLiveOptions(snapshot.options),
    combatEndLabel: snapshot.combatEnd.reason,
    finalizationLabel: snapshot.finalization?.reason ?? null,
    combatLog: snapshot.combatLog.map((entry) => ({
      id: entry.id,
      stepId: entry.stepId,
      sequenceLabel: String(entry.logIndex),
      title: entry.title,
      summary: entry.summary,
      outcomeLabel: labelCode(entry.outcomeClass),
      eventTypeLabels: entry.eventTypes.map(labelCode),
    })),
    auditLog: snapshot.auditLog.map((entry) => ({
      id: entry.id,
      stepId: entry.stepId,
      sequenceLabel: String(entry.sequence),
      accepted: entry.accepted,
      decisionLabel: labelCode(entry.decisionKind),
      rejectionLabel:
        entry.rejectionCode === null ? null : labelCode(entry.rejectionCode),
      eventCount: entry.eventCount,
      traceCount: entry.traceCount,
      stateChanged:
        entry.stateBeforeFingerprint.value !==
        entry.stateAfterFingerprint.value,
    })),
    gameplayFabric: snapshot.gameplayFabric,
    reactionWindow:
      snapshot.currentReactionWindow === null
        ? null
        : {
            id: snapshot.currentReactionWindow.id,
            hookId: snapshot.currentReactionWindow.hookId,
            timingLabel: labelCode(snapshot.currentReactionWindow.timing),
            depthLabel: `${snapshot.currentReactionWindow.depth}/${snapshot.currentReactionWindow.maximumNestedDepth}`,
            currentReactorId: snapshot.currentReactionWindow.currentReactorId,
            options: snapshot.currentReactionWindow.options.map((option) => ({
              id: option.optionId,
              reactorId: option.reactorId,
              opensNestedWindow: option.opensNestedWindow,
              label: `${labelCode(option.optionId)}${option.opensNestedWindow ? " (opens nested window)" : ""}`,
            })),
            responseLabels: snapshot.currentReactionWindow.responses.map(
              (response) =>
                `${response.reactorId}: ${labelCode(response.responseKind)}${response.optionId === null ? "" : ` ${labelCode(response.optionId)}`}`,
            ),
          },
    reactionLifecycleLabels: snapshot.reactionWindowLifecycleLog.map(
      (entry) => `${labelCode(entry.lifecycleKind)}: ${entry.reason}`,
    ),
    reactionAuditLabels: snapshot.reactionAuditLog.map(
      (entry) => `${entry.reactorId}: ${labelCode(entry.decisionKind)} · ${entry.reason}`,
    ),
  };
}

export function projectLiveReactionExecution(
  execution: RulebenchLiveReactionExecutionDto,
): RulebenchLiveReactionExecutionView {
  return {
    accepted: execution.reaction.accepted,
    decisionLabel: labelCode(execution.reaction.decisionKind),
    resumedPendingResolution: execution.reaction.resumedPendingResolution,
    reason: execution.reaction.reason,
    traceLabels: execution.reaction.trace.map((entry) => entry.message),
  };
}

export function projectLiveOptions(
  options: RulebenchLiveCurrentActorOptionsDto,
): RulebenchLiveOptionsView {
  return {
    available: options.available,
    unavailableReason: options.unavailableReason,
    currentActorId: options.currentActorId,
    actions: options.actions.map((action) => ({
      actionId: action.actionId,
      abilityId: action.abilityId,
      name: action.actionName,
      checkKind: action.checkKind,
      available: action.available,
      unavailableReason: action.unavailableReason,
      resourceCostLabels: action.resourceCosts.map(
        (cost) => `${cost.resourceId} costs ${cost.amount}`,
      ),
      resourceLabels: action.resourceStates.map(
        (resource) =>
          `${resource.resourceId} ${resource.current}/${resource.max}`,
      ),
      targetMode: action.targetMode,
      targets: action.targets.map((target) => ({
        id: target.targetId,
        name: target.targetName,
        hitPointLabel: `${target.currentHitPoints}/${target.maxHitPoints} HP`,
        reason: target.reason,
      })),
      targetSets: action.targetSets.map((targetSet) => ({
        id: targetSet.id,
        targetIds: targetSet.targetIds,
        targetCell: targetSet.targetCell,
        rollPolicyLabel: labelCode(targetSet.rollPolicy),
        reason: targetSet.reason,
      })),
      destinations: action.destinations.map((destination) => ({
        x: destination.position.x,
        y: destination.position.y,
        reason: destination.reason,
      })),
    })),
  };
}

export function projectLiveCandidates(
  summary: RulebenchLiveCandidateSummaryDto,
): RulebenchLiveCandidateSummaryView {
  return {
    available: summary.available,
    unavailableReason: summary.unavailableReason,
    currentActorId: summary.currentActorId,
    candidates: summary.candidates.map((candidate) => ({
      actorId: candidate.intent.actorId,
      actionId: candidate.intent.actionId,
      targetId: candidate.intent.targetId,
      targetName: candidate.targetName,
      targetHitPointLabel: `${candidate.targetCurrentHitPoints}/${candidate.targetMaxHitPoints} HP`,
      accepted: candidate.accepted,
      decisionLabel: labelCode(candidate.decisionKind),
      rejectionLabel:
        candidate.rejectionCode === null
          ? null
          : labelCode(candidate.rejectionCode),
      reason: candidate.reason,
    })),
  };
}

export function projectLivePreflight(
  preflight: RulebenchLivePreflightDto,
): RulebenchLivePreflightView {
  return {
    accepted: preflight.accepted,
    actorId: preflight.intent.actorId,
    actionId: preflight.intent.actionId,
    targetId: preflight.intent.targetId,
    decisionLabel: labelCode(preflight.decisionKind),
    rejectionLabel:
      preflight.rejectionCode === null
        ? null
        : labelCode(preflight.rejectionCode),
    targetAccepted: preflight.targetAccepted,
    targetReason: preflight.targetReason,
    reason: preflight.reason,
  };
}

export function projectLiveCommandExecution(
  execution: RulebenchLiveCommandExecutionDto,
): RulebenchLiveCommandExecutionView {
  return {
    accepted: execution.step.accepted,
    stepId: execution.step.stepId,
    title: execution.step.title,
    summary: execution.step.summary,
    decisionLabel: labelCode(execution.step.decisionKind),
    rejectionLabel:
      execution.step.rejectionCode === null
        ? null
        : labelCode(execution.step.rejectionCode),
    eventLabels: execution.step.events.map((event) => labelCode(event.kind)),
    traceLabels: execution.step.trace.map((entry) => entry.message),
    targetResults: execution.step.targetResults.map((target) => ({
      targetId: target.targetId,
      accepted: target.accepted,
      outcomeLabel:
        target.attackOutcome === null ? "No roll" : labelCode(target.attackOutcome),
      damageLabel:
        target.damageAmount === null ? null : `${target.damageAmount} damage`,
      movementLabel:
        target.movementKind === null || target.movementFrom === null || target.movementTo === null
          ? null
          : `${labelCode(target.movementKind)} ${target.movementFrom.x},${target.movementFrom.y} → ${target.movementTo.x},${target.movementTo.y}`,
      resourceLabels: target.resourceChanges.map(
        (resource) =>
          `${resource.resourceId} ${resource.before} → ${resource.after} (${resource.requestedDelta >= 0 ? "+" : ""}${resource.requestedDelta})`,
      ),
      reason: target.reason,
    })),
    stateChanged:
      execution.step.stateBeforeFingerprint.value !==
      execution.step.stateAfterFingerprint.value,
    rollModeLabel: labelCode(execution.step.rollMode),
    generatedRolls: execution.step.generatedRolls.map((roll) => ({
      sequenceLabel: String(roll.sequence + 1),
      purposeLabel: labelCode(roll.requestKind),
      dieExpression: roll.dieExpression,
      valueLabel: String(roll.value),
      sourceLabel: labelCode(roll.sourceMode),
    })),
  };
}

export function projectLiveAutomaticStep(
  step: RulebenchLiveAutomaticStepDto,
): RulebenchLiveAutomaticStepView {
  return {
    accepted: step.accepted,
    decisionLabel: labelCode(step.decisionKind),
    operationLabel:
      step.operationKind === null ? null : labelCode(step.operationKind),
    policyLabel: `${step.policyId} v${step.policyVersion}`,
    selectedActionId: step.selectedActionId,
    selectedTargetId: step.selectedTargetId,
    candidateCount: step.candidateCount,
    acceptedCandidateCount: step.acceptedCandidateCount,
    reason: step.reason,
  };
}

export function projectLiveAutomaticRun(
  run: RulebenchLiveAutomaticRunDto,
): RulebenchLiveAutomaticRunView {
  return {
    id: run.id,
    accepted: run.accepted,
    decisionLabel: labelCode(run.decisionKind),
    policyLabel: `${run.policyId} v${run.policyVersion}`,
    maxSteps: run.maxSteps,
    executedStepCount: run.executedStepCount,
    reason: run.reason,
    steps: run.steps.map(projectLiveAutomaticStep),
    finalLifecycleLabel: labelCode(run.finalSnapshot.lifecyclePhase),
    finalFingerprintLabel: `${run.finalSnapshot.stateFingerprint.algorithm}:${run.finalSnapshot.stateFingerprint.value}`,
  };
}

function labelCode(value: string): string {
  return value
    .replace(/[-_.]+/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .split(" ")
    .map((word) => word.replace(/^./, (letter) => letter.toUpperCase()))
    .join(" ");
}
