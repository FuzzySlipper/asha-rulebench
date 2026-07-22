import { defineScenarioTemplate } from '@asha-rpg/authoring';

import { playBundle } from '../../bundles/src/primary.js';
import {
  demoActionDefinition,
  demoMoveActionDefinition,
} from '../../content/src/index.js';

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
      capabilities: [{ owner: 'vitality', value: { current: 10, max: 10 } }],
    },
    {
      id: 'demo-rival',
      label: 'Demo Rival',
      teamId: 'rivals',
      position: { x: 2, y: 0 },
      definitionIds: [demoActionDefinition.id, demoMoveActionDefinition.id],
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
