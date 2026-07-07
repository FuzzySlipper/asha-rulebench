import { InjectionToken, Injectable, signal } from '@angular/core';
import type { Provider, Signal } from '@angular/core';
import { projectRulebenchScenario, type RulebenchScenarioView } from '@asha-rulebench/domain';
import { browserClock, type ClockPort } from '@asha-rulebench/platform';
import type { ClassifiedError, RulebenchScenarioCatalogSummaryDto } from '@asha-rulebench/protocol';
import { createFakeRulebenchTransport, type RulebenchTransport } from '@asha-rulebench/transport';

export type AsyncState<T> =
  | { readonly kind: 'idle' }
  | { readonly kind: 'loading' }
  | { readonly kind: 'data'; readonly value: T }
  | { readonly kind: 'error'; readonly error: ClassifiedError };

export const RULEBENCH_TRANSPORT = new InjectionToken<RulebenchTransport>('RULEBENCH_TRANSPORT', {
  factory: () => createFakeRulebenchTransport(),
});

export const RULEBENCH_CLOCK = new InjectionToken<ClockPort>('RULEBENCH_CLOCK', {
  factory: () => browserClock,
});

@Injectable()
export class SessionStore {
  private readonly _catalog = signal<AsyncState<readonly RulebenchScenarioCatalogSummaryDto[]>>({ kind: 'idle' });
  readonly catalog: Signal<AsyncState<readonly RulebenchScenarioCatalogSummaryDto[]>> = this._catalog.asReadonly();

  private readonly _selectedScenarioId = signal<string | null>(null);
  readonly selectedScenarioId: Signal<string | null> = this._selectedScenarioId.asReadonly();

  private readonly _scenario = signal<AsyncState<RulebenchScenarioView>>({ kind: 'idle' });
  readonly scenario: Signal<AsyncState<RulebenchScenarioView>> = this._scenario.asReadonly();

  constructor(
    private readonly transport: RulebenchTransport,
    private readonly clock: ClockPort,
  ) {}

  async loadCatalog(): Promise<void> {
    this._catalog.set({ kind: 'loading' });
    const result = await this.transport.loadCatalog();
    if (result.ok) {
      this._catalog.set({ kind: 'data', value: result.value });
      const firstSummary = result.value[0];
      if (this._selectedScenarioId() === null && firstSummary !== undefined) {
        this._selectedScenarioId.set(firstSummary.id);
      }
    } else {
      this._catalog.set({ kind: 'error', error: result.error });
    }
    this.clock.now();
  }

  async selectScenario(scenarioId: string): Promise<void> {
    this._selectedScenarioId.set(scenarioId);
    await this.loadScenario(scenarioId);
  }

  async loadScenario(scenarioId: string | null = this._selectedScenarioId()): Promise<void> {
    this._scenario.set({ kind: 'loading' });
    const result = await this.transport.loadScenario(scenarioId ?? undefined);
    this._scenario.set(
      result.ok
        ? { kind: 'data', value: projectRulebenchScenario(result.value) }
        : { kind: 'error', error: result.error },
    );
    this.clock.now();
  }
}

export function provideRulebenchStoreKernel(): Provider[] {
  return [
    { provide: RULEBENCH_TRANSPORT, useFactory: () => createFakeRulebenchTransport() },
    { provide: RULEBENCH_CLOCK, useValue: browserClock },
    {
      provide: SessionStore,
      deps: [RULEBENCH_TRANSPORT, RULEBENCH_CLOCK],
      useFactory: (transport: RulebenchTransport, clock: ClockPort) => new SessionStore(transport, clock),
    },
  ];
}
