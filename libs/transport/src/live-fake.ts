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
    listScenarios: (requestOptions) =>
      handlers.listScenarios?.(requestOptions) ?? unavailable("listScenarios"),
    listSessions: (requestOptions) =>
      handlers.listSessions?.(requestOptions) ?? unavailable("listSessions"),
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
    runAutomaticStep: (sessionId, request, requestOptions) =>
      handlers.runAutomaticStep?.(sessionId, request, requestOptions) ??
      unavailable("runAutomaticStep"),
    runAutomaticCombat: (sessionId, request, requestOptions) =>
      handlers.runAutomaticCombat?.(sessionId, request, requestOptions) ??
      unavailable("runAutomaticCombat"),
    listReplayPackages: () =>
      handlers.listReplayPackages?.() ?? replayUnavailable("listReplayPackages"),
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
