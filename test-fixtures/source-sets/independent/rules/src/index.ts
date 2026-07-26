import {
  defineRuleset,
  rulesetCalculationSelector,
  rulesetContributionStackingGroup,
} from '@asha-rpg/authoring';
import {
  RPG_CAPABILITY_VERSIONS,
  RPG_OPERATION_VERSIONS,
} from '@asha-rpg/ir';

export const ruleset = defineRuleset({
  schema: { identity: 'asha.rpg.ruleset', major: 1 },
  identity: { id: 'rulebench.independent', version: '1.0.0' },
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
    operations: Object.entries(RPG_OPERATION_VERSIONS).map(([id, version]) => ({
      id,
      version,
    })),
    capabilities: Object.entries(RPG_CAPABILITY_VERSIONS).map(
      ([id, version]) => ({ id, version }),
    ),
    values: [
      {
        kind: 'stat',
        id: 'attack-bonus',
        label: 'Attack bonus',
        numericDomainId: 'signed-bonus',
      },
      {
        kind: 'defense',
        id: 'guard',
        label: 'Guard',
        numericDomainId: 'defense-score',
      },
    ],
    numericDomains: [
      { id: 'signed-bonus', minimum: -20, maximum: 30 },
      { id: 'defense-score', minimum: 0, maximum: 50 },
    ],
    calculationSelectors: [
      {
        id: 'attack',
        version: 1,
        label: 'Attack total',
        numericDomainId: 'signed-bonus',
      },
    ],
    contributionStackingGroups: [
      {
        id: 'circumstance',
        version: 1,
        label: 'Circumstance',
        policy: 'sum',
      },
    ],
  },
});

export const attackSelector = rulesetCalculationSelector(ruleset, 'attack');
export const circumstanceStackingGroup = rulesetContributionStackingGroup(
  ruleset,
  'circumstance',
);
