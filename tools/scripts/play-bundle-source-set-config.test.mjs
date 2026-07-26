import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { promisify } from 'node:util';

import {
  decodePlayBundleSourceSetConfig,
  loadPlayBundleSourceSetConfig,
} from './play-bundle-source-set-config.mjs';

const execFileAsync = promisify(execFile);

test('a missing local source-set config is an empty explicit list', async () => {
  const result = await loadPlayBundleSourceSetConfig(
    '/workspace',
    '.rulebench/source-sets.json',
  );
  assert.deepEqual(result, { schemaVersion: 2, sourceSets: [] });
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
      join(directory, 'source-sets.json'),
      JSON.stringify({
        schemaVersion: 2,
        sourceSets: [
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
    const result = await loadPlayBundleSourceSetConfig(
      directory,
      'source-sets.json',
    );
    assert.equal(result.sourceSets[0]?.label, 'Local rules');
    assert.equal(
      result.sourceSets[1]?.sourceSet.allowedRoots[0],
      '/home/dev/my-rules/rulesets/main',
    );
  } finally {
    await rm(directory, { recursive: true });
  }
});

test('rejects ambiguous or extended local configuration', () => {
  assert.throws(
    () =>
      decodePlayBundleSourceSetConfig({
        schemaVersion: 2,
        sourceSets: [
          { id: 'one', label: 'One', sourceSet: sourceSet('/rulesets/one') },
          { id: 'one', label: 'Two', sourceSet: sourceSet('/rulesets/two') },
        ],
      }),
    /id duplicates/,
  );
  assert.throws(
    () =>
      decodePlayBundleSourceSetConfig({
        schemaVersion: 2,
        sourceSets: [],
        defaultRuleset: 'one',
      }),
    /unexpected defaultRuleset/,
  );
});

test('validates and strips an exact external repository pin', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'rulebench-pin-'));
  const repositoryRoot = join(directory, 'content-repository');
  try {
    await execFileAsync('git', ['init', repositoryRoot]);
    await execFileAsync('git', [
      '-C',
      repositoryRoot,
      'config',
      'user.name',
      'Rulebench Test',
    ]);
    await execFileAsync('git', [
      '-C',
      repositoryRoot,
      'config',
      'user.email',
      'rulebench@example.invalid',
    ]);
    await writeFile(join(repositoryRoot, 'content.txt'), 'fixture\n', 'utf8');
    await execFileAsync('git', ['-C', repositoryRoot, 'add', 'content.txt']);
    await execFileAsync('git', [
      '-C',
      repositoryRoot,
      'commit',
      '-m',
      'fixture',
    ]);
    const revision = (
      await execFileAsync('git', ['-C', repositoryRoot, 'rev-parse', 'HEAD'])
    ).stdout.trim();
    const config = {
      schemaVersion: 2,
      sourceSets: [
        {
          id: 'pinned',
          label: 'Pinned content',
          repository: {
            root: repositoryRoot,
            revision,
          },
          sourceSet: sourceSet(repositoryRoot),
        },
      ],
    };
    await writeFile(
      join(directory, 'source-sets.json'),
      JSON.stringify(config),
      'utf8',
    );

    const result = await loadPlayBundleSourceSetConfig(
      directory,
      'source-sets.json',
    );
    assert.deepEqual(Object.keys(result.sourceSets[0] ?? {}).sort(), [
      'id',
      'label',
      'sourceSet',
    ]);

    config.sourceSets[0].repository.revision = '0'.repeat(40);
    await writeFile(
      join(directory, 'source-sets.json'),
      JSON.stringify(config),
      'utf8',
    );
    await assert.rejects(
      loadPlayBundleSourceSetConfig(directory, 'source-sets.json'),
      /expected revision 0000000000000000000000000000000000000000/,
    );
  } finally {
    await rm(directory, { recursive: true });
  }
});
