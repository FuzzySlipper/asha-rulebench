import { describe, expect, it } from 'vitest';
import { createFakeRulebenchTransport, defaultScenarioCatalog, defaultScenarioReadout } from './index';
import { rustBackedScenarioCatalog } from './generated/rust-scenario-catalog';

describe('RulebenchTransport fixtures', () => {
  it('uses the checked Rust-backed scenario catalog as the default transport payload', async () => {
    expect(defaultScenarioCatalog).toBe(rustBackedScenarioCatalog);

    const transport = createFakeRulebenchTransport();
    const catalogResult = await transport.loadCatalog();
    const result = await transport.loadScenario('hexing-bolt-hit');

    expect(catalogResult.ok).toBe(true);
    if (catalogResult.ok) {
      expect(catalogResult.value.map((summary) => summary.id)).toEqual([
        'hexing-bolt-hit',
        'hexing-bolt-miss',
        'hexing-bolt-self-target-rejected',
      ]);
    }
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultScenarioReadout);
      expect(result.value.selectedAction.attack.modifier).toBe(4);
      expect(result.value.domainEvents.map((event) => event.type)).toEqual([
        'ActionUsed',
        'AttackRolled',
        'DamageApplied',
        'ModifierApplied',
      ]);
      expect(result.value.trace.at(-1)?.phase).toBe('commit');
      expect(result.value.finalState.combatants[1]?.conditions).toEqual(['rattled']);
    }
  });

  it('classifies missing scenario ids as not found', async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadScenario('missing-scenario');

    expect(result).toEqual({
      ok: false,
      error: {
        kind: 'not-found',
        message: 'Scenario not found: missing-scenario',
        retryable: false,
      },
    });
  });
});
