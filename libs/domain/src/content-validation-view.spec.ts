import { describe, expect, it } from 'vitest';
import type { RulebenchContentValidationReadoutDto } from '@asha-rulebench/protocol';
import { projectContentValidationReadout } from './index';

describe('projectContentValidationReadout', () => {
  it('preserves Rust validation identity without deriving semantics', () => {
    const readout: RulebenchContentValidationReadoutDto = {
      scenarioId: 'scenario.invalid',
      scenarioTitle: 'Invalid scenario',
      report: {
        accepted: false,
        errorCount: 1,
        warningCount: 0,
        diagnostics: [{ severity: 'error', code: 'missingAction', contentId: 'action.missing', message: 'Action is missing.' }],
      },
    };

    expect(projectContentValidationReadout(readout)).toEqual({
      scenarioId: 'scenario.invalid', scenarioTitle: 'Invalid scenario', statusLabel: 'Rejected', errorCount: 1, warningCount: 0,
      diagnostics: [{ severityLabel: 'Error', code: 'missingAction', sourceLabel: 'action.missing', message: 'Action is missing.' }],
    });
  });
});
