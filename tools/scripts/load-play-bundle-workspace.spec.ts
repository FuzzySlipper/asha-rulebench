import { describe, expect, it } from 'vitest';

import { loadPlayBundleWorkspace } from './load-play-bundle-workspace.js';

const gatewayRoot = process.cwd();
const minimalRoot = 'test-fixtures/rulesets/minimal';

describe('canonical Ruleset root loader', () => {
  it('discovers distinct Ruleset, Content Pack, and PlayBundle declarations', async () => {
    const result = await loadPlayBundleWorkspace(
      { operation: 'inspect', rulesetRoot: minimalRoot },
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
  });

  it('prepares only an exact declared Content Pack selection', async () => {
    const prepared = await loadPlayBundleWorkspace(
      {
        operation: 'compile',
        rulesetRoot: minimalRoot,
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
        rulesetRoot: minimalRoot,
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

  it('reports build and canonical-layout failures without starting a host', async () => {
    const invalidBuild = await loadPlayBundleWorkspace(
      {
        operation: 'inspect',
        rulesetRoot: 'test-fixtures/rulesets/invalid-build',
      },
      gatewayRoot,
    );
    expect(invalidBuild.ok).toBe(false);
    if (!invalidBuild.ok) {
      expect(invalidBuild.diagnostics[0]?.message).toContain('TS2322');
    }

    const invalidLayout = await loadPlayBundleWorkspace(
      { operation: 'inspect', rulesetRoot: 'test-fixtures/not-a-ruleset' },
      gatewayRoot,
    );
    expect(invalidLayout.ok).toBe(false);
    if (!invalidLayout.ok) {
      expect(invalidLayout.diagnostics[0]?.code).toBe(
        'RULESET_ROOT_LAYOUT_INVALID',
      );
    }
  });
});
