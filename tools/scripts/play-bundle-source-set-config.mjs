import { readFile } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { isAbsolute, resolve } from 'node:path';
import { promisify } from 'node:util';

const SCHEMA_VERSION = 2;
const SOURCE_SET_SCHEMA_VERSION = 1;
const EXPORT_KINDS = new Set([
  'ruleset',
  'contentPack',
  'playBundle',
  'scenarioTemplate',
]);
const execFileAsync = promisify(execFile);

export async function loadPlayBundleSourceSetConfig(workspaceRoot, configPath) {
  const absoluteConfigPath = isAbsolute(configPath)
    ? configPath
    : resolve(workspaceRoot, configPath);
  let source;
  try {
    source = await readFile(absoluteConfigPath, 'utf8');
  } catch (error) {
    if (isMissingFile(error)) return emptyPlayBundleSourceSetConfig();
    throw error;
  }

  let parsed;
  try {
    parsed = JSON.parse(source);
  } catch (error) {
    throw new Error(
      `PlayBundle source-set config ${absoluteConfigPath} is not valid JSON: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  const decoded = decodePlayBundleSourceSetConfig(parsed, absoluteConfigPath);
  await validateRepositoryPins(parsed, workspaceRoot, absoluteConfigPath);
  return decoded;
}

export function decodePlayBundleSourceSetConfig(
  value,
  source = 'PlayBundle source-set config',
) {
  const record = requiredRecord(value, source);
  exactKeys(record, ['schemaVersion', 'sourceSets'], source);
  if (record['schemaVersion'] !== SCHEMA_VERSION) {
    throw new Error(`${source}.schemaVersion must be ${SCHEMA_VERSION}`);
  }
  if (!Array.isArray(record['sourceSets'])) {
    throw new Error(`${source}.sourceSets must be an array`);
  }

  const ids = new Set();
  const sourceSets = record['sourceSets'].map((entry, index) => {
    const path = `${source}.sourceSets[${index}]`;
    const location = requiredRecord(entry, path);
    exactOptionalKeys(
      location,
      ['id', 'label', 'sourceSet'],
      ['repository'],
      path,
    );
    const id = requiredString(location['id'], `${path}.id`);
    const label = requiredString(location['label'], `${path}.label`);
    const sourceSet = decodeSourceSet(
      location['sourceSet'],
      `${path}.sourceSet`,
    );
    if (Object.prototype.hasOwnProperty.call(location, 'repository')) {
      decodeRepositoryPin(location['repository'], `${path}.repository`);
    }
    if (ids.has(id)) throw new Error(`${path}.id duplicates ${id}`);
    ids.add(id);
    return { id, label, sourceSet };
  });

  return { schemaVersion: SCHEMA_VERSION, sourceSets };
}

async function validateRepositoryPins(value, workspaceRoot, source) {
  const record = requiredRecord(value, source);
  const sourceSets = record['sourceSets'];
  if (!Array.isArray(sourceSets)) return;
  for (const [index, entryValue] of sourceSets.entries()) {
    const path = `${source}.sourceSets[${index}]`;
    const entry = requiredRecord(entryValue, path);
    if (!Object.prototype.hasOwnProperty.call(entry, 'repository')) continue;
    const pin = decodeRepositoryPin(entry['repository'], `${path}.repository`);
    const repositoryRoot = isAbsolute(pin.root)
      ? pin.root
      : resolve(workspaceRoot, pin.root);
    let actualRevision;
    try {
      const result = await execFileAsync('git', [
        '-C',
        repositoryRoot,
        'rev-parse',
        'HEAD',
      ]);
      actualRevision = result.stdout.trim();
    } catch (error) {
      throw new Error(
        `${path}.repository could not inspect ${repositoryRoot}: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
    if (actualRevision !== pin.revision) {
      throw new Error(
        `${path}.repository expected revision ${pin.revision}, but ${repositoryRoot} is at ${actualRevision}`,
      );
    }
  }
}

function decodeRepositoryPin(value, path) {
  const repository = requiredRecord(value, path);
  exactKeys(repository, ['root', 'revision'], path);
  const root = requiredString(repository['root'], `${path}.root`);
  const revision = requiredString(
    repository['revision'],
    `${path}.revision`,
  );
  if (!/^[0-9a-f]{40}$/.test(revision)) {
    throw new Error(`${path}.revision must be a full lowercase Git SHA`);
  }
  return { root, revision };
}

function decodeSourceSet(value, path) {
  const sourceSet = requiredRecord(value, path);
  exactKeys(sourceSet, ['schemaVersion', 'allowedRoots', 'entries'], path);
  if (sourceSet['schemaVersion'] !== SOURCE_SET_SCHEMA_VERSION) {
    throw new Error(
      `${path}.schemaVersion must be ${SOURCE_SET_SCHEMA_VERSION}`,
    );
  }
  const allowedRoots = uniqueStrings(
    sourceSet['allowedRoots'],
    `${path}.allowedRoots`,
  );
  if (allowedRoots.length === 0)
    throw new Error(`${path}.allowedRoots must not be empty`);
  if (
    !Array.isArray(sourceSet['entries']) ||
    sourceSet['entries'].length === 0
  ) {
    throw new Error(`${path}.entries must be a non-empty array`);
  }
  const entryIds = new Set();
  const entries = sourceSet['entries'].map((value, index) => {
    const entryPath = `${path}.entries[${index}]`;
    const entry = requiredRecord(value, entryPath);
    exactKeys(
      entry,
      ['id', 'label', 'sourceRoot', 'module', 'exportKinds'],
      entryPath,
    );
    const id = requiredString(entry['id'], `${entryPath}.id`);
    if (entryIds.has(id)) throw new Error(`${entryPath}.id duplicates ${id}`);
    entryIds.add(id);
    const exportKinds = uniqueStrings(
      entry['exportKinds'],
      `${entryPath}.exportKinds`,
    );
    if (
      exportKinds.length === 0 ||
      exportKinds.some((kind) => !EXPORT_KINDS.has(kind))
    ) {
      throw new Error(
        `${entryPath}.exportKinds must contain supported export kinds`,
      );
    }
    return {
      id,
      label: requiredString(entry['label'], `${entryPath}.label`),
      sourceRoot: requiredString(
        entry['sourceRoot'],
        `${entryPath}.sourceRoot`,
      ),
      module: requiredString(entry['module'], `${entryPath}.module`),
      exportKinds,
    };
  });
  const rulesetEntries = entries.filter((entry) =>
    entry.exportKinds.includes('ruleset'),
  );
  if (rulesetEntries.length !== 1) {
    throw new Error(`${path}.entries must declare exactly one ruleset source`);
  }
  return { schemaVersion: SOURCE_SET_SCHEMA_VERSION, allowedRoots, entries };
}

function uniqueStrings(value, path) {
  if (!Array.isArray(value)) throw new Error(`${path} must be an array`);
  const strings = value.map((entry, index) =>
    requiredString(entry, `${path}[${index}]`),
  );
  if (new Set(strings).size !== strings.length)
    throw new Error(`${path} must not contain duplicates`);
  return strings;
}

function emptyPlayBundleSourceSetConfig() {
  return { schemaVersion: SCHEMA_VERSION, sourceSets: [] };
}

function requiredRecord(value, path) {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  return value;
}

function requiredString(value, path) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${path} must be a non-empty string`);
  }
  return value.trim();
}

function exactKeys(record, expectedKeys, path) {
  const expected = new Set(expectedKeys);
  const unexpected = Object.keys(record).filter((key) => !expected.has(key));
  const missing = expectedKeys.filter(
    (key) => !Object.prototype.hasOwnProperty.call(record, key),
  );
  if (unexpected.length > 0 || missing.length > 0) {
    throw new Error(
      `${path} keys must be exactly ${expectedKeys.join(', ')}; missing ${missing.join(', ') || 'none'}; unexpected ${unexpected.join(', ') || 'none'}`,
    );
  }
}

function exactOptionalKeys(record, requiredKeys, optionalKeys, path) {
  const expected = new Set([...requiredKeys, ...optionalKeys]);
  const unexpected = Object.keys(record).filter((key) => !expected.has(key));
  const missing = requiredKeys.filter(
    (key) => !Object.prototype.hasOwnProperty.call(record, key),
  );
  if (unexpected.length > 0 || missing.length > 0) {
    throw new Error(
      `${path} keys must contain ${requiredKeys.join(', ')} and may contain ${optionalKeys.join(', ')}; missing ${missing.join(', ') || 'none'}; unexpected ${unexpected.join(', ') || 'none'}`,
    );
  }
}

function isMissingFile(error) {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    error.code === 'ENOENT'
  );
}
