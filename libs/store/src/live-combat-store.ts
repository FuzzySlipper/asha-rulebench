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
  RulebenchUseActionIntentDto,
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
  readonly destinationCell?: Readonly<{ x: number; y: number }>;
  readonly observedOrigin?: Readonly<{ x: number; y: number }>;
}

export type RulebenchRollMode = "supplied" | "authorityGenerated";

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
  private readonly _defaultRollMode = signal<RulebenchRollMode>("supplied");
  readonly defaultRollMode = this._defaultRollMode.asReadonly();
  private lifecycleGeneration = 0;
  private automationGeneration = 0;
  private automationController: AbortController | null = null;
  private sessionGeneration = 0;

  constructor(
    private readonly transport: RulebenchLiveTransport,
    private readonly clock: ClockPort,
    private readonly replayStore?: ReplayReviewStore,
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
    if (result.ok)
      this.reconcileIntent(projectLiveSessionSnapshot(result.value));
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
    if (result.ok)
      this.reconcileIntent(projectLiveSessionSnapshot(result.value));
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

  setDefaultRollMode(mode: RulebenchRollMode): void {
    this._defaultRollMode.set(mode);
    this.clock.now();
  }

  selectAction(actionId: string): void {
    const snapshot = this.currentSnapshot();
    const actorId = snapshot?.currentActorId ?? "";
    this.setIntent({ actorId, actionId, targetId: "" });
  }

  selectEntityTarget(targetId: string): void {
    const current = this._intent();
    this.setIntent({
      actorId: current.actorId,
      actionId: current.actionId,
      targetId,
    });
  }

  selectCellTarget(destinationCell: Readonly<{ x: number; y: number }>): void {
    const current = this._intent();
    const snapshot = this.currentSnapshot();
    const observedOrigin = snapshot?.participants.find(
      (participant) => participant.id === current.actorId,
    )?.position;
    this.setIntent({
      actorId: current.actorId,
      actionId: current.actionId,
      targetId: "",
      destinationCell,
      ...(observedOrigin === undefined ? {} : { observedOrigin }),
    });
  }

  async preflightIntent(): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._preflight.set({ kind: "loading" });
    const result = await this.transport.preflightIntent(
      request.sessionId,
      protocolIntent(this._intent()),
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
    readonly rollMode?: "supplied" | "authorityGenerated";
    readonly generatedSeed?: number | null;
  }): Promise<void> {
    const request = this.currentRequest();
    if (request === null) return;
    this._submission.set({ kind: "loading" });
    const rollMode = command.rollMode ?? this._defaultRollMode();
    const result = await this.transport.submitIntent(request.sessionId, {
      id: command.id,
      title: command.title,
      summary: command.summary,
      rollStream: rollMode === "supplied" ? command.rollStream : [],
      rollMode,
      generatedSeed:
        rollMode === "authorityGenerated"
          ? (command.generatedSeed ?? this.generatedSeed())
          : null,
      intent: protocolIntent(this._intent()),
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
      this.reconcileIntent(projectLiveSessionSnapshot(result.value.snapshot));
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
      this.reconcileIntent(snapshot);
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
    const configuredSpec = this.configureRollMode(spec);
    const result = await this.transport.runAutomaticStep(
      request.sessionId,
      configuredSpec,
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
        this.reconcileIntent(projectLiveSessionSnapshot(result.value.snapshot));
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
    const configuredSpec = this.configureRollMode(spec);
    const result = await this.transport.runAutomaticCombat(
      request.sessionId,
      configuredSpec,
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
      this.reconcileIntent(
        projectLiveSessionSnapshot(result.value.finalSnapshot),
      );
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
    await Promise.all([
      this.loadSessions(),
      this.replayStore?.loadPackages() ?? Promise.resolve(),
    ]);
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

  private currentSnapshot(): RulebenchLiveSessionView | null {
    const snapshot = this._snapshot();
    return snapshot.kind === "data" ? snapshot.value : null;
  }

  private reconcileIntent(snapshot: RulebenchLiveSessionView): void {
    const current = this._intent();
    if (snapshot.currentActorId !== current.actorId) {
      this._intent.set({
        actorId: snapshot.currentActorId ?? "",
        actionId: "",
        targetId: "",
      });
      this._preflight.set({ kind: "idle" });
      return;
    }
    const action = snapshot.options.actions.find(
      (option) => option.actionId === current.actionId,
    );
    if (action === undefined) {
      this._intent.set({
        actorId: current.actorId,
        actionId: "",
        targetId: "",
      });
      this._preflight.set({ kind: "idle" });
      return;
    }
    if (
      action.targetMode === "entity" &&
      !action.targets.some((target) => target.id === current.targetId)
    ) {
      this._intent.set({
        actorId: current.actorId,
        actionId: current.actionId,
        targetId: "",
      });
      this._preflight.set({ kind: "idle" });
      return;
    }
    if (action.targetMode === "cell") {
      const destination = current.destinationCell;
      const origin = snapshot.participants.find(
        (participant) => participant.id === current.actorId,
      )?.position;
      const destinationStillLegal =
        destination !== undefined &&
        action.destinations.some(
          (option) => option.x === destination.x && option.y === destination.y,
        );
      const originStillCurrent =
        current.observedOrigin !== undefined &&
        origin !== undefined &&
        current.observedOrigin.x === origin.x &&
        current.observedOrigin.y === origin.y;
      if (!destinationStillLegal || !originStillCurrent) {
        this._intent.set({
          actorId: current.actorId,
          actionId: current.actionId,
          targetId: "",
        });
        this._preflight.set({ kind: "idle" });
      }
    }
  }

  private generatedSeed(): number {
    return this.clock.now().getTime() >>> 0;
  }

  private configureRollMode<
    T extends {
      readonly rollStream: readonly number[];
      readonly rollMode?: RulebenchRollMode;
      readonly generatedSeed?: number | null;
    },
  >(spec: T): T {
    const rollMode = spec.rollMode ?? this._defaultRollMode();
    return {
      ...spec,
      rollStream: rollMode === "supplied" ? spec.rollStream : [],
      rollMode,
      generatedSeed:
        rollMode === "authorityGenerated"
          ? (spec.generatedSeed ?? this.generatedSeed())
          : null,
    };
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

function protocolIntent(
  intent: RulebenchLiveIntentInput,
): RulebenchUseActionIntentDto {
  return {
    actorId: intent.actorId,
    actionId: intent.actionId,
    targetId: intent.targetId,
    destinationCell: intent.destinationCell ?? null,
    observedOrigin: intent.observedOrigin ?? null,
  };
}

export function provideLiveCombatStoreKernel(): Provider[] {
  return [
    {
      provide: RULEBENCH_LIVE_TRANSPORT,
      useFactory: () => createLiveRulebenchTransport(),
    },
    {
      provide: ReplayReviewStore,
      deps: [RULEBENCH_LIVE_TRANSPORT],
      useFactory: (transport: RulebenchLiveTransport) =>
        new ReplayReviewStore(transport, browserClock),
    },
    {
      provide: LiveCombatStore,
      deps: [RULEBENCH_LIVE_TRANSPORT, ReplayReviewStore],
      useFactory: (
        transport: RulebenchLiveTransport,
        replayStore: ReplayReviewStore,
      ) => new LiveCombatStore(transport, browserClock, replayStore),
    },
  ];
}
