import { describe, expect, it } from "vitest";
import type { ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchLiveCandidateSummaryDto,
  RulebenchLiveCommandExecutionDto,
  RulebenchLiveAutomaticRunDto,
  RulebenchLiveAutomaticStepDto,
  RulebenchCombatAutomationPolicySpecDto,
  RulebenchCombatSessionCreateRequestDto,
  RulebenchLivePreflightDto,
  RulebenchLiveSessionSnapshotDto,
  RulebenchUseActionIntentDto,
} from "@asha-rulebench/protocol";
import {
  createFakeRulebenchLiveTransport,
  type RulebenchLiveTransport,
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
  targetIds: [],
  targetCell: null,
  destinationCell: null,
  observedOrigin: null,
};

describe("LiveCombatStore", () => {
  it("passes an exact authored-action binding through the store and projects its receipt", async () => {
    const reference = {
      id: "pack.authored.v3",
      version: "3.0.0",
      fingerprint: { algorithm: "pack", value: "exact-pack" },
    };
    const binding = {
      contentPack: reference,
      actionId: "action.binding-glyph",
      actorId: "entity-adept",
    };
    let captured: RulebenchCombatSessionCreateRequestDto | null = null;
    const snapshot = {
      ...makeLiveSessionSnapshot(),
      authoredActionBinding: {
        bindingVersion: "1",
        contentPackRoot: reference,
        contentPackReferences: [reference],
        contentPackSetFingerprint: { algorithm: "set", value: "exact-set" },
        actionId: binding.actionId,
        actionDefinitionFingerprint: {
          algorithm: "action",
          value: "exact-action",
        },
        abilityId: "ability.binding-glyph",
        scenarioId: "scenario",
        actorId: binding.actorId,
        grant: {
          grantKind: "sessionLocalBaseAbility" as const,
          actorId: binding.actorId,
          abilityId: "ability.binding-glyph",
        },
        targetingOperationVocabularyVersion: "2",
        checkVocabularyVersion: "1",
        effectOperationVocabularyVersion: "1",
      },
    };
    const transport = createFakeRulebenchLiveTransport({
      createSession: async (request) => {
        captured = request;
        return { ok: true, value: snapshot };
      },
      listSessions: async () => ({ ok: true, value: [snapshot] }),
      getSessionRecovery: async () => ({
        ok: true,
        value: { sessions: [], issues: [] },
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.createSession("live-session", "scenario", [], null, binding);

    expect(captured).toEqual({
      sessionId: "live-session",
      scenarioId: "scenario",
      participantOrder: [],
      contentPack: null,
      authoredActionBinding: binding,
    });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: {
        authoredActionBinding: {
          actionId: "action.binding-glyph",
          actionFingerprintLabel: "action:exact-action",
        },
      },
    });
  });

  it("projects live capability levels without turning them into permissions", async () => {
    const transport = createFakeRulebenchLiveTransport({
      getCapabilities: async () => ({
        ok: true,
        value: {
          manifestId: "asha-rulebench.capabilities",
          manifestVersion: 4,
          generatedArtifactSchema: "asha-rulebench.capabilities.ts@4",
          governedAshaRevision: "0123456789abcdef",
          operationVocabularyVersion: "2",
          effectVocabularyVersion: "1",
          protocolId: "asha-rulebench.protocol",
          protocolVersion: 9,
          host: {
            adapterId: "rulebench-process-host",
            storageMode: "filesystem",
            contentStorageAdapter: "versionedFilesystem",
            replayStorageAdapter: "versionedFilesystem",
            replayRecoveryMode: "finalizedArchive",
            sessionRecoveryMode: "none",
            authorityViewerMode: "liveAuthorityReadback",
          },
          providers: [],
          rulesets: [],
          packages: [],
          scenarios: [],
          capabilities: [
            {
              id: "session.active-recovery",
              kind: "session",
              version: "none",
              support: {
                declared: false,
                validationSupported: false,
                runtimeExecutable: false,
                protocolExposed: false,
                liveHostExposed: false,
                uiExposed: false,
                regressionCovered: false,
                durableAcrossRestart: false,
              },
              evidence: ["rulebench-process-host.session-recovery-mode:none"],
            },
          ],
        },
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.loadCapabilities();

    expect(store.capabilities()).toMatchObject({
      kind: "data",
      value: {
        hostLabel: "rulebench-process-host · filesystem",
        capabilities: [
          {
            id: "session.active-recovery",
            support: { supportLabel: "Not declared" },
          },
        ],
      },
    });
  });

  it("owns policy catalog, bounded experiment progress, cancellation, and comparison as AsyncState", async () => {
    const matrix = {
      id: "policy-lab-1",
      status: "planned",
      plannedTrialCount: 2,
      completedTrialCount: 0,
      maxStepsPerTrial: 8,
      trials: [],
      reason: "Ready.",
    };
    let experiments = [matrix];
    const transport = createFakeRulebenchLiveTransport({
      listAutomationPolicies: async () => ({
        ok: true,
        value: [
          {
            id: "lowestVitalityTarget",
            version: 1,
            title: "Lowest vitality target",
            summary: "Prefer the weakest target.",
            selector: "lowestVitalityTarget",
            requirement: "anyCombatRuleset",
            compatibility: [
              {
                rulesetId: "rules",
                rulesetVersion: "1",
                compatible: true,
                code: "accepted",
                reason: "Compatible.",
              },
            ],
          },
        ],
      }),
      listExperiments: async () => ({ ok: true, value: experiments }),
      createExperiment: async () => ({ ok: true, value: matrix }),
      advanceExperiment: async () => {
        experiments = [{ ...matrix, status: "running", completedTrialCount: 1 }];
        return { ok: true, value: experiments[0] };
      },
      cancelExperiment: async () => {
        experiments = [{ ...matrix, status: "cancelled", reason: "Cancelled." }];
        return { ok: true, value: experiments[0] };
      },
      compareExperimentTrials: async () => ({
        ok: true,
        value: {
          identical: false,
          firstDivergenceIndex: 0,
          expectedTrialId: "trial-1",
          actualTrialId: "trial-2",
          expectedEvidence: null,
          actualEvidence: null,
          reason: "First decision diverged.",
        },
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.loadAutomationPolicies();
    await store.loadExperiments();
    await store.advanceExperiment("policy-lab-1");
    await store.compareExperimentTrials({
      expectedExperimentId: "policy-lab-1",
      expectedTrialId: "trial-1",
      actualExperimentId: "policy-lab-1",
      actualTrialId: "trial-2",
    });

    expect(store.automationPolicies()).toMatchObject({
      kind: "data",
      value: [{ id: "lowestVitalityTarget" }],
    });
    expect(store.experiments()).toMatchObject({
      kind: "data",
      value: [{ status: "running", completedTrialCount: 1 }],
    });
    expect(store.experimentComparison()).toMatchObject({
      kind: "data",
      value: { identical: false, firstDivergenceIndex: 0 },
    });

    await store.cancelExperiment("policy-lab-1");
    expect(store.experiments()).toMatchObject({
      kind: "data",
      value: [{ status: "cancelled" }],
    });
  });

  it("loads connection, scenarios, snapshot, options, candidates, and preflight through injected transport", async () => {
    const snapshot = makeLiveSessionSnapshot();
    const transport = createFakeRulebenchLiveTransport({
      connect: async () => ({
        ok: true,
        value: {
          protocolId: "asha-rulebench.protocol",
          protocolVersion: 9,
          authoritySurface: "test-authority",
        },
      }),
      listScenarios: async () => ({
        ok: true,
        value: [
          {
            id: "scenario",
            title: "Scenario",
            summary: "Test scenario.",
            rulesetId: "rules",
            rulesetVersion: "1.0.0",
            contentPackId: null,
            contentPackVersion: null,
            participants: [],
          },
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

  it("submits the current authored reaction command and projects the resumed snapshot", async () => {
    let submitted:
      | Parameters<RulebenchLiveTransport["submitReaction"]>[1]
      | undefined;
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({
        ok: true,
        value: makeLiveSessionSnapshot({ reactionWindow: true }),
      }),
      submitReaction: async (_sessionId, command) => {
        submitted = command;
        return {
          ok: true,
          value: {
            reaction: {
              command,
              accepted: true,
              decisionKind: "accepted",
              previousWindow: makeLiveSessionSnapshot({ reactionWindow: true })
                .currentReactionWindow,
              nextWindow: null,
              openedNestedWindow: null,
              resumedPendingResolution: true,
              trace: [],
              reason: "Reaction accepted and pending resolution resumed.",
            },
            snapshot: makeLiveSessionSnapshot({
              raiderHitPoints: 11,
              fingerprint: "state-reaction-resolved",
            }),
          },
        };
      },
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");

    await store.submitReaction({
      windowId: "window-1",
      reactorId: "entity-raider",
      responseKind: "accept",
      optionId: "raider-ward",
    });

    expect(submitted).toEqual({
      windowId: "window-1",
      reactorId: "entity-raider",
      responseKind: "accept",
      optionId: "raider-ward",
    });
    expect(store.reaction()).toMatchObject({
      kind: "data",
      value: { accepted: true, resumedPendingResolution: true },
    });
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: {
        reactionWindow: null,
        participants: [{}, { hitPointLabel: "11/18 HP" }],
      },
    });
  });

  it("owns default roll mode and materializes generated request configuration", async () => {
    let submitted:
      | Parameters<RulebenchLiveTransport["submitIntent"]>[1]
      | undefined;
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      submitIntent: async (_sessionId, command) => {
        submitted = command;
        return { ok: true, value: makeLiveCommandExecution(true) };
      },
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    store.setIntent(intent);
    store.setDefaultRollMode("authorityGenerated");

    await store.submitIntent({
      id: "generated",
      title: "Generated",
      summary: "Generate authority rolls.",
      rollStream: [99, 99],
    });

    expect(submitted).toMatchObject({
      rollMode: "authorityGenerated",
      rollStream: [],
      generatedSeed: fixedClock.now().getTime() >>> 0,
    });
    expect(store.defaultRollMode()).toBe("authorityGenerated");
  });

  it("clears stale targets when action selection or authority actor changes", async () => {
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: makeLiveSessionSnapshot() }),
      submitControl: async () => ({
        ok: true,
        value: {
          commandKind: "advanceTurn",
          accepted: true,
          decisionKind: "accepted",
          previousLifecyclePhase: "inProgress",
          nextLifecyclePhase: "inProgress",
          stateBeforeFingerprint: { algorithm: "test", value: "state-0" },
          stateAfterFingerprint: { algorithm: "test", value: "state-0" },
          reason: "Advanced.",
          snapshot: {
            ...makeLiveSessionSnapshot(),
            currentActorId: "entity-raider",
            turnIndex: 1,
          },
        },
      }),
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    store.selectAction("hexing_bolt");
    store.selectEntityTarget("entity-raider");
    expect(store.intent().targetId).toBe("entity-raider");

    store.selectAction("move.entity-adept");
    expect(store.intent()).toEqual({
      actorId: "entity-adept",
      actionId: "move.entity-adept",
      targetId: "",
    });

    await store.submitControl("advanceTurn");
    expect(store.intent()).toEqual({
      actorId: "entity-raider",
      actionId: "",
      targetId: "",
    });
  });

  it("submits generated target-set intent fields without flattening area selection", async () => {
    const base = makeLiveSessionSnapshot();
    const areaAction = {
      ...base.options.actions[0],
      actionId: "storm-pulse",
      actionName: "Storm Pulse",
      targetMode: "cell" as const,
      targets: [],
      targetSets: [
        {
          id: "area-8-3",
          targetIds: ["entity-bruiser", "entity-raider"],
          targetCell: { x: 8, y: 3 },
          rollPolicy: "shared" as const,
          reason: "Canonical bounded area set.",
        },
      ],
    };
    const snapshot = {
      ...base,
      options: { ...base.options, actions: [areaAction] },
    };
    let observedIntent: RulebenchUseActionIntentDto | null = null;
    const transport = createFakeRulebenchLiveTransport({
      getSession: async () => ({ ok: true, value: snapshot }),
      preflightIntent: async (_sessionId, submittedIntent) => {
        observedIntent = submittedIntent;
        return {
          ok: true,
          value: { ...acceptedLivePreflight, intent: submittedIntent },
        };
      },
    });
    const store = new LiveCombatStore(transport, fixedClock);
    await store.selectSession("live-session");
    store.selectAction("storm-pulse");
    store.selectTargetSet(["entity-bruiser", "entity-raider"], { x: 8, y: 3 });
    await store.preflightIntent();

    expect(observedIntent).toMatchObject({
      actorId: "entity-adept",
      actionId: "storm-pulse",
      targetId: "",
      targetIds: ["entity-bruiser", "entity-raider"],
      targetCell: { x: 8, y: 3 },
      destinationCell: null,
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

  it("loads recovery evidence and performs explicit fork and discard lifecycles", async () => {
    const calls: string[] = [];
    const catalog = {
      sessions: [
        {
          sessionId: "restored-session",
          origin: "restored" as const,
          state: "recoverable" as const,
          generation: 2,
          lastVerifiedFrameId: "2:fingerprint",
          pendingReactionWindowId: "reaction-1",
          actions: ["discard", "fork"] as const,
        },
      ],
      issues: [
        {
          code: "sessionRecoveryFrameMismatch",
          message: "Stored frame did not verify.",
          path: "quarantine/recovery.json",
        },
      ],
    };
    const transport = createFakeRulebenchLiveTransport({
      getSessionRecovery: async () => ({ ok: true, value: catalog }),
      listSessions: async () => ({ ok: true, value: [] }),
      forkRecoveredSession: async (sessionId, newSessionId) => {
        calls.push(`fork:${sessionId}:${newSessionId}`);
        return {
          ok: true,
          value: makeLiveSessionSnapshot({ sessionId: newSessionId }),
        };
      },
      discardRecoveredSession: async (sessionId) => {
        calls.push(`discard:${sessionId}`);
        return {
          ok: true,
          value: makeLiveSessionSnapshot({ sessionId }),
        };
      },
    });
    const store = new LiveCombatStore(transport, fixedClock);

    await store.loadRecovery();
    expect(store.recovery()).toEqual({ kind: "data", value: catalog });

    await store.forkRecoveredSession("restored-session", "explicit-fork");
    expect(store.selectedSessionId()).toBe("explicit-fork");
    expect(store.snapshot()).toMatchObject({
      kind: "data",
      value: { sessionId: "explicit-fork" },
    });

    await store.discardRecoveredSession("restored-session");
    expect(calls).toEqual([
      "fork:restored-session:explicit-fork",
      "discard:restored-session",
    ]);
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
        protocolVersion: 9,
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
    readonly reactionWindow?: boolean;
  } = {},
): RulebenchLiveSessionSnapshotDto {
  const sessionId = options.sessionId ?? "live-session";
  const lifecyclePhase = options.lifecyclePhase ?? "inProgress";
  const raiderHitPoints = options.raiderHitPoints ?? 18;
  const fingerprint = options.fingerprint ?? "state-0";
  return {
    sessionId,
    authoredActionBinding: null,
    authoredScenarioBinding: null,
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
        position: { x: 1, y: 1 },
        movementRemaining: 0,
        movementMaximum: 0,
      },
      {
        id: "entity-raider",
        name: "Raider",
        currentHitPoints: raiderHitPoints,
        maxHitPoints: 18,
        temporaryVitality: 0,
        defeated: false,
        conditions: raiderHitPoints < 18 ? ["rattled"] : [],
        position: { x: 4, y: 1 },
        movementRemaining: 0,
        movementMaximum: 0,
      },
    ],
    board: {
      id: "two-combatant-hexing-bolt",
      width: 6,
      height: 4,
      cells: [],
    },
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
                targetMode: "entity",
                targetSets: [],
                destinations: [],
                targets: [
                  {
                    targetId: "entity-raider",
                    targetName: "Raider",
                    currentHitPoints: raiderHitPoints,
                    maxHitPoints: 18,
                    reason: "Target is legal.",
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
    gameplayFabric: {
      registryDigest: "registry",
      bindingRegistryHash: "bindings",
      moduleStateHash: "module-state",
      runtimeHostHash: "runtime-host",
      reactionFrameHashes: [],
      decisions: [],
      pendingDecisionCount: options.reactionWindow === true ? 1 : 0,
    },
    currentReactionWindow:
      options.reactionWindow === true
        ? {
            id: "window-1",
            hookId: "hexing-bolt.pre-effect",
            timing: "beforeEffect",
            depth: 0,
            maximumNestedDepth: 1,
            parentWindowId: null,
            triggerStepId: "step-1",
            triggerActionId: "hexing_bolt",
            eligibleReactorIds: ["entity-adept", "entity-raider"],
            currentReactorId: "entity-raider",
            options: [
              {
                optionId: "raider-ward",
                reactorId: "entity-raider",
                opensNestedWindow: false,
              },
            ],
            responses: [],
            status: "open",
          }
        : null,
    reactionWindowLifecycleLog: [],
    reactionAuditLog: [],
    finalization: null,
    combatLog: [],
    auditLog: [],
    stateFingerprint: { algorithm: "test", value: fingerprint },
    actionResourceFingerprint: {
      algorithm: "test-resources",
      value: `resources-${fingerprint}`,
    },
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
      targetResults: [],
      trace: [],
      stateBeforeFingerprint: { algorithm: "test", value: "state-0" },
      stateAfterFingerprint: {
        algorithm: "test",
        value: accepted ? "state-1" : "state-0",
      },
      rollMode: "supplied",
      generatedRolls: [],
    },
    snapshot: makeLiveSessionSnapshot({
      raiderHitPoints: accepted ? 9 : 18,
      fingerprint: accepted ? "state-1" : "state-0",
    }),
  };
}
