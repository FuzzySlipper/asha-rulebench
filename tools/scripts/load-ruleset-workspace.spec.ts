import { describe, expect, it } from 'vitest';

import { loadRulesetWorkspace } from './load-ruleset-workspace.js';

const gatewayRoot = process.cwd();

describe('canonical ruleset root loader', () => {
  it('infers independent roots and their shared foundations without a source catalog', async () => {
    const first = await loadRulesetWorkspace(
      root('examples/rulesets/field-manual'),
      gatewayRoot,
    );
    const second = await loadRulesetWorkspace(
      root('examples/rulesets/ember-skirmish'),
      gatewayRoot,
    );
    const upgrade = await loadRulesetWorkspace(
      root('examples/rulesets/field-manual-next'),
      gatewayRoot,
    );

    expect(first.ok).toBe(true);
    expect(second.ok).toBe(true);
    expect(upgrade.ok).toBe(true);
    if (!first.ok || !second.ok || !upgrade.ok) return;
    expect(first.preparedSource).toContain(
      '"compositionIdentity":{"id":"rulebench.fresh-start","version":"1.0.0"}',
    );
    expect(second.preparedSource).toContain(
      '"compositionIdentity":{"id":"rulebench.ember-skirmish.demo","version":"1.0.0"}',
    );
    expect(first.preparedSource).toContain('rulebench.primitives');
    expect(second.preparedSource).toContain('rulebench.primitives');
    expect(upgrade.preparedSource).toContain(
      '"compositionIdentity":{"id":"rulebench.fresh-start","version":"1.1.0"}',
    );
  });

  it('rejects sibling-ruleset imports outside the selected root', async () => {
    const escaped = await loadRulesetWorkspace(
      root('examples/rulesets/escaped-import'),
      gatewayRoot,
    );

    expect(escaped.ok).toBe(false);
    if (!escaped.ok) {
      expect(escaped.diagnostics[0]?.code).toBe(
        'RULESET_WORKSPACE_IMPORT_OUTSIDE_PACKAGE_ROOTS',
      );
      expect(escaped.diagnostics[0]?.message).toContain(
        'examples/rulesets/field-manual/',
      );
    }
  });

  it('fails closed when the selected directory is not a canonical rulesets child', async () => {
    const result = await loadRulesetWorkspace(
      root('examples/foundations/d20'),
      gatewayRoot,
    );

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.diagnostics[0]?.code).toBe('RULESET_ROOT_LAYOUT_INVALID');
    }
  });

  it('reports build failures against the selected module and declaration', async () => {
    const result = await loadRulesetWorkspace(
      root('examples/rulesets/invalid-build'),
      gatewayRoot,
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.diagnostics[0]?.code).toBe('RULESET_WORKSPACE_BUILD_FAILED');
    expect(result.diagnostics[0]?.source).toEqual({
      module: 'ruleset.ts',
      declaration: 'ruleset',
    });
    expect(result.diagnostics[0]?.message).toContain('TS2322');
  });
});

function root(rulesetRoot: string) {
  return { rulesetRoot };
}
