import {
  action,
  actionId,
  composeRuleset,
  constant,
  damage,
  damageType,
  defineActionDefinition,
  defineRulesetPackage,
  defineSupportDefinition,
  definitionReference,
  hostile,
  noRoll,
  onCheck,
  prepareRulesetCompilation,
  rulesetDependency,
  rulesetPackageRequest,
  rulesetPackageSource,
} from '@asha-rpg/authoring';
import type {
  PrepareRulesetResult,
  RulesetCompositionManifest,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

const radiantDamage = defineSupportDefinition({
  kind: 'support',
  id: 'catalog.damage.radiant',
  visibility: 'public',
  extensionPolicy: 'sealed',
  source: {
    module: 'packages/rulebench-primitives.ts',
    declaration: 'radiantDamage',
  },
  references: [],
  presentation: {
    label: 'Radiant damage',
    description: 'A fresh support definition used only by the field manual.',
  },
  semantic: { catalog: 'damageType', id: 'radiant' },
});

const signalFlare = defineActionDefinition({
  kind: 'action',
  id: 'rulebench.signal-flare',
  visibility: 'public',
  extensionPolicy: 'patchable',
  source: {
    module: 'packages/rulebench-field-manual.ts',
    declaration: 'signalFlare',
  },
  references: [
    definitionReference({
      importAs: 'primitives',
      definitionId: 'catalog.damage.radiant',
    }),
  ],
  presentation: {
    label: 'Signal Flare',
    description:
      'A TypeScript-authored declaration compiled by Rust authority.',
    tags: ['fresh', 'inspection'],
  },
  action: action({
    id: actionId('rulebench.signal-flare'),
    name: 'Signal Flare',
    sourcePath: 'packages/rulebench-field-manual.ts',
    targets: hostile({ range: 6 }),
    check: noRoll(),
    program: onCheck({
      noRoll: damage({
        amount: constant(4),
        type: damageType('radiant'),
      }),
    }),
  }),
});

const primitivesPackage = defineRulesetPackage({
  identity: { id: 'rulebench.primitives', version: '1.0.0' },
  entry: {
    module: 'packages/rulebench-primitives.ts',
    declaration: 'default',
  },
  language: { id: 'asha-rpg', version: '^1.0.0' },
  dependencies: [],
  requirements: { operations: [], capabilities: [] },
  definitions: [radiantDamage],
  exports: ['catalog.damage.radiant'],
  policyBindings: [],
  relationships: [],
});

const fieldManualPackage = defineRulesetPackage({
  identity: { id: 'rulebench.field-manual', version: '1.0.0' },
  entry: {
    module: 'packages/rulebench-field-manual.ts',
    declaration: 'default',
  },
  language: { id: 'asha-rpg', version: '^1.0.0' },
  dependencies: [
    rulesetDependency({
      id: 'rulebench.primitives',
      version: '1.0.0',
      importAs: 'primitives',
    }),
  ],
  requirements: {
    operations: [{ id: 'operation.damage', version: 1 }],
    capabilities: [{ id: 'capability.vitality', version: 1 }],
  },
  definitions: [signalFlare],
  exports: ['rulebench.signal-flare'],
  policyBindings: [],
  relationships: [],
});

export const FRESH_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(fieldManualPackage),
    rulesetPackageSource(primitivesPackage),
  ]);

export const FRESH_RULESET_COMPOSITION: RulesetCompositionManifest =
  composeRuleset({
    identity: { id: 'rulebench.fresh-start', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '^1.0.0' },
    base: rulesetPackageRequest({
      id: 'rulebench.field-manual',
      version: '1.0.0',
    }),
    add: [
      rulesetPackageRequest({ id: 'rulebench.primitives', version: '1.0.0' }),
    ],
    overlays: [],
    configure: {},
  });

export function prepareFreshRulebenchRuleset(): PrepareRulesetResult {
  return prepareRulesetCompilation({
    composition: FRESH_RULESET_COMPOSITION,
    packages: FRESH_RULESET_PACKAGE_SOURCES,
  });
}
