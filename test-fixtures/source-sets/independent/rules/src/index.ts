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
      { id: 'operation.damage', version: 1 },
      { id: 'operation.heal', version: 1 },
      { id: 'operation.moveToCell', version: 1 },
    ],
    capabilities: [
      { id: 'capability.defenses', version: 1 },
      { id: 'capability.position', version: 1 },
      { id: 'capability.random', version: 1 },
      { id: 'capability.stats', version: 1 },
      { id: 'capability.vitality', version: 1 },
    ],
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
  },
});
