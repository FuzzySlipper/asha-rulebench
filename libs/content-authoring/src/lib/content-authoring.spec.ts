import { canonicalJson } from '@asha-rpg/authoring';
import { describe, expect, it } from 'vitest';

import {
  FRESH_RULESET_PACKAGE_SOURCES,
  prepareFreshRulebenchRuleset,
} from './content-authoring.js';

describe('fresh Rulebench ruleset package graph', () => {
  it('closes one TypeScript-only action through an exact dependency lock', () => {
    const first = prepareFreshRulebenchRuleset();
    const second = prepareFreshRulebenchRuleset();

    expect(first.ok).toBe(true);
    expect(second.ok).toBe(true);
    if (!first.ok || !second.ok) return;

    expect(canonicalJson(first.prepared)).toBe(canonicalJson(second.prepared));
    expect(first.prepared.exportedRoots).toEqual([
      'catalog.damage.radiant',
      'rulebench.signal-flare',
    ]);
    expect(first.prepared.dependencyLock).toHaveLength(3);
    expect(
      first.prepared.materializedDefinitions.map((definition) => definition.id),
    ).toEqual(['catalog.damage.radiant', 'rulebench.signal-flare']);
    expect(first.prepared.derivationProvenance).toEqual([]);
    expect(first.prepared.overlayProvenance).toEqual([]);
  });

  it('contains immutable declarations and no ambient registration', () => {
    expect(Object.isFrozen(FRESH_RULESET_PACKAGE_SOURCES)).toBe(true);
    expect(
      FRESH_RULESET_PACKAGE_SOURCES.every((source) => Object.isFrozen(source)),
    ).toBe(true);
    expect(
      FRESH_RULESET_PACKAGE_SOURCES.map(
        (source) => source.manifest.identity.id,
      ),
    ).toEqual(['rulebench.field-manual', 'rulebench.primitives']);
  });
});
