import {
  action,
  actionId,
  add,
  applyModifier,
  attack,
  canonicalJson,
  compare,
  composeRuleset,
  constant,
  damage,
  damageType,
  defenseId,
  defineActionDefinition,
  defineRulesetPackage,
  defineSupportDefinition,
  definitionReference,
  dice,
  hostile,
  modifierId,
  moveEntity,
  noRoll,
  onCheck,
  openReaction,
  prepareRulesetCompilation,
  reactionId,
  reactionOptionId,
  readStat,
  refresh,
  resourceId,
  rulesetDependency,
  rulesetPackageRequest,
  rulesetPackageSource,
  sequence,
  spend,
  stackingGroup,
  statId,
  turns,
  when,
} from '@asha-rpg/authoring';
import type {
  AuthoredAction,
  PrepareRulesetResult,
  RulesetCompilerDiagnostic,
  RulesetCompositionManifest,
  RulesetDefinitionReference,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

const supportDefinitions = Object.freeze([
  support('catalog.damage.storm', 'damageType', 'storm', 'Storm damage'),
  support('catalog.stat.power', 'stat', 'power', 'Power'),
  support('catalog.defense.guard', 'defense', 'guard', 'Guard'),
  support('catalog.resource.focus', 'resource', 'focus', 'Focus'),
  support('catalog.modifier.exposed', 'modifier', 'exposed', 'Exposed'),
]);

const primitivesPackage = defineRulesetPackage({
  identity: { id: 'rulebench.primitives', version: '1.0.0' },
  entry: {
    module: 'packages/rulebench-primitives.ts',
    declaration: 'default',
  },
  language: { id: 'asha-rpg', version: '^1.0.0' },
  dependencies: [],
  requirements: { operations: [], capabilities: [] },
  definitions: supportDefinitions,
  exports: supportDefinitions.map((definition) => definition.id),
  policyBindings: [],
  relationships: [],
});

const freshFieldManual = fieldManualPackage('catalog.damage.storm');
const invalidFieldManual = fieldManualPackage('catalog.damage.missing');

export const FRESH_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(freshFieldManual),
    rulesetPackageSource(primitivesPackage),
  ]);

const INVALID_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(invalidFieldManual),
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

export type RulebenchRulesetSourceId = 'fresh' | 'missingSupport';

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
      description:
        'Three TypeScript-authored actions and their support catalogs.',
    },
    {
      id: 'missingSupport',
      label: 'Invalid missing support',
      description:
        'Arc Lash references damage support absent from the package graph.',
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
      readonly diagnostics: readonly RulesetCompilerDiagnostic[];
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
    diagnostics: result.diagnostics,
  };
}

function authoredActions(
  stormDamageDefinitionId: string,
): readonly AuthoredAction[] {
  const power = statId('catalog.stat.power');
  const guard = defenseId('catalog.defense.guard');
  const focus = resourceId('catalog.resource.focus');
  const storm = damageType(stormDamageDefinitionId);

  return Object.freeze([
    action({
      id: actionId('rulebench.tactical-advance'),
      name: 'Tactical Advance',
      sourcePath: 'packages/rulebench-field-manual.ts',
      targets: hostile({ range: 6 }),
      check: noRoll(),
      program: onCheck({
        noRoll: sequence(
          moveEntity({
            subject: 'actor',
            deltaX: constant(2),
            deltaY: constant(0),
            maximumDistance: 2,
            provokes: false,
          }),
          applyModifier({
            modifier: modifierId('catalog.modifier.exposed'),
            value: constant(-2),
            duration: turns(2),
            stacking: refresh(stackingGroup('guard-penalty')),
          }),
        ),
      }),
    }),
    action({
      id: actionId('rulebench.arc-lash'),
      name: 'Arc Lash',
      sourcePath: 'packages/rulebench-field-manual.ts',
      targets: hostile({ range: 3 }),
      check: attack({ modifier: readStat('actor', power), defense: guard }),
      rollScope: 'perTarget',
      costs: [spend(focus, 1)],
      program: onCheck({
        hit: when(
          compare(readStat('actor', power), 'greaterThan', constant(0)),
          damage({
            amount: add(dice({ count: 2, sides: 6 }), constant(1)),
            type: storm,
          }),
          damage({ amount: dice({ count: 1, sides: 6 }), type: storm }),
        ),
      }),
    }),
    action({
      id: actionId('rulebench.wardbreaker-volley'),
      name: 'Wardbreaker Volley',
      sourcePath: 'packages/rulebench-field-manual.ts',
      targets: hostile({ range: 3 }),
      check: noRoll(),
      costs: [spend(focus, 1)],
      program: onCheck({
        noRoll: sequence(
          openReaction({
            id: reactionId('reaction.raise-ward'),
            options: [
              {
                id: reactionOptionId('raise-ward'),
                label: 'Raise ward',
                damageReduction: 3,
              },
            ],
          }),
          damage({ amount: dice({ count: 5, sides: 4 }), type: storm }),
        ),
      }),
    }),
  ]);
}

function fieldManualPackage(stormDamageDefinitionId: string) {
  const actions = authoredActions(stormDamageDefinitionId);
  const definitions = actions.map((authored) =>
    defineActionDefinition({
      kind: 'action',
      id: authored.id,
      visibility: 'public',
      extensionPolicy: 'patchable',
      source: {
        module: 'packages/rulebench-field-manual.ts',
        declaration: authored.id,
      },
      references: actionReferences(authored.id, stormDamageDefinitionId),
      presentation: {
        label: authored.name,
        description: 'Fresh TypeScript content compiled into Rust authority.',
        tags: ['fresh', 'playable'],
      },
      action: authored,
    }),
  );
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
      operations: [
        { id: 'operation.applyModifier', version: 1 },
        { id: 'operation.damage', version: 1 },
        { id: 'operation.move', version: 1 },
        { id: 'operation.openReaction', version: 1 },
      ],
      capabilities: [
        { id: 'capability.defenses', version: 1 },
        { id: 'capability.modifiers', version: 1 },
        { id: 'capability.position', version: 1 },
        { id: 'capability.random', version: 1 },
        { id: 'capability.reactions', version: 1 },
        { id: 'capability.resources', version: 1 },
        { id: 'capability.stats', version: 1 },
        { id: 'capability.vitality', version: 1 },
      ],
    },
    definitions,
    exports: definitions.map((definition) => definition.id),
    policyBindings: [],
    relationships: [],
  });
}

function actionReferences(
  actionIdentity: string,
  stormDamageDefinitionId: string,
): readonly RulesetDefinitionReference[] {
  const references =
    actionIdentity === 'rulebench.tactical-advance'
      ? ['catalog.modifier.exposed', 'catalog.stat.power']
      : actionIdentity === 'rulebench.arc-lash'
        ? [
            stormDamageDefinitionId,
            'catalog.stat.power',
            'catalog.defense.guard',
            'catalog.resource.focus',
          ]
        : [stormDamageDefinitionId, 'catalog.resource.focus'];
  return Object.freeze(
    references.map((definitionId) =>
      definitionReference({ importAs: 'primitives', definitionId }),
    ),
  );
}

function support(
  id: string,
  catalog: 'stat' | 'defense' | 'resource' | 'modifier' | 'damageType',
  semanticId: string,
  label: string,
) {
  return defineSupportDefinition({
    kind: 'support',
    id,
    visibility: 'public',
    extensionPolicy: 'sealed',
    source: {
      module: 'packages/rulebench-primitives.ts',
      declaration: semanticId,
    },
    references: [],
    presentation: { label },
    semantic: { catalog, id: semanticId },
  });
}
