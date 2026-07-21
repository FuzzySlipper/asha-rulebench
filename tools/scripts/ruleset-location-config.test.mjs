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
  assert.deepEqual(result, { schemaVersion: 2, rulesets: [] });
});

const sourceSet = (sourceRoot) => ({
  schemaVersion: 1,
  allowedRoots: [sourceRoot],
  entries: [
    {
      id: 'primary',
      label: 'Primary',
      sourceRoot,
      module: 'src/index.ts',
      exportKinds: ['ruleset', 'contentPack', 'playBundle'],
    },
  ],
});

test('loads friendly source sets without resolving their roots', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'rulebench-locations-'));
  try {
    await writeFile(
      join(directory, 'rulesets.json'),
      JSON.stringify({
        schemaVersion: 2,
        rulesets: [
          {
            id: 'local-rules',
            label: 'Local rules',
            sourceSet: sourceSet('rulesets/local-rules'),
          },
          {
            id: 'external',
            label: 'Independent rules',
            sourceSet: sourceSet('/home/dev/my-rules/rulesets/main'),
          },
        ],
      }),
      'utf8',
    );
    const result = await loadRulesetLocationConfig(directory, 'rulesets.json');
    assert.equal(result.rulesets[0]?.label, 'Local rules');
    assert.equal(
      result.rulesets[1]?.sourceSet.allowedRoots[0],
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
        schemaVersion: 2,
        rulesets: [
          { id: 'one', label: 'One', sourceSet: sourceSet('/rulesets/one') },
          { id: 'one', label: 'Two', sourceSet: sourceSet('/rulesets/two') },
        ],
      }),
    /id duplicates/,
  );
  assert.throws(
    () =>
      decodeRulesetLocationConfig({
        schemaVersion: 2,
        rulesets: [],
        defaultRuleset: 'one',
      }),
    /unexpected defaultRuleset/,
  );
});
