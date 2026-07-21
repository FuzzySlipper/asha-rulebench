import { describe, expect, it } from 'vitest';

import {
  loadPlayBundleWorkspace,
  type PlayBundleSourceExportKind,
  type PlayBundleSourceSet,
} from './load-play-bundle-workspace.js';

const gatewayRoot = process.cwd();
const minimalRoot = 'test-fixtures/rulesets/minimal';

function oneSource(
  sourceRoot: string,
  exportKinds: readonly PlayBundleSourceExportKind[] = [
    'ruleset',
    'contentPack',
    'playBundle',
    'scenarioTemplate',
  ],
): PlayBundleSourceSet {
  return {
    schemaVersion: 1,
    allowedRoots: [sourceRoot],
    entries: [
      {
        id: 'primary',
        label: 'Primary source',
        sourceRoot,
        module: 'src/index.ts',
        exportKinds,
      },
    ],
  };
}

describe('explicit PlayBundle source-set loader', () => {
  it('discovers distinct Ruleset, Content Pack, and PlayBundle declarations', async () => {
    const result = await loadPlayBundleWorkspace(
      { operation: 'inspect', sourceSet: oneSource(minimalRoot) },
      gatewayRoot,
    );

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.preparedSource).toBeNull();
    expect(result.catalog.ruleset).toEqual({
      id: 'rulebench.minimal',
      version: '1.0.0',
    });
    expect(
      result.catalog.contentPacks.map((contentPack) => contentPack.id),
    ).toEqual(['rulebench.minimal.content']);
    expect(result.catalog.playBundles).toEqual([
      expect.objectContaining({
        id: 'rulebench.minimal.play',
        compatible: true,
        contentPackIds: ['rulebench.minimal.content'],
      }),
    ]);
    expect(result.catalog.scenarios).toHaveLength(1);
    expect(result.catalog.scenarios[0]?.presentation.label).toBe(
      'Minimal Scenario',
    );
  });

  it('prepares only an exact declared Content Pack selection', async () => {
    const prepared = await loadPlayBundleWorkspace(
      {
        operation: 'compile',
        sourceSet: oneSource(minimalRoot),
        contentPackIds: ['rulebench.minimal.content'],
      },
      gatewayRoot,
    );
    expect(prepared.ok).toBe(true);
    if (prepared.ok) {
      expect(JSON.parse(prepared.preparedSource ?? '{}')).toMatchObject({
        playBundleIdentity: { id: 'rulebench.minimal.play' },
        ruleset: { identity: { id: 'rulebench.minimal' } },
        contentPacks: [{ id: 'rulebench.minimal.content' }],
      });
    }

    const undeclared = await loadPlayBundleWorkspace(
      {
        operation: 'compile',
        sourceSet: oneSource(minimalRoot),
        contentPackIds: [],
      },
      gatewayRoot,
    );
    expect(undeclared.ok).toBe(false);
    if (!undeclared.ok) {
      expect(undeclared.diagnostics[0]?.code).toBe(
        'PLAY_BUNDLE_SELECTION_NOT_DECLARED',
      );
    }
  });

  it('reports build and missing-entry failures without starting a host', async () => {
    const invalidBuild = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        sourceSet: oneSource('test-fixtures/rulesets/invalid-build'),
      },
      gatewayRoot,
    );
    expect(invalidBuild.ok).toBe(false);
    if (!invalidBuild.ok) {
      expect(invalidBuild.diagnostics[0]?.message).toContain('TS2322');
    }

    const missingEntry = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        sourceSet: oneSource('test-fixtures/not-a-ruleset'),
      },
      gatewayRoot,
    );
    expect(missingEntry.ok).toBe(false);
    if (!missingEntry.ok) {
      expect(missingEntry.diagnostics[0]?.code).toBe(
        'PLAY_BUNDLE_SOURCE_ENTRY_NOT_FOUND',
      );
    }
  });

  it.each([
    ['Ruleset', 'test-fixtures/rulesets/duplicate-ruleset'],
    ['PlayBundle', 'test-fixtures/rulesets/duplicate-play-bundle'],
  ])(
    'rejects distinct exported %s declarations with one identity',
    async (kind, rulesetRoot) => {
      const result = await loadPlayBundleWorkspace(
        { operation: 'inspect', sourceSet: oneSource(rulesetRoot) },
        gatewayRoot,
      );

      expect(result.ok).toBe(false);
      if (result.ok) return;
      expect(result.diagnostics[0]).toMatchObject({
        code: 'PLAY_BUNDLE_SOURCE_EXPORTED_IDENTITY_DUPLICATE',
        path: '$.sourceSet.entries[0]',
      });
      expect(result.diagnostics[0]?.message).toContain(`exported ${kind}`);
    },
  );

  it('composes a Ruleset and an independent content repository through declared roots', async () => {
    const rulesRoot = 'test-fixtures/source-sets/independent/rules';
    const contentRoot = 'test-fixtures/source-sets/independent/content';
    const bundlesRoot = 'test-fixtures/source-sets/independent/bundles';
    const sourceSet: PlayBundleSourceSet = {
      schemaVersion: 1,
      allowedRoots: [rulesRoot, contentRoot, bundlesRoot],
      entries: [
        {
          id: 'rules',
          label: 'Independent rules',
          sourceRoot: rulesRoot,
          module: 'src/index.ts',
          exportKinds: ['ruleset'],
        },
        {
          id: 'content',
          label: 'Independent content',
          sourceRoot: contentRoot,
          module: 'src/index.ts',
          exportKinds: ['contentPack'],
        },
        {
          id: 'bundle',
          label: 'Independent bundle',
          sourceRoot: bundlesRoot,
          module: 'src/primary.ts',
          exportKinds: ['playBundle', 'scenarioTemplate'],
        },
      ],
    };
    const result = await loadPlayBundleWorkspace(
      { operation: 'inspect', sourceSet },
      gatewayRoot,
    );
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.catalog.sourceSet).toEqual(sourceSet);
    expect(result.catalog.ruleset.id).toBe('rulebench.independent');
    expect(result.catalog.contentPacks[0]?.id).toBe(
      'rulebench.independent.content',
    );

    const alternateRulesRoot =
      'test-fixtures/source-sets/independent/alternate-rules';
    const alternate = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        sourceSet: {
          ...sourceSet,
          allowedRoots: [alternateRulesRoot, contentRoot, bundlesRoot],
          entries: [
            {
              ...sourceSet.entries[0]!,
              sourceRoot: alternateRulesRoot,
            },
            sourceSet.entries[1]!,
            {
              ...sourceSet.entries[2]!,
              module: 'src/alternate.ts',
            },
          ],
        },
      },
      gatewayRoot,
    );
    expect(alternate.ok).toBe(true);
    if (!alternate.ok) return;
    expect(alternate.catalog.ruleset.id).toBe(
      'rulebench.independent.alternate',
    );
    expect(alternate.catalog.contentPacks[0]?.id).toBe(
      'rulebench.independent.content',
    );
  });

  it('rejects undeclared cross-root imports', async () => {
    const sourceSet = oneSource(
      'test-fixtures/source-sets/independent/content',
      ['ruleset', 'contentPack', 'playBundle', 'scenarioTemplate'],
    );
    const result = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        sourceSet: {
          ...sourceSet,
          entries: [
            {
              ...sourceSet.entries[0]!,
              module: 'src/invalid-import.ts',
            },
          ],
        },
      },
      gatewayRoot,
    );
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.diagnostics[0]?.code).toBe(
      'PLAY_BUNDLE_SOURCE_IMPORT_OUTSIDE_ALLOWED_ROOTS',
    );
  });

  it('reports the source entries for duplicate identities across roots', async () => {
    const minimal = oneSource(minimalRoot);
    const duplicateRoot = 'test-fixtures/source-sets/duplicate-content';
    const result = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        sourceSet: {
          schemaVersion: 1,
          allowedRoots: [minimalRoot, duplicateRoot],
          entries: [
            minimal.entries[0]!,
            {
              id: 'duplicate-content',
              label: 'Duplicate content',
              sourceRoot: duplicateRoot,
              module: 'src/index.ts',
              exportKinds: ['contentPack'],
            },
          ],
        },
      },
      gatewayRoot,
    );
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.diagnostics[0]).toMatchObject({
      code: 'PLAY_BUNDLE_SOURCE_IDENTITY_DUPLICATE',
      path: '$.sourceSet.entries[1]',
    });
    expect(result.diagnostics[0]?.message).toContain(
      'primary and duplicate-content',
    );
  });
});
