import { defineScenarioTemplate } from '@asha-rpg/authoring';

import { playBundle } from '../../bundles/src/primary.js';
import { demoActionDefinition } from '../../content/src/index.js';

export const scenario = defineScenarioTemplate({
  identity: { id: 'rulebench.independent.scenario', version: '1.0.0' },
  playBundle: playBundle.identity,
  presentation: { label: 'Independent source scenario' },
  board: { width: 3, height: 1, cells: [] },
  participants: [
    {
      id: 'demo-hero',
      label: 'Demo Hero',
      teamId: 'allies',
      position: { x: 0, y: 0 },
      definitionIds: [demoActionDefinition.id],
      capabilities: [{ owner: 'vitality', value: { current: 10, max: 10 } }],
    },
    {
      id: 'demo-rival',
      label: 'Demo Rival',
      teamId: 'rivals',
      position: { x: 2, y: 0 },
      definitionIds: [demoActionDefinition.id],
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
