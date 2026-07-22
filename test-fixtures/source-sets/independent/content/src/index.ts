import {
  action,
  actionId,
  ally,
  cells,
  constant,
  contentPackSource,
  defineActionDefinition,
  defineContentPack,
  heal,
  moveToCell,
  noRoll,
  onCheck,
} from '@asha-rpg/authoring';

const demoAction = action({
  id: actionId('action.rulebench.demo-rest'),
  name: 'Catch Breath',
  sourcePath: 'test-fixtures/source-sets/independent/content/src/index.ts',
  targets: ally({ range: 0 }),
  check: noRoll(),
  program: onCheck({ noRoll: heal({ amount: constant(1) }) }),
});

const demoMoveAction = action({
  id: actionId('action.rulebench.demo-move'),
  name: 'Move',
  sourcePath: 'test-fixtures/source-sets/independent/content/src/index.ts',
  targets: cells({ range: 2 }),
  check: noRoll(),
  program: onCheck({
    noRoll: moveToCell({ maximumDistance: 2, provokes: true }),
  }),
});

export const demoActionDefinition = defineActionDefinition({
  id: demoAction.id,
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: { module: 'src/index.ts', declaration: 'demoActionDefinition' },
  presentation: {
    label: 'Catch Breath',
    description: 'Recover one hit point.',
    tags: ['recovery'],
  },
  action: demoAction,
});

export const demoMoveActionDefinition = defineActionDefinition({
  id: demoMoveAction.id,
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: { module: 'src/index.ts', declaration: 'demoMoveActionDefinition' },
  presentation: {
    label: 'Move',
    description: 'Move to an authority-approved destination.',
    tags: ['movement'],
  },
  action: demoMoveAction,
});

export const contentPack = defineContentPack({
  identity: { id: 'rulebench.independent.content', version: '1.0.0' },
  entry: { module: 'src/index.ts', declaration: 'contentPack' },
  definitions: [demoActionDefinition, demoMoveActionDefinition],
});

export const contentSource = contentPackSource(contentPack);
