import { Injectable, InjectionToken, signal } from "@angular/core";
import type { Provider, Signal } from "@angular/core";
import {
  projectLiveAutomaticRun,
  projectLiveAutomaticStep,
  projectLiveCandidates,
  projectLiveCommandExecution,
  projectLiveOptions,
  projectLivePreflight,
  projectLiveSessionSnapshot,
  type RulebenchLiveAutomaticRunView,
  type RulebenchLiveAutomaticStepView,
  type RulebenchLiveCandidateSummaryView,
  type RulebenchLiveCommandExecutionView,
  type RulebenchLiveOptionsView,
  type RulebenchLivePreflightView,
  type RulebenchLiveSessionView,
} from "@asha-rulebench/domain";
import { browserClock, type ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchAutomaticRunSpecDto,
  RulebenchAutomaticStepSpecDto,
  RulebenchCombatControlCommandKindDto,
  RulebenchLiveTransportErrorDto,
  RulebenchProtocolHandshakeDto,
  RulebenchScenarioOptionDto,
} from "@asha-rulebench/protocol";
import {
  createLiveRulebenchTransport,
  type RulebenchLiveTransport,
} from "@asha-rulebench/transport";
import type { AsyncState } from "./async-state";
import { ReplayReviewStore } from "./replay-review-store";

type LiveState<T> = AsyncState<T, RulebenchLiveTransportErrorDto>;

export interface RulebenchLiveIntentInput {
  readonly actorId: string;
  readonly actionId: string;
  readonly targetId: string;
}

export const RULEBENCH_LIVE_TRANSPORT =
  new InjectionToken<RulebenchLiveTransport>("RULEBENCH_LIVE_TRANSPORT", {
    factory: () => createLiveRulebenchTransport(),
  });

@Injectable()
export class LiveCombatStore {
  private readonly _connection = signal<
    LiveState<RulebenchProtocolHandshakeDto>
  >({ kind: "idle" });
  readonly connection: Signal<LiveState<RulebenchProtocolHandshakeDto>> =
    this._connection.asReadonly();
  private readonly _scenarios = signal<
    LiveState<readonly RulebenchScenarioOptionDto[]>
  >({ kind: "idle" });
  readonly scenarios: Signal<LiveState<readonly RulebenchScenarioOptionDto[]>> =
    this._scenarios.asReadonly();
  private readonly _sessions = signal<
    LiveState<readonly RulebenchLiveSessionView[]>
  >({ kind: "idle" });
  readonly sessions: Signal<LiveState<readonly RulebenchLiveSessionView[]>> =
    this._sessions.asReadonly();
  private readonly _snapshot = signal<LiveState<RulebenchLiveSessionView>>({
    kind: "idle",
  });
  readonly snapshot: Signal<LiveState<RulebenchLiveSessionView>> =
    this._snapshot.asReadonly();
  private readonly _options = signal<LiveState<RulebenchLiveOptionsView>>({
    kind: "idle",
  });
  readonly options: Signal<LiveState<RulebenchLiveOptionsView>> =
    this._options.asReadonly();
  private readonly _candidates = signal<
    LiveState<RulebenchLiveCandidateSummaryView>
  >({ kind: "idle" });
  readonly candidates: Signal<LiveState<RulebenchLiveCandidateSummaryView>> =
    this._candidates.asReadonly();
  private readonly _preflight = signal<LiveState<RulebenchLivePreflightView>>({
    kind: "idle",
  });
  readonly preflight: Signal<LiveState<RulebenchLivePreflightView>> =
    this._preflight.asReadonly();
  private readonly _submission = signal<
    LiveState<RulebenchLiveCommandExecutionView>
  >({ kind: "idle" });
  readonly submission: Signal<LiveState<RulebenchLiveCommandExecutionView>> =
    this._submission.asReadonly();
  private readonly _control = signal<LiveState<RulebenchLiveSessionView>>({
    kind: "idle",
  });
  readonly control: Signal<LiveState<RulebenchLiveSessionView>> =
    this._control.asReadonly();
  private readonly _automaticStep = signal<
    LiveState<RulebenchLiveAutomaticStepView>
  >({ kind: "idle" });
  readonly automaticStep = this._automaticStep.asReadonly();
  private readonly _automaticRun = signal<
    LiveState<RulebenchLiveAutomaticRunView>
  >({ kind: "idle" });
  readonly automaticRun = this._automaticRun.asReadonly();
  private readonly _selectedScenarioId = signal<string | null>(null);
  readonly selectedScenarioId = this._selectedScenarioId.asReadonly();
  private readonly _selectedSessionId = signal<string | null>(null);
  readonly selectedSessionId = this._selectedSessionId.asReadonly();
  private readonly _intent = signal<RulebenchLiveIntentInput>({
    actorId: "",
    actionId: "",
    targetId: "",
  });
  readonly intent = this._intent.asReadonly();
  private lifecycleGeneration = 0;
  private automationGeneration = 0;
  private automationController: AbortController | null = null;
  private sessionGeneration = 0;

  constructor(
    private readonly transport: RulebenchLiveTransport,
    private readonly clock: ClockPort,
  ) {}

  async connect(): Promise<void> {
    const generation = this.lifecycleGeneration;
    this._connection.set({ kind: "loading" });
    const result = await this.transport.connect();
    if (generation !== this.lifecycleGeneration) return;
    this._connection.set(
      result.ok
        ? { kind: "data", value: result.value }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async loadScenarios(): Promise<void> {
    const generation = this.lifecycleGeneration;
    this._scenarios.set({ kind: "loading" });
    const result = await this.transport.listScenarios();
    if (generation !== this.lifecycleGeneration) return;
    if (result.ok) {
      this._scenarios.set({ kind: "data", value: result.value });
      if (this._selectedScenarioId() === null)
        this._selectedScenarioId.set(result.value[0]?.id ?? null);
    } else {
      this._scenarios.set({ kind: "error", error: result.error });
    }
    this.clock.now();
  }

  selectScenario(scenarioId: string): void {
    this._selectedScenarioId.set(scenarioId);
    this.clock.now();
  }

  async loadSessions(): Promise<void> {
    const generation = this.lifecycleGeneration;
    this._sessions.set({ kind: "loading" });
    const result = await this.transport.listSessions();
    if (generation !== this.lifecycleGeneration) return;
    this._sessions.set(
      result.ok
        ? { kind: "data", value: result.value.map(projectLiveSessionSnapshot) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async createSession(
    sessionId: string,
    scenarioId: string,
    participantOrder: readonly string[],
  ): Promise<void> {
    this.selectSessionIdentity(sessionId);
    const generation = this.sessionGeneration;
    this._snapshot.set({ kind: "loading" });
    const result = await this.transport.createSession({
      sessionId,
      scenarioId,
      participantOrder,
    });
    if (!this.isCurrent(sessionId, generation)) return;
    this._snapshot.set(
      result.ok
        ? { kind: "data", value: projectLiveSessionSnapshot(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async selectSession(sessionId: string): Promise<void> {
    this.selectSessionIdentity(sessionId);
    await this.refreshSession();
  }

  async refreshSession(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._snapshot.set({ kind: "loading" });
    const result = await this.transport.getSession(request.sessionId);
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    this._snapshot.set(
      result.ok
        ? { kind: "data", value: projectLiveSessionSnapshot(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async refreshOptions(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._options.set({ kind: "loading" });
    const result = await this.transport.getCurrentActorOptions(
      request.sessionId,
    );
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    this._options.set(
      result.ok
        ? { kind: "data", value: projectLiveOptions(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async refreshCandidates(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._candidates.set({ kind: "loading" });
    const result = await this.transport.listCandidates(request.sessionId);
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    this._candidates.set(
      result.ok
        ? { kind: "data", value: projectLiveCandidates(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  setIntent(intent: RulebenchLiveIntentInput): void {
    this._intent.set(intent);
    this._preflight.set({ kind: "idle" });
    this.clock.now();
  }

  async preflightIntent(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._preflight.set({ kind: "loading" });
    const result = await this.transport.preflightIntent(
      request.sessionId,
      this._intent(),
    );
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    this._preflight.set(
      result.ok
        ? { kind: "data", value: projectLivePreflight(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async submitIntent(command: {
    readonly id: string;
    readonly title: string;
    readonly summary: string;
    readonly rollStream: readonly number[];
  }): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._submission.set({ kind: "loading" });
    const result = await this.transport.submitIntent(request.sessionId, {
      ...command,
      intent: this._intent(),
    });
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    if (result.ok) {
      this._submission.set({
        kind: "data",
        value: projectLiveCommandExecution(result.value),
      });
      this._snapshot.set({
        kind: "data",
        value: projectLiveSessionSnapshot(result.value.snapshot),
      });
    } else {
      this._submission.set({ kind: "error", error: result.error });
    }
    this.clock.now();
  }

  async submitControl(
    kind: RulebenchCombatControlCommandKindDto,
  ): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._control.set({ kind: "loading" });
    const result = await this.transport.submitControl(request.sessionId, {
      kind,
    });
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    if (result.ok) {
      const snapshot = projectLiveSessionSnapshot(result.value.snapshot);
      this._control.set({ kind: "data", value: snapshot });
      this._snapshot.set({ kind: "data", value: snapshot });
    } else {
      this._control.set({ kind: "error", error: result.error });
    }
    this.clock.now();
  }

  async runAutomaticStep(spec: RulebenchAutomaticStepSpecDto): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    const automation = this.beginAutomation();
    this._automaticStep.set({ kind: "loading" });
    const result = await this.transport.runAutomaticStep(
      request.sessionId,
      spec,
      { signal: automation.controller.signal },
    );
    if (!this.isCurrentAutomation(request, automation.generation)) return;
    this.automationController = null;
    if (result.ok) {
      this._automaticStep.set({
        kind: "data",
        value: projectLiveAutomaticStep(result.value),
      });
      if (result.value.snapshot !== null) {
        this._snapshot.set({
          kind: "data",
          value: projectLiveSessionSnapshot(result.value.snapshot),
        });
      }
    } else {
      this._automaticStep.set({ kind: "error", error: result.error });
    }
    this.clock.now();
  }

  async runAutomaticCombat(spec: RulebenchAutomaticRunSpecDto): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    const automation = this.beginAutomation();
    this._automaticRun.set({ kind: "loading" });
    const result = await this.transport.runAutomaticCombat(
      request.sessionId,
      spec,
      { signal: automation.controller.signal },
    );
    if (!this.isCurrentAutomation(request, automation.generation)) return;
    this.automationController = null;
    if (result.ok) {
      this._automaticRun.set({
        kind: "data",
        value: projectLiveAutomaticRun(result.value),
      });
      this._snapshot.set({
        kind: "data",
        value: projectLiveSessionSnapshot(result.value.finalSnapshot),
      });
    } else {
      this._automaticRun.set({ kind: "error", error: result.error });
    }
    this.clock.now();
  }

  cancelAutomation(): void {
    this.automationGeneration += 1;
    this.automationController?.abort();
    this.automationController = null;
    this._automaticStep.set({ kind: "idle" });
    this._automaticRun.set({ kind: "idle" });
    this.clock.now();
  }

  async closeSession(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    const result = await this.transport.closeSession(request.sessionId);
    if (!this.isCurrent(request.sessionId, request.generation)) return;
    if (!result.ok) {
      this._snapshot.set({ kind: "error", error: result.error });
      this.clock.now();
      return;
    }
    this.clearSessionState();
    await this.loadSessions();
  }

  disconnect(): void {
    this.transport.disconnect();
    this.lifecycleGeneration += 1;
    this.sessionGeneration += 1;
    this._connection.set({ kind: "idle" });
    this._scenarios.set({ kind: "idle" });
    this._sessions.set({ kind: "idle" });
    this._selectedScenarioId.set(null);
    this.clearSessionState();
    this.clock.now();
  }

  private selectSessionIdentity(sessionId: string): void {
    this.cancelAutomation();
    this.sessionGeneration += 1;
    this._selectedSessionId.set(sessionId);
    this._options.set({ kind: "idle" });
    this._candidates.set({ kind: "idle" });
    this._preflight.set({ kind: "idle" });
    this._submission.set({ kind: "idle" });
    this._control.set({ kind: "idle" });
  }

  private clearSessionState(): void {
    this.cancelAutomation();
    this.sessionGeneration += 1;
    this._selectedSessionId.set(null);
    this._snapshot.set({ kind: "idle" });
    this._options.set({ kind: "idle" });
    this._candidates.set({ kind: "idle" });
    this._preflight.set({ kind: "idle" });
    this._submission.set({ kind: "idle" });
    this._control.set({ kind: "idle" });
    this._intent.set({ actorId: "", actionId: "", targetId: "" });
  }

  private currentRequest(): {
    readonly sessionId: string;
    readonly generation: number;
  } | null {
    const sessionId = this._selectedSessionId();
    return sessionId === null
      ? null
      : { sessionId, generation: this.sessionGeneration };
  }

  private isCurrent(sessionId: string, generation: number): boolean {
    return (
      this._selectedSessionId() === sessionId &&
      this.sessionGeneration === generation
    );
  }

  private beginAutomation(): {
    readonly generation: number;
    readonly controller: AbortController;
  } {
    this.automationGeneration += 1;
    this.automationController?.abort();
    const controller = new AbortController();
    this.automationController = controller;
    return { generation: this.automationGeneration, controller };
  }

  private isCurrentAutomation(
    request: { readonly sessionId: string; readonly generation: number },
    automationGeneration: number,
  ): boolean {
    return (
      this.isCurrent(request.sessionId, request.generation) &&
      this.automationGeneration === automationGeneration
    );
  }
}

export function provideLiveCombatStoreKernel(): Provider[] {
  return [
    {
      provide: RULEBENCH_LIVE_TRANSPORT,
      useFactory: () => createLiveRulebenchTransport(),
    },
    {
      provide: LiveCombatStore,
      deps: [RULEBENCH_LIVE_TRANSPORT],
      useFactory: (transport: RulebenchLiveTransport) =>
        new LiveCombatStore(transport, browserClock),
    },
    {
      provide: ReplayReviewStore,
      deps: [RULEBENCH_LIVE_TRANSPORT],
      useFactory: (transport: RulebenchLiveTransport) =>
        new ReplayReviewStore(transport, browserClock),
    },
  ];
}
