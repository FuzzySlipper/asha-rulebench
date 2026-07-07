import type {
  Result,
  RulebenchScenarioCatalogDto,
  RulebenchScenarioCatalogSummaryDto,
  RulebenchScenarioReadoutDto,
} from '@asha-rulebench/protocol';
import { rustBackedScenarioCatalog } from './generated/rust-scenario-catalog';

export interface RulebenchTransport {
  readonly loadCatalog: () => Promise<Result<readonly RulebenchScenarioCatalogSummaryDto[]>>;
  readonly loadScenario: (scenarioId?: string) => Promise<Result<RulebenchScenarioReadoutDto>>;
}

export const defaultScenarioCatalog: RulebenchScenarioCatalogDto = rustBackedScenarioCatalog;
export const defaultScenarioId: string = firstScenarioId(defaultScenarioCatalog);
export const defaultScenarioReadout: RulebenchScenarioReadoutDto = requireScenarioReadout(
  defaultScenarioCatalog,
  defaultScenarioId,
);

export const createFakeRulebenchTransport = (
  catalog: RulebenchScenarioCatalogDto = defaultScenarioCatalog,
): RulebenchTransport => ({
  loadCatalog: async () => ({ ok: true, value: catalog.summaries }),
  loadScenario: async (scenarioId: string = firstScenarioId(catalog)) => {
    const readout = scenarioReadout(catalog, scenarioId);
    return readout === null
      ? {
          ok: false,
          error: {
            kind: 'not-found',
            message: `Scenario not found: ${scenarioId}`,
            retryable: false,
          },
        }
      : { ok: true, value: readout };
  },
});

function firstScenarioId(catalog: RulebenchScenarioCatalogDto): string {
  const firstSummary = catalog.summaries[0];
  return firstSummary?.id ?? '';
}

function requireScenarioReadout(catalog: RulebenchScenarioCatalogDto, scenarioId: string): RulebenchScenarioReadoutDto {
  const readout = scenarioReadout(catalog, scenarioId);
  if (readout === null) {
    throw new Error(`Default scenario readout is missing: ${scenarioId}`);
  }
  return readout;
}

function scenarioReadout(catalog: RulebenchScenarioCatalogDto, scenarioId: string): RulebenchScenarioReadoutDto | null {
  return catalog.readouts.find((readout) => readout.id === scenarioId) ?? null;
}
