import { describe, expect, it } from 'vitest';

import { loadRulesetWorkspace } from './load-ruleset-workspace.js';

const gatewayRoot = process.cwd();

describe('explicit ruleset workspace loader', () => {
  it('loads independent root workspaces without a source catalog', async () => {
    const first = await loadRulesetWorkspace(
      workspace('examples/rulesets/field-manual-v1'),
      gatewayRoot,
    );
    const second = await loadRulesetWorkspace(
      workspace('examples/rulesets/field-manual-v1_1'),
      gatewayRoot,
    );

    expect(first.ok).toBe(true);
    expect(second.ok).toBe(true);
    if (!first.ok || !second.ok) return;
    expect(first.preparedSource).toContain(
      '"compositionIdentity":{"id":"rulebench.fresh-start","version":"1.0.0"}',
    );
    expect(second.preparedSource).toContain(
      '"compositionIdentity":{"id":"rulebench.fresh-start","version":"1.1.0"}',
    );
  });

  it('anchors source fingerprints to the explicit module without changing materialized semantics', async () => {
    const first = await loadRulesetWorkspace(
      workspace('examples/rulesets/field-manual-v1'),
      gatewayRoot,
    );
    const moved = await loadRulesetWorkspace(
      {
        ...workspace('examples/rulesets/field-manual-v1'),
        module: 'src/moved-ruleset.ts',
      },
      gatewayRoot,
    );

    expect(first.ok).toBe(true);
    expect(moved.ok).toBe(true);
    if (!first.ok || !moved.ok) return;
    expect(sourceFingerprints(first.preparedSource)).not.toEqual(
      sourceFingerprints(moved.preparedSource),
    );
    expect(materializedDefinitions(first.preparedSource)).toEqual(
      materializedDefinitions(moved.preparedSource),
    );
  });

  it('does not discover unexported declarations or side-effect package values', async () => {
    const unexported = await loadRulesetWorkspace(
      {
        ...workspace('examples/rulesets/field-manual-v1'),
        declaration: 'FIELD_MANUAL_V1_WORKSPACE',
      },
      gatewayRoot,
    );
    const sideEffect = await loadRulesetWorkspace(
      workspace('examples/rulesets/side-effect'),
      gatewayRoot,
    );

    expect(unexported.ok).toBe(false);
    if (!unexported.ok) {
      expect(unexported.diagnostics[0]?.code).toBe(
        'RULESET_WORKSPACE_DECLARATION_NOT_EXPORTED',
      );
    }
    expect(sideEffect.ok).toBe(true);
    if (sideEffect.ok) {
      expect(sideEffect.preparedSource).not.toContain('catalog.damage.missing');
    }
  });

  it('reports build failures against the selected module and declaration', async () => {
    const result = await loadRulesetWorkspace(
      workspace('examples/rulesets/invalid-build'),
      gatewayRoot,
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.diagnostics[0]?.code).toBe('RULESET_WORKSPACE_BUILD_FAILED');
    expect(result.diagnostics[0]?.source).toEqual({
      module: 'src/ruleset.ts',
      declaration: 'ruleset',
    });
    expect(result.diagnostics[0]?.message).toContain('TS2322');
  });
});

function workspace(workspaceRoot: string) {
  return {
    workspaceRoot,
    packageRoots: ['.', '../shared'],
    module: 'src/ruleset.ts',
    declaration: 'ruleset',
  };
}

function sourceFingerprints(source: string): readonly string[] {
  const prepared = preparedRecord(source);
  const sourcePackages = prepared['sourcePackages'];
  if (!Array.isArray(sourcePackages)) throw new Error('sourcePackages missing');
  return sourcePackages.map((entry) =>
    requiredString(entry, 'sourceFingerprint'),
  );
}

function materializedDefinitions(source: string): unknown {
  const prepared = preparedRecord(source);
  return prepared['materializedDefinitions'];
}

function preparedRecord(source: string): Readonly<Record<string, unknown>> {
  const value: unknown = JSON.parse(source);
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error('prepared source must be an object');
  }
  return value;
}

function requiredString(value: unknown, key: string): string {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${key} owner must be an object`);
  }
  const member = value[key];
  if (typeof member !== 'string') throw new Error(`${key} must be a string`);
  return member;
}
