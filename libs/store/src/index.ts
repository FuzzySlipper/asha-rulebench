export * from "./replay-review-store";
export * from "./async-state";
export * from "./live-combat-store";
export * from "./content-workspace-store";

import { InjectionToken, Injectable, signal } from '@angular/core';
import type { Provider, Signal } from '@angular/core';
import {
  projectRulebenchViewerSessionStep,
  projectRulebenchViewerScenario,
  type RulebenchCombatSessionStepView,
  type RulebenchScenarioView,
} from '@asha-rulebench/domain';
import { browserClock, type ClockPort } from '@asha-rulebench/platform';
import type {
  RulebenchLiveTransportErrorDto,
  RulebenchViewerScenarioSummaryDto,
  RulebenchViewerSessionSummaryDto,
} from '@asha-rulebench/protocol';
import type { RulebenchLiveTransport } from '@asha-rulebench/transport';
import type { AsyncState } from "./async-state";
import { provideLiveCombatStoreKernel, RULEBENCH_LIVE_TRANSPORT } from './live-combat-store';
import { provideContentWorkbenchStoreKernel } from './content-workspace-store';

export const RULEBENCH_CLOCK = new InjectionToken<ClockPort>('RULEBENCH_CLOCK', {
  factory: () => browserClock,
});

@Injectable()
export class SessionStore {
  private readonly _catalog = signal<AsyncState<readonly RulebenchViewerScenarioSummaryDto[], RulebenchLiveTransportErrorDto>>({ kind: 'idle' });
  readonly catalog = this._catalog.asReadonly();

  private readonly _selectedScenarioId = signal<string | null>(null);
  readonly selectedScenarioId: Signal<string | null> = this._selectedScenarioId.asReadonly();

  private readonly _scenario = signal<AsyncState<RulebenchScenarioView, RulebenchLiveTransportErrorDto>>({ kind: 'idle' });
  readonly scenario = this._scenario.asReadonly();

  private readonly _sessionCatalog = signal<AsyncState<readonly RulebenchViewerSessionSummaryDto[], RulebenchLiveTransportErrorDto>>({ kind: 'idle' });
  readonly sessionCatalog = this._sessionCatalog.asReadonly();

  private readonly _selectedSessionId = signal<string | null>(null);
  readonly selectedSessionId: Signal<string | null> = this._selectedSessionId.asReadonly();

  private readonly _selectedSessionStepId = signal<string | null>(null);
  readonly selectedSessionStepId: Signal<string | null> = this._selectedSessionStepId.asReadonly();

  private readonly _sessionStep = signal<AsyncState<RulebenchCombatSessionStepView, RulebenchLiveTransportErrorDto>>({ kind: 'idle' });
  readonly sessionStep = this._sessionStep.asReadonly();

  private catalogRequest = 0;
  private scenarioRequest = 0;
  private sessionCatalogRequest = 0;
  private sessionStepRequest = 0;
  private catalogController: AbortController | null = null;
  private scenarioController: AbortController | null = null;
  private sessionCatalogController: AbortController | null = null;
  private sessionStepController: AbortController | null = null;

  constructor(
    private readonly transport: RulebenchLiveTransport,
    private readonly clock: ClockPort,
  ) {}

  async loadCatalog(): Promise<void> {
    const request = this.beginCatalogRequest();
    this._catalog.set({ kind: 'loading' });
    const result = await this.transport.listViewerScenarios({ signal: request.controller.signal });
    if (request.generation !== this.catalogRequest) {
      return;
    }
    if (result.ok) {
      this._catalog.set({ kind: 'data', value: result.value });
      const firstSummary = result.value[0];
      const selectedScenarioId = this._selectedScenarioId();
      if (!result.value.some((summary) => summary.id === selectedScenarioId)) {
        this._selectedScenarioId.set(firstSummary?.id ?? null);
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
    if (scenarioId === null) {
      this._scenario.set({
        kind: 'error',
        error: viewerSelectionError('viewerScenarioRequired', 'Select a live authority scenario to inspect.'),
      });
      return;
    }
    const request = this.beginScenarioRequest();
    this._scenario.set({ kind: 'loading' });
    const result = await this.transport.getViewerScenario(scenarioId, { signal: request.controller.signal });
    if (request.generation !== this.scenarioRequest) {
      return;
    }
    this._scenario.set(
      result.ok
        ? { kind: 'data', value: projectRulebenchViewerScenario(result.value) }
        : { kind: 'error', error: result.error },
    );
    this.clock.now();
  }

  async loadSessionCatalog(): Promise<void> {
    const request = this.beginSessionCatalogRequest();
    this._sessionCatalog.set({ kind: 'loading' });
    const result = await this.transport.listViewerSessions({ signal: request.controller.signal });
    if (request.generation !== this.sessionCatalogRequest) {
      return;
    }
    if (result.ok) {
      this._sessionCatalog.set({ kind: 'data', value: result.value });
      const firstSession = result.value[0];
      const firstStep = firstSession?.steps[0];
      const selectedSessionId = this._selectedSessionId();
      const selectedSession = result.value.find((session) => session.id === selectedSessionId);
      if (selectedSession === undefined) {
        this._selectedSessionId.set(firstSession?.id ?? null);
        this._selectedSessionStepId.set(firstStep?.id ?? null);
      } else if (!selectedSession.steps.some((step) => step.id === this._selectedSessionStepId())) {
        this._selectedSessionStepId.set(selectedSession.steps[0]?.id ?? null);
      }
    } else {
      this._sessionCatalog.set({ kind: 'error', error: result.error });
    }
    this.clock.now();
  }

  async selectSessionStep(sessionId: string, stepId: string): Promise<void> {
    this._selectedSessionId.set(sessionId);
    this._selectedSessionStepId.set(stepId);
    await this.loadSessionStep(sessionId, stepId);
  }

  async loadSessionStep(
    sessionId: string | null = this._selectedSessionId(),
    stepId: string | null = this._selectedSessionStepId(),
  ): Promise<void> {
    if (sessionId === null || stepId === null) {
      this._sessionStep.set({
        kind: 'error',
        error: viewerSelectionError('viewerSessionStepRequired', 'Select a live authority session step to inspect.'),
      });
      return;
    }
    const request = this.beginSessionStepRequest();
    this._sessionStep.set({ kind: 'loading' });
    const result = await this.transport.getViewerSessionStep(sessionId, stepId, {
      signal: request.controller.signal,
    });
    if (request.generation !== this.sessionStepRequest) {
      return;
    }
    if (result.ok) {
      this._selectedSessionId.set(result.value.sessionId);
      this._selectedSessionStepId.set(result.value.step.id);
      this._sessionStep.set({ kind: 'data', value: projectRulebenchViewerSessionStep(result.value) });
    } else {
      this._sessionStep.set({ kind: 'error', error: result.error });
    }
    this.clock.now();
  }

  async nextSessionStep(): Promise<void> {
    await this.selectAdjacentSessionStep(1);
  }

  async previousSessionStep(): Promise<void> {
    await this.selectAdjacentSessionStep(-1);
  }

  private async selectAdjacentSessionStep(offset: 1 | -1): Promise<void> {
    const catalog = this._sessionCatalog();
    if (catalog.kind !== 'data') {
      this.clock.now();
      return;
    }

    const currentSessionId = this._selectedSessionId();
    const currentStepId = this._selectedSessionStepId();
    const session = catalog.value.find((candidate) => candidate.id === currentSessionId) ?? catalog.value[0];
    if (session === undefined || session.steps.length === 0) {
      this.clock.now();
      return;
    }

    const currentIndex = session.steps.findIndex((step) => step.id === currentStepId);
    const baseIndex = currentIndex >= 0 ? currentIndex : 0;
    const nextIndex = Math.min(Math.max(baseIndex + offset, 0), session.steps.length - 1);
    const nextStep = session.steps[nextIndex];
    if (nextStep === undefined) {
      this.clock.now();
      return;
    }

    await this.selectSessionStep(session.id, nextStep.id);
  }

  retryCatalog(): Promise<void> {
    return this.loadCatalog();
  }

  retryScenario(): Promise<void> {
    return this.loadScenario();
  }

  retrySessionCatalog(): Promise<void> {
    return this.loadSessionCatalog();
  }

  retrySessionStep(): Promise<void> {
    return this.loadSessionStep();
  }

  private beginCatalogRequest(): { readonly generation: number; readonly controller: AbortController } {
    this.catalogRequest += 1;
    this.catalogController?.abort();
    this.catalogController = new AbortController();
    return { generation: this.catalogRequest, controller: this.catalogController };
  }

  private beginScenarioRequest(): { readonly generation: number; readonly controller: AbortController } {
    this.scenarioRequest += 1;
    this.scenarioController?.abort();
    this.scenarioController = new AbortController();
    return { generation: this.scenarioRequest, controller: this.scenarioController };
  }

  private beginSessionCatalogRequest(): { readonly generation: number; readonly controller: AbortController } {
    this.sessionCatalogRequest += 1;
    this.sessionCatalogController?.abort();
    this.sessionCatalogController = new AbortController();
    return { generation: this.sessionCatalogRequest, controller: this.sessionCatalogController };
  }

  private beginSessionStepRequest(): { readonly generation: number; readonly controller: AbortController } {
    this.sessionStepRequest += 1;
    this.sessionStepController?.abort();
    this.sessionStepController = new AbortController();
    return { generation: this.sessionStepRequest, controller: this.sessionStepController };
  }
}

function viewerSelectionError(code: string, message: string): RulebenchLiveTransportErrorDto {
  return { kind: 'selection', code, message, retryable: false };
}

export function provideRulebenchStoreKernel(): Provider[] {
  return [
    ...provideLiveCombatStoreKernel(),
    ...provideContentWorkbenchStoreKernel(),
    { provide: RULEBENCH_CLOCK, useValue: browserClock },
    {
      provide: SessionStore,
      deps: [RULEBENCH_LIVE_TRANSPORT, RULEBENCH_CLOCK],
      useFactory: (transport: RulebenchLiveTransport, clock: ClockPort) => new SessionStore(transport, clock),
    },
  ];
}
