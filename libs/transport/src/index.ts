import type {
  Result,
  RulebenchAutomaticRunReadoutDto,
  RulebenchCombatControlHistoryReadoutDto,
  RulebenchCombatScriptReadoutDto,
  RulebenchCombatSessionCatalogDto,
  RulebenchCombatSessionStepReadoutDto,
  RulebenchCombatSessionSummaryDto,
  RulebenchContentValidationCatalogDto,
  RulebenchContentValidationReadoutDto,
  RulebenchRulesetCatalogDto,
  RulebenchScenarioCatalogDto,
  RulebenchScenarioCatalogSummaryDto,
  RulebenchScenarioReadoutDto,
} from "@asha-rulebench/protocol";
import { rustBackedCombatSessionCatalog } from "./generated/rust-combat-session";
import {
  rustBackedContentValidationCatalog,
  rustBackedRulesetCatalog,
  rustBackedScenarioCatalog,
} from "./generated/rust-scenario-catalog";

export interface RulebenchTransport {
  readonly loadRulesetCatalog: () => Promise<
    Result<RulebenchRulesetCatalogDto>
  >;
  readonly loadContentValidationReport: (
    scenarioId?: string,
  ) => Promise<Result<RulebenchContentValidationReadoutDto>>;
  readonly loadCatalog: () => Promise<
    Result<readonly RulebenchScenarioCatalogSummaryDto[]>
  >;
  readonly loadScenario: (
    scenarioId?: string,
  ) => Promise<Result<RulebenchScenarioReadoutDto>>;
  readonly loadSessionCatalog: () => Promise<
    Result<readonly RulebenchCombatSessionSummaryDto[]>
  >;
  readonly loadSessionStep: (
    sessionId?: string,
    stepId?: string,
  ) => Promise<Result<RulebenchCombatSessionStepReadoutDto>>;
  readonly loadSessionControlHistory: (
    sessionId?: string,
  ) => Promise<Result<RulebenchCombatControlHistoryReadoutDto>>;
  readonly loadSessionScriptReadout: (
    scriptId?: string,
  ) => Promise<Result<RulebenchCombatScriptReadoutDto>>;
  readonly loadSessionAutomaticRunReadout: (
    runId?: string,
  ) => Promise<Result<RulebenchAutomaticRunReadoutDto>>;
}

export const defaultRulesetCatalog: RulebenchRulesetCatalogDto =
  rustBackedRulesetCatalog;
export const defaultContentValidationCatalog: RulebenchContentValidationCatalogDto =
  rustBackedContentValidationCatalog;
export const defaultContentValidationScenarioId: string =
  firstContentValidationScenarioId(defaultContentValidationCatalog);
export const defaultContentValidationReport: RulebenchContentValidationReadoutDto =
  requireContentValidationReport(
    defaultContentValidationCatalog,
    defaultContentValidationScenarioId,
  );
export const defaultScenarioCatalog: RulebenchScenarioCatalogDto =
  rustBackedScenarioCatalog;
export const defaultScenarioId: string = firstScenarioId(
  defaultScenarioCatalog,
);
export const defaultScenarioReadout: RulebenchScenarioReadoutDto =
  requireScenarioReadout(defaultScenarioCatalog, defaultScenarioId);
export const defaultCombatSessionCatalog: RulebenchCombatSessionCatalogDto =
  rustBackedCombatSessionCatalog;
export const defaultCombatSessionId: string = firstSessionId(
  defaultCombatSessionCatalog,
);
export const defaultCombatSessionStepId: string = firstSessionStepId(
  defaultCombatSessionCatalog,
  defaultCombatSessionId,
);
export const defaultCombatSessionStepReadout: RulebenchCombatSessionStepReadoutDto =
  requireSessionStepReadout(
    defaultCombatSessionCatalog,
    defaultCombatSessionId,
    defaultCombatSessionStepId,
  );
export const defaultCombatControlHistoryId: string = firstControlHistoryId(
  defaultCombatSessionCatalog,
);
export const defaultCombatControlHistoryReadout: RulebenchCombatControlHistoryReadoutDto =
  requireSessionControlHistory(
    defaultCombatSessionCatalog,
    defaultCombatControlHistoryId,
  );
export const defaultCombatScriptReadoutId: string = firstScriptReadoutId(
  defaultCombatSessionCatalog,
);
export const defaultCombatScriptReadout: RulebenchCombatScriptReadoutDto =
  requireSessionScriptReadout(
    defaultCombatSessionCatalog,
    defaultCombatScriptReadoutId,
  );
export const defaultCombatAutomaticRunReadoutId: string =
  firstAutomaticRunReadoutId(defaultCombatSessionCatalog);
export const defaultCombatAutomaticRunReadout: RulebenchAutomaticRunReadoutDto =
  requireSessionAutomaticRunReadout(
    defaultCombatSessionCatalog,
    defaultCombatAutomaticRunReadoutId,
  );

export const createFakeRulebenchTransport = (
  catalog: RulebenchScenarioCatalogDto = defaultScenarioCatalog,
  sessionCatalog: RulebenchCombatSessionCatalogDto = defaultCombatSessionCatalog,
  rulesetCatalog: RulebenchRulesetCatalogDto = defaultRulesetCatalog,
  validationCatalog: RulebenchContentValidationCatalogDto = defaultContentValidationCatalog,
): RulebenchTransport => ({
  loadRulesetCatalog: async () => ({ ok: true, value: rulesetCatalog }),
  loadContentValidationReport: async (
    scenarioId: string = firstContentValidationScenarioId(validationCatalog),
  ) => {
    const report = contentValidationReport(validationCatalog, scenarioId);
    return report === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: `Content validation report not found: ${scenarioId}`,
            retryable: false,
          },
        }
      : { ok: true, value: report };
  },
  loadCatalog: async () => ({ ok: true, value: catalog.summaries }),
  loadScenario: async (scenarioId: string = firstScenarioId(catalog)) => {
    const readout = scenarioReadout(catalog, scenarioId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: `Scenario not found: ${scenarioId}`,
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
  loadSessionCatalog: async () => ({
    ok: true,
    value: sessionCatalog.summaries,
  }),
  loadSessionStep: async (
    sessionId: string = firstSessionId(sessionCatalog),
    stepId: string = firstSessionStepId(sessionCatalog, sessionId),
  ) => {
    const readout = sessionStepReadout(sessionCatalog, sessionId, stepId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: sessionMissingMessage(sessionCatalog, sessionId, stepId),
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
  loadSessionControlHistory: async (
    sessionId: string = firstControlHistoryId(sessionCatalog),
  ) => {
    const readout = sessionControlHistory(sessionCatalog, sessionId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: `Combat control history not found: ${sessionId}`,
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
  loadSessionScriptReadout: async (
    scriptId: string = firstScriptReadoutId(sessionCatalog),
  ) => {
    const readout = sessionScriptReadout(sessionCatalog, scriptId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: `Combat script readout not found: ${scriptId}`,
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
  loadSessionAutomaticRunReadout: async (
    runId: string = firstAutomaticRunReadoutId(sessionCatalog),
  ) => {
    const readout = sessionAutomaticRunReadout(sessionCatalog, runId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: "not-found",
            message: `Combat automatic run readout not found: ${runId}`,
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
});

function firstContentValidationScenarioId(
  catalog: RulebenchContentValidationCatalogDto,
): string {
  const firstReport = catalog.reports[0];
  return firstReport?.scenarioId ?? "";
}

function requireContentValidationReport(
  catalog: RulebenchContentValidationCatalogDto,
  scenarioId: string,
): RulebenchContentValidationReadoutDto {
  const report = contentValidationReport(catalog, scenarioId);
  if (report === null) {
    throw new Error(
      `Default content validation report is missing: ${scenarioId}`,
    );
  }
  return report;
}

function contentValidationReport(
  catalog: RulebenchContentValidationCatalogDto,
  scenarioId: string,
): RulebenchContentValidationReadoutDto | null {
  return (
    catalog.reports.find((report) => report.scenarioId === scenarioId) ?? null
  );
}

function firstScenarioId(catalog: RulebenchScenarioCatalogDto): string {
  const firstSummary = catalog.summaries[0];
  return firstSummary?.id ?? "";
}

function requireScenarioReadout(
  catalog: RulebenchScenarioCatalogDto,
  scenarioId: string,
): RulebenchScenarioReadoutDto {
  const readout = scenarioReadout(catalog, scenarioId);
  if (readout === null) {
    throw new Error(`Default scenario readout is missing: ${scenarioId}`);
  }
  return readout;
}

function scenarioReadout(
  catalog: RulebenchScenarioCatalogDto,
  scenarioId: string,
): RulebenchScenarioReadoutDto | null {
  return catalog.readouts.find((readout) => readout.id === scenarioId) ?? null;
}

function firstSessionId(catalog: RulebenchCombatSessionCatalogDto): string {
  const firstSummary = catalog.summaries[0];
  return firstSummary?.id ?? "";
}

function firstSessionStepId(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
): string {
  const summary = catalog.summaries.find(
    (candidate) => candidate.id === sessionId,
  );
  const firstStep = summary?.steps[0];
  return firstStep?.id ?? "";
}

function requireSessionStepReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
  stepId: string,
): RulebenchCombatSessionStepReadoutDto {
  const readout = sessionStepReadout(catalog, sessionId, stepId);
  if (readout === null) {
    throw new Error(
      `Default combat session step readout is missing: ${sessionId} / ${stepId}`,
    );
  }
  return readout;
}

function firstControlHistoryId(
  catalog: RulebenchCombatSessionCatalogDto,
): string {
  const firstReadout = catalog.controlHistoryReadouts[0];
  return firstReadout?.sessionId ?? "";
}

function requireSessionControlHistory(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
): RulebenchCombatControlHistoryReadoutDto {
  const readout = sessionControlHistory(catalog, sessionId);
  if (readout === null) {
    throw new Error(
      `Default combat control history readout is missing: ${sessionId}`,
    );
  }
  return readout;
}

function sessionControlHistory(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
): RulebenchCombatControlHistoryReadoutDto | null {
  return (
    catalog.controlHistoryReadouts.find(
      (readout) => readout.sessionId === sessionId,
    ) ?? null
  );
}

function firstScriptReadoutId(
  catalog: RulebenchCombatSessionCatalogDto,
): string {
  const firstReadout = catalog.scriptReadouts[0];
  return firstReadout?.scriptId ?? "";
}

function requireSessionScriptReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  scriptId: string,
): RulebenchCombatScriptReadoutDto {
  const readout = sessionScriptReadout(catalog, scriptId);
  if (readout === null) {
    throw new Error(`Default combat script readout is missing: ${scriptId}`);
  }
  return readout;
}

function sessionScriptReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  scriptId: string,
): RulebenchCombatScriptReadoutDto | null {
  return (
    catalog.scriptReadouts.find((readout) => readout.scriptId === scriptId) ??
    null
  );
}

function firstAutomaticRunReadoutId(
  catalog: RulebenchCombatSessionCatalogDto,
): string {
  const firstReadout = catalog.automaticRunReadouts[0];
  return firstReadout?.id ?? "";
}

function requireSessionAutomaticRunReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  runId: string,
): RulebenchAutomaticRunReadoutDto {
  const readout = sessionAutomaticRunReadout(catalog, runId);
  if (readout === null) {
    throw new Error(
      `Default combat automatic run readout is missing: ${runId}`,
    );
  }
  return readout;
}

function sessionAutomaticRunReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  runId: string,
): RulebenchAutomaticRunReadoutDto | null {
  return (
    catalog.automaticRunReadouts.find((readout) => readout.id === runId) ?? null
  );
}

function sessionStepReadout(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
  stepId: string,
): RulebenchCombatSessionStepReadoutDto | null {
  return (
    catalog.readouts.find(
      (readout) =>
        readout.sessionId === sessionId && readout.step.id === stepId,
    ) ?? null
  );
}

function sessionMissingMessage(
  catalog: RulebenchCombatSessionCatalogDto,
  sessionId: string,
  stepId: string,
): string {
  const session = catalog.summaries.find((summary) => summary.id === sessionId);
  return session === undefined
    ? `Combat session not found: ${sessionId}`
    : `Combat session step not found: ${sessionId} / ${stepId}`;
}
