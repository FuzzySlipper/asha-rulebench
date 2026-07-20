import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import {
  decodeRulesetLocationConfig,
  loadRulesetLocationConfig,
} from './ruleset-location-config.mjs';

test('a missing local ruleset config is an empty explicit location list', async () => {
  const result = await loadRulesetLocationConfig(
    '/workspace',
    '.rulebench/rulesets.json',
  );
  assert.deepEqual(result, { schemaVersion: 1, rulesets: [] });
});

test('loads friendly ruleset locations without resolving their roots', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'rulebench-locations-'));
  try {
    await writeFile(
      join(directory, 'rulesets.json'),
      JSON.stringify({
        schemaVersion: 1,
        rulesets: [
          {
            id: 'field-manual',
            label: 'Field Manual',
            rulesetRoot: 'examples/rulesets/field-manual',
          },
          {
            id: 'external',
            label: 'Independent rules',
            rulesetRoot: '/home/dev/my-rules/rulesets/main',
          },
        ],
      }),
      'utf8',
    );
    const result = await loadRulesetLocationConfig(directory, 'rulesets.json');
    assert.equal(result.rulesets[0]?.label, 'Field Manual');
    assert.equal(
      result.rulesets[1]?.rulesetRoot,
      '/home/dev/my-rules/rulesets/main',
    );
  } finally {
    await rm(directory, { recursive: true });
  }
});

test('rejects ambiguous or extended local configuration', () => {
  assert.throws(
    () =>
      decodeRulesetLocationConfig({
        schemaVersion: 1,
        rulesets: [
          { id: 'one', label: 'One', rulesetRoot: '/rulesets/one' },
          { id: 'two', label: 'Two', rulesetRoot: '/rulesets/one' },
        ],
      }),
    /rulesetRoot duplicates/,
  );
  assert.throws(
    () =>
      decodeRulesetLocationConfig({
        schemaVersion: 1,
        rulesets: [],
        defaultRuleset: 'one',
      }),
    /unexpected defaultRuleset/,
  );
});
