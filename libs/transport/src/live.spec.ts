import { describe, expect, it } from "vitest";
import type {
  RulebenchCombatAutomationPolicySpecDto,
  RulebenchProtocolHandshakeDto,
  RulebenchScenarioOptionDto,
} from "@asha-rulebench/protocol";
import {
  createLiveRulebenchTransport,
  RULEBENCH_PROTOCOL_VERSION,
} from "./live";
import { createFakeRulebenchLiveTransport } from "./live-fake";

const handshake: RulebenchProtocolHandshakeDto = {
  protocolId: "asha-rulebench.protocol",
  protocolVersion: RULEBENCH_PROTOCOL_VERSION,
  authoritySurface: "asha-rulebench.local-authority.v0",
};

describe("live Rulebench transport", () => {
  it("maps every public operation to the versioned host route without changing request DTOs", async () => {
    const calls: Array<{
      readonly url: string;
      readonly method: string | undefined;
      readonly version: string | null;
      readonly body: BodyInit | null | undefined;
    }> = [];
    const responseBodies: unknown[] = [
      handshake,
      ...Array.from({ length: 32 }, () => ({})),
    ];
    const fetchRequest: typeof fetch = async (input, init) => {
      calls.push({
        url: String(input),
        method: init?.method,
        version: new Headers(init?.headers).get("x-rulebench-protocol-version"),
        body: init?.body,
      });
      return Response.json(responseBodies.shift());
    };
    const transport = createLiveRulebenchTransport({
      apiBaseUrl: "http://rulebench.test/api/rulebench/v1/",
      fetch: fetchRequest,
    });
    const createRequest = {
      sessionId: "session/one",
      scenarioId: "hexing-bolt-hit",
      participantOrder: ["entity-adept", "entity-raider"],
    };
    const intent = {
      actorId: "adept",
      actionId: "hexing-bolt",
      targetId: "raider",
      destinationCell: null,
      observedOrigin: null,
    };
    const intentCommand = {
      id: "step-1",
      title: "Hexing Bolt",
      summary: "Adept attacks Raider.",
      intent,
      rollStream: [17, 5],
      rollMode: "authorityGenerated" as const,
      generatedSeed: 42,
    };
    const policy: RulebenchCombatAutomationPolicySpecDto = {
      id: "first-accepted-candidate",
      version: 1,
      noCandidateBehavior: "advanceTurn",
    };
    const automaticStep = {
      id: "auto-step",
      title: "Automatic step",
      summary: "Run one authority-selected step.",
      rollStream: [17, 5],
      rollMode: "authorityGenerated" as const,
      generatedSeed: 43,
      policy,
    };
    const automaticRun = {
      ...automaticStep,
      id: "auto-run",
      title: "Automatic run",
      maxSteps: 10,
    };

    await transport.connect();
    await transport.listScenarios();
    await transport.listViewerScenarios();
    await transport.getViewerScenario("scenario/one");
    await transport.listViewerSessions();
    await transport.getViewerSessionStep("session/one", "step/one");
    await transport.listSessions();
    await transport.getSessionRecovery();
    await transport.discardRecoveredSession(createRequest.sessionId);
    await transport.forkRecoveredSession(
      createRequest.sessionId,
      "fork/session",
    );
    await transport.createSession(createRequest);
    await transport.getSession(createRequest.sessionId);
    await transport.closeSession(createRequest.sessionId);
    await transport.getCurrentActorOptions(createRequest.sessionId);
    await transport.preflightIntent(createRequest.sessionId, intent);
    await transport.listCandidates(createRequest.sessionId);
    await transport.submitIntent(createRequest.sessionId, intentCommand);
    await transport.submitControl(createRequest.sessionId, {
      kind: "advanceTurn",
    });
    const reactionCommand = {
      windowId: "window-1",
      reactorId: "entity-adept",
      responseKind: "pass" as const,
      optionId: null,
    };
    await transport.submitReaction(createRequest.sessionId, reactionCommand);
    await transport.runAutomaticStep(createRequest.sessionId, automaticStep);
    await transport.runAutomaticCombat(createRequest.sessionId, automaticRun);
    const contentReference = {
      id: "pack.authored",
      version: "1.0.0",
      fingerprint: { algorithm: "fnv1a64", value: "abc123" },
    };
    await transport.listContentWorkspace();
    await transport.importContent('{"formatVersion":1}', "reject");
    await transport.reviewContent(contentReference);
    await transport.compareContent('{"formatVersion":1}');
    await transport.activateContent(contentReference);
    await transport.deactivateContent(contentReference);
    await transport.deleteContent(contentReference);
    await transport.listReplayPackages();
    await transport.loadReplayPackage("replay/one");
    await transport.loadReplayVerification("replay/one");
    await transport.compareReplayPackages("expected", "actual");
    await transport.getCapabilities();

    expect(calls.map(({ method, url }) => `${method} ${url}`)).toEqual([
      "GET http://rulebench.test/api/rulebench/v1/handshake",
      "GET http://rulebench.test/api/rulebench/v1/scenarios",
      "GET http://rulebench.test/api/rulebench/v1/viewer/scenarios",
      "GET http://rulebench.test/api/rulebench/v1/viewer/scenarios/scenario%2Fone",
      "GET http://rulebench.test/api/rulebench/v1/viewer/sessions",
      "GET http://rulebench.test/api/rulebench/v1/viewer/sessions/session%2Fone/steps/step%2Fone",
      "GET http://rulebench.test/api/rulebench/v1/sessions",
      "GET http://rulebench.test/api/rulebench/v1/session-recovery",
      "DELETE http://rulebench.test/api/rulebench/v1/session-recovery/session%2Fone",
      "POST http://rulebench.test/api/rulebench/v1/session-recovery/session%2Fone/fork",
      "POST http://rulebench.test/api/rulebench/v1/sessions",
      "GET http://rulebench.test/api/rulebench/v1/sessions/session%2Fone",
      "DELETE http://rulebench.test/api/rulebench/v1/sessions/session%2Fone",
      "GET http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/options",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/preflight",
      "GET http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/candidates",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/intents",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/controls",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/reactions",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/automatic-step",
      "POST http://rulebench.test/api/rulebench/v1/sessions/session%2Fone/automatic-run",
      "GET http://rulebench.test/api/rulebench/v1/content",
      "POST http://rulebench.test/api/rulebench/v1/content/import",
      "POST http://rulebench.test/api/rulebench/v1/content/review",
      "POST http://rulebench.test/api/rulebench/v1/content/compare",
      "POST http://rulebench.test/api/rulebench/v1/content/activate",
      "POST http://rulebench.test/api/rulebench/v1/content/deactivate",
      "POST http://rulebench.test/api/rulebench/v1/content/delete",
      "GET http://rulebench.test/api/rulebench/v1/replays",
      "GET http://rulebench.test/api/rulebench/v1/replays/replay%2Fone",
      "POST http://rulebench.test/api/rulebench/v1/replays/replay%2Fone/verify",
      "POST http://rulebench.test/api/rulebench/v1/replays/compare",
      "GET http://rulebench.test/api/rulebench/v1/capabilities",
    ]);
    expect(calls.every((call) => call.version === "4")).toBe(true);
    expect(calls[9]?.body).toBe(
      JSON.stringify({ newSessionId: "fork/session" }),
    );
    expect(calls[10]?.body).toBe(JSON.stringify(createRequest));
    expect(calls[14]?.body).toBe(JSON.stringify(intent));
    expect(calls[16]?.body).toBe(JSON.stringify(intentCommand));
    expect(calls[18]?.body).toBe(JSON.stringify(reactionCommand));
    expect(calls[19]?.body).toBe(JSON.stringify(automaticStep));
    expect(calls[20]?.body).toBe(JSON.stringify(automaticRun));
    expect(calls[31]?.body).toBe(
      JSON.stringify({
        expectedPackageId: "expected",
        actualPackageId: "actual",
      }),
    );
  });

  it("tracks a verified handshake and rejects a mismatched protocol without repairing it", async () => {
    const connected = createLiveRulebenchTransport({
      fetch: async () => Response.json(handshake),
    });
    const connectedResult = await connected.connect();

    expect(connectedResult).toEqual({ ok: true, value: handshake });
    expect(connected.connectionState()).toEqual({
      kind: "connected",
      handshake,
    });

    const mismatched = createLiveRulebenchTransport({
      fetch: async () => Response.json({ ...handshake, protocolVersion: 5 }),
    });
    const mismatchResult = await mismatched.connect();

    expect(mismatchResult).toEqual({
      ok: false,
      error: {
        kind: "protocol",
        code: "handshakeMismatch",
        message:
          "Expected asha-rulebench.protocol v4; received asha-rulebench.protocol v5.",
        retryable: false,
      },
    });
    expect(mismatched.connectionState().kind).toBe("disconnected");
  });

  it("preserves host errors and classifies malformed or unreachable responses", async () => {
    const hostError = {
      kind: "bridge",
      code: "unknownSession",
      message: "Unknown combat session handle: missing.",
      retryable: false,
    };
    const rejected = createLiveRulebenchTransport({
      fetch: async () => Response.json(hostError, { status: 404 }),
    });
    await expect(rejected.getSession("missing")).resolves.toEqual({
      ok: false,
      error: hostError,
    });

    const malformed = createLiveRulebenchTransport({
      fetch: async () => new Response("not json", { status: 200 }),
    });
    await expect(malformed.listScenarios()).resolves.toEqual({
      ok: false,
      error: {
        kind: "serialization",
        code: "invalidJsonResponse",
        message: "Rulebench host returned invalid JSON with HTTP 200.",
        retryable: false,
      },
    });

    const unreachable = createLiveRulebenchTransport({
      fetch: async () => {
        throw new TypeError("connection refused");
      },
    });
    await expect(unreachable.listSessions()).resolves.toEqual({
      ok: false,
      error: {
        kind: "network",
        code: "requestFailed",
        message: "connection refused",
        retryable: true,
      },
    });
    expect(unreachable.connectionState().kind).toBe("disconnected");
  });

  it("cancels in-flight requests when disconnected and cleans up external abort listeners", async () => {
    const fetchRequest: typeof fetch = async (_input, init) =>
      new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener(
          "abort",
          () => reject(new DOMException("aborted", "AbortError")),
          { once: true },
        );
      });
    const transport = createLiveRulebenchTransport({ fetch: fetchRequest });

    const pending = transport.listSessions();
    transport.disconnect();

    await expect(pending).resolves.toEqual({
      ok: false,
      error: {
        kind: "cancellation",
        code: "requestAborted",
        message: "Rulebench host request was cancelled.",
        retryable: false,
      },
    });
    expect(transport.connectionState()).toEqual({
      kind: "disconnected",
      error: null,
    });
  });
});

describe("fake live Rulebench transport", () => {
  it("has interface parity and returns configured authority evidence by identity", async () => {
    const scenarios: readonly RulebenchScenarioOptionDto[] = [
      {
        id: "scenario",
        title: "Scenario",
        summary: "Authority fixture.",
        rulesetId: "rules",
        rulesetVersion: "1.0.0",
        contentPackId: null,
        contentPackVersion: null,
        participants: [],
      },
    ];
    const transport = createFakeRulebenchLiveTransport({
      connect: async () => ({ ok: true, value: handshake }),
      listScenarios: async () => ({ ok: true, value: scenarios }),
    });

    await transport.connect();
    const result = await transport.listScenarios();

    expect(result.ok && result.value).toBe(scenarios);
    expect(transport.connectionState()).toEqual({
      kind: "connected",
      handshake,
    });
    await expect(transport.listSessions()).resolves.toEqual({
      ok: false,
      error: {
        kind: "fake",
        code: "handlerNotConfigured",
        message: "Fake live transport handler is not configured: listSessions.",
        retryable: false,
      },
    });
  });
});
