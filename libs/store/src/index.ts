import { InjectionToken, Injectable, signal } from '@angular/core';
import type { Provider, Signal } from '@angular/core';
import { projectRulebenchStatus, type RulebenchStatusView } from '@asha-rulebench/domain';
import { browserClock, type ClockPort } from '@asha-rulebench/platform';
import type { ClassifiedError } from '@asha-rulebench/protocol';
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
  private readonly _status = signal<AsyncState<RulebenchStatusView>>({ kind: 'idle' });
  readonly status: Signal<AsyncState<RulebenchStatusView>> = this._status.asReadonly();

  constructor(
    private readonly transport: RulebenchTransport,
    private readonly clock: ClockPort,
  ) {}

  async load(): Promise<void> {
    this._status.set({ kind: 'loading' });
    const result = await this.transport.loadStatus();
    this._status.set(
      result.ok
        ? { kind: 'data', value: projectRulebenchStatus(result.value) }
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
