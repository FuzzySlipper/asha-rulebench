import { mkdir, mkdtemp, rm, stat } from 'node:fs/promises';
import {
  dirname,
  extname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from 'node:path';
import { pathToFileURL } from 'node:url';

import {
  canonicalJson,
  prepareRulesetCompilation,
  stableFingerprint,
} from '@asha-rpg/authoring';
import type { RulesetCompilerDiagnostic } from '@asha-rpg/authoring';
import ts from 'typescript';

import { isRulesetWorkspaceDeclaration } from '../../libs/content-authoring/src/index.js';

export interface RulesetWorkspaceInput {
  readonly workspaceRoot: string;
  readonly packageRoots: readonly string[];
  readonly module: string;
  readonly declaration: string;
}

export type RulesetWorkspaceLoadResult =
  | {
      readonly ok: true;
      readonly preparedSource: string;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly RulesetCompilerDiagnostic[];
    };

type RulesetWorkspaceFailure = Extract<
  RulesetWorkspaceLoadResult,
  { readonly ok: false }
>;

interface ResolvedWorkspaceInput extends RulesetWorkspaceInput {
  readonly resolvedWorkspaceRoot: string;
  readonly resolvedPackageRoots: readonly string[];
  readonly resolvedModule: string;
}

const RESULT_PREFIX = 'RULEBENCH_WORKSPACE_RESULT:';

export async function loadRulesetWorkspace(
  input: unknown,
  gatewayRoot: string,
): Promise<RulesetWorkspaceLoadResult> {
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
        '$.packageRoots',
        `Imported source ${normalizedPath(escapedSource.fileName)} is outside the explicit package roots`,
        resolved.value,
      );
    }
    const disallowedImport = authoredSourceFiles
      .map(disallowedModuleSpecifier)
      .find((specifier) => specifier !== null);
    if (disallowedImport !== undefined) {
      return failure(
        'RULESET_WORKSPACE_IMPORT_NOT_ALLOWED',
        '$.module',
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
        '$.module',
        error instanceof Error ? error.message : String(error),
        resolved.value,
      );
    }
    if (!isRecord(moduleNamespace)) {
      return failure(
        'RULESET_WORKSPACE_MODULE_INVALID',
        '$.module',
        'The selected module did not expose an ES module namespace',
        resolved.value,
      );
    }
    if (!(resolved.value.declaration in moduleNamespace)) {
      return failure(
        'RULESET_WORKSPACE_DECLARATION_NOT_EXPORTED',
        '$.declaration',
        `Module does not export ${resolved.value.declaration}`,
        resolved.value,
      );
    }
    const declaration = moduleNamespace[resolved.value.declaration];
    if (!isRulesetWorkspaceDeclaration(declaration)) {
      return failure(
        'RULESET_WORKSPACE_DECLARATION_INVALID',
        '$.declaration',
        'Export must be immutable package and composition data',
        resolved.value,
      );
    }

    const sourceRoot =
      compilerOptions.rootDir ?? resolved.value.resolvedWorkspaceRoot;
    const sourceGraphFingerprint = stableFingerprint(
      authoredSourceFiles
        .map((sourceFile) => ({
          module: normalizedPath(relative(sourceRoot, sourceFile.fileName)),
          source: sourceFile.text,
        }))
        .sort((left, right) => left.module.localeCompare(right.module)),
    );
    const entrypoint = {
      module: normalizedPath(resolved.value.module),
      declaration: resolved.value.declaration,
      packageRoots: resolved.value.packageRoots.map(normalizedPath),
      sourceGraphFingerprint,
    };
    const packages = declaration.packages.map((source) => ({
      manifest: source.manifest,
      sourceFingerprint: stableFingerprint({
        entrypoint,
        packageSourceFingerprint: source.sourceFingerprint,
      }),
    }));
    const prepared = prepareRulesetCompilation({
      composition: declaration.composition,
      packages,
    });
    if (!prepared.ok) return prepared;
    return {
      ok: true,
      preparedSource: canonicalJson(prepared.prepared),
      diagnostics: [],
    };
  } finally {
    await rm(buildRoot, { recursive: true, force: true });
  }
}

async function resolveWorkspaceInput(
  input: unknown,
  gatewayRoot: string,
): Promise<
  | { readonly ok: true; readonly value: ResolvedWorkspaceInput }
  | RulesetWorkspaceFailure
> {
  if (!isRecord(input)) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_INPUT_INVALID',
      '$',
      'Compile input must be an object',
    );
  }
  const exactKeys = ['declaration', 'module', 'packageRoots', 'workspaceRoot'];
  if (
    Object.keys(input).length !== exactKeys.length ||
    !exactKeys.every((key) => key in input)
  ) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_INPUT_INVALID',
      '$',
      `Compile input must contain exactly ${exactKeys.join(', ')}`,
    );
  }
  const workspaceRoot = input['workspaceRoot'];
  const packageRoots = input['packageRoots'];
  const module = input['module'];
  const declaration = input['declaration'];
  if (typeof workspaceRoot !== 'string' || workspaceRoot.trim().length === 0) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_ROOT_INVALID',
      '$.workspaceRoot',
      'workspaceRoot must be a non-empty path',
    );
  }
  if (
    typeof module !== 'string' ||
    module.trim().length === 0 ||
    isAbsolute(module)
  ) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_MODULE_INVALID',
      '$.module',
      'module must be a non-empty path relative to workspaceRoot',
    );
  }
  if (
    typeof declaration !== 'string' ||
    !/^[$A-Z_a-z][$0-9A-Z_a-z]*$/.test(declaration)
  ) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_DECLARATION_INVALID',
      '$.declaration',
      'declaration must be one exported JavaScript identifier',
    );
  }
  if (
    !Array.isArray(packageRoots) ||
    packageRoots.length === 0 ||
    !packageRoots.every(
      (packageRoot) =>
        typeof packageRoot === 'string' && packageRoot.trim().length > 0,
    )
  ) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_PACKAGE_ROOTS_INVALID',
      '$.packageRoots',
      'packageRoots must contain at least one explicit path',
    );
  }
  const resolvedWorkspaceRoot = resolve(gatewayRoot, workspaceRoot);
  const resolvedModule = resolve(resolvedWorkspaceRoot, module);
  if (!isWithinRoot(resolvedModule, resolvedWorkspaceRoot)) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_MODULE_OUTSIDE_ROOT',
      '$.module',
      'module must remain within workspaceRoot',
    );
  }
  const resolvedPackageRoots = packageRoots.map((packageRoot) =>
    resolve(resolvedWorkspaceRoot, packageRoot),
  );
  if (!isWithinAnyRoot(resolvedModule, resolvedPackageRoots)) {
    return failureWithoutSource(
      'RULESET_WORKSPACE_MODULE_OUTSIDE_PACKAGE_ROOTS',
      '$.packageRoots',
      'module must be contained by an explicit package root',
    );
  }
  const resolvedValue: ResolvedWorkspaceInput = {
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
      'RULESET_WORKSPACE_SOURCE_NOT_FOUND',
      '$.module',
      error instanceof Error ? error.message : String(error),
      resolvedValue,
    );
  }
  return { ok: true, value: resolvedValue };
}

function typescriptDiagnostic(
  diagnostic: ts.Diagnostic,
  input: ResolvedWorkspaceInput,
): RulesetCompilerDiagnostic {
  const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n');
  if (diagnostic.file === undefined || diagnostic.start === undefined) {
    return diagnosticValue(
      'RULESET_WORKSPACE_BUILD_FAILED',
      '$.module',
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
  input: RulesetWorkspaceInput,
): RulesetWorkspaceFailure {
  return {
    ok: false,
    diagnostics: [diagnosticValue(code, path, message, input)],
  };
}

function failureWithoutSource(
  code: string,
  path: string,
  message: string,
): RulesetWorkspaceFailure {
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
  input: Pick<RulesetWorkspaceInput, 'module' | 'declaration'>,
): RulesetCompilerDiagnostic {
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
      'RULESET_WORKSPACE_INPUT_INVALID',
      '$',
      error instanceof Error ? error.message : String(error),
    );
    process.stdout.write(`${RESULT_PREFIX}${JSON.stringify(result)}\n`);
    return;
  }
  const result = await loadRulesetWorkspace(input, process.cwd());
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
