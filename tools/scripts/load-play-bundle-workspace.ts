import { mkdir, mkdtemp, rm, stat } from 'node:fs/promises';
import { dirname, extname, join, relative, resolve, sep } from 'node:path';
import { pathToFileURL } from 'node:url';

import { canonicalJson, preparePlayBundle } from '@asha-rpg/authoring';
import type {
  ContentPackSource,
  PlayBundleCompilerDiagnostic,
  PlayBundleManifest,
  Ruleset,
  ScenarioTemplate,
} from '@asha-rpg/authoring';
import ts from 'typescript';

import {
  isContentPackSource,
  isPlayBundleManifest,
  isRuleset,
  isScenarioTemplate,
} from '../../libs/content-authoring/src/index.js';

export interface PlayBundleSelectionInput {
  readonly operation: 'inspect' | 'compile';
  readonly sourceSet: PlayBundleSourceSet;
  readonly contentPackIds?: readonly string[];
}

export type PlayBundleSourceExportKind =
  | 'ruleset'
  | 'contentPack'
  | 'playBundle'
  | 'scenarioTemplate';

export interface PlayBundleSourceEntry {
  readonly id: string;
  readonly label: string;
  readonly sourceRoot: string;
  readonly module: string;
  readonly exportKinds: readonly PlayBundleSourceExportKind[];
}

export interface PlayBundleSourceSet {
  readonly schemaVersion: 1;
  readonly allowedRoots: readonly string[];
  readonly entries: readonly PlayBundleSourceEntry[];
}

export interface RulesetSourceCatalog {
  readonly sourceSet: PlayBundleSourceSet;
  readonly ruleset: { readonly id: string; readonly version: string };
  readonly contentPacks: readonly {
    readonly id: string;
    readonly version: string;
    readonly label: string;
    readonly requirements: readonly string[];
  }[];
  readonly playBundles: readonly {
    readonly id: string;
    readonly version: string;
    readonly contentPackIds: readonly string[];
    readonly compatible: boolean;
    readonly diagnostics: readonly PlayBundleCompilerDiagnostic[];
  }[];
  readonly scenarios: readonly ReturnType<typeof catalogScenario>[];
}

export type PlayBundleWorkspaceLoadResult =
  | {
      readonly ok: true;
      readonly catalog: RulesetSourceCatalog;
      readonly preparedSource: string | null;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly PlayBundleCompilerDiagnostic[];
    };

type PlayBundleWorkspaceFailure = Extract<
  PlayBundleWorkspaceLoadResult,
  { readonly ok: false }
>;

interface ResolvedSourceEntry extends PlayBundleSourceEntry {
  readonly resolvedSourceRoot: string;
  readonly resolvedModule: string;
}

interface ResolvedSourceSet {
  readonly sourceSet: PlayBundleSourceSet;
  readonly resolvedAllowedRoots: readonly string[];
  readonly entries: readonly ResolvedSourceEntry[];
}

interface Located<Value> {
  readonly value: Value;
  readonly entry: ResolvedSourceEntry;
  readonly entryIndex: number;
}

const RESULT_PREFIX = 'RULEBENCH_PLAY_BUNDLE_RESULT:';

export async function loadPlayBundleWorkspace(
  input: unknown,
  gatewayRoot: string,
): Promise<PlayBundleWorkspaceLoadResult> {
  const resolved = await resolveSourceSetInput(input, gatewayRoot);
  if (!resolved.ok) return resolved;

  const buildRoot = await createBuildRoot(gatewayRoot);
  try {
    const compilerOptions: ts.CompilerOptions = {
      // Authoring sources are data modules. Emit one explicit ESM format instead
      // of inheriting each external checkout's nearest package.json mode and
      // then evaluating mixed CommonJS output inside Rulebench's ESM temp tree.
      // The emit transformer below also turns every resolved relative edge into
      // an exact output-file specifier so the graph follows Node's ESM contract.
      module: ts.ModuleKind.ES2022,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      target: ts.ScriptTarget.ES2022,
      rootDir: commonAncestor(resolved.value.resolvedAllowedRoots),
      outDir: buildRoot,
      strict: true,
      noEmitOnError: true,
      skipLibCheck: true,
      types: ['node'],
      baseUrl: resolve(gatewayRoot),
      paths: {
        '@asha-rpg/authoring': [
          'node_modules/@asha-rpg/authoring/dist/index.d.ts',
        ],
        '@asha-rpg/ir': ['node_modules/@asha-rpg/ir/dist/index.d.ts'],
      },
    };
    const program = ts.createProgram(
      resolved.value.entries.map((entry) => entry.resolvedModule),
      compilerOptions,
    );
    const authoredSourceFiles = program
      .getSourceFiles()
      .filter((sourceFile) => !sourceFile.isDeclarationFile);
    const escapedSource = authoredSourceFiles.find(
      (sourceFile) =>
        !isWithinAnyRoot(
          sourceFile.fileName,
          resolved.value.resolvedAllowedRoots,
        ),
    );
    if (escapedSource !== undefined) {
      return failure(
        'PLAY_BUNDLE_SOURCE_IMPORT_OUTSIDE_ALLOWED_ROOTS',
        '$.sourceSet.allowedRoots',
        `Imported source ${normalizedPath(escapedSource.fileName)} is outside the explicitly allowed roots`,
        resolved.value,
      );
    }
    const disallowedImport = authoredSourceFiles
      .map(disallowedModuleSpecifier)
      .find((specifier) => specifier !== null);
    if (disallowedImport !== undefined) {
      return failure(
        'PLAY_BUNDLE_SOURCE_IMPORT_NOT_ALLOWED',
        '$.sourceSet.entries',
        `Authoring modules may import only relative modules and published Asha RPG packages, not ${disallowedImport}`,
        resolved.value,
      );
    }

    const buildDiagnostics = ts.getPreEmitDiagnostics(program);
    if (buildDiagnostics.length > 0) {
      return {
        ok: false,
        diagnostics: buildDiagnostics.map((diagnostic) =>
          typescriptDiagnostic(diagnostic, resolved.value),
        ),
      };
    }
    const compilerRoot =
      compilerOptions.rootDir ??
      commonAncestor(resolved.value.resolvedAllowedRoots);
    const emittedSources = new Set(
      authoredSourceFiles.map((sourceFile) => resolve(sourceFile.fileName)),
    );
    const emit = program.emit(undefined, undefined, undefined, false, {
      before: [
        nodeEsmModuleSpecifierTransformer(
          compilerOptions,
          compilerRoot,
          buildRoot,
          emittedSources,
        ),
      ],
    });
    if (emit.emitSkipped || emit.diagnostics.length > 0) {
      return {
        ok: false,
        diagnostics: emit.diagnostics.map((diagnostic) =>
          typescriptDiagnostic(diagnostic, resolved.value),
        ),
      };
    }

    const locatedRulesets: Located<Ruleset>[] = [];
    const locatedContentPacks: Located<ContentPackSource>[] = [];
    const locatedPlayBundles: Located<PlayBundleManifest>[] = [];
    const locatedScenarios: Located<ScenarioTemplate>[] = [];
    for (const [entryIndex, entry] of resolved.value.entries.entries()) {
      const emittedModule = emittedModulePath(
        entry.resolvedModule,
        compilerOptions.rootDir ??
          commonAncestor(resolved.value.resolvedAllowedRoots),
        buildRoot,
      );
      let moduleNamespace: unknown;
      try {
        moduleNamespace = await import(
          `${pathToFileURL(emittedModule).href}?load=${Date.now()}`
        );
      } catch (error: unknown) {
        return failureForEntry(
          'PLAY_BUNDLE_SOURCE_EVALUATION_FAILED',
          `$.sourceSet.entries[${entryIndex}].module`,
          error instanceof Error ? error.message : String(error),
          entry,
        );
      }
      if (!isRecord(moduleNamespace)) {
        return failureForEntry(
          'PLAY_BUNDLE_SOURCE_MODULE_INVALID',
          `$.sourceSet.entries[${entryIndex}].module`,
          'The selected module did not expose an ES module namespace',
          entry,
        );
      }
      const discovery = discoverAuthoringValues(moduleNamespace);
      if (!discovery.ok) {
        return failureForEntry(
          'PLAY_BUNDLE_SOURCE_EXPORTED_IDENTITY_DUPLICATE',
          `$.sourceSet.entries[${entryIndex}]`,
          `Distinct exported ${discovery.kind} declarations in source ${entry.id} share identity ${discovery.identity}`,
          entry,
        );
      }
      const kindMismatch = firstUndeclaredExportKind(
        discovery.value,
        entry.exportKinds,
      );
      if (kindMismatch !== null) {
        return failureForEntry(
          'PLAY_BUNDLE_SOURCE_EXPORT_KIND_UNDECLARED',
          `$.sourceSet.entries[${entryIndex}].exportKinds`,
          `Source ${entry.id} exports ${kindMismatch}, but that kind is not declared`,
          entry,
        );
      }
      locatedRulesets.push(
        ...discovery.value.rulesets.map((value) => ({
          value,
          entry,
          entryIndex,
        })),
      );
      locatedContentPacks.push(
        ...discovery.value.contentPacks.map((value) => ({
          value,
          entry,
          entryIndex,
        })),
      );
      locatedPlayBundles.push(
        ...discovery.value.playBundles.map((value) => ({
          value,
          entry,
          entryIndex,
        })),
      );
      locatedScenarios.push(
        ...discovery.value.scenarios.map((value) => ({
          value,
          entry,
          entryIndex,
        })),
      );
    }
    const aggregate = aggregateLocatedValues({
      rulesets: locatedRulesets,
      contentPacks: locatedContentPacks,
      playBundles: locatedPlayBundles,
      scenarios: locatedScenarios,
    });
    if (!aggregate.ok) {
      return failureForEntry(
        'PLAY_BUNDLE_SOURCE_IDENTITY_DUPLICATE',
        `$.sourceSet.entries[${aggregate.current.entryIndex}]`,
        `Distinct ${aggregate.kind} declarations share identity ${aggregate.identity} in sources ${aggregate.previous.entry.id} and ${aggregate.current.entry.id}`,
        aggregate.current.entry,
      );
    }
    const discovered = aggregate.value;
    if (discovered.rulesets.length !== 1) {
      return failure(
        'PLAY_BUNDLE_SOURCE_RULESET_COUNT_INVALID',
        '$.sourceSet.entries',
        `Expected exactly one exported Ruleset across all sources, found ${discovered.rulesets.length}`,
        resolved.value,
      );
    }
    if (discovered.contentPacks.length === 0) {
      return failure(
        'PLAY_BUNDLE_SOURCE_CONTENT_PACK_REQUIRED',
        '$.sourceSet.entries',
        'The source set must export at least one Content Pack source',
        resolved.value,
      );
    }
    if (discovered.playBundles.length === 0) {
      return failure(
        'PLAY_BUNDLE_SOURCE_PLAY_BUNDLE_REQUIRED',
        '$.sourceSet.entries',
        'The source set must export at least one explicit PlayBundle',
        resolved.value,
      );
    }
    const ruleset = discovered.rulesets[0];
    if (ruleset === undefined) {
      return failure(
        'PLAY_BUNDLE_SOURCE_RULESET_COUNT_INVALID',
        '$.sourceSet.entries',
        'The source set did not expose a Ruleset',
        resolved.value,
      );
    }
    const mismatchedBundle = discovered.playBundles.find(
      (bundle) =>
        bundle.ruleset.identity.id !== ruleset.identity.id ||
        bundle.ruleset.identity.version !== ruleset.identity.version,
    );
    if (mismatchedBundle !== undefined) {
      return failure(
        'PLAY_BUNDLE_RULESET_MISMATCH',
        '$.sourceSet.entries',
        `PlayBundle ${mismatchedBundle.identity.id}@${mismatchedBundle.identity.version} does not use the selected Ruleset ${ruleset.identity.id}@${ruleset.identity.version}`,
        resolved.value,
      );
    }
    const mismatchedScenario = discovered.scenarios.find(
      (scenario) =>
        !discovered.playBundles.some(
          (bundle) =>
            bundle.identity.id === scenario.playBundle.id &&
            bundle.identity.version === scenario.playBundle.version,
        ),
    );
    if (mismatchedScenario !== undefined) {
      return failure(
        'SCENARIO_TEMPLATE_PLAY_BUNDLE_NOT_DECLARED',
        '$.sourceSet.entries',
        `Scenario template ${mismatchedScenario.identity.id}@${mismatchedScenario.identity.version} names undeclared PlayBundle ${mismatchedScenario.playBundle.id}@${mismatchedScenario.playBundle.version}`,
        resolved.value,
      );
    }
    const catalog = sourceCatalog(
      resolved.value.sourceSet,
      ruleset,
      discovered.contentPacks,
      discovered.playBundles,
      discovered.scenarios,
    );
    if (inputOperation(input) === 'inspect') {
      return { ok: true, catalog, preparedSource: null, diagnostics: [] };
    }
    const selectedIds = selectedContentPackIds(input);
    if (!selectedIds.ok) return selectedIds;
    const matchingBundles = discovered.playBundles.filter((bundle) =>
      sameStrings(bundleContentPackIds(bundle), selectedIds.value),
    );
    if (matchingBundles.length !== 1) {
      return failure(
        'PLAY_BUNDLE_SELECTION_NOT_DECLARED',
        '$.contentPackIds',
        `Selected Content Packs must match exactly one declared PlayBundle; matched ${matchingBundles.length}`,
        resolved.value,
      );
    }
    const bundle = matchingBundles[0];
    if (bundle === undefined) {
      return failure(
        'PLAY_BUNDLE_SELECTION_NOT_DECLARED',
        '$.contentPackIds',
        'Selected Content Packs do not match a declared PlayBundle',
        resolved.value,
      );
    }
    const prepared = preparePlayBundle({
      bundle,
      contentPacks: discovered.contentPacks,
    });
    if (!prepared.ok) return prepared;
    return {
      ok: true,
      catalog,
      preparedSource: canonicalJson(prepared.prepared),
      diagnostics: [],
    };
  } finally {
    await rm(buildRoot, { recursive: true, force: true });
  }
}

function firstUndeclaredExportKind(
  discovered: {
    readonly rulesets: readonly Ruleset[];
    readonly contentPacks: readonly ContentPackSource[];
    readonly playBundles: readonly PlayBundleManifest[];
    readonly scenarios: readonly ScenarioTemplate[];
  },
  allowed: readonly PlayBundleSourceExportKind[],
): PlayBundleSourceExportKind | null {
  const checks: readonly [PlayBundleSourceExportKind, number][] = [
    ['ruleset', discovered.rulesets.length],
    ['contentPack', discovered.contentPacks.length],
    ['playBundle', discovered.playBundles.length],
    ['scenarioTemplate', discovered.scenarios.length],
  ];
  return (
    checks.find(([kind, count]) => count > 0 && !allowed.includes(kind))?.[0] ??
    null
  );
}

function aggregateLocatedValues(input: {
  readonly rulesets: readonly Located<Ruleset>[];
  readonly contentPacks: readonly Located<ContentPackSource>[];
  readonly playBundles: readonly Located<PlayBundleManifest>[];
  readonly scenarios: readonly Located<ScenarioTemplate>[];
}):
  | {
      readonly ok: true;
      readonly value: {
        readonly rulesets: readonly Ruleset[];
        readonly contentPacks: readonly ContentPackSource[];
        readonly playBundles: readonly PlayBundleManifest[];
        readonly scenarios: readonly ScenarioTemplate[];
      };
    }
  | {
      readonly ok: false;
      readonly kind:
        | 'Ruleset'
        | 'Content Pack'
        | 'PlayBundle'
        | 'ScenarioTemplate';
      readonly identity: string;
      readonly previous: Located<unknown>;
      readonly current: Located<unknown>;
    } {
  const duplicateRuleset = duplicateLocatedIdentity(
    input.rulesets,
    (located) => located.value.identity,
  );
  if (duplicateRuleset !== null)
    return { ok: false, kind: 'Ruleset', ...duplicateRuleset };
  const duplicateContentPack = duplicateLocatedIdentity(
    input.contentPacks,
    (located) => located.value.manifest.identity,
  );
  if (duplicateContentPack !== null)
    return { ok: false, kind: 'Content Pack', ...duplicateContentPack };
  const duplicatePlayBundle = duplicateLocatedIdentity(
    input.playBundles,
    (located) => located.value.identity,
  );
  if (duplicatePlayBundle !== null)
    return { ok: false, kind: 'PlayBundle', ...duplicatePlayBundle };
  const duplicateScenario = duplicateLocatedIdentity(
    input.scenarios,
    (located) => located.value.identity,
  );
  if (duplicateScenario !== null)
    return { ok: false, kind: 'ScenarioTemplate', ...duplicateScenario };
  return {
    ok: true,
    value: {
      rulesets: uniqueByIdentity(
        input.rulesets.map(({ value }) => value),
        (value) => value.identity,
      ),
      contentPacks: uniqueByIdentity(
        input.contentPacks.map(({ value }) => value),
        (value) => value.manifest.identity,
      ),
      playBundles: uniqueByIdentity(
        input.playBundles.map(({ value }) => value),
        (value) => value.identity,
      ),
      scenarios: uniqueByIdentity(
        input.scenarios.map(({ value }) => value),
        (value) => value.identity,
      ),
    },
  };
}

function duplicateLocatedIdentity<Value>(
  values: readonly Located<Value>[],
  identity: (value: Located<Value>) => {
    readonly id: string;
    readonly version: string;
  },
): {
  readonly identity: string;
  readonly previous: Located<Value>;
  readonly current: Located<Value>;
} | null {
  const seen = new Map<string, Located<Value>>();
  for (const located of values) {
    const currentIdentity = identity(located);
    const key = currentIdentity.id;
    const previous = seen.get(key);
    if (previous !== undefined && previous.value !== located.value) {
      const previousIdentity = identity(previous);
      return {
        identity: `${key} at versions ${previousIdentity.version} and ${currentIdentity.version}`,
        previous,
        current: located,
      };
    }
    seen.set(key, located);
  }
  return null;
}

function discoverAuthoringValues(
  moduleNamespace: Readonly<Record<string, unknown>>,
):
  | {
      readonly ok: true;
      readonly value: {
        readonly rulesets: readonly Ruleset[];
        readonly contentPacks: readonly ContentPackSource[];
        readonly playBundles: readonly PlayBundleManifest[];
        readonly scenarios: readonly ScenarioTemplate[];
      };
    }
  | {
      readonly ok: false;
      readonly kind:
        | 'Ruleset'
        | 'Content Pack'
        | 'PlayBundle'
        | 'ScenarioTemplate';
      readonly identity: string;
    } {
  const values = Object.values(moduleNamespace);
  const rulesets = values.filter(isRuleset);
  const contentPacks = values.filter(isContentPackSource);
  const playBundles = values.filter(isPlayBundleManifest);
  const scenarios = values.filter(isScenarioTemplate);

  const duplicateRuleset = duplicateIdentity(
    rulesets,
    (value) => value.identity,
  );
  if (duplicateRuleset !== null) {
    return { ok: false, kind: 'Ruleset', identity: duplicateRuleset };
  }
  const duplicateContentPack = duplicateIdentity(
    contentPacks,
    (value) => value.manifest.identity,
  );
  if (duplicateContentPack !== null) {
    return { ok: false, kind: 'Content Pack', identity: duplicateContentPack };
  }
  const duplicatePlayBundle = duplicateIdentity(
    playBundles,
    (value) => value.identity,
  );
  if (duplicatePlayBundle !== null) {
    return { ok: false, kind: 'PlayBundle', identity: duplicatePlayBundle };
  }
  const duplicateScenario = duplicateIdentity(
    scenarios,
    (value) => value.identity,
  );
  if (duplicateScenario !== null) {
    return { ok: false, kind: 'ScenarioTemplate', identity: duplicateScenario };
  }

  return {
    ok: true,
    value: {
      rulesets: uniqueByIdentity(rulesets, (value) => value.identity),
      contentPacks: uniqueByIdentity(
        contentPacks,
        (value) => value.manifest.identity,
      ),
      playBundles: uniqueByIdentity(playBundles, (value) => value.identity),
      scenarios: uniqueByIdentity(scenarios, (value) => value.identity),
    },
  };
}

function duplicateIdentity<Value>(
  values: readonly Value[],
  identity: (value: Value) => { readonly id: string; readonly version: string },
): string | null {
  const discovered = new Map<
    string,
    { readonly value: Value; readonly version: string }
  >();
  for (const value of values) {
    const current = identity(value);
    const key = current.id;
    const previous = discovered.get(key);
    if (previous !== undefined && previous.value !== value) {
      return `${key} at versions ${previous.version} and ${current.version}`;
    }
    discovered.set(key, { value, version: current.version });
  }
  return null;
}

function uniqueByIdentity<Value>(
  values: readonly Value[],
  identity: (value: Value) => { readonly id: string; readonly version: string },
): readonly Value[] {
  const unique = new Map<string, Value>();
  for (const value of values) {
    const current = identity(value);
    unique.set(`${current.id}@${current.version}`, value);
  }
  return [...unique.values()].sort((left, right) => {
    const leftIdentity = identity(left);
    const rightIdentity = identity(right);
    return `${leftIdentity.id}@${leftIdentity.version}`.localeCompare(
      `${rightIdentity.id}@${rightIdentity.version}`,
    );
  });
}

function sourceCatalog(
  sourceSet: PlayBundleSourceSet,
  ruleset: Ruleset,
  contentPacks: readonly ContentPackSource[],
  playBundles: readonly PlayBundleManifest[],
  scenarios: readonly ScenarioTemplate[],
): RulesetSourceCatalog {
  return {
    sourceSet,
    ruleset: { id: ruleset.identity.id, version: ruleset.identity.version },
    contentPacks: contentPacks.map((source) => ({
      id: source.manifest.identity.id,
      version: source.manifest.identity.version,
      label: readableId(source.manifest.identity.id),
      requirements: contentPackRequirements(source),
    })),
    playBundles: playBundles.map((bundle) => {
      const preparation = preparePlayBundle({ bundle, contentPacks });
      return {
        id: bundle.identity.id,
        version: bundle.identity.version,
        contentPackIds: bundleContentPackIds(bundle),
        compatible: preparation.ok,
        diagnostics: preparation.diagnostics,
      };
    }),
    scenarios: scenarios.map(catalogScenario),
  };
}

function catalogScenario(template: ScenarioTemplate) {
  return {
    schema: template.schema,
    identity: template.identity,
    playBundle: template.playBundle,
    presentation: {
      label: template.presentation.label,
      description: template.presentation.description ?? null,
    },
    board: {
      ...template.board,
      cells: template.board.cells.map((cell) => ({
        ...cell,
        capabilities: cell.capabilities.map((capability) => ({
          ...capability,
          definitionId: capability.definitionId ?? null,
        })),
      })),
    },
    participants: template.participants,
    turn: template.turn,
    randomSource: template.randomSource,
  };
}

function contentPackRequirements(source: ContentPackSource): readonly string[] {
  const requirements = source.manifest.requirements;
  return [
    ...requirements.operations.map((value) => `${value.id}@${value.version}`),
    ...requirements.capabilities.map((value) => `${value.id}@${value.version}`),
    ...requirements.values.map((value) => `${value.kind}:${value.id}`),
    ...requirements.numericDomains.map((value) => `numeric-domain:${value}`),
  ].sort((left, right) => left.localeCompare(right));
}

function readableId(id: string): string {
  const segment = id.split('.').at(-1) ?? id;
  return segment
    .split(/[-_]/)
    .filter((part) => part.length > 0)
    .map((part) => `${part[0]?.toUpperCase() ?? ''}${part.slice(1)}`)
    .join(' ');
}

function bundleContentPackIds(bundle: PlayBundleManifest): readonly string[] {
  return [
    ...new Set([
      bundle.base.id,
      ...bundle.add.map((request) => request.id),
      ...bundle.overlays.map((request) => request.id),
    ]),
  ].sort((left, right) => left.localeCompare(right));
}

function inputOperation(input: unknown): 'inspect' | 'compile' | null {
  if (!isRecord(input)) return null;
  const operation = input['operation'];
  return operation === 'inspect' || operation === 'compile' ? operation : null;
}

function selectedContentPackIds(
  input: unknown,
):
  | { readonly ok: true; readonly value: readonly string[] }
  | PlayBundleWorkspaceFailure {
  if (!isRecord(input) || !Array.isArray(input['contentPackIds'])) {
    return failureWithoutSource(
      'PLAY_BUNDLE_CONTENT_PACK_SELECTION_INVALID',
      '$.contentPackIds',
      'contentPackIds must be an array of unique non-empty Content Pack IDs',
    );
  }
  const values = input['contentPackIds'];
  if (
    values.some(
      (value) => typeof value !== 'string' || value.trim().length === 0,
    )
  ) {
    return failureWithoutSource(
      'PLAY_BUNDLE_CONTENT_PACK_SELECTION_INVALID',
      '$.contentPackIds',
      'contentPackIds must contain only non-empty strings',
    );
  }
  const normalized = values.map((value) => value.trim());
  if (new Set(normalized).size !== normalized.length) {
    return failureWithoutSource(
      'PLAY_BUNDLE_CONTENT_PACK_SELECTION_DUPLICATE',
      '$.contentPackIds',
      'contentPackIds must not contain duplicates',
    );
  }
  return {
    ok: true,
    value: normalized.sort((left, right) => left.localeCompare(right)),
  };
}

function sameStrings(
  left: readonly string[],
  right: readonly string[],
): boolean {
  return (
    left.length === right.length &&
    left.every((value, index) => value === right[index])
  );
}

async function resolveSourceSetInput(
  input: unknown,
  gatewayRoot: string,
): Promise<
  | { readonly ok: true; readonly value: ResolvedSourceSet }
  | PlayBundleWorkspaceFailure
> {
  if (!isRecord(input)) {
    return failureWithoutSource(
      'PLAY_BUNDLE_WORKSPACE_INPUT_INVALID',
      '$',
      'Workspace input must be an object',
    );
  }
  const operation = inputOperation(input);
  if (operation === null) {
    return failureWithoutSource(
      'PLAY_BUNDLE_WORKSPACE_OPERATION_INVALID',
      '$.operation',
      'operation must be inspect or compile',
    );
  }
  const exactKeys =
    operation === 'inspect'
      ? ['operation', 'sourceSet']
      : ['operation', 'sourceSet', 'contentPackIds'];
  if (
    Object.keys(input).length !== exactKeys.length ||
    !exactKeys.every((key) => key in input)
  ) {
    return failureWithoutSource(
      'PLAY_BUNDLE_WORKSPACE_INPUT_INVALID',
      '$',
      `Workspace input must contain exactly ${exactKeys.join(', ')}`,
    );
  }
  const decoded = decodeSourceSet(input['sourceSet']);
  if (!decoded.ok) return decoded;
  const sourceSet = decoded.value;
  const resolvedAllowedRoots = sourceSet.allowedRoots.map((root) =>
    resolve(gatewayRoot, root),
  );
  const entries = sourceSet.entries.map((entry) => ({
    ...entry,
    resolvedSourceRoot: resolve(gatewayRoot, entry.sourceRoot),
    resolvedModule: resolve(gatewayRoot, entry.sourceRoot, entry.module),
  }));
  const outsideAllowedRoot = entries.find(
    (entry) => !isWithinAnyRoot(entry.resolvedSourceRoot, resolvedAllowedRoots),
  );
  if (outsideAllowedRoot !== undefined) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_ROOT_NOT_ALLOWED',
      `$.sourceSet.entries[${entries.indexOf(outsideAllowedRoot)}].sourceRoot`,
      `Source root ${outsideAllowedRoot.sourceRoot} is outside allowedRoots`,
    );
  }
  const escapedModule = entries.find(
    (entry) => !isWithinRoot(entry.resolvedModule, entry.resolvedSourceRoot),
  );
  if (escapedModule !== undefined) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_MODULE_OUTSIDE_ROOT',
      `$.sourceSet.entries[${entries.indexOf(escapedModule)}].module`,
      `Entry module ${escapedModule.module} is outside source root ${escapedModule.sourceRoot}`,
    );
  }
  const resolvedValue: ResolvedSourceSet = {
    sourceSet,
    resolvedAllowedRoots,
    entries,
  };
  try {
    for (const allowedRoot of resolvedAllowedRoots) {
      const rootStat = await stat(allowedRoot);
      if (!rootStat.isDirectory())
        throw new Error(`${allowedRoot} is not a directory`);
    }
    for (const entry of entries) {
      const rootStat = await stat(entry.resolvedSourceRoot);
      if (!rootStat.isDirectory())
        throw new Error(`${entry.resolvedSourceRoot} is not a directory`);
      const moduleStat = await stat(entry.resolvedModule);
      if (!moduleStat.isFile())
        throw new Error(`${entry.resolvedModule} is not a file`);
    }
  } catch (error: unknown) {
    return failure(
      'PLAY_BUNDLE_SOURCE_ENTRY_NOT_FOUND',
      '$.sourceSet',
      error instanceof Error ? error.message : String(error),
      resolvedValue,
    );
  }
  return { ok: true, value: resolvedValue };
}

function decodeSourceSet(
  value: unknown,
):
  | { readonly ok: true; readonly value: PlayBundleSourceSet }
  | PlayBundleWorkspaceFailure {
  if (
    !isRecord(value) ||
    !hasExactKeys(value, ['schemaVersion', 'allowedRoots', 'entries'])
  ) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_SET_INVALID',
      '$.sourceSet',
      'sourceSet must contain exactly schemaVersion, allowedRoots, entries',
    );
  }
  if (value['schemaVersion'] !== 1) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_SET_VERSION_UNSUPPORTED',
      '$.sourceSet.schemaVersion',
      'sourceSet.schemaVersion must be 1',
    );
  }
  const roots = decodeUniqueStrings(value['allowedRoots']);
  if (roots === null || roots.length === 0) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_ALLOWED_ROOTS_INVALID',
      '$.sourceSet.allowedRoots',
      'allowedRoots must be a non-empty array of unique non-empty paths',
    );
  }
  if (!Array.isArray(value['entries']) || value['entries'].length === 0) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_ENTRIES_INVALID',
      '$.sourceSet.entries',
      'entries must be a non-empty array',
    );
  }
  const entries: PlayBundleSourceEntry[] = [];
  const ids = new Set<string>();
  for (const [index, candidate] of value['entries'].entries()) {
    const path = `$.sourceSet.entries[${index}]`;
    if (
      !isRecord(candidate) ||
      !hasExactKeys(candidate, [
        'id',
        'label',
        'sourceRoot',
        'module',
        'exportKinds',
      ])
    ) {
      return failureWithoutSource(
        'PLAY_BUNDLE_SOURCE_ENTRY_INVALID',
        path,
        'Each entry must contain exactly id, label, sourceRoot, module, exportKinds',
      );
    }
    const id = nonEmptyString(candidate['id']);
    const label = nonEmptyString(candidate['label']);
    const sourceRoot = nonEmptyString(candidate['sourceRoot']);
    const module = nonEmptyString(candidate['module']);
    const exportKinds = decodeExportKinds(candidate['exportKinds']);
    if (
      id === null ||
      label === null ||
      sourceRoot === null ||
      module === null ||
      exportKinds === null ||
      exportKinds.length === 0
    ) {
      return failureWithoutSource(
        'PLAY_BUNDLE_SOURCE_ENTRY_INVALID',
        path,
        'Entry fields must be non-empty and exportKinds must contain unique supported kinds',
      );
    }
    if (ids.has(id)) {
      return failureWithoutSource(
        'PLAY_BUNDLE_SOURCE_ENTRY_ID_DUPLICATE',
        `${path}.id`,
        `Source entry ID ${id} is duplicated`,
      );
    }
    ids.add(id);
    entries.push({ id, label, sourceRoot, module, exportKinds });
  }
  const rulesetEntries = entries.filter((entry) =>
    entry.exportKinds.includes('ruleset'),
  );
  if (rulesetEntries.length !== 1) {
    return failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_RULESET_ENTRY_COUNT_INVALID',
      '$.sourceSet.entries',
      `Exactly one source entry must declare ruleset exports; found ${rulesetEntries.length}`,
    );
  }
  return {
    ok: true,
    value: { schemaVersion: 1, allowedRoots: roots, entries },
  };
}

function decodeExportKinds(
  value: unknown,
): readonly PlayBundleSourceExportKind[] | null {
  const strings = decodeUniqueStrings(value);
  if (strings === null) return null;
  const kinds: PlayBundleSourceExportKind[] = [];
  for (const value of strings) {
    const kind = sourceExportKind(value);
    if (kind === null) return null;
    kinds.push(kind);
  }
  return kinds;
}

function decodeUniqueStrings(value: unknown): readonly string[] | null {
  if (!Array.isArray(value)) return null;
  const strings: string[] = [];
  for (const candidate of value) {
    const decoded = nonEmptyString(candidate);
    if (decoded === null) return null;
    strings.push(decoded);
  }
  if (new Set(strings).size !== strings.length) return null;
  return strings;
}

function sourceExportKind(value: string): PlayBundleSourceExportKind | null {
  if (
    value === 'ruleset' ||
    value === 'contentPack' ||
    value === 'playBundle' ||
    value === 'scenarioTemplate'
  ) {
    return value;
  }
  return null;
}

function nonEmptyString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0
    ? value.trim()
    : null;
}

function hasExactKeys(
  value: Readonly<Record<string, unknown>>,
  expected: readonly string[],
): boolean {
  return (
    Object.keys(value).length === expected.length &&
    expected.every((key) => key in value)
  );
}

function typescriptDiagnostic(
  diagnostic: ts.Diagnostic,
  input: ResolvedSourceSet,
): PlayBundleCompilerDiagnostic {
  const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n');
  if (diagnostic.file === undefined || diagnostic.start === undefined) {
    return diagnosticValue(
      'PLAY_BUNDLE_SOURCE_BUILD_FAILED',
      '$.sourceSet.entries',
      `TS${diagnostic.code}: ${message}`,
      input,
    );
  }
  const position = diagnostic.file.getLineAndCharacterOfPosition(
    diagnostic.start,
  );
  const sourceEntry = input.entries.find((entry) =>
    isWithinRoot(diagnostic.file?.fileName ?? '', entry.resolvedSourceRoot),
  );
  const sourcePath =
    sourceEntry === undefined
      ? normalizedPath(diagnostic.file.fileName)
      : normalizedPath(
          relative(sourceEntry.resolvedSourceRoot, diagnostic.file.fileName),
        );
  return diagnosticValue(
    'PLAY_BUNDLE_SOURCE_BUILD_FAILED',
    `${sourcePath}:${position.line + 1}:${position.character + 1}`,
    `TS${diagnostic.code}: ${message}`,
    sourceEntry ?? input,
  );
}

function failure(
  code: string,
  path: string,
  message: string,
  input: ResolvedSourceSet,
): PlayBundleWorkspaceFailure {
  return {
    ok: false,
    diagnostics: [diagnosticValue(code, path, message, input)],
  };
}

function failureForEntry(
  code: string,
  path: string,
  message: string,
  entry: ResolvedSourceEntry,
): PlayBundleWorkspaceFailure {
  return {
    ok: false,
    diagnostics: [diagnosticValue(code, path, message, entry)],
  };
}

function failureWithoutSource(
  code: string,
  path: string,
  message: string,
): PlayBundleWorkspaceFailure {
  return {
    ok: false,
    diagnostics: [
      {
        stage: 'source',
        severity: 'error',
        code,
        path,
        message,
      },
    ],
  };
}

function diagnosticValue(
  code: string,
  path: string,
  message: string,
  input: ResolvedSourceSet | ResolvedSourceEntry,
): PlayBundleCompilerDiagnostic {
  const entry = 'entries' in input ? input.entries[0] : input;
  return {
    stage: 'source',
    severity: 'error',
    code,
    path,
    message,
    ...(entry === undefined
      ? {}
      : {
          source: {
            module: normalizedPath(join(entry.sourceRoot, entry.module)),
            declaration: `source entry ${entry.id}`,
          },
        }),
  };
}

function isWithinAnyRoot(path: string, roots: readonly string[]): boolean {
  return roots.some((root) => isWithinRoot(path, root));
}

function disallowedModuleSpecifier(sourceFile: ts.SourceFile): string | null {
  let disallowed: string | null = null;
  const visit = (node: ts.Node): void => {
    if (disallowed !== null) return;
    if (
      (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
      node.moduleSpecifier !== undefined &&
      ts.isStringLiteral(node.moduleSpecifier)
    ) {
      const specifier = node.moduleSpecifier.text;
      if (
        specifier !== '@asha-rpg/authoring' &&
        specifier !== '@asha-rpg/ir' &&
        !specifier.startsWith('./') &&
        !specifier.startsWith('../')
      ) {
        disallowed = specifier;
        return;
      }
    }
    if (
      ts.isImportEqualsDeclaration(node) ||
      (ts.isCallExpression(node) &&
        (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
          (ts.isIdentifier(node.expression) &&
            node.expression.text === 'require')))
    ) {
      disallowed = 'dynamic or require-style import';
      return;
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return disallowed;
}

function nodeEsmModuleSpecifierTransformer(
  compilerOptions: ts.CompilerOptions,
  compilerRoot: string,
  outputRoot: string,
  emittedSources: ReadonlySet<string>,
): ts.TransformerFactory<ts.SourceFile> {
  return (context) => {
    const visit: ts.Visitor = (node) => {
      if (
        ts.isImportDeclaration(node) &&
        ts.isStringLiteral(node.moduleSpecifier)
      ) {
        const specifier = executableRelativeModuleSpecifier(
          node.getSourceFile().fileName,
          node.moduleSpecifier.text,
          compilerOptions,
          compilerRoot,
          outputRoot,
          emittedSources,
        );
        if (specifier !== null) {
          return context.factory.updateImportDeclaration(
            node,
            node.modifiers,
            node.importClause,
            context.factory.createStringLiteral(specifier),
            node.attributes,
          );
        }
      }
      if (
        ts.isExportDeclaration(node) &&
        node.moduleSpecifier !== undefined &&
        ts.isStringLiteral(node.moduleSpecifier)
      ) {
        const specifier = executableRelativeModuleSpecifier(
          node.getSourceFile().fileName,
          node.moduleSpecifier.text,
          compilerOptions,
          compilerRoot,
          outputRoot,
          emittedSources,
        );
        if (specifier !== null) {
          return context.factory.updateExportDeclaration(
            node,
            node.modifiers,
            node.isTypeOnly,
            node.exportClause,
            context.factory.createStringLiteral(specifier),
            node.attributes,
          );
        }
      }
      return ts.visitEachChild(node, visit, context);
    };

    return (sourceFile) => ts.visitEachChild(sourceFile, visit, context);
  };
}

function executableRelativeModuleSpecifier(
  sourceFileName: string,
  authoredSpecifier: string,
  compilerOptions: ts.CompilerOptions,
  compilerRoot: string,
  outputRoot: string,
  emittedSources: ReadonlySet<string>,
): string | null {
  if (
    !authoredSpecifier.startsWith('./') &&
    !authoredSpecifier.startsWith('../')
  ) {
    return null;
  }
  const resolution = ts.resolveModuleName(
    authoredSpecifier,
    sourceFileName,
    compilerOptions,
    ts.sys,
  ).resolvedModule;
  if (resolution === undefined) return null;

  const resolvedTarget = resolve(resolution.resolvedFileName);
  if (!emittedSources.has(resolvedTarget)) return null;

  const sourceOutput = emittedModulePath(
    sourceFileName,
    compilerRoot,
    outputRoot,
  );
  const targetOutput = emittedModulePath(
    resolvedTarget,
    compilerRoot,
    outputRoot,
  );
  const outputRelative = normalizedPath(
    relative(dirname(sourceOutput), targetOutput),
  );
  return outputRelative.startsWith('.')
    ? outputRelative
    : `./${outputRelative}`;
}

function isWithinRoot(path: string, root: string): boolean {
  const child = relative(resolve(root), resolve(path));
  return child === '' || (!child.startsWith(`..${sep}`) && child !== '..');
}

function commonAncestor(paths: readonly string[]): string {
  let ancestor = resolve(paths[0] ?? process.cwd());
  for (const path of paths.slice(1)) {
    const candidate = resolve(path);
    while (!isWithinRoot(candidate, ancestor)) {
      const parent = dirname(ancestor);
      if (parent === ancestor) return ancestor;
      ancestor = parent;
    }
  }
  return ancestor;
}

function emittedModulePath(
  modulePath: string,
  rootDir: string,
  outputRoot: string,
): string {
  const relativeModule = relative(rootDir, modulePath);
  const extension = extname(relativeModule);
  const outputExtension =
    extension === '.mts' ? '.mjs' : extension === '.cts' ? '.cjs' : '.js';
  return join(
    outputRoot,
    `${relativeModule.slice(0, -extension.length)}${outputExtension}`,
  );
}

function normalizedPath(path: string): string {
  return path.split(sep).join('/');
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

async function createBuildRoot(gatewayRoot: string): Promise<string> {
  const parent = join(resolve(gatewayRoot), 'tmp', 'play-bundle-sources');
  await mkdir(parent, { recursive: true });
  return mkdtemp(join(parent, 'build-'));
}

async function readStandardInput(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString('utf8');
}

async function run(): Promise<void> {
  let input: unknown;
  try {
    input = JSON.parse(await readStandardInput());
  } catch (error: unknown) {
    const result = failureWithoutSource(
      'PLAY_BUNDLE_SOURCE_INPUT_INVALID',
      '$',
      error instanceof Error ? error.message : String(error),
    );
    process.stdout.write(`${RESULT_PREFIX}${JSON.stringify(result)}\n`);
    return;
  }
  const result = await loadPlayBundleWorkspace(input, process.cwd());
  process.stdout.write(`${RESULT_PREFIX}${JSON.stringify(result)}\n`);
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await run();
}

export function parseWorkspaceLoaderOutput(output: string): unknown {
  const line = output
    .split('\n')
    .reverse()
    .find((candidate) => candidate.startsWith(RESULT_PREFIX));
  if (line === undefined) {
    throw new Error('authoring subprocess did not return a workspace result');
  }
  return JSON.parse(line.slice(RESULT_PREFIX.length));
}
