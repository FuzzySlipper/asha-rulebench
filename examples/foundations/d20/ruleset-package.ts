import {
  actionPatch,
  defineMixinDefinition,
  defineRulesetCatalog,
  defineRulesetPackage,
  patchParameter,
} from '@asha-rpg/authoring';

export const primitivesCatalog = defineRulesetCatalog({
  packageId: 'rulebench.primitives',
  sourceModule: 'foundations/d20/ruleset-package.ts',
  entries: {
    storm: {
      definitionId: 'catalog.damage.storm',
      category: 'damageType',
      id: 'storm',
      label: 'Storm damage',
    },
    power: {
      definitionId: 'catalog.stat.power',
      category: 'stat',
      id: 'power',
      label: 'Power',
    },
    guard: {
      definitionId: 'catalog.defense.guard',
      category: 'defense',
      id: 'guard',
      label: 'Guard',
    },
    focus: {
      definitionId: 'catalog.resource.focus',
      category: 'resource',
      id: 'focus',
      label: 'Focus',
    },
    exposed: {
      definitionId: 'catalog.modifier.exposed',
      category: 'modifier',
      id: 'exposed',
      label: 'Exposed',
    },
  },
});

const mixinDefinitions = Object.freeze([
  defineMixinDefinition({
    kind: 'mixin',
    id: 'rulebench.double-range',
    visibility: 'public',
    extensionPolicy: 'sealed',
    source: {
      module: 'foundations/d20/ruleset-package.ts',
      declaration: 'doubleRange',
    },
    parameters: [{ id: 'factor', type: 'number' }],
    patch: actionPatch.semantic.maximumRange.adjust({
      multiply: patchParameter('factor'),
    }),
  }),
  defineMixinDefinition({
    kind: 'mixin',
    id: 'rulebench.extend-range',
    visibility: 'public',
    extensionPolicy: 'sealed',
    source: {
      module: 'foundations/d20/ruleset-package.ts',
      declaration: 'extendRange',
    },
    parameters: [{ id: 'amount', type: 'number', default: 1 }],
    patch: actionPatch.semantic.maximumRange.adjust({
      add: patchParameter('amount'),
    }),
  }),
]);

export const primitivesPackage = defineRulesetPackage({
  identity: { id: 'rulebench.primitives', version: '1.0.0' },
  entry: {
    module: 'foundations/d20/ruleset-package.ts',
    declaration: 'primitivesPackage',
  },
  definitions: [...primitivesCatalog.definitions, ...mixinDefinitions],
});
