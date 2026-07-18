import {
  action,
  actionId,
  canonicalJson,
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

const signalFlare = signalFlareDefinition('catalog.damage.radiant');
const invalidSignalFlare = signalFlareDefinition('catalog.damage.missing');

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

const fieldManualPackage = fieldManualPackageFor(signalFlare);
const invalidFieldManualPackage = fieldManualPackageFor(invalidSignalFlare);

export const FRESH_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(fieldManualPackage),
    rulesetPackageSource(primitivesPackage),
  ]);

const INVALID_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(invalidFieldManualPackage),
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

export type RulebenchRulesetSourceId = 'fresh' | 'missing-support';

export interface RulebenchRulesetSourceOption {
  readonly id: RulebenchRulesetSourceId;
  readonly label: string;
  readonly description: string;
}

export const RULEBENCH_RULESET_SOURCE_OPTIONS: readonly RulebenchRulesetSourceOption[] =
  Object.freeze([
    {
      id: 'fresh',
      label: 'Valid field manual',
      description: 'Signal Flare and its radiant support definition.',
    },
    {
      id: 'missing-support',
      label: 'Invalid missing support',
      description:
        'Signal Flare references a definition absent from the package graph.',
    },
  ]);

export type PreparedRulebenchRulesetSource =
  | {
      readonly ok: true;
      readonly preparedSource: string;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly {
        readonly stage: string;
        readonly severity: 'error';
        readonly code: string;
        readonly path: string;
        readonly message: string;
      }[];
    };

export function prepareRulebenchRulesetSource(
  sourceId: RulebenchRulesetSourceId,
): PreparedRulebenchRulesetSource {
  const result = prepareRulesetCompilation({
    composition: FRESH_RULESET_COMPOSITION,
    packages:
      sourceId === 'fresh'
        ? FRESH_RULESET_PACKAGE_SOURCES
        : INVALID_RULESET_PACKAGE_SOURCES,
  });
  if (result.ok) {
    return {
      ok: true,
      preparedSource: canonicalJson(result.prepared),
      diagnostics: [],
    };
  }
  return {
    ok: false,
    diagnostics: result.diagnostics.map((diagnostic) => ({
      stage: diagnostic.stage,
      severity: diagnostic.severity,
      code: diagnostic.code,
      path: diagnostic.path,
      message: diagnostic.message,
    })),
  };
}

function signalFlareDefinition(damageDefinitionId: string) {
  return defineActionDefinition({
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
        definitionId: damageDefinitionId,
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
          type: damageType(damageDefinitionId),
        }),
      }),
    }),
  });
}

function fieldManualPackageFor(
  definition: ReturnType<typeof signalFlareDefinition>,
) {
  return defineRulesetPackage({
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
    definitions: [definition],
    exports: ['rulebench.signal-flare'],
    policyBindings: [],
    relationships: [],
  });
}
