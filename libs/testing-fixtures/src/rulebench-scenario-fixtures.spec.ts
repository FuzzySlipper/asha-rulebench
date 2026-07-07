import { describe, expect, it } from 'vitest';
import type { RulebenchScenarioReadoutDto } from '@asha-rulebench/protocol';
import { makeTwoCombatantScenarioReadout, twoCombatantScenarioReadout } from './index';

describe('twoCombatantScenarioReadout', () => {
  it('covers board, action, target, events, trace, and final state', () => {
    expect(twoCombatantScenarioReadout.grid.width).toBeGreaterThan(0);
    expect(twoCombatantScenarioReadout.combatants).toHaveLength(2);
    expect(twoCombatantScenarioReadout.selectedAction.actorId).toBe('entity-adept');
    expect(twoCombatantScenarioReadout.selectedTarget.legality).toBe('accepted');
    expect(twoCombatantScenarioReadout.domainEvents.map((event) => event.type)).toEqual([
      'ActionUsed',
      'AttackRolled',
      'DamageApplied',
      'ModifierApplied',
    ]);
    expect(twoCombatantScenarioReadout.trace.map((entry) => entry.phase)).toEqual([
      'proposal',
      'validation',
      'resolution',
      'commit',
    ]);
    expect(twoCombatantScenarioReadout.finalState.combatants[1]?.conditions).toEqual(['rattled']);
  });

  it('constructs typed fixture variants without changing accepted readout shape', () => {
    const scenario: RulebenchScenarioReadoutDto = makeTwoCombatantScenarioReadout({
      title: 'Variant title',
      seedLabel: 'roll-stream:20',
    });

    expect(scenario.title).toBe('Variant title');
    expect(scenario.seedLabel).toBe('roll-stream:20');
    expect(scenario.domainEvents).toBe(twoCombatantScenarioReadout.domainEvents);
  });
});
