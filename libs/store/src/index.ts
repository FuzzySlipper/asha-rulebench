export * from "./replay-review-store";
export * from "./async-state";
export * from "./live-combat-store";

import { InjectionToken, Injectable, signal } from '@angular/core';
import type { Provider, Signal } from '@angular/core';
import {
  projectContentImportReadout,
  projectContentValidationReadout,
  projectRulebenchCombatSessionStep,
  projectRulebenchScenario,
  type RulebenchContentImportView,
  type RulebenchContentValidationView,
  type RulebenchCombatSessionStepView,
  type RulebenchScenarioView,
} from '@asha-rulebench/domain';
import { browserClock, type ClockPort } from '@asha-rulebench/platform';
import type {
  RulebenchCombatSessionSummaryDto,
  RulebenchScenarioCatalogSummaryDto,
} from '@asha-rulebench/protocol';
import { createFakeRulebenchTransport, type RulebenchTransport } from '@asha-rulebench/transport';
import type { AsyncState } from "./async-state";
import { provideLiveCombatStoreKernel } from './live-combat-store';

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

  private readonly _sessionCatalog = signal<AsyncState<readonly RulebenchCombatSessionSummaryDto[]>>({ kind: 'idle' });
  readonly sessionCatalog: Signal<AsyncState<readonly RulebenchCombatSessionSummaryDto[]>> =
    this._sessionCatalog.asReadonly();

  private readonly _selectedSessionId = signal<string | null>(null);
  readonly selectedSessionId: Signal<string | null> = this._selectedSessionId.asReadonly();

  private readonly _selectedSessionStepId = signal<string | null>(null);
  readonly selectedSessionStepId: Signal<string | null> = this._selectedSessionStepId.asReadonly();

  private readonly _sessionStep = signal<AsyncState<RulebenchCombatSessionStepView>>({ kind: 'idle' });
  readonly sessionStep: Signal<AsyncState<RulebenchCombatSessionStepView>> = this._sessionStep.asReadonly();

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

  async loadSessionCatalog(): Promise<void> {
    this._sessionCatalog.set({ kind: 'loading' });
    const result = await this.transport.loadSessionCatalog();
    if (result.ok) {
      this._sessionCatalog.set({ kind: 'data', value: result.value });
      const firstSession = result.value[0];
      const firstStep = firstSession?.steps[0];
      if (this._selectedSessionId() === null && firstSession !== undefined) {
        this._selectedSessionId.set(firstSession.id);
      }
      if (this._selectedSessionStepId() === null && firstStep !== undefined) {
        this._selectedSessionStepId.set(firstStep.id);
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
    this._sessionStep.set({ kind: 'loading' });
    const result = await this.transport.loadSessionStep(sessionId ?? undefined, stepId ?? undefined);
    if (result.ok) {
      this._selectedSessionId.set(result.value.sessionId);
      this._selectedSessionStepId.set(result.value.step.id);
      this._sessionStep.set({ kind: 'data', value: projectRulebenchCombatSessionStep(result.value) });
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
}

@Injectable()
export class ContentStore {
  private readonly _imports = signal<AsyncState<readonly RulebenchContentImportView[]>>({ kind: 'idle' });
  readonly imports: Signal<AsyncState<readonly RulebenchContentImportView[]>> = this._imports.asReadonly();

  private readonly _selectedImportId = signal<string | null>(null);
  readonly selectedImportId: Signal<string | null> = this._selectedImportId.asReadonly();

  private readonly _validation = signal<AsyncState<RulebenchContentValidationView>>({ kind: 'idle' });
  readonly validation: Signal<AsyncState<RulebenchContentValidationView>> = this._validation.asReadonly();

  constructor(
    private readonly transport: RulebenchTransport,
    private readonly clock: ClockPort,
  ) {}

  async loadImportExamples(): Promise<void> {
    this._imports.set({ kind: 'loading' });
    const result = await this.transport.loadContentImportExamples();
    this._imports.set(
      result.ok
        ? { kind: 'data', value: result.value.map(projectContentImportReadout) }
        : { kind: 'error', error: result.error },
    );
    if (result.ok && this._selectedImportId() === null) {
      this._selectedImportId.set(result.value[0]?.exampleId ?? null);
    }
    this.clock.now();
  }

  selectImport(exampleId: string): void {
    this._selectedImportId.set(exampleId);
    this.clock.now();
  }

  async loadValidation(scenarioId?: string): Promise<void> {
    this._validation.set({ kind: 'loading' });
    const result = await this.transport.loadContentValidationReport(scenarioId);
    this._validation.set(
      result.ok
        ? { kind: 'data', value: projectContentValidationReadout(result.value) }
        : { kind: 'error', error: result.error },
    );
    this.clock.now();
  }

  clear(): void {
    this._imports.set({ kind: 'idle' });
    this._validation.set({ kind: 'idle' });
    this._selectedImportId.set(null);
    this.clock.now();
  }
}

export function provideRulebenchStoreKernel(): Provider[] {
  return [
    ...provideLiveCombatStoreKernel(),
    { provide: RULEBENCH_TRANSPORT, useFactory: () => createFakeRulebenchTransport() },
    { provide: RULEBENCH_CLOCK, useValue: browserClock },
    {
      provide: SessionStore,
      deps: [RULEBENCH_TRANSPORT, RULEBENCH_CLOCK],
      useFactory: (transport: RulebenchTransport, clock: ClockPort) => new SessionStore(transport, clock),
    },
    {
      provide: ContentStore,
      deps: [RULEBENCH_TRANSPORT, RULEBENCH_CLOCK],
      useFactory: (transport: RulebenchTransport, clock: ClockPort) => new ContentStore(transport, clock),
    },
  ];
}
