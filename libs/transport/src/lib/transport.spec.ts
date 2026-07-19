import type { RulesetWorkspaceResponseDto } from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { createRulesetTransport, type JsonHttpClient } from './transport.js';

describe('ruleset transport', () => {
  it('uses generated compile and portable archive routes', async () => {
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
    await transport.compile({
      workspaceRoot: 'examples/rulesets/field-manual-v1',
      packageRoots: ['.', '../shared'],
      module: 'src/ruleset.ts',
      declaration: 'ruleset',
    });
    await transport.restoreCheckpoint();
    await transport.replay();

    expect(requests).toEqual([
      {
        method: 'POST',
        path: '/api/ruleset/compile',
        body: {
          workspaceRoot: 'examples/rulesets/field-manual-v1',
          packageRoots: ['.', '../shared'],
          module: 'src/ruleset.ts',
          declaration: 'ruleset',
        },
      },
      {
        method: 'POST',
        path: '/api/ruleset/checkpoint/restore',
        body: undefined,
      },
      {
        method: 'POST',
        path: '/api/ruleset/replay',
        body: undefined,
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
