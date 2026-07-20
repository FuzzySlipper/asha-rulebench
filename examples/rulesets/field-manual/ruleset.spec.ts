import { canonicalJson, prepareRulesetCompilation } from '@asha-rpg/authoring';
import { describe, expect, it } from 'vitest';

import { ruleset } from './ruleset.js';

describe('example Rulebench ruleset package graph', () => {
  it('closes four TypeScript-only actions through an exact materialized dependency lock', () => {
    const first = prepareRuleset(ruleset);
    const second = prepareRuleset(ruleset);

    expect(first.ok).toBe(true);
    expect(second.ok).toBe(true);
    if (!first.ok || !second.ok) return;

    expect(canonicalJson(first.prepared)).toBe(canonicalJson(second.prepared));
    expect(first.prepared.exportedRoots).toEqual([
      'rulebench.arc-lash',
      'rulebench.arc-lash-stormfront',
      'rulebench.tactical-advance',
      'rulebench.wardbreaker-volley',
    ]);
    expect(first.prepared.dependencyLock).toHaveLength(6);
    expect(
      first.prepared.materializedDefinitions.map((definition) => definition.id),
    ).toEqual([
      'catalog.damage.storm',
      'catalog.defense.guard',
      'catalog.modifier.exposed',
      'catalog.resource.focus',
      'catalog.stat.power',
      ...first.prepared.exportedRoots,
    ]);
    expect(
      first.prepared.materializedDefinitions
        .filter((definition) => definition.visibility === 'support')
        .map((definition) => definition.id),
    ).toEqual([
      'catalog.damage.storm',
      'catalog.defense.guard',
      'catalog.modifier.exposed',
      'catalog.resource.focus',
      'catalog.stat.power',
    ]);
    expect(first.prepared.requiredOperations).toContainEqual({
      id: 'operation.openReaction',
      version: 1,
    });
    expect(first.prepared.derivationProvenance).toHaveLength(1);
    expect(first.prepared.derivationProvenance[0]?.mixins).toHaveLength(2);
    expect(first.prepared.overlayProvenance).toHaveLength(2);
    expect(
      first.prepared.overlayProvenance.map((overlay) => overlay.plane),
    ).toEqual(['semantic', 'presentation']);
  });

  it('contains immutable declarations and no ambient registration', () => {
    expect(Object.isFrozen(ruleset.packages)).toBe(true);
    expect(ruleset.packages.every((source) => Object.isFrozen(source))).toBe(
      true,
    );
    expect(
      ruleset.packages.map((source) => source.manifest.identity.id),
    ).toEqual([
      'rulebench.field-manual',
      'rulebench.primitives',
      'rulebench.stormfront-balance',
      'rulebench.stormfront-presentation',
    ]);
  });

  it('exports a self-contained declaration with the expected materialized semantics', () => {
    const accepted = prepareRuleset(ruleset);

    expect(accepted.ok).toBe(true);
    if (accepted.ok) {
      const source = canonicalJson(accepted.prepared);
      expect(source).toContain('rulebench.wardbreaker-volley');
      expect(source).toContain('"count":5');
      expect(source).toContain('"sides":4');
      expect(source).toContain('"sides":6');
    }
  });
});

function prepareRuleset(declaration: typeof ruleset) {
  return prepareRulesetCompilation(declaration);
}
