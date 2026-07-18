import { describe, expect, it } from 'vitest';

import {
  decodeRulesetWorkspaceResponse,
  RulesetProtocolDecodeError,
} from './decode-ruleset-response.js';

const emptyResponse = {
  ok: true,
  status: 'noActiveRuleset',
  activeArtifact: null,
  candidateArtifact: null,
  upgradeImpact: null,
  activationRevision: 0,
  gameplayAvailable: false,
  gameplay: null,
  diagnostics: [],
};

describe('ruleset protocol decoder', () => {
  it('accepts the generated empty lifecycle response', () => {
    expect(decodeRulesetWorkspaceResponse(emptyResponse)).toEqual(
      emptyResponse,
    );
  });

  it('fails closed for unknown protocol fields and unsafe revision values', () => {
    expect(() =>
      decodeRulesetWorkspaceResponse({
        ...emptyResponse,
        hiddenRuntimeState: {},
      }),
    ).toThrow(RulesetProtocolDecodeError);
    expect(() =>
      decodeRulesetWorkspaceResponse({
        ...emptyResponse,
        activationRevision: -1,
      }),
    ).toThrow('$.activationRevision');
  });

  it('retains typed source context on compiler diagnostics', () => {
    const response = {
      ...emptyResponse,
      ok: false,
      diagnostics: [
        {
          stage: 'graph',
          severity: 'error',
          code: 'RULESET_DEFINITION_REFERENCE_MISSING',
          path: '$.definitions[0].references[0]',
          message: 'missing support',
          packageId: 'rulebench.field-manual',
          definitionId: 'rulebench.signal-flare',
          source: {
            module: 'packages/rulebench-field-manual.ts',
            declaration: 'signalFlare',
          },
          graphPath: ['rulebench.field-manual', 'catalog.damage.missing'],
          expected: 'exported support definition',
          actual: 'missing',
        },
      ],
    };

    expect(decodeRulesetWorkspaceResponse(response)).toEqual(response);
  });

  it('decodes an exact pre-activation upgrade impact report', () => {
    const response = {
      ...emptyResponse,
      upgradeImpact: {
        fromArtifactId: 'artifact-1.0',
        toArtifactId: 'artifact-1.1',
        sourceChanges: ['field-manual 1.0.0 → 1.1.0'],
        definitions: [
          {
            definitionId: 'rulebench.arc-lash-stormfront',
            change: 'changed',
            descendant: true,
            causes: ['primary base identity or fingerprint changed'],
            fields: [
              {
                plane: 'semantic',
                path: '$.semantic.program.hit.amount.right.value',
                before: '1',
                after: '2',
              },
            ],
          },
        ],
      },
    };

    expect(decodeRulesetWorkspaceResponse(response)).toEqual(response);
  });
});
