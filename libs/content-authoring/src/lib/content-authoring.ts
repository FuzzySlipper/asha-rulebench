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
  defineDerivedDefinition,
  defineMixinDefinition,
  defineRulesetPackage,
  defineRulesetRelationship,
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

const mixinDefinitions = Object.freeze([
  defineMixinDefinition({
    kind: 'mixin',
    id: 'rulebench.double-range',
    visibility: 'public',
    extensionPolicy: 'sealed',
    source: {
      module: 'packages/rulebench-primitives.ts',
      declaration: 'doubleRange',
    },
    references: [],
    parameters: [{ id: 'factor', type: 'number' }],
    patch: {
      version: 1,
      operations: [
        {
          kind: 'adjustNumber',
          plane: 'semantic',
          path: fieldPath('targets', 'maximumRange'),
          multiply: { parameter: 'factor' },
          add: 0,
        },
      ],
    },
  }),
  defineMixinDefinition({
    kind: 'mixin',
    id: 'rulebench.extend-range',
    visibility: 'public',
    extensionPolicy: 'sealed',
    source: {
      module: 'packages/rulebench-primitives.ts',
      declaration: 'extendRange',
    },
    references: [],
    parameters: [{ id: 'amount', type: 'number', default: 1 }],
    patch: {
      version: 1,
      operations: [
        {
          kind: 'adjustNumber',
          plane: 'semantic',
          path: fieldPath('targets', 'maximumRange'),
          multiply: 1,
          add: { parameter: 'amount' },
        },
      ],
    },
  }),
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
  definitions: [...supportDefinitions, ...mixinDefinitions],
  exports: [...supportDefinitions, ...mixinDefinitions].map(
    (definition) => definition.id,
  ),
  policyBindings: [],
  relationships: [],
});

const freshVariant = buildFreshRulesetVariant({
  version: '1.0.0',
  arcLashDamageBonus: 1,
});
const upgradeVariant = buildFreshRulesetVariant({
  version: '1.1.0',
  arcLashDamageBonus: 2,
});
const invalidFieldManual = fieldManualPackage('catalog.damage.missing', {
  version: '1.0.0',
  arcLashDamageBonus: 1,
});

export const FRESH_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  freshVariant.packages;
export const UPGRADE_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  upgradeVariant.packages;

const INVALID_RULESET_PACKAGE_SOURCES: readonly RulesetPackageSource[] =
  Object.freeze([
    rulesetPackageSource(invalidFieldManual),
    rulesetPackageSource(primitivesPackage),
  ]);

export const FRESH_RULESET_COMPOSITION: RulesetCompositionManifest =
  freshVariant.composition;
export const UPGRADE_RULESET_COMPOSITION: RulesetCompositionManifest =
  upgradeVariant.composition;

export function prepareFreshRulebenchRuleset(): PrepareRulesetResult {
  return prepareRulesetCompilation({
    composition: FRESH_RULESET_COMPOSITION,
    packages: FRESH_RULESET_PACKAGE_SOURCES,
  });
}

export type RulebenchRulesetSourceId =
  | 'fresh'
  | 'freshUpgrade'
  | 'missingSupport';

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
        'Four TypeScript-authored actions, including a derived and overlaid action, plus their support catalogs.',
    },
    {
      id: 'freshUpgrade',
      label: 'Field manual 1.1 candidate',
      description:
        'A candidate package upgrade that changes Arc Lash damage and its derived descendant without activation.',
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
  const selection = rulesetSourceSelection(sourceId);
  const result = prepareRulesetCompilation({
    composition: selection.composition,
    packages: selection.packages,
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

function rulesetSourceSelection(sourceId: RulebenchRulesetSourceId): {
  readonly composition: RulesetCompositionManifest;
  readonly packages: readonly RulesetPackageSource[];
} {
  switch (sourceId) {
    case 'fresh':
      return freshVariant;
    case 'freshUpgrade':
      return upgradeVariant;
    case 'missingSupport':
      return {
        composition: composeFreshRuleset([], '1.0.0'),
        packages: INVALID_RULESET_PACKAGE_SOURCES,
      };
  }
}

function buildFreshRulesetVariant(options: {
  readonly version: string;
  readonly arcLashDamageBonus: number;
}): {
  readonly composition: RulesetCompositionManifest;
  readonly packages: readonly RulesetPackageSource[];
} {
  const fieldManual = fieldManualPackage('catalog.damage.storm', options);
  const basePackages: readonly RulesetPackageSource[] = Object.freeze([
    rulesetPackageSource(fieldManual),
    rulesetPackageSource(primitivesPackage),
  ]);
  const basePrepared = prepareRulesetCompilation({
    composition: composeFreshRuleset([], options.version),
    packages: basePackages,
  });
  if (!basePrepared.ok) {
    throw new Error(
      `fresh derivation failed: ${canonicalJson(basePrepared.diagnostics)}`,
    );
  }
  const arcDerivation = basePrepared.prepared.derivationProvenance.find(
    (provenance) => provenance.definitionId === 'rulebench.arc-lash-stormfront',
  );
  if (arcDerivation === undefined) {
    throw new Error(
      'fresh derivation did not materialize rulebench.arc-lash-stormfront',
    );
  }
  const semanticOverlay = overlayPackage({
    id: 'rulebench.stormfront-balance',
    version: options.version,
    targetPackageVersion: options.version,
    expectedFingerprint: arcDerivation.materializedFingerprint,
    plane: 'semantic',
    path: fieldPath('targets', 'maximumRange'),
    value: 8,
  });
  const semanticPrepared = prepareRulesetCompilation({
    composition: composeFreshRuleset(
      ['rulebench.stormfront-balance'],
      options.version,
    ),
    packages: [...basePackages, rulesetPackageSource(semanticOverlay)],
  });
  if (!semanticPrepared.ok) {
    throw new Error(
      `fresh semantic overlay failed: ${canonicalJson(semanticPrepared.diagnostics)}`,
    );
  }
  const semanticOverlayProvenance =
    semanticPrepared.prepared.overlayProvenance[0];
  if (semanticOverlayProvenance === undefined) {
    throw new Error('fresh semantic overlay did not emit provenance');
  }
  const presentationOverlay = overlayPackage({
    id: 'rulebench.stormfront-presentation',
    version: options.version,
    targetPackageVersion: options.version,
    expectedFingerprint: semanticOverlayProvenance.afterFingerprint,
    plane: 'presentation',
    path: fieldPath('label'),
    value: 'Arc Lash: Stormfront',
  });
  return {
    composition: composeFreshRuleset(
      ['rulebench.stormfront-balance', 'rulebench.stormfront-presentation'],
      options.version,
    ),
    packages: Object.freeze([
      ...basePackages,
      rulesetPackageSource(semanticOverlay),
      rulesetPackageSource(presentationOverlay),
    ]),
  };
}

function authoredActions(
  stormDamageDefinitionId: string,
  arcLashDamageBonus: number,
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
            amount: add(
              dice({ count: 2, sides: 6 }),
              constant(arcLashDamageBonus),
            ),
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

function fieldManualPackage(
  stormDamageDefinitionId: string,
  options: {
    readonly version: string;
    readonly arcLashDamageBonus: number;
  },
) {
  const actions = authoredActions(
    stormDamageDefinitionId,
    options.arcLashDamageBonus,
  );
  const definitions = actions.map((authored) =>
    defineActionDefinition({
      kind: 'action',
      id: authored.id,
      visibility: 'public',
      extensionPolicy:
        authored.id === 'rulebench.arc-lash' ? 'derivable' : 'patchable',
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
  const derivedArcLash = defineDerivedDefinition({
    kind: 'derived',
    id: 'rulebench.arc-lash-stormfront',
    materializesAs: 'action',
    visibility: 'public',
    extensionPolicy: 'patchable',
    source: {
      module: 'packages/rulebench-field-manual.ts',
      declaration: 'arcLashStormfront',
    },
    references: [],
    presentation: { label: 'Arc Lash Stormfront' },
  });
  return defineRulesetPackage({
    identity: { id: 'rulebench.field-manual', version: options.version },
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
    definitions: [...definitions, derivedArcLash],
    exports: [...definitions, derivedArcLash].map(
      (definition) => definition.id,
    ),
    policyBindings: [],
    relationships: [
      defineRulesetRelationship({
        kind: 'derivesFrom',
        definitionId: derivedArcLash.id,
        target: definitionReference({ definitionId: 'rulebench.arc-lash' }),
        mixins: [
          {
            target: definitionReference({
              importAs: 'primitives',
              definitionId: 'rulebench.double-range',
            }),
            parameters: { factor: 2 },
          },
          {
            target: definitionReference({
              importAs: 'primitives',
              definitionId: 'rulebench.extend-range',
            }),
            parameters: { amount: 1 },
          },
        ],
        localPatch: {
          version: 1,
          operations: [
            {
              kind: 'setScalar',
              plane: 'presentation',
              path: fieldPath('description'),
              value:
                'Derived from Arc Lash through two ordered mixins and a local patch.',
            },
          ],
        },
        version: 1,
      }),
    ],
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

function composeFreshRuleset(
  overlays: readonly string[],
  fieldManualVersion: string,
): RulesetCompositionManifest {
  return composeRuleset({
    identity: { id: 'rulebench.fresh-start', version: fieldManualVersion },
    language: { id: 'asha-rpg', version: '^1.0.0' },
    base: rulesetPackageRequest({
      id: 'rulebench.field-manual',
      version: fieldManualVersion,
    }),
    add: [],
    overlays: overlays.map((id) =>
      rulesetPackageRequest({ id, version: fieldManualVersion }),
    ),
    configure: {},
  });
}

function overlayPackage(options: {
  readonly id: string;
  readonly version: string;
  readonly targetPackageVersion: string;
  readonly expectedFingerprint: string;
  readonly plane: 'semantic' | 'presentation';
  readonly path: ReturnType<typeof fieldPath>;
  readonly value: string | number | boolean;
}) {
  return defineRulesetPackage({
    identity: { id: options.id, version: options.version },
    entry: { module: `packages/${options.id}.ts`, declaration: 'default' },
    language: { id: 'asha-rpg', version: '^1.0.0' },
    dependencies: [
      rulesetDependency({
        id: 'rulebench.field-manual',
        version: options.targetPackageVersion,
        importAs: 'fieldManual',
      }),
    ],
    requirements: { operations: [], capabilities: [] },
    definitions: [],
    exports: [],
    policyBindings: [],
    relationships: [
      defineRulesetRelationship({
        kind: 'patches',
        definitionId: `${options.id}.patch`,
        target: definitionReference({
          importAs: 'fieldManual',
          definitionId: 'rulebench.arc-lash-stormfront',
        }),
        targetPackage: {
          id: 'rulebench.field-manual',
          version: options.targetPackageVersion,
        },
        expectedFingerprint: options.expectedFingerprint,
        patch: {
          version: 1,
          operations: [
            {
              kind: 'setScalar',
              plane: options.plane,
              path: options.path,
              value: options.value,
            },
          ],
        },
        plane: options.plane,
        conflictPolicy: 'reject',
        version: 1,
      }),
    ],
  });
}

function fieldPath(...names: readonly string[]) {
  return names.map((name) => ({ kind: 'field' as const, name }));
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
