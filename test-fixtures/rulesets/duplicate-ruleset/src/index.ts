import {
  composePlayBundle,
  contentPackRequest,
  contentPackSource,
  defineContentPack,
  defineRuleset,
} from '@asha-rpg/authoring';

const rulesetInput = {
  schema: { identity: 'asha.rpg.ruleset', major: 1 },
  identity: { id: 'rulebench.duplicate-ruleset', version: '1.0.0' },
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
} as const;

export const firstRuleset = defineRuleset(rulesetInput);
export const secondRuleset = defineRuleset(rulesetInput);

export const contentPack = defineContentPack({
  identity: { id: 'rulebench.duplicate-ruleset.content', version: '1.0.0' },
  entry: { module: 'src/index.ts', declaration: 'contentPack' },
  definitions: [],
});

export const contentSource = contentPackSource(contentPack);

export const playBundle = composePlayBundle({
  identity: { id: 'rulebench.duplicate-ruleset.play', version: '1.0.0' },
  ruleset: firstRuleset,
  base: contentPackRequest(contentPack.identity),
  add: [],
  overlays: [],
  configure: {},
});
