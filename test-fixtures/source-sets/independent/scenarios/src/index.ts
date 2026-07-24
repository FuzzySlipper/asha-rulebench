import { defineScenarioTemplate } from '@asha-rpg/authoring';

import { playBundle } from '../../bundles/src/primary.js';
import {
  demoActionDefinition,
  demoMoveActionDefinition,
} from '../../content/src/index.js';
import {
  coordinatedFlankerFeature,
  holdTheLineFeature,
  positionalStrikeDefinition,
  vanguardClass,
} from '../../positional-content/src/index.js';

export const scenario = defineScenarioTemplate({
  identity: { id: 'rulebench.independent.scenario', version: '1.0.0' },
  playBundle: playBundle.identity,
  presentation: { label: 'Independent source scenario' },
  board: {
    width: 3,
    height: 2,
    cells: [
      { id: 'cell-0-0', position: { x: 0, y: 0 }, capabilities: [] },
      {
        id: 'cell-1-0',
        position: { x: 1, y: 0 },
        capabilities: [
          {
            id: 'capability.traversal',
            version: 1,
            value: {
              kind: 'traversal',
              passable: false,
              movementCost: 1,
            },
          },
        ],
      },
      { id: 'cell-2-0', position: { x: 2, y: 0 }, capabilities: [] },
      { id: 'cell-0-1', position: { x: 0, y: 1 }, capabilities: [] },
      { id: 'cell-1-1', position: { x: 1, y: 1 }, capabilities: [] },
      { id: 'cell-2-1', position: { x: 2, y: 1 }, capabilities: [] },
    ],
  },
  participants: [
    {
      id: 'demo-hero',
      label: 'Demo Hero',
      teamId: 'allies',
      position: { x: 0, y: 0 },
      definitionIds: [demoActionDefinition.id, demoMoveActionDefinition.id],
      featureDefinitionIds: [],
      items: [],
      equipment: [],
      capabilities: [{ owner: 'vitality', value: { current: 10, max: 10 } }],
    },
    {
      id: 'demo-rival',
      label: 'Demo Rival',
      teamId: 'rivals',
      position: { x: 2, y: 0 },
      definitionIds: [demoActionDefinition.id, demoMoveActionDefinition.id],
      featureDefinitionIds: [],
      items: [],
      equipment: [],
      capabilities: [{ owner: 'vitality', value: { current: 8, max: 8 } }],
    },
  ],
  turn: {
    initiativeOrder: ['demo-hero', 'demo-rival'],
    currentActorId: 'demo-hero',
    round: 1,
    turn: 1,
  },
  randomSource: {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.roll-tape',
    sourceVersion: 1,
  },
});

export const positionalContributionsScenario = defineScenarioTemplate({
  identity: {
    id: 'rulebench.independent.positional-contributions',
    version: '1.0.0',
  },
  playBundle: playBundle.identity,
  presentation: {
    label: 'Independent positional contributions',
    description:
      'A Vanguard begins in a flank while adjacent to two living hostiles.',
  },
  board: {
    width: 3,
    height: 2,
    cells: [
      { id: 'cell-0-0', position: { x: 0, y: 0 }, capabilities: [] },
      { id: 'cell-1-0', position: { x: 1, y: 0 }, capabilities: [] },
      { id: 'cell-2-0', position: { x: 2, y: 0 }, capabilities: [] },
      { id: 'cell-0-1', position: { x: 0, y: 1 }, capabilities: [] },
      { id: 'cell-1-1', position: { x: 1, y: 1 }, capabilities: [] },
      { id: 'cell-2-1', position: { x: 2, y: 1 }, capabilities: [] },
    ],
  },
  participants: [
    {
      id: 'vanguard',
      label: 'Demo Vanguard',
      teamId: 'allies',
      position: { x: 0, y: 0 },
      definitionIds: [positionalStrikeDefinition.id],
      classDefinitionId: vanguardClass.id,
      featureDefinitionIds: [
        coordinatedFlankerFeature.id,
        holdTheLineFeature.id,
      ],
      items: [],
      equipment: [],
      capabilities: [
        { owner: 'vitality', value: { current: 10, max: 10 } },
        { owner: 'stat', id: 'attack-bonus', value: 5 },
        { owner: 'defense', id: 'guard', value: 15 },
      ],
    },
    {
      id: 'flanking-ally',
      label: 'Flanking Ally',
      teamId: 'allies',
      position: { x: 2, y: 0 },
      definitionIds: [demoActionDefinition.id],
      featureDefinitionIds: [],
      items: [],
      equipment: [],
      capabilities: [{ owner: 'vitality', value: { current: 10, max: 10 } }],
    },
    {
      id: 'practice-target',
      label: 'Practice Target',
      teamId: 'rivals',
      position: { x: 1, y: 0 },
      definitionIds: [positionalStrikeDefinition.id],
      featureDefinitionIds: [],
      items: [],
      equipment: [],
      capabilities: [
        { owner: 'vitality', value: { current: 8, max: 8 } },
        { owner: 'stat', id: 'attack-bonus', value: 5 },
        { owner: 'defense', id: 'guard', value: 17 },
      ],
    },
    {
      id: 'second-hostile',
      label: 'Second Hostile',
      teamId: 'rivals',
      position: { x: 0, y: 1 },
      definitionIds: [demoActionDefinition.id],
      featureDefinitionIds: [],
      items: [],
      equipment: [],
      capabilities: [{ owner: 'vitality', value: { current: 8, max: 8 } }],
    },
  ],
  turn: {
    initiativeOrder: [
      'vanguard',
      'practice-target',
      'flanking-ally',
      'second-hostile',
    ],
    currentActorId: 'vanguard',
    round: 1,
    turn: 1,
  },
  randomSource: {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.roll-tape',
    sourceVersion: 1,
  },
});
