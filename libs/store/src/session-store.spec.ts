import { describe, expect, it } from 'vitest';
import type { ClockPort } from '@asha-rulebench/platform';
import type { ClassifiedError } from '@asha-rulebench/protocol';
import { defaultScenarioReadout, type RulebenchTransport } from '@asha-rulebench/transport';
import { SessionStore } from './index';

const fixedClock: ClockPort = {
  now: () => new Date('2026-07-07T00:00:00.000Z'),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

describe('SessionStore', () => {
  it('loads a scenario through transport and exposes projected async data', async () => {
    const store = new SessionStore(
      {
        loadScenario: async () => ({ ok: true, value: defaultScenarioReadout }),
      },
      fixedClock,
    );

    await store.loadScenario();

    const state = store.scenario();
    expect(state.kind).toBe('data');
    if (state.kind === 'data') {
      expect(state.value.title).toBe('Hexing Bolt Opening');
      expect(state.value.selectedAction.actorLabel).toBe('Adept');
      expect(state.value.timeline.map((row) => row.typeLabel)).toContain('DamageApplied');
    }
  });

  it('maps transport failures to error async state without mutating through callers', async () => {
    const error: ClassifiedError = {
      kind: 'network',
      message: 'Scenario transport unavailable',
      retryable: true,
    };
    const transport: RulebenchTransport = {
      loadScenario: async () => ({ ok: false, error }),
    };
    const store = new SessionStore(transport, fixedClock);

    await store.loadScenario();

    expect(store.scenario()).toEqual({ kind: 'error', error });
  });
});
