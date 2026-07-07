import { describe, expect, it } from 'vitest';
import { createFakeRulebenchTransport, defaultScenarioReadout } from './index';
import { rustBackedScenarioReadout } from './generated/rust-scenario-readout';

describe('RulebenchTransport fixtures', () => {
  it('uses the checked Rust-backed scenario readout as the default transport payload', async () => {
    expect(defaultScenarioReadout).toBe(rustBackedScenarioReadout);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadScenario();

    expect(result.ok).toBe(true);
    if (result.ok) {
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
});
