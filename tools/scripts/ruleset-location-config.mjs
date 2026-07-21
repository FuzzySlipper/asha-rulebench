import { readFile } from 'node:fs/promises';
import { isAbsolute, resolve } from 'node:path';

const SCHEMA_VERSION = 2;
const SOURCE_SET_SCHEMA_VERSION = 1;
const EXPORT_KINDS = new Set([
  'ruleset',
  'contentPack',
  'playBundle',
  'scenarioTemplate',
]);

export async function loadRulesetLocationConfig(workspaceRoot, configPath) {
  const absoluteConfigPath = isAbsolute(configPath)
    ? configPath
    : resolve(workspaceRoot, configPath);
  let source;
  try {
    source = await readFile(absoluteConfigPath, 'utf8');
  } catch (error) {
    if (isMissingFile(error)) return emptyRulesetLocationConfig();
    throw error;
  }

  let parsed;
  try {
    parsed = JSON.parse(source);
  } catch (error) {
    throw new Error(
      `ruleset location config ${absoluteConfigPath} is not valid JSON: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  return decodeRulesetLocationConfig(parsed, absoluteConfigPath);
}

export function decodeRulesetLocationConfig(value, source = 'ruleset config') {
  const record = requiredRecord(value, source);
  exactKeys(record, ['schemaVersion', 'rulesets'], source);
  if (record['schemaVersion'] !== SCHEMA_VERSION) {
    throw new Error(`${source}.schemaVersion must be ${SCHEMA_VERSION}`);
  }
  if (!Array.isArray(record['rulesets'])) {
    throw new Error(`${source}.rulesets must be an array`);
  }

  const ids = new Set();
  const rulesets = record['rulesets'].map((entry, index) => {
    const path = `${source}.rulesets[${index}]`;
    const location = requiredRecord(entry, path);
    exactKeys(location, ['id', 'label', 'sourceSet'], path);
    const id = requiredString(location['id'], `${path}.id`);
    const label = requiredString(location['label'], `${path}.label`);
    const sourceSet = decodeSourceSet(
      location['sourceSet'],
      `${path}.sourceSet`,
    );
    if (ids.has(id)) throw new Error(`${path}.id duplicates ${id}`);
    ids.add(id);
    return { id, label, sourceSet };
  });

  return { schemaVersion: SCHEMA_VERSION, rulesets };
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

function emptyRulesetLocationConfig() {
  return { schemaVersion: SCHEMA_VERSION, rulesets: [] };
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

function isMissingFile(error) {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    error.code === 'ENOENT'
  );
}
