import { describe, expect, it } from 'vitest';
import type { RulebenchScenarioReadoutDto } from '@asha-rulebench/protocol';
import { projectRulebenchScenario } from './index';

const scenarioReadout: RulebenchScenarioReadoutDto = {
  id: 'two-combatant-hexing-bolt',
  title: 'Hexing Bolt Opening',
  summary: 'A focused readout fixture.',
  seedLabel: 'roll-stream:17,5',
  grid: {
    width: 3,
    height: 2,
    cells: [{ x: 1, y: 1, terrainTags: ['cover'] }],
  },
  combatants: [
    {
      id: 'entity-adept',
      name: 'Adept',
      team: 'ally',
      position: { x: 0, y: 0 },
      hitPoints: { current: 24, max: 24 },
      defenses: [{ id: 'nerve', label: 'Nerve', value: 15 }],
      conditions: [],
      isActor: true,
    },
    {
      id: 'entity-raider',
      name: 'Raider',
      team: 'enemy',
      position: { x: 2, y: 0 },
      hitPoints: { current: 9, max: 18 },
      defenses: [{ id: 'nerve', label: 'Nerve', value: 13 }],
      conditions: ['rattled'],
      isActor: false,
    },
  ],
  selectedAction: {
    id: 'hexing_bolt',
    name: 'Hexing Bolt',
    actorId: 'entity-adept',
    targetIds: ['entity-raider'],
    actionText: 'Mind vs Nerve at range 10',
    effectText: 'Psychic damage and rattled on hit',
  },
  selectedTarget: {
    targetId: 'entity-raider',
    legality: 'accepted',
    reason: 'Target is hostile, within range, and visible.',
  },
  domainEvents: [
    {
      sequence: 1,
      type: 'ActionUsed',
      summary: 'Adept used Hexing Bolt against Raider.',
      entityIds: ['entity-adept', 'entity-raider'],
    },
    {
      sequence: 2,
      type: 'DamageApplied',
      summary: 'Raider took 9 psychic damage.',
      entityIds: ['entity-raider'],
    },
  ],
  trace: [
    {
      sequence: 1,
      phase: 'proposal',
      status: 'info',
      message: 'UseActionIntent received.',
      detail: 'Actor proposed action.',
    },
    {
      sequence: 2,
      phase: 'validation',
      status: 'accepted',
      message: 'Target legality accepted.',
      detail: 'The target is hostile, in range, and visible.',
    },
    {
      sequence: 3,
      phase: 'commit',
      status: 'accepted',
      message: 'DomainEvents committed.',
      detail: 'Accepted facts were recorded.',
    },
  ],
  finalState: {
    summary: 'Raider is damaged and rattled.',
    combatants: [
      {
        id: 'entity-adept',
        name: 'Adept',
        hitPoints: { current: 24, max: 24 },
        conditions: [],
      },
      {
        id: 'entity-raider',
        name: 'Raider',
        hitPoints: { current: 9, max: 18 },
        conditions: ['rattled'],
      },
    ],
  },
};

describe('projectRulebenchScenario', () => {
  it('maps scenario readouts into UI-ready board, timeline, trace, and final-state views', () => {
    const view = projectRulebenchScenario(scenarioReadout);

    expect(view.title).toBe('Hexing Bolt Opening');
    expect(view.board.cells).toHaveLength(6);
    expect(view.board.cells.find((cell) => cell.x === 0 && cell.y === 0)?.occupantIds).toEqual(['entity-adept']);
    expect(view.board.cells.find((cell) => cell.x === 1 && cell.y === 1)?.terrainLabel).toBe('cover');
    expect(view.combatants[0]?.hitPointLabel).toBe('24/24 HP');
    expect(view.combatants[0]?.conditionLabels).toEqual(['None']);
    expect(view.selectedAction.actorLabel).toBe('Adept');
    expect(view.selectedAction.targetLabels).toEqual(['Raider']);
    expect(view.selectedTarget.legalityLabel).toBe('Accepted');
    expect(view.timeline[0]?.participantLabels).toEqual(['Adept', 'Raider']);
    expect(view.traceGroups.map((group) => group.phaseLabel)).toEqual(['Proposal', 'Validation', 'Commit']);
    expect(view.finalState.combatants[1]?.conditionLabels).toEqual(['rattled']);
  });

  it('maps empty grids and unknown entity references without manufacturing authority facts', () => {
    const view = projectRulebenchScenario({
      ...scenarioReadout,
      grid: { width: 1, height: 1, cells: [] },
      combatants: [],
      selectedAction: {
        ...scenarioReadout.selectedAction,
        actorId: 'missing-actor',
        targetIds: ['missing-target'],
      },
      selectedTarget: {
        targetId: 'missing-target',
        legality: 'rejected',
        reason: 'No target exists.',
      },
      domainEvents: [
        {
          sequence: 1,
          type: 'IntentRejected',
          summary: 'The intent was rejected.',
          entityIds: ['missing-target'],
        },
      ],
      trace: [
        {
          sequence: 1,
          phase: 'validation',
          status: 'rejected',
          message: 'Target missing.',
          detail: 'No target exists.',
        },
      ],
      finalState: {
        summary: 'No state changed.',
        combatants: [],
      },
    });

    expect(view.board.cells).toEqual([{ x: 0, y: 0, terrainLabel: 'clear', occupantIds: [] }]);
    expect(view.selectedAction.actorLabel).toBe('missing-actor');
    expect(view.selectedTarget.legalityLabel).toBe('Rejected');
    expect(view.timeline[0]?.participantLabels).toEqual(['missing-target']);
    expect(view.traceGroups[0]?.entries[0]?.statusLabel).toBe('Rejected');
    expect(view.finalState.summary).toBe('No state changed.');
  });
});
