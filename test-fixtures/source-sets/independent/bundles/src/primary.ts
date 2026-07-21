import {
  composePlayBundle,
  contentPackRequest,
  defineScenarioTemplate,
} from '@asha-rpg/authoring';

import { contentPack } from '../../content/src/index.js';
import { ruleset } from '../../rules/src/index.js';

export const playBundle = composePlayBundle({
  identity: { id: 'rulebench.independent.play', version: '1.0.0' },
  ruleset,
  base: contentPackRequest(contentPack.identity),
  add: [],
  overlays: [],
  configure: {},
});

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
