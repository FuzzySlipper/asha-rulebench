import { describe, expect, it } from "vitest";
import type { ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchLiveCandidateSummaryDto,
  RulebenchLiveCommandExecutionDto,
  RulebenchLiveAutomaticRunDto,
  RulebenchLiveAutomaticStepDto,
  RulebenchCombatAutomationPolicySpecDto,
  RulebenchLivePreflightDto,
  RulebenchLiveSessionSnapshotDto,
} from "@asha-rulebench/protocol";
import {
  createFakeRulebenchLiveTransport,
  type RulebenchLiveTransportResult,
} from "@asha-rulebench/transport";
import { LiveCombatStore } from "./live-combat-store";

const fixedClock: ClockPort = {
  now: () => new Date("2026-07-10T00:00:00.000Z"),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

const intent = {
  actorId: "entity-adept",
  actionId: "hexing_bolt",
  targetId: "entity-raider",
};

describe("LiveCombatStore", () => {
  it("loads connection, scenarios, snapshot, options, candidates, and preflight through injected transport", async () => {
    const snapshot = makeLiveSessionSnapshot();
    const transport = createFakeRulebenchLiveTransport({
      connect: async () => ({
        ok: true,
        value: {
          protocolId: "asha-rulebench.protocol",
          protocolVersion: 1,
          authoritySurface: "test-authority",
        },
      }),
      listScenarios: async () => ({
        ok: true,
        value: [
          { id: "scenario", title: "Scenario", summary: "Test scenario." },
        ],
      }),
      getSession: async () => ({ ok: true, value: snapshot }),
      getCurrentActorOptions: async () => ({
        ok: true,
        value: snapshot.options,
      }),
      listCandidates: async () => ({ ok: true, value: acceptedLiveCandidates }),
      preflightIntent: async () => ({ ok: true, value: acceptedLivePreflight }),
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.connect();
    await store.loadScenarios();
    await store.selectSession("live-session");
    await store.refreshOptions();
    await store.refreshCandidates();
    store.setIntent(intent);
    await store.preflightIntent();

    expect(store.connection().kind).toBe("data");
    expect(store.selectedScenarioId()).toBe("scenario");
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { sessionId: "live-session" },
    });
    expect(store.options()).toMatchObject({
      kind: "data",
      value: { available: true },
    });
    expect(store.candidates()).toMatchObject({
      kind: "data",
      value: { candidates: [{ accepted: true }] },
    });
    expect(store.preflight()).toMatchObject({
      kind: "data",
      value: { accepted: true },
    });
  });

  it("applies accepted and rejected command snapshots exactly as returned by authority", async () => {
    let accepted = true;
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      submitIntent: async () => ({
        ok: true,
        value: makeLiveCommandExecution(accepted),
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    store.setIntent(intent);

    await store.submitIntent({
      id: "accepted",
      title: "Accepted",
      summary: "Accepted.",
      rollStream: [17, 5],
    });
    expect(store.submission()).toMatchObject({
      kind: "data",
      value: { accepted: true, stateChanged: true },
    });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { participants: [{}, { hitPointLabel: "9/18 HP" }] },
    });

    accepted = false;
    await store.submitIntent({
      id: "rejected",
      title: "Rejected",
      summary: "Rejected.",
      rollStream: [],
    });
    expect(store.submission()).toMatchObject({
      kind: "data",
      value: {
        accepted: false,
        stateChanged: false,
        rejectionLabel: "Invalid Target",
      },
    });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { participants: [{}, { hitPointLabel: "18/18 HP" }] },
    });
  });

  it("ignores a stale session response after selection changes", async () => {
    const first =
      deferred<RulebenchLiveTransportResult<RulebenchLiveSessionSnapshotDto>>();
    const transport = createFakeRulebenchLiveTransport({
      getSession: async (sessionId) =>
        sessionId === "first"
          ? first.promise
          : {
              ok: true,
              value: makeLiveSessionSnapshot({ sessionId: "second" }),
            },
    });
    const store = new LiveCombatStore(transport, fixedClock);

    const firstSelection = store.selectSession("first");
    expect(store.snapshot()).toEqual({ kind: "loading" });
    await store.selectSession("second");
    first.resolve({
      ok: true,
      value: makeLiveSessionSnapshot({ sessionId: "first" }),
    });
    await firstSelection;

    expect(store.selectedSessionId()).toBe("second");
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { sessionId: "second" },
    });
  });

  it("preserves classified transport errors and clears all state on disconnect", async () => {
    const error = {
      kind: "bridge",
      code: "unknownSession",
      message: "Missing.",
      retryable: false,
    };
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: false, error }),
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.selectSession("missing");
    expect(store.snapshot()).toEqual({ kind: "error", error });

    store.disconnect();
    expect(store.connection()).toEqual({ kind: "idle" });
    expect(store.snapshot()).toEqual({ kind: "idle" });
    expect(store.selectedSessionId()).toBeNull();
    expect(store.intent()).toEqual({ actorId: "", actionId: "", targetId: "" });
    expect(transport.connectionState()).toEqual({
      kind: "disconnected",
      error: null,
    });
  });

  it("does not let a pending connection overwrite disconnected cleanup state", async () => {
    const connection = deferred<
      RulebenchLiveTransportResult<{
        readonly protocolId: string;
        readonly protocolVersion: number;
        readonly authoritySurface: string;
      }>
    >();
    const transport = createFakeRulebenchLiveTransport({
      connect: async () => connection.promise,
    });
    const store = new LiveCombatStore(transport, fixedClock);

    const pending = store.connect();
    expect(store.connection()).toEqual({ kind: "loading" });
    store.disconnect();
    connection.resolve({
      ok: true,
      value: {
        protocolId: "asha-rulebench.protocol",
        protocolVersion: 1,
        authoritySurface: "late",
      },
    });
    await pending;

    expect(store.connection()).toEqual({ kind: "idle" });
  });

  it("clears the selected session after authority close and refreshes the session list", async () => {
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      closeSession: async () => ({
        ok: true,
        value: makeLiveSessionSnapshot({ lifecyclePhase: "ended" }),
      }),
      listSessions: async () => ({ ok: true, value: [] }),
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");

    await store.closeSession();

    expect(store.selectedSessionId()).toBeNull();
    expect(store.snapshot()).toEqual({ kind: "idle" });
    expect(store.sessions()).toEqual({ kind: "data", value: [] });
  });

  it("projects Rust automatic step and bounded-run decisions and applies returned snapshots", async () => {
    const finalSnapshot = makeLiveSessionSnapshot({
      raiderHitPoints: 9,
      fingerprint: "automatic",
    });
    const automaticStep: RulebenchLiveAutomaticStepDto = {
      accepted: true,
      decisionKind: "submitCandidate",
      operationKind: "submitCandidate",
      lifecyclePhase: "inProgress",
      currentActorId: "entity-adept",
      policyId: "firstAcceptedCandidate",
      policyVersion: 1,
      selectedActionId: "hexing_bolt",
      selectedTargetId: "entity-raider",
      candidateCount: 1,
      acceptedCandidateCount: 1,
      submittedStep: null,
      reason: "Rust selected the first accepted candidate.",
      snapshot: finalSnapshot,
    };
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      runAutomaticStep: async () => ({ ok: true, value: automaticStep }),
      runAutomaticCombat: async () => ({
        ok: true,
        value: {
          id: "run",
          title: "Run",
          summary: "Bounded run.",
          accepted: true,
          decisionKind: "stoppedAtMaxSteps",
          maxSteps: 1,
          executedStepCount: 1,
          policyId: "firstAcceptedCandidate",
          policyVersion: 1,
          steps: [automaticStep],
          finalSnapshot,
          reason: "Run reached its configured step limit.",
        },
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    const policy: RulebenchCombatAutomationPolicySpecDto = {
      id: "firstAcceptedCandidate",
      version: 1,
      noCandidateBehavior: "advanceTurn",
    };

    await store.runAutomaticStep({
      id: "step",
      title: "Step",
      summary: "Step.",
      rollStream: [17, 5],
      policy,
    });
    expect(store.automaticStep()).toMatchObject({
      kind: "data",
      value: {
        decisionLabel: "Submit Candidate",
        selectedActionId: "hexing_bolt",
      },
    });
    await store.runAutomaticCombat({
      id: "run",
      title: "Run",
      summary: "Run.",
      maxSteps: 1,
      rollStream: [17, 5],
      policy,
    });
    expect(store.automaticRun()).toMatchObject({
      kind: "data",
      value: {
        executedStepCount: 1,
        maxSteps: 1,
        decisionLabel: "Stopped At Max Steps",
      },
    });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { fingerprintLabel: "test:automatic" },
    });
  });

  it("interrupts a pending automatic run without allowing its late response to overwrite state", async () => {
    const run =
      deferred<RulebenchLiveTransportResult<RulebenchLiveAutomaticRunDto>>();
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      runAutomaticCombat: async () => run.promise,
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    const pending = store.runAutomaticCombat({
      id: "run",
      title: "Run",
      summary: "Run.",
      maxSteps: 2,
      rollStream: [17, 5],
      policy: {
        id: "firstAcceptedCandidate",
        version: 1,
        noCandidateBehavior: "stopRun",
      },
    });
    expect(store.automaticRun()).toEqual({ kind: "loading" });

    store.cancelAutomation();
    run.resolve({
      ok: true,
      value: {
        id: "late",
        title: "Late",
        summary: "Late.",
        accepted: true,
        decisionKind: "stoppedAtMaxSteps",
        maxSteps: 2,
        executedStepCount: 0,
        policyId: "firstAcceptedCandidate",
        policyVersion: 1,
        steps: [],
        finalSnapshot: makeLiveSessionSnapshot({ fingerprint: "late" }),
        reason: "Late response.",
      },
    });
    await pending;

    expect(store.automaticRun()).toEqual({ kind: "idle" });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { fingerprintLabel: "test:state-0" },
    });
  });
});

function deferred<T>(): {
  readonly promise: Promise<T>;
  readonly resolve: (value: T) => void;
} {
  let resolvePromise: ((value: T) => void) | undefined;
  const promise = new Promise<T>((resolve) => {
    resolvePromise = resolve;
  });
  return {
    promise,
    resolve: (value) => {
      if (resolvePromise === undefined)
        throw new Error("Deferred promise was not initialized.");
      resolvePromise(value);
    },
  };
}

const acceptedLivePreflight: RulebenchLivePreflightDto = {
  intent,
  accepted: true,
  decisionKind: "accepted",
  rejectionCode: null,
  currentActorId: "entity-adept",
  targetId: "entity-raider",
  targetAccepted: true,
  targetReason: "Target accepted.",
  resourceCosts: [],
  actionResource: null,
  reason: "Command accepted by preflight.",
};

const acceptedLiveCandidates: RulebenchLiveCandidateSummaryDto = {
  roundNumber: 1,
  turnIndex: 0,
  lifecyclePhase: "inProgress",
  currentActorId: "entity-adept",
  available: true,
  unavailableReason: null,
  candidates: [
    {
      intent,
      abilityId: "ability.hexing-bolt",
      targetName: "Raider",
      targetCurrentHitPoints: 18,
      targetMaxHitPoints: 18,
      accepted: true,
      decisionKind: "accepted",
      rejectionCode: null,
      reason: "Candidate accepted.",
    },
  ],
};

function makeLiveSessionSnapshot(
  options: {
    readonly sessionId?: string;
    readonly lifecyclePhase?: RulebenchLiveSessionSnapshotDto["lifecyclePhase"];
    readonly raiderHitPoints?: number;
    readonly fingerprint?: string;
  } = {},
): RulebenchLiveSessionSnapshotDto {
  const sessionId = options.sessionId ?? "live-session";
  const lifecyclePhase = options.lifecyclePhase ?? "inProgress";
  const raiderHitPoints = options.raiderHitPoints ?? 18;
  const fingerprint = options.fingerprint ?? "state-0";
  return {
    sessionId,
    nextStepIndex: 0,
    lifecyclePhase,
    startedAtStep: lifecyclePhase === "ready" ? null : 0,
    endedAtStep: lifecyclePhase === "ended" ? 1 : null,
    roundNumber: 1,
    turnIndex: 0,
    participantOrder: ["entity-adept", "entity-raider"],
    currentActorId: lifecyclePhase === "ended" ? null : "entity-adept",
    participants: [
      {
        id: "entity-adept",
        name: "Adept",
        currentHitPoints: 24,
        maxHitPoints: 24,
        temporaryVitality: 0,
        defeated: false,
        conditions: [],
      },
      {
        id: "entity-raider",
        name: "Raider",
        currentHitPoints: raiderHitPoints,
        maxHitPoints: 18,
        temporaryVitality: 0,
        defeated: false,
        conditions: raiderHitPoints < 18 ? ["rattled"] : [],
      },
    ],
    options: {
      roundNumber: 1,
      turnIndex: 0,
      lifecyclePhase,
      currentActorId: lifecyclePhase === "ended" ? null : "entity-adept",
      currentActorDefeated: false,
      available: lifecyclePhase === "inProgress",
      unavailableReason:
        lifecyclePhase === "inProgress" ? null : lifecyclePhase,
      actions:
        lifecyclePhase === "inProgress"
          ? [
              {
                actionId: "hexing_bolt",
                abilityId: "ability.hexing-bolt",
                actionName: "Hexing Bolt",
                available: true,
                unavailableReason: null,
                resourceCosts: [],
                resourceStates: [],
                targets: [
                  {
                    targetId: "entity-raider",
                    targetName: "Raider",
                    currentHitPoints: raiderHitPoints,
                    maxHitPoints: 18,
                  },
                ],
              },
            ]
          : [],
    },
    combatEnd: {
      shouldEnd: lifecyclePhase === "ended",
      conditionKind: lifecyclePhase === "ended" ? "explicitEnd" : "ongoing",
      outcomeKind: lifecyclePhase === "ended" ? "explicitEnd" : "ongoing",
      activeSides: ["ally", "enemy"],
      defeatedSides: [],
      winningSides: [],
      reason: "Authority lifecycle readout.",
    },
    finalization: null,
    combatLog: [],
    auditLog: [],
    stateFingerprint: { algorithm: "test", value: fingerprint },
  };
}

function makeLiveCommandExecution(
  accepted: boolean,
): RulebenchLiveCommandExecutionDto {
  return {
    step: {
      sessionId: "live-session",
      stepId: accepted ? "accepted" : "rejected",
      stepIndex: 0,
      title: "Command",
      summary: "Authority result.",
      outcomeClass: accepted ? "acceptedHit" : "rejectedInvalidCommand",
      accepted,
      decisionKind: accepted ? "acceptedByResolver" : "rejectedByPreflight",
      rejectionCode: accepted ? null : "invalidTarget",
      intent,
      rolls: [],
      events: accepted
        ? [{ kind: "damageApplied", summary: "Raider took damage." }]
        : [],
      trace: [],
      stateBeforeFingerprint: { algorithm: "test", value: "state-0" },
      stateAfterFingerprint: {
        algorithm: "test",
        value: accepted ? "state-1" : "state-0",
      },
    },
    snapshot: makeLiveSessionSnapshot({
      raiderHitPoints: accepted ? 9 : 18,
      fingerprint: accepted ? "state-1" : "state-0",
    }),
  };
}
