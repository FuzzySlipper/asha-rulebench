import type { Result, RulebenchScenarioReadoutDto } from '@asha-rulebench/protocol';
import { rustBackedScenarioReadout } from './generated/rust-scenario-readout';

export interface RulebenchTransport {
  readonly loadScenario: () => Promise<Result<RulebenchScenarioReadoutDto>>;
}

export const defaultScenarioReadout: RulebenchScenarioReadoutDto = rustBackedScenarioReadout;

export const createFakeRulebenchTransport = (
  scenario: RulebenchScenarioReadoutDto = defaultScenarioReadout,
): RulebenchTransport => ({
  loadScenario: async () => ({ ok: true, value: scenario }),
});
