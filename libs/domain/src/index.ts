export * from './live-combat';
export * from './replay-review';
export * from './content-workspace';

import type {
  RulebenchCombatLogEntryDto,
  RulebenchCombatSessionStepReadoutDto,
  RulebenchCombatantDto,
  RulebenchDomainEventDto,
  RulebenchFinalStateDto,
  RulebenchScenarioReadoutDto,
  RulebenchTraceEntryDto,
  RulebenchContentImportDiagnosticDto,
  RulebenchContentImportReadoutDto,
  RulebenchContentValidationReadoutDto,
  RulebenchViewerFinalStateDto,
  RulebenchViewerScenarioReadoutDto,
  RulebenchViewerSessionStepReadoutDto,
} from '@asha-rulebench/protocol';

export interface RulebenchContentValidationView {
  readonly scenarioId: string;
  readonly scenarioTitle: string;
  readonly statusLabel: string;
  readonly errorCount: number;
  readonly warningCount: number;
  readonly diagnostics: readonly RulebenchContentValidationDiagnosticView[];
}

export interface RulebenchContentValidationDiagnosticView {
  readonly severityLabel: string;
  readonly code: string;
  readonly sourceLabel: string;
  readonly message: string;
}

export function projectContentValidationReadout(
  readout: RulebenchContentValidationReadoutDto,
): RulebenchContentValidationView {
  return {
    scenarioId: readout.scenarioId,
    scenarioTitle: readout.scenarioTitle,
    statusLabel: readout.report.accepted ? 'Accepted' : 'Rejected',
    errorCount: readout.report.errorCount,
    warningCount: readout.report.warningCount,
    diagnostics: readout.report.diagnostics.map((diagnostic) => ({
      severityLabel: diagnostic.severity === 'error' ? 'Error' : 'Warning',
      code: diagnostic.code,
      sourceLabel: diagnostic.contentId ?? readout.scenarioId,
      message: diagnostic.message,
    })),
  };
}

export interface RulebenchContentImportView {
  readonly exampleId: string;
  readonly packLabel: string;
  readonly fingerprintLabel: string;
  readonly statusLabel: string;
  readonly errorCount: number;
  readonly warningCount: number;
  readonly diagnostics: readonly RulebenchContentImportDiagnosticView[];
}

export interface RulebenchContentImportDiagnosticView {
  readonly severityLabel: string;
  readonly code: string;
  readonly locationLabel: string;
  readonly message: string;
}

export function projectContentImportReadout(readout: RulebenchContentImportReadoutDto): RulebenchContentImportView {
  return {
    exampleId: readout.exampleId,
    packLabel: `${readout.pack.id}@${readout.pack.version}`,
    fingerprintLabel:
      readout.pack.fingerprint === null
        ? 'Not accepted'
        : `${readout.pack.fingerprint.algorithm}:${readout.pack.fingerprint.value}`,
    statusLabel: readout.accepted ? 'Accepted' : 'Rejected',
    errorCount: readout.errorCount,
    warningCount: readout.warningCount,
    diagnostics: readout.diagnostics.map(projectContentImportDiagnostic),
  };
}

function projectContentImportDiagnostic(
  diagnostic: RulebenchContentImportDiagnosticDto,
): RulebenchContentImportDiagnosticView {
  const referenceLabel = diagnostic.referenceId === null ? '' : ` / ${diagnostic.referenceId}`;
  const definitionLabel = diagnostic.definitionKind === null ? '' : ` / ${diagnostic.definitionKind}`;
  return {
    severityLabel: diagnostic.severity === 'error' ? 'Error' : 'Warning',
    code: diagnostic.code,
    locationLabel: `${diagnostic.path}${definitionLabel}${referenceLabel}`,
    message: diagnostic.message,
  };
}

export interface RulebenchScenarioView {
  readonly title: string;
  readonly summary: string;
  readonly seedLabel: string;
  readonly board: RulebenchBoardView;
  readonly combatants: readonly RulebenchCombatantView[];
  readonly selectedAction: RulebenchSelectedActionView;
  readonly selectedTarget: RulebenchSelectedTargetView;
  readonly timeline: readonly RulebenchTimelineRowView[];
  readonly traceGroups: readonly RulebenchTraceGroupView[];
  readonly finalState: RulebenchFinalStateView;
}

export interface RulebenchBoardView {
  readonly width: number;
  readonly height: number;
  readonly cells: readonly RulebenchBoardCellView[];
}

export interface RulebenchBoardCellView {
  readonly x: number;
  readonly y: number;
  readonly terrainLabel: string;
  readonly occupantIds: readonly string[];
}

export interface RulebenchCombatantView {
  readonly id: string;
  readonly name: string;
  readonly teamLabel: string;
  readonly positionLabel: string;
  readonly hitPointLabel: string;
  readonly defenseLabels: readonly string[];
  readonly conditionLabels: readonly string[];
  readonly isActor: boolean;
}

export interface RulebenchSelectedActionView {
  readonly name: string;
  readonly actorLabel: string;
  readonly targetLabels: readonly string[];
  readonly actionText: string;
  readonly effectText: string;
}

export interface RulebenchSelectedTargetView {
  readonly targetLabel: string;
  readonly legalityLabel: string;
  readonly reason: string;
}

export interface RulebenchTimelineRowView {
  readonly sequenceLabel: string;
  readonly typeLabel: string;
  readonly summary: string;
  readonly participantLabels: readonly string[];
}

export interface RulebenchTraceGroupView {
  readonly phaseLabel: string;
  readonly entries: readonly RulebenchTraceEntryView[];
}

export interface RulebenchTraceEntryView {
  readonly sequenceLabel: string;
  readonly statusLabel: string;
  readonly message: string;
  readonly detail: string;
}

export interface RulebenchFinalStateView {
  readonly summary: string;
  readonly combatants: readonly RulebenchFinalCombatantStateView[];
}

export interface RulebenchFinalCombatantStateView {
  readonly id: string;
  readonly name: string;
  readonly hitPointLabel: string;
  readonly conditionLabels: readonly string[];
}

export interface RulebenchCombatSessionStepView {
  readonly sessionId: string;
  readonly step: RulebenchCombatSessionStepSummaryView;
  readonly command: RulebenchCommandAttemptView;
  readonly scenario: RulebenchScenarioView;
  readonly combatLog: readonly RulebenchCombatLogEntryView[];
  readonly stateBefore: RulebenchFinalStateView;
  readonly stateAfter: RulebenchFinalStateView;
}

export interface RulebenchCombatSessionStepSummaryView {
  readonly id: string;
  readonly indexLabel: string;
  readonly title: string;
  readonly summary: string;
  readonly outcomeLabel: string;
  readonly logIndexLabel: string;
}

export interface RulebenchCommandAttemptView {
  readonly stepId: string;
  readonly stepIndexLabel: string;
  readonly actorId: string;
  readonly actionId: string;
  readonly targetId: string;
  readonly rollStreamLabel: string;
  readonly outcomeLabel: string;
}

export interface RulebenchCombatLogEntryView {
  readonly id: string;
  readonly stepId: string;
  readonly logIndexLabel: string;
  readonly title: string;
  readonly summary: string;
  readonly outcomeLabel: string;
  readonly eventTypeLabels: readonly string[];
}

type TracePhase = RulebenchTraceEntryDto['phase'];

const tracePhaseOrder: readonly TracePhase[] = ['proposal', 'validation', 'resolution', 'commit'];

export function projectRulebenchScenario(readout: RulebenchScenarioReadoutDto): RulebenchScenarioView {
  const combatantLabels = createCombatantLabels(readout.combatants);

  return {
    title: readout.title,
    summary: readout.summary,
    seedLabel: readout.seedLabel,
    board: projectBoard(readout),
    combatants: readout.combatants.map(projectCombatant),
    selectedAction: {
      name: readout.selectedAction.name,
      actorLabel: labelForId(combatantLabels, readout.selectedAction.actorId),
      targetLabels: readout.selectedAction.targetIds.map((targetId) => labelForId(combatantLabels, targetId)),
      actionText: readout.selectedAction.actionText,
      effectText: readout.selectedAction.effectText,
    },
    selectedTarget: {
      targetLabel: labelForId(combatantLabels, readout.selectedTarget.targetId),
      legalityLabel: labelLegality(readout.selectedTarget.legality),
      reason: readout.selectedTarget.reason,
    },
    timeline: readout.domainEvents.map((event) => projectTimelineRow(event, combatantLabels)),
    traceGroups: projectTraceGroups(readout.trace),
    finalState: projectFinalState(readout.finalState),
  };
}

export function projectRulebenchCombatSessionStep(
  readout: RulebenchCombatSessionStepReadoutDto,
): RulebenchCombatSessionStepView {
  return {
    sessionId: readout.sessionId,
    step: {
      id: readout.step.id,
      indexLabel: String(readout.step.index + 1),
      title: readout.step.title,
      summary: readout.step.summary,
      outcomeLabel: labelOutcomeClass(readout.step.outcomeClass),
      logIndexLabel: String(readout.step.logIndex),
    },
    command: {
      stepId: readout.command.stepId,
      stepIndexLabel: String(readout.command.stepIndex + 1),
      actorId: readout.command.actorId,
      actionId: readout.command.actionId,
      targetId: readout.command.targetId,
      rollStreamLabel: readout.command.rollStream.join(','),
      outcomeLabel: labelOutcomeClass(readout.command.outcomeClass),
    },
    scenario: projectRulebenchScenario(readout.scenarioReadout),
    combatLog: readout.combatLog.map(projectCombatLogEntry),
    stateBefore: projectFinalState(readout.stateBefore),
    stateAfter: projectFinalState(readout.stateAfter),
  };
}

export function projectRulebenchViewerScenario(
  readout: RulebenchViewerScenarioReadoutDto,
): RulebenchScenarioView {
  const combatantLabels = new Map(
    readout.combatants.map((combatant) => [combatant.id, combatant.name]),
  );
  const selectedTarget = readout.selectedTarget;
  return {
    title: readout.identity.title,
    summary: readout.identity.summary,
    seedLabel: readout.identity.seedLabel,
    board: {
      width: readout.board.width,
      height: readout.board.height,
      cells: readout.board.cells.map((cell) => ({
        x: cell.position.x,
        y: cell.position.y,
        terrainLabel: labelTerrain(cell.terrainTags),
        occupantIds: cell.occupantIds,
      })),
    },
    combatants: readout.combatants.map((combatant) => ({
      id: combatant.id,
      name: combatant.name,
      teamLabel: combatant.team === 'ally' ? 'Ally' : 'Enemy',
      positionLabel: labelPosition(combatant.positionX, combatant.positionY),
      hitPointLabel: labelHitPoints(combatant.currentHitPoints, combatant.maxHitPoints),
      defenseLabels: combatant.defenses.map((defense) => `${defense.label} ${defense.value}`),
      conditionLabels: labelConditions(combatant.conditions),
      isActor: combatant.isActor,
    })),
    selectedAction: {
      name: readout.selectedAction.name,
      actorLabel: labelForId(combatantLabels, readout.selectedAction.actorId),
      targetLabels: readout.selectedAction.targetIds.map((targetId) =>
        labelForId(combatantLabels, targetId),
      ),
      actionText: readout.selectedAction.actionText,
      effectText: readout.selectedAction.effectText,
    },
    selectedTarget: selectedTarget === null
      ? {
          targetLabel: 'No target readback',
          legalityLabel: 'Not resolved',
          reason: 'Authority did not emit a target-legality readback.',
        }
      : {
          targetLabel: labelForId(combatantLabels, selectedTarget.targetId),
          legalityLabel: selectedTarget.accepted ? 'Accepted' : 'Rejected',
          reason: selectedTarget.reason,
        },
    timeline: readout.domainEvents.map((event) => ({
      sequenceLabel: String(event.sequence),
      typeLabel: humanizeIdentifier(event.kind),
      summary: event.summary,
      participantLabels: event.entityIds.map((id) => labelForId(combatantLabels, id)),
    })),
    traceGroups: projectTraceGroups(readout.trace),
    finalState: projectViewerFinalState(readout.finalState),
  };
}

export function projectRulebenchViewerSessionStep(
  readout: RulebenchViewerSessionStepReadoutDto,
): RulebenchCombatSessionStepView {
  return {
    sessionId: readout.sessionId,
    step: {
      id: readout.step.id,
      indexLabel: String(readout.step.index + 1),
      title: readout.step.title,
      summary: readout.step.summary,
      outcomeLabel: labelOutcomeClass(readout.step.outcomeClass),
      logIndexLabel: String(readout.step.logIndex),
    },
    command: {
      stepId: readout.command.stepId,
      stepIndexLabel: String(readout.command.stepIndex + 1),
      actorId: readout.command.actorId,
      actionId: readout.command.actionId,
      targetId: readout.command.targetId,
      rollStreamLabel: readout.command.rollStream.join(','),
      outcomeLabel: labelOutcomeClass(readout.command.outcomeClass),
    },
    scenario: projectRulebenchViewerScenario(readout.scenario),
    combatLog: readout.combatLog.map((entry) => ({
      id: entry.id,
      stepId: entry.stepId,
      logIndexLabel: String(entry.logIndex),
      title: entry.title,
      summary: entry.summary,
      outcomeLabel: labelOutcomeClass(entry.outcomeClass),
      eventTypeLabels: entry.eventTypes.map(humanizeIdentifier),
    })),
    stateBefore: projectViewerFinalState(readout.stateBefore),
    stateAfter: projectViewerFinalState(readout.stateAfter),
  };
}

function projectViewerFinalState(state: RulebenchViewerFinalStateDto): RulebenchFinalStateView {
  return {
    summary: state.summary,
    combatants: state.combatants.map((combatant) => ({
      id: combatant.id,
      name: combatant.name,
      hitPointLabel: labelHitPoints(combatant.currentHitPoints, combatant.maxHitPoints),
      conditionLabels: labelConditions(combatant.conditions),
    })),
  };
}

function projectBoard(readout: RulebenchScenarioReadoutDto): RulebenchBoardView {
  const terrainByPosition = new Map(
    readout.grid.cells.map((cell) => [positionKey(cell.x, cell.y), labelTerrain(cell.terrainTags)]),
  );
  const occupantsByPosition = new Map<string, string[]>();

  for (const combatant of readout.combatants) {
    const key = positionKey(combatant.position.x, combatant.position.y);
    const existingOccupants = occupantsByPosition.get(key) ?? [];
    occupantsByPosition.set(key, [...existingOccupants, combatant.id]);
  }

  const cells: RulebenchBoardCellView[] = [];
  for (let y = 0; y < readout.grid.height; y += 1) {
    for (let x = 0; x < readout.grid.width; x += 1) {
      const key = positionKey(x, y);
      cells.push({
        x,
        y,
        terrainLabel: terrainByPosition.get(key) ?? 'clear',
        occupantIds: occupantsByPosition.get(key) ?? [],
      });
    }
  }

  return {
    width: readout.grid.width,
    height: readout.grid.height,
    cells,
  };
}

function projectCombatant(combatant: RulebenchCombatantDto): RulebenchCombatantView {
  return {
    id: combatant.id,
    name: combatant.name,
    teamLabel: combatant.team === 'ally' ? 'Ally' : 'Enemy',
    positionLabel: labelPosition(combatant.position.x, combatant.position.y),
    hitPointLabel: labelHitPoints(combatant.hitPoints.current, combatant.hitPoints.max),
    defenseLabels: combatant.defenses.map((defense) => `${defense.label} ${defense.value}`),
    conditionLabels: labelConditions(combatant.conditions),
    isActor: combatant.isActor,
  };
}

function projectTimelineRow(
  event: RulebenchDomainEventDto,
  combatantLabels: ReadonlyMap<string, string>,
): RulebenchTimelineRowView {
  return {
    sequenceLabel: String(event.sequence),
    typeLabel: event.type,
    summary: event.summary,
    participantLabels: event.entityIds.map((entityId) => labelForId(combatantLabels, entityId)),
  };
}

function projectTraceGroups(trace: readonly RulebenchTraceEntryDto[]): readonly RulebenchTraceGroupView[] {
  return tracePhaseOrder
    .map((phase) => ({
      phaseLabel: labelTracePhase(phase),
      entries: trace.filter((entry) => entry.phase === phase).map(projectTraceEntry),
    }))
    .filter((group) => group.entries.length > 0);
}

function projectTraceEntry(entry: RulebenchTraceEntryDto): RulebenchTraceEntryView {
  return {
    sequenceLabel: String(entry.sequence),
    statusLabel: labelTraceStatus(entry.status),
    message: entry.message,
    detail: entry.detail,
  };
}

function projectCombatLogEntry(entry: RulebenchCombatLogEntryDto): RulebenchCombatLogEntryView {
  return {
    id: entry.id,
    stepId: entry.stepId,
    logIndexLabel: String(entry.logIndex),
    title: entry.title,
    summary: entry.summary,
    outcomeLabel: labelOutcomeClass(entry.outcomeClass),
    eventTypeLabels: entry.eventTypes,
  };
}

function projectFinalState(finalState: RulebenchFinalStateDto): RulebenchFinalStateView {
  return {
    summary: finalState.summary,
    combatants: finalState.combatants.map((combatant) => ({
      id: combatant.id,
      name: combatant.name,
      hitPointLabel: labelHitPoints(combatant.hitPoints.current, combatant.hitPoints.max),
      conditionLabels: labelConditions(combatant.conditions),
    })),
  };
}

function createCombatantLabels(combatants: readonly RulebenchCombatantDto[]): ReadonlyMap<string, string> {
  return new Map(combatants.map((combatant) => [combatant.id, combatant.name]));
}

function labelForId(labels: ReadonlyMap<string, string>, id: string): string {
  return labels.get(id) ?? id;
}

function labelPosition(x: number, y: number): string {
  return `(${x}, ${y})`;
}

function labelHitPoints(current: number, max: number): string {
  return `${current}/${max} HP`;
}

function labelConditions(conditions: readonly string[]): readonly string[] {
  return conditions.length === 0 ? ['None'] : conditions;
}

function labelTerrain(tags: readonly string[]): string {
  return tags.length === 0 ? 'clear' : tags.join(', ');
}

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/[-_]/g, ' ')
    .replace(/^./, (first) => first.toUpperCase());
}

function labelLegality(legality: RulebenchScenarioReadoutDto['selectedTarget']['legality']): string {
  return legality === 'accepted' ? 'Accepted' : 'Rejected';
}

function labelOutcomeClass(outcomeClass: RulebenchCombatSessionStepReadoutDto['step']['outcomeClass']): string {
  switch (outcomeClass) {
    case 'acceptedHit':
      return 'Accepted hit';
    case 'acceptedMiss':
      return 'Accepted miss';
    case 'rejectedTargetLegality':
      return 'Rejected target';
    case 'rejectedInvalidCommand':
      return 'Rejected invalid command';
  }
}

function labelTracePhase(phase: TracePhase): string {
  switch (phase) {
    case 'proposal':
      return 'Proposal';
    case 'validation':
      return 'Validation';
    case 'resolution':
      return 'Resolution';
    case 'commit':
      return 'Commit';
  }
}

function labelTraceStatus(status: RulebenchTraceEntryDto['status']): string {
  switch (status) {
    case 'accepted':
      return 'Accepted';
    case 'rejected':
      return 'Rejected';
    case 'info':
      return 'Info';
  }
}

function positionKey(x: number, y: number): string {
  return `${x}:${y}`;
}
export * from "./capabilities";
