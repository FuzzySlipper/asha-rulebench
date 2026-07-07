import { describe, expect, it } from 'vitest';
import type { ClockPort } from '@asha-rulebench/platform';
import type { ClassifiedError, RulebenchScenarioCatalogSummaryDto } from '@asha-rulebench/protocol';
import { createFakeRulebenchTransport, defaultScenarioCatalog, defaultScenarioReadout, type RulebenchTransport } from '@asha-rulebench/transport';
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

  it('maps transport failures to error async state without mutating through callers', async () => {
    const error: ClassifiedError = {
      kind: 'network',
      message: 'Scenario transport unavailable',
      retryable: true,
    };
    const summaries: readonly RulebenchScenarioCatalogSummaryDto[] = [];
    const transport: RulebenchTransport = {
      loadCatalog: async () => ({ ok: true, value: summaries }),
      loadScenario: async () => ({ ok: false, error }),
    };
    const store = new SessionStore(transport, fixedClock);

    await store.loadScenario();

    expect(store.scenario()).toEqual({ kind: 'error', error });
  });
});
