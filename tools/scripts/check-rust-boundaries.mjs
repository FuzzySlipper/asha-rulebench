import { readFileSync, readdirSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const root = process.cwd();
const cratesRoot = join(root, 'rulebench-rs', 'crates');
const hostsRoot = join(root, 'rulebench-rs', 'hosts');
const ashaRepository = 'https://github.com/FuzzySlipper/asha-engine.git';
const ashaRevision = '67ce55dba602ad61e1b9ca3b0ad01a22fa4fe148';
const ashaVersionRequirement = '^0.1';

const allowedDependencies = new Map([
  ['rulebench-core', new Set()],
  ['rulebench-ruleset', new Set(['rulebench-core'])],
  ['rulebench-content', new Set(['rulebench-core', 'rulebench-ruleset'])],
  [
    'rulebench-gameplay-module',
    new Set(['asha-gameplay-module-sdk', 'asha-runtime-session-composition']),
  ],
  [
    'rulebench-combat',
    new Set([
      'rulebench-core',
      'rulebench-content',
      'rulebench-gameplay-module',
      'rulebench-ruleset',
    ]),
  ],
  ['rulebench-replay', new Set(['rulebench-core', 'rulebench-combat'])],
  [
    'rulebench-rules',
    new Set(['rulebench-core', 'rulebench-ruleset', 'rulebench-content', 'rulebench-combat', 'rulebench-replay']),
  ],
  ['rulebench-protocol', new Set(['rulebench-rules'])],
  ['rulebench-bridge', new Set(['rulebench-protocol', 'rulebench-rules'])],
  ['rulebench-fixtures', new Set(['rulebench-rules'])],
  ['rulebench-codegen', new Set(['rulebench-fixtures', 'rulebench-protocol'])],
  ['rulebench-authority', new Set(['rulebench-rules', 'rulebench-fixtures', 'rulebench-codegen', 'rulebench-bridge'])],
  ['rulebench-process-host', new Set(['rulebench-bridge', 'rulebench-fixtures', 'rulebench-protocol'])],
]);

const portableCrates = new Set([
  'rulebench-core',
  'rulebench-ruleset',
  'rulebench-content',
  'rulebench-combat',
  'rulebench-gameplay-module',
  'rulebench-replay',
  'rulebench-rules',
]);

const publicAshaDependencies = new Set([
  'asha-gameplay-module-sdk',
  'asha-runtime-session-composition',
]);

const manifests = readWorkspaceManifests();
const failures = validateWorkspace(manifests);

runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(`check:rust-boundaries ok (${manifests.size} crates)`);

function readWorkspaceManifests() {
  const manifests = new Map();
  for (const workspaceRoot of [cratesRoot, hostsRoot]) {
    for (const entry of readdirSync(workspaceRoot, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;

      const crateName = entry.name;
      const manifestPath = join(workspaceRoot, crateName, 'Cargo.toml');
      const manifest = readFileSync(manifestPath, 'utf8');
      manifests.set(crateName, {
        manifestPath,
        dependencies: parseLocalDependencies(manifest),
      });
    }
  }
  return manifests;
}

function parseLocalDependencies(manifest) {
  const dependencies = [];
  let section = '';
  const lines = manifest.split('\n');

  for (const [index, line] of lines.entries()) {
    const sectionMatch = /^\[([^\]]+)]\s*$/.exec(line);
    if (sectionMatch !== null) {
      section = sectionMatch[1];
      continue;
    }

    if (!['dependencies', 'dev-dependencies', 'build-dependencies'].includes(section)) continue;

    const dependencyMatch = /^([A-Za-z0-9_-]+)\s*=\s*(.*)$/.exec(line);
    if (dependencyMatch === null) continue;

    const specification = dependencyMatch[2];
    const pathMatch = /\bpath\s*=\s*"([^"]+)"/.exec(specification);
    const isRulebenchDependency = dependencyMatch[1].startsWith('rulebench-');
    const isAshaDependency = dependencyMatch[1].startsWith('asha-');
    if (!isRulebenchDependency && !isAshaDependency && pathMatch === null) continue;

    dependencies.push({
      name: dependencyMatch[1],
      path: pathMatch?.[1] ?? null,
      git: /\bgit\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      revision: /\brev\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      version: /\bversion\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      line: index + 1,
    });
  }

  return dependencies;
}

function validateWorkspace(manifests) {
  const failures = [];
  const actualCrates = new Set(manifests.keys());

  for (const crateName of actualCrates) {
    if (!allowedDependencies.has(crateName)) {
      failures.push(`Unknown Rust workspace crate: ${crateName}. Add an explicit boundary policy before it can join the workspace.`);
    }
  }

  for (const crateName of allowedDependencies.keys()) {
    if (!actualCrates.has(crateName)) {
      failures.push(`Boundary policy names missing workspace crate: ${crateName}.`);
    }
  }

  for (const [crateName, manifest] of manifests) {
    for (const dependency of manifest.dependencies) {
      failures.push(...validateDependency(crateName, dependency.name, `${relative(root, manifest.manifestPath)}:${dependency.line}`));
      failures.push(...validateAshaDistribution(crateName, dependency, `${relative(root, manifest.manifestPath)}:${dependency.line}`));
      if (dependency.path !== null) {
        failures.push(...validatePortablePath(crateName, dependency.name, manifest.manifestPath, dependency.path, `${relative(root, manifest.manifestPath)}:${dependency.line}`));
      }
    }
  }

  return failures;
}

function validateAshaDistribution(crateName, dependency, location) {
  if (!dependency.name.startsWith('asha-')) return [];
  if (!publicAshaDependencies.has(dependency.name)) {
    return [`${location}: ${crateName} imports unapproved ASHA crate ${dependency.name}.`];
  }
  const failures = [];
  if (dependency.path !== null) failures.push(`${location}: shipping ASHA dependencies must not use sibling paths.`);
  if (dependency.git !== ashaRepository) failures.push(`${location}: ASHA dependency must use the governed repository.`);
  if (dependency.revision !== ashaRevision) failures.push(`${location}: ASHA dependency must use the reviewed exact revision ${ashaRevision}.`);
  if (dependency.version !== ashaVersionRequirement) failures.push(`${location}: ASHA dependency must use compatible version ${ashaVersionRequirement}.`);
  return failures;
}

function validateDependency(crateName, dependencyName, location) {
  const allowed = allowedDependencies.get(crateName);
  if (allowed === undefined) return [];

  if (publicAshaDependencies.has(dependencyName)) {
    return allowed.has(dependencyName)
      ? []
      : [`${location}: ${crateName} may not consume ASHA public crate ${dependencyName}.`];
  }

  if (!allowedDependencies.has(dependencyName)) {
    return [`${location}: ${crateName} depends on unknown local crate ${dependencyName}. Add an explicit boundary policy first.`];
  }

  if (allowed.has(dependencyName)) return [];

  return [`${location}: ${crateName} must not depend on ${dependencyName}. This violates the Rulebench crate dependency direction.`];
}

function validatePortablePath(crateName, dependencyName, manifestPath, dependencyPath, location) {
  if (!portableCrates.has(crateName)) return [];

  const resolvedPath = resolve(join(manifestPath, '..'), dependencyPath);
  const crateRelativePath = relative(cratesRoot, resolvedPath);
  const staysInsideCrates = crateRelativePath !== '' && !crateRelativePath.startsWith('..') && !crateRelativePath.startsWith('/');

  if (staysInsideCrates) return [];

  return [`${location}: portable crate ${crateName} may only path-depend inside rulebench-rs/crates; ASHA facades use the governed Git distribution (${dependencyPath}).`];
}

function runFocusedFailureTests() {
  const versionedReverseDependency = parseLocalDependencies('[dependencies]\nrulebench-authority = "0.1.0"')[0];
  const reverseDependencyErrors = validateDependency(
    'rulebench-core',
    versionedReverseDependency?.name ?? '',
    'self-test:reverse-dependency',
  );
  if (reverseDependencyErrors.length === 0) {
    throw new Error('Boundary self-test failed: a portable crate may depend on rulebench-authority by version.');
  }

  const frontendPathErrors = validatePortablePath(
    'rulebench-core',
    'rulebench-store',
    join(cratesRoot, 'rulebench-core', 'Cargo.toml'),
    '../../../libs/store',
    'self-test:frontend-path',
  );
  if (frontendPathErrors.length === 0) {
    throw new Error('Boundary self-test failed: a portable crate may path-depend on a frontend surface.');
  }

  const staleAshaErrors = validateAshaDistribution(
    'rulebench-gameplay-module',
    {
      name: 'asha-gameplay-module-sdk',
      path: null,
      git: ashaRepository,
      revision: '0'.repeat(40),
      version: ashaVersionRequirement,
    },
    'self-test:stale-asha-revision',
  );
  if (staleAshaErrors.length === 0) {
    throw new Error('Boundary self-test failed: a stale ASHA revision was accepted.');
  }

  assertRejected(
    validateAshaDistribution(
      'rulebench-gameplay-module',
      ashaDependency({ git: 'https://github.com/example/asha-fork.git' }),
      'self-test:forked-asha-repository',
    ),
    'a forked ASHA repository was accepted',
  );
  assertRejected(
    validateAshaDistribution(
      'rulebench-gameplay-module',
      ashaDependency({ version: '*' }),
      'self-test:unbounded-asha-version',
    ),
    'an unbounded ASHA version was accepted',
  );
  assertRejected(
    validateAshaDistribution(
      'rulebench-gameplay-module',
      ashaDependency({ path: '../../../asha' }),
      'self-test:sibling-asha-path',
    ),
    'a sibling ASHA path was accepted',
  );
  assertRejected(
    validateAshaDistribution(
      'rulebench-gameplay-module',
      ashaDependency({ name: 'asha-gameplay-runtime-host' }),
      'self-test:private-asha-crate',
    ),
    'an unapproved ASHA crate was accepted',
  );
  assertRejected(
    validateDependency(
      'rulebench-replay',
      'rulebench-fixtures',
      'self-test:replay-to-fixtures',
    ),
    'a portable owner was allowed to import product fixtures',
  );
  assertRejected(
    validateDependency(
      'rulebench-core',
      'rulebench-unknown',
      'self-test:unknown-rulebench-crate',
    ),
    'an unknown Rulebench crate was accepted',
  );
  assertRejected(
    validateDependency(
      'rulebench-replay',
      'asha-gameplay-module-sdk',
      'self-test:asha-owner-bypass',
    ),
    'an ASHA public crate was imported outside its approved downstream adapter',
  );
}

function ashaDependency(overrides) {
  return {
    name: 'asha-gameplay-module-sdk',
    path: null,
    git: ashaRepository,
    revision: ashaRevision,
    version: ashaVersionRequirement,
    ...overrides,
  };
}

function assertRejected(failures, message) {
  if (failures.length === 0) {
    throw new Error(`Boundary self-test failed: ${message}.`);
  }
}
