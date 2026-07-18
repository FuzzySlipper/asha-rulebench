import type { RulesetWorkspaceResponseDto } from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { createRulesetTransport, type JsonHttpClient } from './transport.js';

describe('ruleset transport', () => {
  it('sends freshly prepared source through the generated compile request shape', async () => {
    const requests: {
      readonly method: 'GET' | 'POST';
      readonly path: string;
      readonly body: unknown;
    }[] = [];
    const http: JsonHttpClient = {
      request: async (method, path, body) => {
        requests.push({ method, path, body });
        return emptyResponse();
      },
    };

    const transport = createRulesetTransport(http);
    await transport.compile('fresh');

    expect(requests).toEqual([
      {
        method: 'POST',
        path: '/api/ruleset/compile',
        body: {
          sourceId: 'fresh',
        },
      },
    ]);
  });
});

function emptyResponse(): RulesetWorkspaceResponseDto {
  return {
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
}
