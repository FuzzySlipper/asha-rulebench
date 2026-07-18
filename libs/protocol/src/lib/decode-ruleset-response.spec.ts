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
  activationRevision: 0,
  gameplayAvailable: false,
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
});
