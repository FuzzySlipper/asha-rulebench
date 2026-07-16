import type {
  RulebenchLiveTransportErrorDto,
  RulebenchProtocolHandshakeDto,
  RulebenchReplayArchiveErrorDto,
} from "@asha-rulebench/protocol";
import type { ReplayReviewResult } from "./replay-review";
import type {
  RulebenchLiveConnectionState,
  RulebenchLiveRequestOptions,
  RulebenchLiveTransport,
  RulebenchLiveTransportResult,
} from "./live";

type RulebenchLiveTransportHandlers = Omit<
  RulebenchLiveTransport,
  "connectionState" | "disconnect"
>;

export function createFakeRulebenchLiveTransport(
  handlers: Partial<RulebenchLiveTransportHandlers> = {},
): RulebenchLiveTransport {
  let connectionState: RulebenchLiveConnectionState = { kind: "idle" };
  const unavailable = <T>(
    operation: string,
  ): Promise<RulebenchLiveTransportResult<T>> =>
    Promise.resolve({
      ok: false,
      error: fakeError(
        "handlerNotConfigured",
        `Fake live transport handler is not configured: ${operation}.`,
      ),
    });
  const replayUnavailable = <T>(
    operation: string,
  ): Promise<ReplayReviewResult<T>> =>
    Promise.resolve({
      ok: false,
      error: replayFakeError(
        "handlerNotConfigured",
        `Fake live transport handler is not configured: ${operation}.`,
      ),
    });
  const connect = async (
    requestOptions?: RulebenchLiveRequestOptions,
  ): Promise<RulebenchLiveTransportResult<RulebenchProtocolHandshakeDto>> => {
    connectionState = { kind: "connecting" };
    const result =
      handlers.connect === undefined
        ? await unavailable<RulebenchProtocolHandshakeDto>("connect")
        : await handlers.connect(requestOptions);
    connectionState = result.ok
      ? { kind: "connected", handshake: result.value }
      : { kind: "disconnected", error: result.error };
    return result;
  };

  return {
    connectionState: () => connectionState,
    connect,
    disconnect: () => {
      connectionState = { kind: "disconnected", error: null };
    },
    getCapabilities: (requestOptions) =>
      handlers.getCapabilities?.(requestOptions) ??
      unavailable("getCapabilities"),
    listAutomationPolicies: (requestOptions) =>
      handlers.listAutomationPolicies?.(requestOptions) ??
      unavailable("listAutomationPolicies"),
    listExperiments: (requestOptions) =>
      handlers.listExperiments?.(requestOptions) ??
      unavailable("listExperiments"),
    createExperiment: (matrix, requestOptions) =>
      handlers.createExperiment?.(matrix, requestOptions) ??
      unavailable("createExperiment"),
    getExperiment: (experimentId, requestOptions) =>
      handlers.getExperiment?.(experimentId, requestOptions) ??
      unavailable("getExperiment"),
    advanceExperiment: (experimentId, requestOptions) =>
      handlers.advanceExperiment?.(experimentId, requestOptions) ??
      unavailable("advanceExperiment"),
    cancelExperiment: (experimentId, requestOptions) =>
      handlers.cancelExperiment?.(experimentId, requestOptions) ??
      unavailable("cancelExperiment"),
    compareExperimentTrials: (comparison, requestOptions) =>
      handlers.compareExperimentTrials?.(comparison, requestOptions) ??
      unavailable("compareExperimentTrials"),
    listScenarios: (requestOptions) =>
      handlers.listScenarios?.(requestOptions) ?? unavailable("listScenarios"),
    listViewerScenarios: (requestOptions) =>
      handlers.listViewerScenarios?.(requestOptions) ??
      unavailable("listViewerScenarios"),
    getViewerScenario: (scenarioId, requestOptions) =>
      handlers.getViewerScenario?.(scenarioId, requestOptions) ??
      unavailable("getViewerScenario"),
    listViewerSessions: (requestOptions) =>
      handlers.listViewerSessions?.(requestOptions) ??
      unavailable("listViewerSessions"),
    getViewerSessionStep: (sessionId, stepId, requestOptions) =>
      handlers.getViewerSessionStep?.(sessionId, stepId, requestOptions) ??
      unavailable("getViewerSessionStep"),
    listSessions: (requestOptions) =>
      handlers.listSessions?.(requestOptions) ?? unavailable("listSessions"),
    getSessionRecovery: (requestOptions) =>
      handlers.getSessionRecovery?.(requestOptions) ??
      unavailable("getSessionRecovery"),
    discardRecoveredSession: (sessionId, requestOptions) =>
      handlers.discardRecoveredSession?.(sessionId, requestOptions) ??
      unavailable("discardRecoveredSession"),
    forkRecoveredSession: (sessionId, newSessionId, requestOptions) =>
      handlers.forkRecoveredSession?.(
        sessionId,
        newSessionId,
        requestOptions,
      ) ?? unavailable("forkRecoveredSession"),
    createSession: (request, requestOptions) =>
      handlers.createSession?.(request, requestOptions) ??
      unavailable("createSession"),
    getSession: (sessionId, requestOptions) =>
      handlers.getSession?.(sessionId, requestOptions) ??
      unavailable("getSession"),
    closeSession: (sessionId, requestOptions) =>
      handlers.closeSession?.(sessionId, requestOptions) ??
      unavailable("closeSession"),
    getCurrentActorOptions: (sessionId, requestOptions) =>
      handlers.getCurrentActorOptions?.(sessionId, requestOptions) ??
      unavailable("getCurrentActorOptions"),
    preflightIntent: (sessionId, intent, requestOptions) =>
      handlers.preflightIntent?.(sessionId, intent, requestOptions) ??
      unavailable("preflightIntent"),
    listCandidates: (sessionId, requestOptions) =>
      handlers.listCandidates?.(sessionId, requestOptions) ??
      unavailable("listCandidates"),
    submitIntent: (sessionId, command, requestOptions) =>
      handlers.submitIntent?.(sessionId, command, requestOptions) ??
      unavailable("submitIntent"),
    submitControl: (sessionId, command, requestOptions) =>
      handlers.submitControl?.(sessionId, command, requestOptions) ??
      unavailable("submitControl"),
    submitReaction: (sessionId, command, requestOptions) =>
      handlers.submitReaction?.(sessionId, command, requestOptions) ??
      unavailable("submitReaction"),
    runAutomaticStep: (sessionId, request, requestOptions) =>
      handlers.runAutomaticStep?.(sessionId, request, requestOptions) ??
      unavailable("runAutomaticStep"),
    runAutomaticCombat: (sessionId, request, requestOptions) =>
      handlers.runAutomaticCombat?.(sessionId, request, requestOptions) ??
      unavailable("runAutomaticCombat"),
    listContentWorkspace: (requestOptions) =>
      handlers.listContentWorkspace?.(requestOptions) ??
      unavailable("listContentWorkspace"),
    importContent: (authoredPayload, replacementPolicy, requestOptions) =>
      handlers.importContent?.(
        authoredPayload,
        replacementPolicy,
        requestOptions,
      ) ?? unavailable("importContent"),
    validateContent: (authoredPayload, requestOptions) =>
      handlers.validateContent?.(authoredPayload, requestOptions) ??
      unavailable("validateContent"),
    reviewContent: (reference, requestOptions) =>
      handlers.reviewContent?.(reference, requestOptions) ??
      unavailable("reviewContent"),
    compareContent: (authoredPayload, requestOptions) =>
      handlers.compareContent?.(authoredPayload, requestOptions) ??
      unavailable("compareContent"),
    activateContent: (reference, requestOptions) =>
      handlers.activateContent?.(reference, requestOptions) ??
      unavailable("activateContent"),
    deactivateContent: (reference, requestOptions) =>
      handlers.deactivateContent?.(reference, requestOptions) ??
      unavailable("deactivateContent"),
    deleteContent: (reference, requestOptions) =>
      handlers.deleteContent?.(reference, requestOptions) ??
      unavailable("deleteContent"),
    listReplayPackages: () =>
      handlers.listReplayPackages?.() ??
      replayUnavailable("listReplayPackages"),
    loadReplayPackage: (packageId) =>
      handlers.loadReplayPackage?.(packageId) ??
      replayUnavailable("loadReplayPackage"),
    loadReplayVerification: (packageId) =>
      handlers.loadReplayVerification?.(packageId) ??
      replayUnavailable("loadReplayVerification"),
    compareReplayPackages: (expectedPackageId, actualPackageId) =>
      handlers.compareReplayPackages?.(expectedPackageId, actualPackageId) ??
      replayUnavailable("compareReplayPackages"),
  };
}

function replayFakeError(
  code: string,
  message: string,
): RulebenchReplayArchiveErrorDto {
  return { kind: "storage", code, message, retryable: false };
}

function fakeError(
  code: string,
  message: string,
): RulebenchLiveTransportErrorDto {
  return { kind: "fake", code, message, retryable: false };
}
