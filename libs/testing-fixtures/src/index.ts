import type { RulebenchScenarioReadoutDto } from '@asha-rulebench/protocol';

export {
  authorActionResolutionModule,
  authorRulesetDefinition,
  authorTurnControlModule,
} from './ruleset-authoring';
export type {
  ActionResolutionModuleOptions,
  RulesetDefinitionOptions,
  TurnControlModuleOptions,
} from './ruleset-authoring';

export const twoCombatantScenarioReadout: RulebenchScenarioReadoutDto = {
  id: 'two-combatant-hexing-bolt',
  title: 'Hexing Bolt Opening',
  summary: 'A focused two-combatant fixture for proving board, event, trace, and final-state readouts.',
  seedLabel: 'roll-stream:17,5',
  grid: {
    width: 6,
    height: 4,
    cells: [
      { x: 1, y: 1, terrainTags: ['clear'] },
      { x: 4, y: 1, terrainTags: ['clear'] },
      { x: 2, y: 2, terrainTags: ['cover'] },
    ],
  },
  combatants: [
    {
      id: 'entity-adept',
      name: 'Adept',
      team: 'ally',
      sideId: 'ally',
      position: { x: 1, y: 1 },
      hitPoints: { current: 24, max: 24 },
      defenses: [
        { id: 'guard', label: 'Guard', value: 16 },
        { id: 'nerve', label: 'Nerve', value: 15 },
      ],
      conditions: [],
      isActor: true,
    },
    {
      id: 'entity-raider',
      name: 'Raider',
      team: 'enemy',
      sideId: 'enemy',
      position: { x: 4, y: 1 },
      hitPoints: { current: 9, max: 18 },
      defenses: [
        { id: 'guard', label: 'Guard', value: 14 },
        { id: 'nerve', label: 'Nerve', value: 13 },
      ],
      conditions: ['rattled'],
      isActor: false,
    },
  ],
  selectedAction: {
    id: 'hexing_bolt',
    name: 'Hexing Bolt',
    actorId: 'entity-adept',
    targetIds: ['entity-raider'],
    range: 10,
    lineOfSightRequired: true,
    visibleTargetIds: ['entity-raider'],
    attack: {
      modifier: 4,
      defenseId: 'nerve',
      defenseLabel: 'Nerve',
    },
    savingThrow: null,
    contested: null,
    hit: {
      damageBonus: 4,
      damageType: 'psychic',
      modifierId: 'rattled',
      modifierLabel: 'rattled',
      modifierDuration: 'until end of next turn',
    },
    actionText: 'Mind vs Nerve at range 10',
    effectText: '1d8 + Mind psychic damage and rattled until end of next turn on hit',
  },
  selectedTarget: {
    targetId: 'entity-raider',
    legality: 'accepted',
    reason: 'Target is hostile, within range, and line of sight is clear.',
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
      type: 'AttackRolled',
      summary: 'Attack rolled 17 + 4 vs Nerve 13: hit.',
      entityIds: ['entity-adept', 'entity-raider'],
    },
    {
      sequence: 3,
      type: 'DamageApplied',
      summary: 'Raider took 9 psychic damage.',
      entityIds: ['entity-raider'],
    },
    {
      sequence: 4,
      type: 'ModifierApplied',
      summary: 'Raider became rattled until end of next turn.',
      entityIds: ['entity-raider'],
    },
  ],
  trace: [
    {
      sequence: 1,
      phase: 'proposal',
      status: 'info',
      message: 'UseActionIntent received.',
      detail: 'Actor entity-adept proposed action hexing_bolt against entity-raider.',
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
      phase: 'resolution',
      status: 'accepted',
      message: 'Hit branch selected.',
      detail: 'Roll stream supplied 17; total 21 beats Nerve 13.',
    },
    {
      sequence: 4,
      phase: 'commit',
      status: 'accepted',
      message: 'DomainEvents committed.',
      detail: 'ActionUsed, AttackRolled, DamageApplied, and ModifierApplied became accepted facts.',
    },
  ],
  finalState: {
    summary: 'Raider is damaged and rattled; Adept is unchanged.',
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

export interface RulebenchScenarioFixtureOverrides {
  readonly title?: string;
  readonly seedLabel?: string;
}

export function makeTwoCombatantScenarioReadout(
  overrides: RulebenchScenarioFixtureOverrides = {},
): RulebenchScenarioReadoutDto {
  return {
    ...twoCombatantScenarioReadout,
    title: overrides.title ?? twoCombatantScenarioReadout.title,
    seedLabel: overrides.seedLabel ?? twoCombatantScenarioReadout.seedLabel,
  };
}
