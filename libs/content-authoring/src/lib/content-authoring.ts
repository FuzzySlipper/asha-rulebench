import {
  action,
  actionPatch,
  actionId,
  add,
  applyModifier,
  attack,
  canonicalJson,
  compare,
  composeRuleset,
  constant,
  damage,
  defineActionDefinition,
  defineMixinDefinition,
  defineRulesetCatalog,
  defineRulesetOverlay,
  defineRulesetPackage,
  definitionReference,
  deriveAction,
  dice,
  hostile,
  moveEntity,
  noRoll,
  onCheck,
  openReaction,
  patchParameter,
  prepareRulesetCompilation,
  reactionId,
  reactionOptionId,
  readStat,
  refresh,
  rulesetDependency,
  rulesetPackageRequest,
  rulesetPackageSource,
  sequence,
  spend,
  stackingGroup,
  turns,
  when,
} from '@asha-rpg/authoring';
import type {
  AuthoredAction,
  PrepareRulesetResult,
  RulesetCatalogReference,
  RulesetCompilerDiagnostic,
  RulesetCompositionManifest,
  RulesetPatch,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

const primitivesCatalog = defineRulesetCatalog({
  packageId: 'rulebench.primitives',
  sourceModule: 'packages/rulebench-primitives.ts',
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

const missingDamageCatalog = defineRulesetCatalog({
  packageId: 'rulebench.primitives',
  sourceModule: 'packages/rulebench-invalid.ts',
  entries: {
    missing: {
      definitionId: 'catalog.damage.missing',
      category: 'damageType',
      id: 'missing',
      label: 'Missing damage',
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
      module: 'packages/rulebench-primitives.ts',
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
      module: 'packages/rulebench-primitives.ts',
      declaration: 'extendRange',
    },
    parameters: [{ id: 'amount', type: 'number', default: 1 }],
    patch: actionPatch.semantic.maximumRange.adjust({
      add: patchParameter('amount'),
    }),
  }),
]);

const primitivesPackage = defineRulesetPackage({
  identity: { id: 'rulebench.primitives', version: '1.0.0' },
  entry: {
    module: 'packages/rulebench-primitives.ts',
    declaration: 'default',
  },
  definitions: [...primitivesCatalog.definitions, ...mixinDefinitions],
});

const freshVariant = buildFreshRulesetVariant({
  version: '1.0.0',
  arcLashDamageBonus: 1,
});
const upgradeVariant = buildFreshRulesetVariant({
  version: '1.1.0',
  arcLashDamageBonus: 2,
});
const invalidFieldManual = fieldManualPackage(
  missingDamageCatalog.references.missing,
  {
    version: '1.0.0',
    arcLashDamageBonus: 1,
  },
);

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
  const fieldManual = fieldManualPackage(
    primitivesCatalog.references.storm,
    options,
  );
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
    patch: actionPatch.semantic.maximumRange.set(8),
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
    patch: actionPatch.presentation.label.set('Arc Lash: Stormfront'),
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
  storm: RulesetCatalogReference<'damageType', 'rulebench.primitives'>,
  arcLashDamageBonus: number,
): readonly AuthoredAction[] {
  const { power, guard, focus, exposed } = primitivesCatalog.references;

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
            modifier: exposed,
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
  storm: RulesetCatalogReference<'damageType', 'rulebench.primitives'>,
  options: {
    readonly version: string;
    readonly arcLashDamageBonus: number;
  },
) {
  const actions = authoredActions(storm, options.arcLashDamageBonus);
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
      presentation: {
        label: authored.name,
        description: 'Fresh TypeScript content compiled into Rust authority.',
        tags: ['fresh', 'playable'],
      },
      action: authored,
    }),
  );
  const derivedArcLash = deriveAction({
    id: 'rulebench.arc-lash-stormfront',
    visibility: 'public',
    extensionPolicy: 'patchable',
    source: {
      module: 'packages/rulebench-field-manual.ts',
      declaration: 'arcLashStormfront',
    },
    presentation: { label: 'Arc Lash Stormfront' },
    base: definitionReference({ definitionId: 'rulebench.arc-lash' }),
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
    patch: actionPatch.presentation.description.set(
      'Derived from Arc Lash through two ordered mixins and a local patch.',
    ),
  });
  return defineRulesetPackage({
    identity: { id: 'rulebench.field-manual', version: options.version },
    entry: {
      module: 'packages/rulebench-field-manual.ts',
      declaration: 'default',
    },
    dependencies: [
      rulesetDependency({
        id: 'rulebench.primitives',
        version: '1.0.0',
        importAs: 'primitives',
      }),
    ],
    definitions: [...definitions, derivedArcLash.definition],
    relationships: [derivedArcLash.relationship],
  });
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
  readonly patch: RulesetPatch;
}) {
  return defineRulesetPackage({
    identity: { id: options.id, version: options.version },
    entry: { module: `packages/${options.id}.ts`, declaration: 'default' },
    dependencies: [
      rulesetDependency({
        id: 'rulebench.field-manual',
        version: options.targetPackageVersion,
        importAs: 'fieldManual',
      }),
    ],
    definitions: [],
    relationships: [
      defineRulesetOverlay({
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
        patch: options.patch,
      }),
    ],
  });
}
