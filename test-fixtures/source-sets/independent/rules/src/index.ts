import { defineRuleset } from '@asha-rpg/authoring';

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
    operations: [
      { id: 'operation.heal', version: 1 },
      { id: 'operation.moveToCell', version: 1 },
    ],
    capabilities: [
      { id: 'capability.position', version: 1 },
      { id: 'capability.vitality', version: 1 },
    ],
    values: [],
    numericDomains: [],
  },
});
