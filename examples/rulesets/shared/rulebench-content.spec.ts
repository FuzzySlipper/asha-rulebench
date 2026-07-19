import { canonicalJson, prepareRulesetCompilation } from '@asha-rpg/authoring';
import { describe, expect, it } from 'vitest';

import {
  FIELD_MANUAL_V1_1_WORKSPACE,
  FIELD_MANUAL_V1_WORKSPACE,
  INVALID_MISSING_SUPPORT_WORKSPACE,
} from './rulebench-content.js';

describe('example Rulebench ruleset package graph', () => {
  it('closes four TypeScript-only actions through an exact materialized dependency lock', () => {
    const first = prepareRuleset(FIELD_MANUAL_V1_WORKSPACE);
    const second = prepareRuleset(FIELD_MANUAL_V1_WORKSPACE);

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
    expect(Object.isFrozen(FIELD_MANUAL_V1_WORKSPACE.packages)).toBe(true);
    expect(
      FIELD_MANUAL_V1_WORKSPACE.packages.every((source) =>
        Object.isFrozen(source),
      ),
    ).toBe(true);
    expect(
      FIELD_MANUAL_V1_WORKSPACE.packages.map(
        (source) => source.manifest.identity.id,
      ),
    ).toEqual([
      'rulebench.field-manual',
      'rulebench.primitives',
      'rulebench.stormfront-balance',
      'rulebench.stormfront-presentation',
    ]);
  });

  it('exports independent declarations and exposes invalid graph diagnostics', () => {
    const accepted = prepareRuleset(FIELD_MANUAL_V1_WORKSPACE);
    const upgrade = prepareRuleset(FIELD_MANUAL_V1_1_WORKSPACE);
    const rejected = prepareRuleset(INVALID_MISSING_SUPPORT_WORKSPACE);

    expect(accepted.ok).toBe(true);
    if (accepted.ok) {
      const source = canonicalJson(accepted.prepared);
      expect(source).toContain('rulebench.wardbreaker-volley');
      expect(source).toContain('"count":5');
      expect(source).toContain('"sides":4');
      expect(source).toContain('"sides":6');
    }
    expect(upgrade.ok).toBe(true);
    if (accepted.ok && upgrade.ok) {
      const upgradeSource = canonicalJson(upgrade.prepared);
      const acceptedSource = canonicalJson(accepted.prepared);
      expect(upgradeSource).toContain(
        '"compositionIdentity":{"id":"rulebench.fresh-start","version":"1.1.0"}',
      );
      expect(upgradeSource).not.toBe(acceptedSource);
      expect(upgradeSource).toContain('"value":2');
    }
    expect(rejected.ok).toBe(false);
    if (!rejected.ok) {
      const diagnostic = rejected.diagnostics.find(
        (entry) => entry.code === 'RULESET_DEFINITION_REFERENCE_MISSING',
      );
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

function prepareRuleset(declaration: typeof FIELD_MANUAL_V1_WORKSPACE) {
  return prepareRulesetCompilation(declaration);
}
