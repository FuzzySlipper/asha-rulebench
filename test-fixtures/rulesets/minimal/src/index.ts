import {
  composePlayBundle,
  contentPackRequest,
  contentPackSource,
  defineContentPack,
  defineRuleset,
  defineScenarioTemplate,
} from '@asha-rpg/authoring';

export const minimalRuleset = defineRuleset({
  schema: { identity: 'asha.rpg.ruleset', major: 1 },
  identity: { id: 'rulebench.minimal', version: '1.0.0' },
  language: { id: 'asha-rpg', version: '1.0.0' },
  models: {
    checks: { id: 'check.d20-roll-over', version: 1 },
    turns: { id: 'turn.ordered-one-action', version: 1 },
    initiative: { id: 'initiative.scenario-ordered', version: 1 },
    reactions: { id: 'reaction.before-damage-choice', version: 1 },
    actionEconomy: {
      id: 'action-economy.one-action-plus-reaction',
      version: 1,
    },
  },
  provides: {
    operations: [],
    capabilities: [],
    values: [],
    numericDomains: [],
  },
});

export const minimalContentPack = defineContentPack({
  identity: { id: 'rulebench.minimal.content', version: '1.0.0' },
  entry: { module: 'src/index.ts', declaration: 'minimalContentPack' },
  definitions: [],
});

export const minimalContentSource = contentPackSource(minimalContentPack);

export const minimalPlayBundle = composePlayBundle({
  identity: { id: 'rulebench.minimal.play', version: '1.0.0' },
  ruleset: minimalRuleset,
  base: contentPackRequest(minimalContentPack.identity),
  add: [],
  overlays: [],
  configure: {},
});

export const minimalScenario = defineScenarioTemplate({
  identity: { id: 'rulebench.minimal.scenario', version: '1.0.0' },
  playBundle: minimalPlayBundle.identity,
  presentation: { label: 'Minimal Scenario' },
  board: { width: 3, height: 3, cells: [] },
  participants: [],
  turn: {
    initiativeOrder: [],
    currentActorId: '',
    round: 1,
    turn: 1,
  },
  randomSource: {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.system',
    sourceVersion: 1,
  },
});
