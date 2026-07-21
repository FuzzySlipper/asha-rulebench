import { mkdir, mkdtemp, rm, stat } from 'node:fs/promises';
import {
  basename,
  dirname,
  extname,
  join,
  relative,
  resolve,
  sep,
} from 'node:path';
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
  readonly rulesetRoot: string;
  readonly contentPackIds?: readonly string[];
}

export interface RulesetRootCatalog {
  readonly rulesetRoot: string;
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
      readonly catalog: RulesetRootCatalog;
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

interface ResolvedRulesetRoot {
  readonly rulesetRoot: string;
  readonly workspaceRoot: string;
  readonly packageRoots: readonly string[];
  readonly module: 'src/index.ts';
  readonly declaration: 'discovered exports';
  readonly resolvedWorkspaceRoot: string;
  readonly resolvedPackageRoots: readonly string[];
  readonly resolvedModule: string;
}

const RESULT_PREFIX = 'RULEBENCH_PLAY_BUNDLE_RESULT:';

export async function loadPlayBundleWorkspace(
  input: unknown,
  gatewayRoot: string,
): Promise<PlayBundleWorkspaceLoadResult> {
  const resolved = await resolveWorkspaceInput(input, gatewayRoot);
  if (!resolved.ok) return resolved;

  const buildRoot = await createBuildRoot(gatewayRoot);
  try {
    const compilerOptions: ts.CompilerOptions = {
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      target: ts.ScriptTarget.ES2022,
      rootDir: commonAncestor(resolved.value.resolvedPackageRoots),
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
      [resolved.value.resolvedModule],
      compilerOptions,
    );
    const authoredSourceFiles = program
      .getSourceFiles()
      .filter((sourceFile) => !sourceFile.isDeclarationFile);
    const escapedSource = authoredSourceFiles.find(
      (sourceFile) =>
        !isWithinAnyRoot(
          sourceFile.fileName,
          resolved.value.resolvedPackageRoots,
        ),
    );
    if (escapedSource !== undefined) {
      return failure(
        'RULESET_WORKSPACE_IMPORT_OUTSIDE_PACKAGE_ROOTS',
        '$.rulesetRoot',
        `Imported source ${normalizedPath(escapedSource.fileName)} is outside the selected ruleset root and its repository foundations`,
        resolved.value,
      );
    }
    const disallowedImport = authoredSourceFiles
      .map(disallowedModuleSpecifier)
      .find((specifier) => specifier !== null);
    if (disallowedImport !== undefined) {
      return failure(
        'RULESET_WORKSPACE_IMPORT_NOT_ALLOWED',
        '$.rulesetRoot',
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
    const emit = program.emit();
    if (emit.emitSkipped || emit.diagnostics.length > 0) {
      return {
        ok: false,
        diagnostics: emit.diagnostics.map((diagnostic) =>
          typescriptDiagnostic(diagnostic, resolved.value),
        ),
      };
    }

    const emittedModule = emittedModulePath(
      resolved.value.resolvedModule,
      compilerOptions.rootDir ?? resolved.value.resolvedWorkspaceRoot,
      buildRoot,
    );
    let moduleNamespace: unknown;
    try {
      moduleNamespace = await import(
        `${pathToFileURL(emittedModule).href}?load=${Date.now()}`
      );
    } catch (error: unknown) {
      return failure(
        'RULESET_WORKSPACE_EVALUATION_FAILED',
        '$.rulesetRoot',
        error instanceof Error ? error.message : String(error),
        resolved.value,
      );
    }
    if (!isRecord(moduleNamespace)) {
      return failure(
        'RULESET_WORKSPACE_MODULE_INVALID',
        '$.rulesetRoot',
        'The selected module did not expose an ES module namespace',
        resolved.value,
      );
    }
    const discovery = discoverAuthoringValues(moduleNamespace);
    if (!discovery.ok) {
      return failure(
        'RULESET_ROOT_EXPORTED_IDENTITY_DUPLICATE',
        '$.rulesetRoot',
        `Distinct exported ${discovery.kind} declarations share identity ${discovery.identity}`,
        resolved.value,
      );
    }
    const discovered = discovery.value;
    if (discovered.rulesets.length !== 1) {
      return failure(
        'RULESET_ROOT_RULESET_COUNT_INVALID',
        '$.rulesetRoot',
        `Expected one exported Ruleset, found ${discovered.rulesets.length}`,
        resolved.value,
      );
    }
    if (discovered.contentPacks.length === 0) {
      return failure(
        'RULESET_ROOT_CONTENT_PACK_REQUIRED',
        '$.rulesetRoot',
        'The Ruleset root must export at least one Content Pack source',
        resolved.value,
      );
    }
    if (discovered.playBundles.length === 0) {
      return failure(
        'RULESET_ROOT_PLAY_BUNDLE_REQUIRED',
        '$.rulesetRoot',
        'The Ruleset root must export at least one explicit PlayBundle',
        resolved.value,
      );
    }
    const ruleset = discovered.rulesets[0];
    if (ruleset === undefined) {
      return failure(
        'RULESET_ROOT_RULESET_COUNT_INVALID',
        '$.rulesetRoot',
        'The Ruleset root did not expose a Ruleset',
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
        'PLAY_BUNDLE_RULESET_ROOT_MISMATCH',
        '$.rulesetRoot',
        `PlayBundle ${mismatchedBundle.identity.id}@${mismatchedBundle.identity.version} does not use the root Ruleset ${ruleset.identity.id}@${ruleset.identity.version}`,
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
        '$.rulesetRoot',
        `Scenario template ${mismatchedScenario.identity.id}@${mismatchedScenario.identity.version} names undeclared PlayBundle ${mismatchedScenario.playBundle.id}@${mismatchedScenario.playBundle.version}`,
        resolved.value,
      );
    }
    const catalog = rootCatalog(
      resolved.value.rulesetRoot,
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
  const discovered = new Map<string, Value>();
  for (const value of values) {
    const current = identity(value);
    const key = `${current.id}@${current.version}`;
    const previous = discovered.get(key);
    if (previous !== undefined && previous !== value) return key;
    discovered.set(key, value);
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

function rootCatalog(
  rulesetRoot: string,
  ruleset: Ruleset,
  contentPacks: readonly ContentPackSource[],
  playBundles: readonly PlayBundleManifest[],
  scenarios: readonly ScenarioTemplate[],
): RulesetRootCatalog {
  return {
    rulesetRoot,
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

async function resolveWorkspaceInput(
  input: unknown,
  gatewayRoot: string,
): Promise<
  | { readonly ok: true; readonly value: ResolvedRulesetRoot }
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
      ? ['operation', 'rulesetRoot']
      : ['operation', 'rulesetRoot', 'contentPackIds'];
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
  const rulesetRoot = input['rulesetRoot'];
  if (typeof rulesetRoot !== 'string' || rulesetRoot.trim().length === 0) {
    return failureWithoutSource(
      'RULESET_ROOT_PATH_INVALID',
      '$.rulesetRoot',
      'rulesetRoot must be a non-empty path',
    );
  }
  const resolvedWorkspaceRoot = resolve(gatewayRoot, rulesetRoot);
  const rulesetsDirectory = dirname(resolvedWorkspaceRoot);
  if (basename(rulesetsDirectory) !== 'rulesets') {
    return failureWithoutSource(
      'RULESET_ROOT_LAYOUT_INVALID',
      '$.rulesetRoot',
      'rulesetRoot must be a direct child of a rulesets directory',
    );
  }
  const module = 'src/index.ts';
  const declaration = 'discovered exports';
  const workspaceRoot = rulesetRoot;
  const resolvedModule = join(resolvedWorkspaceRoot, module);
  const repositoryRoot = dirname(rulesetsDirectory);
  const resolvedFoundationsRoot = join(repositoryRoot, 'foundations');
  const resolvedPackageRoots = [resolvedWorkspaceRoot];
  try {
    const foundationsStat = await stat(resolvedFoundationsRoot);
    if (!foundationsStat.isDirectory()) {
      return failureWithoutSource(
        'RULESET_ROOT_FOUNDATIONS_INVALID',
        '$.rulesetRoot',
        'The conventional foundations path exists but is not a directory',
      );
    }
    resolvedPackageRoots.push(resolvedFoundationsRoot);
  } catch (error: unknown) {
    if (!isMissingPath(error)) {
      return failureWithoutSource(
        'RULESET_ROOT_FOUNDATIONS_UNREADABLE',
        '$.rulesetRoot',
        error instanceof Error ? error.message : String(error),
      );
    }
  }
  const packageRoots = resolvedPackageRoots.map((packageRoot) =>
    normalizedPath(relative(resolvedWorkspaceRoot, packageRoot) || '.'),
  );
  const resolvedValue: ResolvedRulesetRoot = {
    rulesetRoot,
    workspaceRoot,
    packageRoots,
    module,
    declaration,
    resolvedWorkspaceRoot,
    resolvedPackageRoots,
    resolvedModule,
  };
  try {
    const moduleStat = await stat(resolvedModule);
    if (!moduleStat.isFile()) throw new Error('not a file');
    for (const packageRoot of resolvedPackageRoots) {
      const packageStat = await stat(packageRoot);
      if (!packageStat.isDirectory())
        throw new Error(`${packageRoot} is not a directory`);
    }
  } catch (error: unknown) {
    return failure(
      'RULESET_ROOT_ENTRY_NOT_FOUND',
      '$.rulesetRoot',
      error instanceof Error ? error.message : String(error),
      resolvedValue,
    );
  }
  return { ok: true, value: resolvedValue };
}

function typescriptDiagnostic(
  diagnostic: ts.Diagnostic,
  input: ResolvedRulesetRoot,
): PlayBundleCompilerDiagnostic {
  const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n');
  if (diagnostic.file === undefined || diagnostic.start === undefined) {
    return diagnosticValue(
      'RULESET_WORKSPACE_BUILD_FAILED',
      '$.rulesetRoot',
      `TS${diagnostic.code}: ${message}`,
      input,
    );
  }
  const position = diagnostic.file.getLineAndCharacterOfPosition(
    diagnostic.start,
  );
  const sourcePath = normalizedPath(
    relative(input.resolvedWorkspaceRoot, diagnostic.file.fileName),
  );
  return diagnosticValue(
    'RULESET_WORKSPACE_BUILD_FAILED',
    `${sourcePath}:${position.line + 1}:${position.character + 1}`,
    `TS${diagnostic.code}: ${message}`,
    input,
  );
}

function failure(
  code: string,
  path: string,
  message: string,
  input: ResolvedRulesetRoot,
): PlayBundleWorkspaceFailure {
  return {
    ok: false,
    diagnostics: [diagnosticValue(code, path, message, input)],
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
  input: Pick<ResolvedRulesetRoot, 'module' | 'declaration'>,
): PlayBundleCompilerDiagnostic {
  return {
    stage: 'source',
    severity: 'error',
    code,
    path,
    message,
    source: { module: input.module, declaration: input.declaration },
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

function isMissingPath(error: unknown): boolean {
  return isRecord(error) && 'code' in error && error['code'] === 'ENOENT';
}

async function createBuildRoot(gatewayRoot: string): Promise<string> {
  const parent = join(resolve(gatewayRoot), 'tmp', 'ruleset-workspaces');
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
      'RULESET_ROOT_INPUT_INVALID',
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
