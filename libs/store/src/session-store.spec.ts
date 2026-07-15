import { describe, expect, it } from "vitest";
import type { ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchLiveTransportErrorDto,
  RulebenchViewerScenarioReadoutDto,
  RulebenchViewerScenarioSummaryDto,
  RulebenchViewerSessionStepReadoutDto,
  RulebenchViewerSessionSummaryDto,
} from "@asha-rulebench/protocol";
import {
  createFakeRulebenchLiveTransport,
  type RulebenchLiveTransport,
  type RulebenchLiveTransportResult,
} from "@asha-rulebench/transport";
import { SessionStore } from "./index";

const fixedClock: ClockPort = {
  now: () => new Date("2026-07-07T00:00:00.000Z"),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

const scenarioSummary: RulebenchViewerScenarioSummaryDto = {
  id: "provider-scenario",
  title: "Provider Scenario",
  summary: "Authority-owned scenario evidence.",
  seedLabel: "roll-stream:17,5",
  outcomeClass: "acceptedHit",
};

const scenarioReadout: RulebenchViewerScenarioReadoutDto = {
  identity: scenarioSummary,
  board: {
    id: "provider-board",
    width: 2,
    height: 1,
    cells: [
      {
        position: { x: 0, y: 0 },
        terrainTags: [],
        blocksMovement: false,
        occupantIds: ["actor"],
      },
      {
        position: { x: 1, y: 0 },
        terrainTags: ["cover"],
        blocksMovement: false,
        occupantIds: ["target"],
      },
    ],
  },
  combatants: [
    {
      id: "actor",
      name: "Adept",
      team: "ally",
      sideId: "allies",
      currentHitPoints: 12,
      maxHitPoints: 12,
      temporaryVitality: 0,
      conditions: [],
      positionX: 0,
      positionY: 0,
      defenses: [{ id: "nerve", label: "Nerve", value: 14 }],
      isActor: true,
    },
    {
      id: "target",
      name: "Raider",
      team: "enemy",
      sideId: "enemies",
      currentHitPoints: 9,
      maxHitPoints: 18,
      temporaryVitality: 0,
      conditions: ["rattled"],
      positionX: 1,
      positionY: 0,
      defenses: [{ id: "nerve", label: "Nerve", value: 13 }],
      isActor: false,
    },
  ],
  selectedAction: {
    id: "hexing-bolt",
    name: "Hexing Bolt",
    actorId: "actor",
    targetIds: ["target"],
    actionText: "A ranged spell attack.",
    effectText: "Deals damage and rattles the target.",
  },
  selectedTarget: {
    targetId: "target",
    accepted: true,
    reason: "Target is in range and visible.",
  },
  domainEvents: [
    {
      sequence: 1,
      kind: "damageApplied",
      summary: "target took 9 psychic damage.",
      entityIds: ["target"],
    },
  ],
  trace: [
    {
      sequence: 1,
      phase: "commit",
      status: "accepted",
      message: "State committed.",
      detail: "Authority projection updated.",
    },
  ],
  finalState: {
    summary: "Raider has 9 vitality remaining.",
    combatants: [
      {
        id: "actor",
        name: "Adept",
        currentHitPoints: 12,
        maxHitPoints: 12,
        temporaryVitality: 0,
        conditions: [],
        positionX: 0,
        positionY: 0,
      },
      {
        id: "target",
        name: "Raider",
        currentHitPoints: 9,
        maxHitPoints: 18,
        temporaryVitality: 0,
        conditions: ["rattled"],
        positionX: 1,
        positionY: 0,
      },
    ],
  },
};

const sessionStepSummary: RulebenchViewerSessionSummaryDto["steps"][number] = {
  id: "provider-step",
  index: 0,
  title: "Adept hits Raider",
  summary: "The opening attack hits.",
  outcomeClass: "acceptedHit",
  logIndex: 1,
};

const sessionSummary: RulebenchViewerSessionSummaryDto = {
  id: "provider-session",
  title: "Provider Session",
  summary: "Authority-owned transcript.",
  seedLabel: "session-seed",
  steps: [sessionStepSummary],
};

const sessionStep: RulebenchViewerSessionStepReadoutDto = {
  sessionId: sessionSummary.id,
  step: sessionStepSummary,
  command: {
    stepId: "provider-step",
    stepIndex: 0,
    actorId: "actor",
    actionId: "hexing-bolt",
    targetId: "target",
    rollStream: [17, 5],
    outcomeClass: "acceptedHit",
  },
  scenario: scenarioReadout,
  combatLog: [
    {
      id: "log-1",
      stepId: "provider-step",
      logIndex: 1,
      title: "Adept hits Raider",
      summary: "Raider took damage.",
      outcomeClass: "acceptedHit",
      eventTypes: ["damageApplied"],
    },
  ],
  stateBefore: {
    ...scenarioReadout.finalState,
    summary: "Raider begins at full vitality.",
    combatants: scenarioReadout.finalState.combatants.map((combatant) =>
      combatant.id === "target"
        ? { ...combatant, currentHitPoints: 18, conditions: [] }
        : combatant,
    ),
  },
  stateAfter: scenarioReadout.finalState,
};

describe("SessionStore live authority viewer", () => {
  it("loads provider-driven scenario and session catalogs and records initial selections", async () => {
    const store = new SessionStore(viewerTransport(), fixedClock);

    await store.loadCatalog();
    await store.loadSessionCatalog();

    expect(store.catalog()).toEqual({ kind: "data", value: [scenarioSummary] });
    expect(store.selectedScenarioId()).toBe(scenarioSummary.id);
    expect(store.sessionCatalog()).toEqual({ kind: "data", value: [sessionSummary] });
    expect(store.selectedSessionId()).toBe(sessionSummary.id);
    expect(store.selectedSessionStepId()).toBe(sessionSummary.steps[0]?.id);
  });

  it("projects scenario and transcript readbacks without TypeScript authority mutation", async () => {
    const store = new SessionStore(viewerTransport(), fixedClock);
    await store.loadCatalog();
    await store.loadSessionCatalog();

    await store.loadScenario();
    await store.loadSessionStep();

    const scenario = store.scenario();
    expect(scenario.kind).toBe("data");
    if (scenario.kind === "data") {
      expect(scenario.value.selectedAction.actorLabel).toBe("Adept");
      expect(scenario.value.timeline[0]).toMatchObject({
        typeLabel: "Damage Applied",
        participantLabels: ["Raider"],
      });
      expect(scenario.value.finalState.combatants[1]?.hitPointLabel).toBe("9/18 HP");
    }
    const step = store.sessionStep();
    expect(step.kind).toBe("data");
    if (step.kind === "data") {
      expect(step.value.command.rollStreamLabel).toBe("17,5");
      expect(step.value.stateBefore.combatants[1]?.hitPointLabel).toBe("18/18 HP");
      expect(step.value.stateAfter.combatants[1]?.conditionLabels).toEqual(["rattled"]);
    }
  });

  it("classifies missing selections before making a request", async () => {
    const store = new SessionStore(viewerTransport(), fixedClock);

    await store.loadScenario();
    await store.loadSessionStep();

    expect(store.scenario()).toMatchObject({
      kind: "error",
      error: { code: "viewerScenarioRequired", retryable: false },
    });
    expect(store.sessionStep()).toMatchObject({
      kind: "error",
      error: { code: "viewerSessionStepRequired", retryable: false },
    });
  });

  it("preserves classified host failures and explicit retry", async () => {
    const error: RulebenchLiveTransportErrorDto = {
      kind: "network",
      code: "requestFailed",
      message: "Authority viewer unavailable.",
      retryable: true,
    };
    let attempt = 0;
    const transport = viewerTransport({
      listViewerScenarios: async () => {
        attempt += 1;
        return attempt === 1
          ? { ok: false, error }
          : { ok: true, value: [scenarioSummary] };
      },
    });
    const store = new SessionStore(transport, fixedClock);

    await store.loadCatalog();
    expect(store.catalog()).toEqual({ kind: "error", error });

    await store.retryCatalog();
    expect(store.catalog()).toEqual({ kind: "data", value: [scenarioSummary] });
  });

  it("aborts superseded requests and ignores their stale response", async () => {
    let resolveFirst: ((result: RulebenchLiveTransportResult<readonly RulebenchViewerScenarioSummaryDto[]>) => void) | null = null;
    let firstSignal: AbortSignal | undefined;
    let attempt = 0;
    const transport = viewerTransport({
      listViewerScenarios: (options) => {
        attempt += 1;
        if (attempt === 1) {
          firstSignal = options?.signal;
          return new Promise((resolve) => {
            resolveFirst = resolve;
          });
        }
        return Promise.resolve({ ok: true, value: [scenarioSummary] });
      },
    });
    const store = new SessionStore(transport, fixedClock);

    const first = store.loadCatalog();
    await store.loadCatalog();
    expect(firstSignal?.aborted).toBe(true);
    resolveFirst?.({ ok: true, value: [] });
    await first;

    expect(store.catalog()).toEqual({ kind: "data", value: [scenarioSummary] });
  });
});

function viewerTransport(
  overrides: Partial<Pick<RulebenchLiveTransport, "listViewerScenarios">> = {},
): RulebenchLiveTransport {
  return createFakeRulebenchLiveTransport({
    listViewerScenarios: async () => ({ ok: true, value: [scenarioSummary] }),
    getViewerScenario: async (scenarioId) =>
      scenarioId === scenarioSummary.id
        ? { ok: true, value: scenarioReadout }
        : { ok: false, error: missingError(`Scenario not found: ${scenarioId}`) },
    listViewerSessions: async () => ({ ok: true, value: [sessionSummary] }),
    getViewerSessionStep: async (sessionId, stepId) =>
      sessionId === sessionSummary.id && stepId === sessionStep.step.id
        ? { ok: true, value: sessionStep }
        : { ok: false, error: missingError(`Session step not found: ${sessionId}/${stepId}`) },
    ...overrides,
  });
}

function missingError(message: string): RulebenchLiveTransportErrorDto {
  return { kind: "bridge", code: "notFound", message, retryable: false };
}
