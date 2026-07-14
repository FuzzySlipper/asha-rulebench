import type {
  RulebenchAutomaticRunSpecDto,
  RulebenchAutomaticStepSpecDto,
  RulebenchCombatControlCommandDto,
  RulebenchCombatSessionCreateRequestDto,
  RulebenchCombatSessionIntentCommandDto,
  RulebenchLiveAutomaticRunDto,
  RulebenchLiveAutomaticStepDto,
  RulebenchLiveCandidateSummaryDto,
  RulebenchLiveCommandExecutionDto,
  RulebenchLiveControlExecutionDto,
  RulebenchLiveCurrentActorOptionsDto,
  RulebenchLivePreflightDto,
  RulebenchLiveReactionExecutionDto,
  RulebenchLiveSessionSnapshotDto,
  RulebenchLiveTransportErrorDto,
  RulebenchProtocolHandshakeDto,
  RulebenchReplayArchiveErrorDto,
  RulebenchReplayArchiveMetadataDto,
  RulebenchReplayComparisonReadoutDto,
  RulebenchReplayPackageReviewDto,
  RulebenchReplayVerificationReadoutDto,
  RulebenchReactionCommandSpecDto,
  RulebenchScenarioOptionDto,
  RulebenchUseActionIntentDto,
} from "@asha-rulebench/protocol";
import type { ReplayReviewResult, ReplayReviewTransport } from "./replay-review";

export const RULEBENCH_PROTOCOL_ID = "asha-rulebench.protocol";
export const RULEBENCH_PROTOCOL_VERSION = 3;

const DEFAULT_API_BASE_URL = "/api/rulebench/v1";

export type RulebenchLiveTransportResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: RulebenchLiveTransportErrorDto };

export type RulebenchLiveConnectionState =
  | { readonly kind: "idle" }
  | { readonly kind: "connecting" }
  | {
      readonly kind: "connected";
      readonly handshake: RulebenchProtocolHandshakeDto;
    }
  | {
      readonly kind: "disconnected";
      readonly error: RulebenchLiveTransportErrorDto | null;
    };

export interface RulebenchLiveRequestOptions {
  readonly signal?: AbortSignal;
}

export interface RulebenchLiveTransport extends ReplayReviewTransport {
  readonly connectionState: () => RulebenchLiveConnectionState;
  readonly connect: (
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchProtocolHandshakeDto>>;
  readonly disconnect: () => void;
  readonly listScenarios: (
    options?: RulebenchLiveRequestOptions,
  ) => Promise<
    RulebenchLiveTransportResult<readonly RulebenchScenarioOptionDto[]>
  >;
  readonly listSessions: (
    options?: RulebenchLiveRequestOptions,
  ) => Promise<
    RulebenchLiveTransportResult<readonly RulebenchLiveSessionSnapshotDto[]>
  >;
  readonly createSession: (
    request: RulebenchCombatSessionCreateRequestDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveSessionSnapshotDto>>;
  readonly getSession: (
    sessionId: string,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveSessionSnapshotDto>>;
  readonly closeSession: (
    sessionId: string,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveSessionSnapshotDto>>;
  readonly getCurrentActorOptions: (
    sessionId: string,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<
    RulebenchLiveTransportResult<RulebenchLiveCurrentActorOptionsDto>
  >;
  readonly preflightIntent: (
    sessionId: string,
    intent: RulebenchUseActionIntentDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLivePreflightDto>>;
  readonly listCandidates: (
    sessionId: string,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveCandidateSummaryDto>>;
  readonly submitIntent: (
    sessionId: string,
    command: RulebenchCombatSessionIntentCommandDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveCommandExecutionDto>>;
  readonly submitControl: (
    sessionId: string,
    command: RulebenchCombatControlCommandDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveControlExecutionDto>>;
  readonly submitReaction: (
    sessionId: string,
    command: RulebenchReactionCommandSpecDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveReactionExecutionDto>>;
  readonly runAutomaticStep: (
    sessionId: string,
    request: RulebenchAutomaticStepSpecDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveAutomaticStepDto>>;
  readonly runAutomaticCombat: (
    sessionId: string,
    request: RulebenchAutomaticRunSpecDto,
    options?: RulebenchLiveRequestOptions,
  ) => Promise<RulebenchLiveTransportResult<RulebenchLiveAutomaticRunDto>>;
}

export interface RulebenchLiveTransportOptions {
  readonly apiBaseUrl?: string;
  readonly fetch?: typeof fetch;
  readonly protocolId?: string;
  readonly protocolVersion?: number;
}

export function createLiveRulebenchTransport(
  options: RulebenchLiveTransportOptions = {},
): RulebenchLiveTransport {
  const apiBaseUrl = stripTrailingSlash(
    options.apiBaseUrl ?? DEFAULT_API_BASE_URL,
  );
  const fetchRequest = options.fetch ?? globalThis.fetch.bind(globalThis);
  const protocolId = options.protocolId ?? RULEBENCH_PROTOCOL_ID;
  const protocolVersion = options.protocolVersion ?? RULEBENCH_PROTOCOL_VERSION;
  const activeRequests = new Set<AbortController>();
  let connectionState: RulebenchLiveConnectionState = { kind: "idle" };

  const request = async <T>(
    method: "GET" | "POST" | "DELETE",
    path: string,
    body: object | undefined,
    requestOptions: RulebenchLiveRequestOptions | undefined,
  ): Promise<RulebenchLiveTransportResult<T>> => {
    const controller = new AbortController();
    const externalSignal = requestOptions?.signal;
    const abortFromExternalSignal = (): void => controller.abort();
    if (externalSignal !== undefined) {
      if (externalSignal.aborted) {
        controller.abort();
      } else {
        externalSignal.addEventListener("abort", abortFromExternalSignal, {
          once: true,
        });
      }
    }
    activeRequests.add(controller);

    const headers: Record<string, string> = {
      "x-rulebench-protocol-version": String(protocolVersion),
    };
    const init: RequestInit = {
      method,
      headers,
      signal: controller.signal,
    };
    if (body !== undefined) {
      headers["content-type"] = "application/json";
      init.body = JSON.stringify(body);
    }

    try {
      const response = await fetchRequest(`${apiBaseUrl}${path}`, init);
      if (!response.ok) {
        const decodedError = await decodeJsonResponse<unknown>(response);
        if (!decodedError.ok) {
          return decodedError;
        }
        return isLiveTransportError(decodedError.value)
          ? { ok: false, error: decodedError.value }
          : {
              ok: false,
              error: transportError(
                "serialization",
                "invalidErrorResponse",
                `Rulebench host returned HTTP ${response.status} without a valid protocol error.`,
                false,
              ),
            };
      }
      return decodeJsonResponse<T>(response);
    } catch (error: unknown) {
      const aborted = controller.signal.aborted;
      const transportFailure = aborted
        ? transportError(
            "cancellation",
            "requestAborted",
            "Rulebench host request was cancelled.",
            false,
          )
        : transportError(
            "network",
            "requestFailed",
            error instanceof Error
              ? error.message
              : "Rulebench host request failed.",
            true,
          );
      if (!aborted) {
        connectionState = { kind: "disconnected", error: transportFailure };
      }
      return { ok: false, error: transportFailure };
    } finally {
      activeRequests.delete(controller);
      externalSignal?.removeEventListener("abort", abortFromExternalSignal);
    }
  };

  const sessionPath = (sessionId: string): string =>
    `/sessions/${encodeURIComponent(sessionId)}`;
  const replayPath = (packageId: string): string =>
    `/replays/${encodeURIComponent(packageId)}`;
  const replayRequest = async <T>(
    method: "GET" | "POST",
    path: string,
    body?: object,
  ): Promise<ReplayReviewResult<T>> => {
    const result = await request<T>(method, path, body, undefined);
    return result.ok
      ? result
      : { ok: false, error: replayArchiveError(result.error) };
  };

  return {
    connectionState: () => connectionState,
    connect: async (requestOptions) => {
      connectionState = { kind: "connecting" };
      const result = await request<RulebenchProtocolHandshakeDto>(
        "GET",
        "/handshake",
        undefined,
        requestOptions,
      );
      if (!result.ok) {
        connectionState = { kind: "disconnected", error: result.error };
        return result;
      }
      if (
        result.value.protocolId !== protocolId ||
        result.value.protocolVersion !== protocolVersion
      ) {
        const error = transportError(
          "protocol",
          "handshakeMismatch",
          `Expected ${protocolId} v${protocolVersion}; received ${result.value.protocolId} v${result.value.protocolVersion}.`,
          false,
        );
        connectionState = { kind: "disconnected", error };
        return { ok: false, error };
      }
      connectionState = { kind: "connected", handshake: result.value };
      return result;
    },
    disconnect: () => {
      for (const controller of activeRequests) {
        controller.abort();
      }
      activeRequests.clear();
      connectionState = { kind: "disconnected", error: null };
    },
    listScenarios: (requestOptions) =>
      request("GET", "/scenarios", undefined, requestOptions),
    listSessions: (requestOptions) =>
      request("GET", "/sessions", undefined, requestOptions),
    createSession: (createRequest, requestOptions) =>
      request("POST", "/sessions", createRequest, requestOptions),
    getSession: (sessionId, requestOptions) =>
      request("GET", sessionPath(sessionId), undefined, requestOptions),
    closeSession: (sessionId, requestOptions) =>
      request("DELETE", sessionPath(sessionId), undefined, requestOptions),
    getCurrentActorOptions: (sessionId, requestOptions) =>
      request(
        "GET",
        `${sessionPath(sessionId)}/options`,
        undefined,
        requestOptions,
      ),
    preflightIntent: (sessionId, intent, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/preflight`,
        intent,
        requestOptions,
      ),
    listCandidates: (sessionId, requestOptions) =>
      request(
        "GET",
        `${sessionPath(sessionId)}/candidates`,
        undefined,
        requestOptions,
      ),
    submitIntent: (sessionId, command, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/intents`,
        command,
        requestOptions,
      ),
    submitControl: (sessionId, command, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/controls`,
        command,
        requestOptions,
      ),
    submitReaction: (sessionId, command, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/reactions`,
        command,
        requestOptions,
      ),
    runAutomaticStep: (sessionId, automaticRequest, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/automatic-step`,
        automaticRequest,
        requestOptions,
      ),
    runAutomaticCombat: (sessionId, automaticRequest, requestOptions) =>
      request(
        "POST",
        `${sessionPath(sessionId)}/automatic-run`,
        automaticRequest,
        requestOptions,
      ),
    listReplayPackages: () =>
      replayRequest<readonly RulebenchReplayArchiveMetadataDto[]>(
        "GET",
        "/replays",
      ),
    loadReplayPackage: (packageId) =>
      replayRequest<RulebenchReplayPackageReviewDto>(
        "GET",
        replayPath(packageId),
      ),
    loadReplayVerification: (packageId) =>
      replayRequest<RulebenchReplayVerificationReadoutDto>(
        "POST",
        `${replayPath(packageId)}/verify`,
      ),
    compareReplayPackages: (expectedPackageId, actualPackageId) =>
      replayRequest<RulebenchReplayComparisonReadoutDto>(
        "POST",
        "/replays/compare",
        { expectedPackageId, actualPackageId },
      ),
  };
}

function replayArchiveError(
  error: RulebenchLiveTransportErrorDto,
): RulebenchReplayArchiveErrorDto {
  const kind: RulebenchReplayArchiveErrorDto["kind"] =
    error.code === "unknownReplayPackage"
      ? "notFound"
      : error.code === "corruptReplayPackage"
        ? "corrupt"
        : error.code === "unsupportedReplayPackageVersion"
          ? "unsupportedVersion"
          : error.code === "invalidReplayPackage" ||
              error.code === "replayCombatNotFinalized"
            ? "invalidPackage"
            : "storage";
  return { kind, code: error.code, message: error.message, retryable: error.retryable };
}

async function decodeJsonResponse<T>(
  response: Response,
): Promise<RulebenchLiveTransportResult<T>> {
  try {
    const value: T = await response.json();
    return { ok: true, value };
  } catch {
    return {
      ok: false,
      error: transportError(
        "serialization",
        "invalidJsonResponse",
        `Rulebench host returned invalid JSON with HTTP ${response.status}.`,
        false,
      ),
    };
  }
}

function isLiveTransportError(
  value: unknown,
): value is RulebenchLiveTransportErrorDto {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  return (
    "kind" in value &&
    typeof value.kind === "string" &&
    "code" in value &&
    typeof value.code === "string" &&
    "message" in value &&
    typeof value.message === "string" &&
    "retryable" in value &&
    typeof value.retryable === "boolean"
  );
}

function transportError(
  kind: string,
  code: string,
  message: string,
  retryable: boolean,
): RulebenchLiveTransportErrorDto {
  return { kind, code, message, retryable };
}

function stripTrailingSlash(value: string): string {
  return value.endsWith("/") ? value.slice(0, -1) : value;
}
