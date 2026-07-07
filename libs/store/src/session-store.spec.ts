import { describe, expect, it } from 'vitest';
import type { ClockPort } from '@asha-rulebench/platform';
import type {
  ClassifiedError,
  RulebenchCombatSessionSummaryDto,
  RulebenchScenarioCatalogSummaryDto,
} from '@asha-rulebench/protocol';
import {
  createFakeRulebenchTransport,
  defaultCombatSessionCatalog,
  defaultCombatSessionStepReadout,
  defaultScenarioCatalog,
  defaultScenarioReadout,
  type RulebenchTransport,
} from '@asha-rulebench/transport';
import { SessionStore } from './index';

const fixedClock: ClockPort = {
  now: () => new Date('2026-07-07T00:00:00.000Z'),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

describe('SessionStore', () => {
  it('loads the Rust-backed catalog through transport and records the first selected id', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadCatalog();

    const catalog = store.catalog();
    expect(catalog.kind).toBe('data');
    if (catalog.kind === 'data') {
      expect(catalog.value.map((summary) => summary.id)).toEqual([
        'hexing-bolt-hit',
        'hexing-bolt-miss',
        'hexing-bolt-self-target-rejected',
      ]);
    }
    expect(store.selectedScenarioId()).toBe('hexing-bolt-hit');
  });

  it('loads a scenario through transport and exposes projected async data', async () => {
    const store = new SessionStore(
      {
        loadCatalog: async () => ({ ok: true, value: defaultScenarioCatalog.summaries }),
        loadScenario: async () => ({ ok: true, value: defaultScenarioReadout }),
        loadSessionCatalog: async () => ({ ok: true, value: defaultCombatSessionCatalog.summaries }),
        loadSessionStep: async () => ({ ok: true, value: defaultCombatSessionStepReadout }),
      },
      fixedClock,
    );

    await store.loadScenario();

    const state = store.scenario();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.title).toBe('Hexing Bolt Hit');
      expect(state.value.selectedAction.actorLabel).toBe('Adept');
      expect(state.value.timeline.map((row) => row.typeLabel)).toContain('DamageApplied');
    }
  });

  it('selects a scenario id and exposes the selected Rust-backed readout', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadCatalog();
    await store.selectScenario('hexing-bolt-miss');

    expect(store.selectedScenarioId()).toBe('hexing-bolt-miss');
    const state = store.scenario();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.title).toBe('Hexing Bolt Miss');
      expect(state.value.timeline.map((row) => row.typeLabel)).toEqual(['ActionUsed', 'AttackRolled']);
      expect(state.value.finalState.combatants[1]?.hitPointLabel).toBe('18/18 HP');
    }
  });

  it('maps missing scenario ids to error async state', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.selectScenario('missing-scenario');

    expect(store.selectedScenarioId()).toBe('missing-scenario');
    expect(store.scenario()).toEqual({
      kind: 'error',
      error: {
        kind: 'not-found',
        message: 'Scenario not found: missing-scenario',
        retryable: false,
      },
    });
  });

  it('loads the Rust-backed combat session catalog and records first session step selection', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadSessionCatalog();

    const catalog = store.sessionCatalog();
    expect(catalog.kind).toBe('data');
    if (catalog.kind === 'data') {
      expect(catalog.value[0]?.id).toBe('hexing-bolt-opening-exchange');
      expect(catalog.value[0]?.steps.map((step) => step.id)).toEqual([
        'adept-hexing-bolt-hit',
        'adept-hexing-bolt-miss',
        'adept-hexing-bolt-self-target-rejected',
      ]);
    }
    expect(store.selectedSessionId()).toBe('hexing-bolt-opening-exchange');
    expect(store.selectedSessionStepId()).toBe('adept-hexing-bolt-hit');
  });

  it('loads a combat session step through transport and exposes projected async data', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadSessionStep();

    const state = store.sessionStep();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(store.selectedSessionId()).toBe('hexing-bolt-opening-exchange');
      expect(store.selectedSessionStepId()).toBe('adept-hexing-bolt-hit');
      expect(state.value.step.title).toBe('Adept hits Raider');
      expect(state.value.command.rollStreamLabel).toBe('17,5');
      expect(state.value.combatLog[0]?.eventTypeLabels).toEqual([
        'ActionUsed',
        'AttackRolled',
        'DamageApplied',
        'ModifierApplied',
      ]);
      expect(state.value.stateBefore.combatants[1]?.hitPointLabel).toBe('18/18 HP');
      expect(state.value.stateAfter.combatants[1]?.conditionLabels).toEqual(['rattled']);
    }
  });

  it('selects a combat session step and exposes the selected Rust-backed readout', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.selectSessionStep('hexing-bolt-opening-exchange', 'adept-hexing-bolt-self-target-rejected');

    expect(store.selectedSessionId()).toBe('hexing-bolt-opening-exchange');
    expect(store.selectedSessionStepId()).toBe('adept-hexing-bolt-self-target-rejected');
    const state = store.sessionStep();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.step.outcomeLabel).toBe('Rejected target');
      expect(state.value.scenario.selectedTarget.legalityLabel).toBe('Rejected');
      expect(state.value.scenario.timeline).toEqual([]);
      expect(state.value.stateAfter.combatants[1]?.hitPointLabel).toBe('9/18 HP');
    }
  });

  it('steps forward and backward through combat session summaries without computing combat state', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadSessionCatalog();
    await store.loadSessionStep();
    await store.nextSessionStep();

    expect(store.selectedSessionStepId()).toBe('adept-hexing-bolt-miss');
    let state = store.sessionStep();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.step.outcomeLabel).toBe('Accepted miss');
      expect(state.value.stateAfter.combatants[1]?.hitPointLabel).toBe('9/18 HP');
    }

    await store.previousSessionStep();

    expect(store.selectedSessionStepId()).toBe('adept-hexing-bolt-hit');
    state = store.sessionStep();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.step.outcomeLabel).toBe('Accepted hit');
    }
  });

  it('maps missing combat session ids to error async state', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.selectSessionStep('missing-session', 'missing-step');

    expect(store.selectedSessionId()).toBe('missing-session');
    expect(store.selectedSessionStepId()).toBe('missing-step');
    expect(store.sessionStep()).toEqual({
      kind: 'error',
      error: {
        kind: 'not-found',
        message: 'Combat session not found: missing-session',
        retryable: false,
      },
    });
  });

  it('maps missing combat session step ids to error async state', async () => {
    const store = new SessionStore(createFakeRulebenchTransport(), fixedClock);

    await store.selectSessionStep('hexing-bolt-opening-exchange', 'missing-step');

    expect(store.sessionStep()).toEqual({
      kind: 'error',
      error: {
        kind: 'not-found',
        message: 'Combat session step not found: hexing-bolt-opening-exchange / missing-step',
        retryable: false,
      },
    });
  });

  it('maps transport failures to error async state without mutating through callers', async () => {
    const error: ClassifiedError = {
      kind: 'network',
      message: 'Scenario transport unavailable',
      retryable: true,
    };
    const summaries: readonly RulebenchScenarioCatalogSummaryDto[] = [];
    const sessionSummaries: readonly RulebenchCombatSessionSummaryDto[] = [];
    const transport: RulebenchTransport = {
      loadCatalog: async () => ({ ok: true, value: summaries }),
      loadScenario: async () => ({ ok: false, error }),
      loadSessionCatalog: async () => ({ ok: true, value: sessionSummaries }),
      loadSessionStep: async () => ({ ok: false, error }),
    };
    const store = new SessionStore(transport, fixedClock);

    await store.loadScenario();
    await store.loadSessionStep();

    expect(store.scenario()).toEqual({ kind: 'error', error });
    expect(store.sessionStep()).toEqual({ kind: 'error', error });
  });
});
