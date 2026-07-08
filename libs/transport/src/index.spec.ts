import { describe, expect, it } from 'vitest';
import {
  createFakeRulebenchTransport,
  defaultCombatSessionCatalog,
  defaultCombatControlHistoryReadout,
  defaultCombatSessionStepReadout,
  defaultContentValidationCatalog,
  defaultContentValidationReport,
  defaultRulesetCatalog,
  defaultScenarioCatalog,
  defaultScenarioReadout,
} from './index';
import { rustBackedCombatSessionCatalog } from './generated/rust-combat-session';
import {
  rustBackedContentValidationCatalog,
  rustBackedRulesetCatalog,
  rustBackedScenarioCatalog,
} from './generated/rust-scenario-catalog';

describe('RulebenchTransport fixtures', () => {
  it('uses the checked Rust-backed ruleset catalog as the default transport payload', async () => {
    expect(defaultRulesetCatalog).toBe(rustBackedRulesetCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadRulesetCatalog();

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultRulesetCatalog);
      expect(result.value.selectedRulesetId).toBe('asha-rulebench.hexing-bolt.v0');
      expect(result.value.rulesets).toEqual([
        {
          id: 'asha-rulebench.hexing-bolt.v0',
          name: 'Hexing Bolt Fixture Rules',
          version: '0.0.0',
          summary: 'Local single-action fixture ruleset for authority incubation.',
        },
      ]);
    }
  });

  it('uses the checked Rust-backed content validation catalog as the default transport payload', async () => {
    expect(defaultContentValidationCatalog).toBe(rustBackedContentValidationCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadContentValidationReport('hexing-bolt-hit');

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultContentValidationReport);
      expect(result.value.scenarioId).toBe('hexing-bolt-hit');
      expect(result.value.scenarioTitle).toBe('Hexing Bolt Hit');
      expect(result.value.report).toEqual({
        accepted: true,
        errorCount: 0,
        warningCount: 0,
        diagnostics: [],
      });
    }
  });

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

  it('classifies missing content validation report ids as not found', async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadContentValidationReport('missing-scenario');

    expect(result).toEqual({
      ok: false,
      error: {
        kind: 'not-found',
        message: 'Content validation report not found: missing-scenario',
        retryable: false,
      },
    });
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

  it('uses the checked Rust-backed combat session catalog as the default transport payload', async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const catalogResult = await transport.loadSessionCatalog();
    const result = await transport.loadSessionStep('hexing-bolt-opening-exchange', 'adept-hexing-bolt-hit');

    expect(catalogResult.ok).toBe(true);
    if (catalogResult.ok) {
      expect(catalogResult.value[0]?.steps.map((step) => step.id)).toEqual([
        'adept-hexing-bolt-hit',
        'adept-hexing-bolt-miss',
        'adept-hexing-bolt-self-target-rejected',
      ]);
    }
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatSessionStepReadout);
      expect(result.value.combatLog[0]?.eventTypes).toEqual([
        'ActionUsed',
        'AttackRolled',
        'DamageApplied',
        'ModifierApplied',
      ]);
      expect(result.value.stateBefore.combatants[1]?.hitPoints.current).toBe(18);
      expect(result.value.stateAfter.combatants[1]?.conditions).toEqual(['rattled']);
    }
  });

  it('uses the checked Rust-backed control history fixture as the default transport payload', async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadSessionControlHistory('hexing-bolt-control-sequence');

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatControlHistoryReadout);
      expect(result.value.history.map((entry) => entry.commandKind)).toEqual([
        'explicitStart',
        'advanceTurn',
        'explicitEnd',
        'advanceTurn',
      ]);
      expect(result.value.history.map((entry) => entry.decisionKind)).toEqual([
        'accepted',
        'accepted',
        'accepted',
        'rejectedByLifecycle',
      ]);
      expect(result.value.history[0]?.lifecycleTransitionSequence).toBe(0);
      expect(result.value.history[1]?.turnTransitionSequence).toBe(0);
      expect(result.value.history[3]?.reason).toBe('Combat is already ended.');
    }
  });

  it('classifies missing combat control history ids as not found', async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionControlHistory('missing-control-history');

    expect(result).toEqual({
      ok: false,
      error: {
        kind: 'not-found',
        message: 'Combat control history not found: missing-control-history',
        retryable: false,
      },
    });
  });

  it('classifies missing combat session ids as not found', async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionStep('missing-session', 'missing-step');

    expect(result).toEqual({
      ok: false,
      error: {
        kind: 'not-found',
        message: 'Combat session not found: missing-session',
        retryable: false,
      },
    });
  });

  it('classifies missing combat session step ids as not found', async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionStep('hexing-bolt-opening-exchange', 'missing-step');

    expect(result).toEqual({
      ok: false,
      error: {
        kind: 'not-found',
        message: 'Combat session step not found: hexing-bolt-opening-exchange / missing-step',
        retryable: false,
      },
    });
  });
});
