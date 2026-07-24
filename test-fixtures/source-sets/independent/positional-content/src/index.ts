import {
  action,
  actionId,
  attack,
  constant,
  contentPackSource,
  damage,
  defineActionDefinition,
  defineCharacterClassDefinition,
  defineCharacterFeatureDefinition,
  defineContentCatalog,
  defineContentPack,
  definitionReference,
  hostile,
  onCheck,
  readStat,
  rulesetDefense,
  rulesetStat,
} from '@asha-rpg/authoring';

import { ruleset } from '../../rules/src/index.js';

const attackBonus = rulesetStat(ruleset, 'attack-bonus');
const guard = rulesetDefense(ruleset, 'guard');
const positionalCatalog = defineContentCatalog({
  packageId: 'rulebench.independent.positional-content',
  sourceModule: 'src/index.ts',
  entries: {
    practice: {
      definitionId: 'damage-type.practice',
      category: 'damageType',
      id: 'practice',
      label: 'Practice',
    },
  },
});

const positionalStrikeAction = action({
  id: actionId('action.rulebench.positional-strike'),
  name: 'Positional Strike',
  sourcePath:
    'test-fixtures/source-sets/independent/positional-content/src/index.ts',
  targets: hostile({ range: 1 }),
  check: attack({ modifier: readStat('actor', attackBonus), defense: guard }),
  rollScope: 'perTarget',
  program: onCheck({
    hit: damage({
      amount: constant(1),
      type: positionalCatalog.references.practice,
    }),
  }),
});

export const positionalStrikeDefinition = defineActionDefinition({
  id: positionalStrikeAction.id,
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: { module: 'src/index.ts', declaration: 'positionalStrikeDefinition' },
  presentation: {
    label: 'Positional Strike',
    description: 'Make a basic attack using the configured attack bonus.',
    tags: ['attack'],
  },
  action: positionalStrikeAction,
});

export const coordinatedFlankerFeature = defineCharacterFeatureDefinition({
  id: 'feature.rulebench.coordinated-flanker',
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: {
    module: 'src/index.ts',
    declaration: 'coordinatedFlankerFeature',
  },
  presentation: {
    label: 'Coordinated Flanker',
    description: 'Gain +2 on attacks while flanking the selected target.',
    tags: ['positional', 'talent'],
  },
  characterFeature: {
    rollContributions: [
      {
        id: 'coordinated-flanker',
        selector: 'attack',
        condition: { kind: 'actorFlanksTarget' },
        amount: 2,
      },
    ],
  },
});

export const holdTheLineFeature = defineCharacterFeatureDefinition({
  id: 'feature.rulebench.hold-the-line',
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: { module: 'src/index.ts', declaration: 'holdTheLineFeature' },
  presentation: {
    label: 'Hold the Line',
    description:
      'Gain +1 on attacks while at least two living hostiles are adjacent.',
    tags: ['positional', 'talent'],
  },
  characterFeature: {
    rollContributions: [
      {
        id: 'hold-the-line',
        selector: 'attack',
        condition: {
          kind: 'actorSurrounded',
          minimumHostiles: 2,
        },
        amount: 1,
      },
    ],
  },
});

export const vanguardClass = defineCharacterClassDefinition({
  id: 'class.rulebench.vanguard',
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: { module: 'src/index.ts', declaration: 'vanguardClass' },
  presentation: {
    label: 'Vanguard',
    description: 'A positional combatant used by the real-host browser gate.',
    tags: ['class'],
  },
  characterClass: {
    featureDefinitions: [
      definitionReference({ definitionId: coordinatedFlankerFeature.id }),
      definitionReference({ definitionId: holdTheLineFeature.id }),
    ],
  },
});

export const positionalContentPack = defineContentPack({
  identity: {
    id: 'rulebench.independent.positional-content',
    version: '1.0.0',
  },
  entry: { module: 'src/index.ts', declaration: 'positionalContentPack' },
  definitions: [
    coordinatedFlankerFeature,
    holdTheLineFeature,
    ...positionalCatalog.definitions,
    positionalStrikeDefinition,
    vanguardClass,
  ],
});

export const positionalContentSource = contentPackSource(positionalContentPack);
