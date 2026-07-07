import { InjectionToken, Injectable, signal } from '@angular/core';
import type { Provider, Signal } from '@angular/core';
import { projectTemplateStatus, type TemplateStatusView } from '@template/domain';
import { browserClock, type ClockPort } from '@template/platform';
import type { ClassifiedError } from '@template/protocol';
import { createFakeTemplateTransport, type TemplateTransport } from '@template/transport';

export type AsyncState<T> =
  | { readonly kind: 'idle' }
  | { readonly kind: 'loading' }
  | { readonly kind: 'data'; readonly value: T }
  | { readonly kind: 'error'; readonly error: ClassifiedError };

export const TEMPLATE_TRANSPORT = new InjectionToken<TemplateTransport>('TEMPLATE_TRANSPORT', {
  factory: () => createFakeTemplateTransport(),
});

export const TEMPLATE_CLOCK = new InjectionToken<ClockPort>('TEMPLATE_CLOCK', {
  factory: () => browserClock,
});

@Injectable()
export class SessionStore {
  private readonly _status = signal<AsyncState<TemplateStatusView>>({ kind: 'idle' });
  readonly status: Signal<AsyncState<TemplateStatusView>> = this._status.asReadonly();

  constructor(
    private readonly transport: TemplateTransport,
    private readonly clock: ClockPort,
  ) {}

  async load(): Promise<void> {
    this._status.set({ kind: 'loading' });
    const result = await this.transport.loadStatus();
    this._status.set(
      result.ok
        ? { kind: 'data', value: projectTemplateStatus(result.value) }
        : { kind: 'error', error: result.error },
    );
    this.clock.now();
  }
}

export function provideTemplateStoreKernel(): Provider[] {
  return [
    { provide: TEMPLATE_TRANSPORT, useFactory: () => createFakeTemplateTransport() },
    { provide: TEMPLATE_CLOCK, useValue: browserClock },
    {
      provide: SessionStore,
      deps: [TEMPLATE_TRANSPORT, TEMPLATE_CLOCK],
      useFactory: (transport: TemplateTransport, clock: ClockPort) => new SessionStore(transport, clock),
    },
  ];
}
