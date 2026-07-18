import { canonicalJson } from '@asha-rpg/authoring';
import { describe, expect, it } from 'vitest';

import {
  FRESH_RULESET_PACKAGE_SOURCES,
  prepareFreshRulebenchRuleset,
  prepareRulebenchRulesetSource,
} from './content-authoring.js';

describe('fresh Rulebench ruleset package graph', () => {
  it('closes three TypeScript-only actions through an exact dependency lock', () => {
    const first = prepareFreshRulebenchRuleset();
    const second = prepareFreshRulebenchRuleset();

    expect(first.ok).toBe(true);
    expect(second.ok).toBe(true);
    if (!first.ok || !second.ok) return;

    expect(canonicalJson(first.prepared)).toBe(canonicalJson(second.prepared));
    expect(first.prepared.exportedRoots).toEqual([
      'catalog.damage.storm',
      'catalog.defense.guard',
      'catalog.modifier.exposed',
      'catalog.resource.focus',
      'catalog.stat.power',
      'rulebench.arc-lash',
      'rulebench.tactical-advance',
      'rulebench.wardbreaker-volley',
    ]);
    expect(first.prepared.dependencyLock).toHaveLength(3);
    expect(
      first.prepared.materializedDefinitions.map((definition) => definition.id),
    ).toEqual(first.prepared.exportedRoots);
    expect(first.prepared.requiredOperations).toContainEqual({
      id: 'operation.openReaction',
      version: 1,
    });
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

  it('prepares the selected source on demand and exposes invalid graph diagnostics', () => {
    const accepted = prepareRulebenchRulesetSource('fresh');
    const rejected = prepareRulebenchRulesetSource('missingSupport');

    expect(accepted.ok).toBe(true);
    if (accepted.ok) {
      expect(accepted.preparedSource).toContain('rulebench.wardbreaker-volley');
      expect(accepted.preparedSource).toContain('"count":5');
      expect(accepted.preparedSource).toContain('"sides":4');
      expect(accepted.preparedSource).toContain('"sides":6');
    }
    expect(rejected.ok).toBe(false);
    if (!rejected.ok) {
      const diagnostic = rejected.diagnostics[0];
      expect(diagnostic?.code).toBe('RULESET_DEFINITION_REFERENCE_MISSING');
      expect(diagnostic?.packageId).toBe('rulebench.field-manual');
      expect(diagnostic?.definitionId).toBe('rulebench.arc-lash');
      expect(diagnostic?.source).toEqual({
        module: 'packages/rulebench-field-manual.ts',
        declaration: 'rulebench.arc-lash',
      });
    }
  });
});
