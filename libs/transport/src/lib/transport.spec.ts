import type { RulesetWorkspaceResponseDto } from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { createRulesetTransport, type JsonHttpClient } from './transport.js';

describe('ruleset transport', () => {
  it('sends intents and reactions without browser-authored random evidence', async () => {
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
      rulesetRoot: 'examples/rulesets/field-manual',
    });
    await transport.command({
      expectedRevision: 2,
      actionId: 'action.arc-lash',
      actorId: 'hero',
      targetIds: ['raider'],
    });
    await transport.react({
      expectedRevision: 2,
      reactionId: 'reaction.raise-ward',
      optionId: 'raise-ward',
    });
    await transport.restoreCheckpoint();
    await transport.replay();

    expect(requests).toEqual([
      {
        method: 'POST',
        path: '/api/ruleset/compile',
        body: {
          rulesetRoot: 'examples/rulesets/field-manual',
        },
      },
      {
        method: 'POST',
        path: '/api/ruleset/command',
        body: {
          expectedRevision: 2,
          actionId: 'action.arc-lash',
          actorId: 'hero',
          targetIds: ['raider'],
        },
      },
      {
        method: 'POST',
        path: '/api/ruleset/reaction',
        body: {
          expectedRevision: 2,
          reactionId: 'reaction.raise-ward',
          optionId: 'raise-ward',
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
