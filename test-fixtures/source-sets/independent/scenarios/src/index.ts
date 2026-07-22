import { defineScenarioTemplate } from '@asha-rpg/authoring';

import { playBundle } from '../../bundles/src/primary.js';

export const scenario = defineScenarioTemplate({
  identity: { id: 'rulebench.independent.scenario', version: '1.0.0' },
  playBundle: playBundle.identity,
  presentation: { label: 'Independent source scenario' },
  board: { width: 1, height: 1, cells: [] },
  participants: [],
  turn: { initiativeOrder: [], currentActorId: '', round: 1, turn: 1 },
  randomSource: {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.system',
    sourceVersion: 1,
  },
});
