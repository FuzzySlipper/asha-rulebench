import { readFile } from 'node:fs/promises';
import { isAbsolute, resolve } from 'node:path';

const SCHEMA_VERSION = 1;

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
  const roots = new Set();
  const rulesets = record['rulesets'].map((entry, index) => {
    const path = `${source}.rulesets[${index}]`;
    const location = requiredRecord(entry, path);
    exactKeys(location, ['id', 'label', 'rulesetRoot'], path);
    const id = requiredString(location['id'], `${path}.id`);
    const label = requiredString(location['label'], `${path}.label`);
    const rulesetRoot = requiredString(
      location['rulesetRoot'],
      `${path}.rulesetRoot`,
    );
    if (ids.has(id)) throw new Error(`${path}.id duplicates ${id}`);
    if (roots.has(rulesetRoot)) {
      throw new Error(`${path}.rulesetRoot duplicates ${rulesetRoot}`);
    }
    ids.add(id);
    roots.add(rulesetRoot);
    return { id, label, rulesetRoot };
  });

  return { schemaVersion: SCHEMA_VERSION, rulesets };
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
