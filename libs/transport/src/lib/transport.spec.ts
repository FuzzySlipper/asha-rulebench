import type { PlayWorkspaceResponseDto } from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { createPlayTransport, type JsonHttpClient } from './transport.js';

describe('play transport', () => {
  it('keeps Ruleset discovery, PlayBundle activation, Scenario, and Session routes distinct', async () => {
    const requests: {
      readonly method: 'GET' | 'POST';
      readonly path: string;
      readonly body: unknown;
    }[] = [];
    const http: JsonHttpClient = {
      request: async (method, path, body) => {
        requests.push({ method, path, body });
        if (path === '/api/rulesets/config') {
          return { schemaVersion: 1, rulesets: [] };
        }
        if (path === '/api/rulesets/inspect') {
          return { ok: true, catalog: null, diagnostics: [] };
        }
        return emptyResponse();
      },
    };
    const transport = createPlayTransport(http);
    const root = 'test-fixtures/rulesets/minimal';

    await transport.rulesetLocations();
    await transport.inspectRuleset({ rulesetRoot: root });
    await transport.compile({
      rulesetRoot: root,
      contentPackIds: ['rulebench.minimal.content'],
    });
    await transport.activatePlayBundle();
    await transport.startScenario({
      schema: { id: 'asha.rpg.scenario', version: 1 },
      playBundleId: 'artifact-1',
      board: { width: 5, height: 3, cells: [] },
      participants: [],
      turn: {
        initiativeOrder: [],
        currentActorId: '',
        round: 1,
        turn: 1,
      },
      randomSource: automaticRandomSource(),
    });
    await transport.command({
      expectedRevision: 2,
      actionId: 'action.arc-lash',
      actorId: 'hero',
      targetIds: ['raider'],
    });

    expect(requests.map(({ method, path }) => `${method} ${path}`)).toEqual([
      'GET /api/rulesets/config',
      'POST /api/rulesets/inspect',
      'POST /api/play-bundle/compile',
      'POST /api/play-bundle/activate',
      'POST /api/scenario/start',
      'POST /api/session/command',
    ]);
    expect(requests.at(-1)?.body).toEqual({
      expectedRevision: 2,
      actionId: 'action.arc-lash',
      actorId: 'hero',
      targetIds: ['raider'],
    });
  });

  it('strictly decodes configured Ruleset locations without a product default', async () => {
    const http: JsonHttpClient = {
      request: async () => ({
        schemaVersion: 1,
        rulesets: [
          {
            id: 'd20-fantasy',
            label: 'd20 Fantasy',
            rulesetRoot: '/rules/rulesets/d20-fantasy',
          },
        ],
      }),
    };

    await expect(createPlayTransport(http).rulesetLocations()).resolves.toEqual(
      {
        schemaVersion: 1,
        rulesets: [
          {
            id: 'd20-fantasy',
            label: 'd20 Fantasy',
            rulesetRoot: '/rules/rulesets/d20-fantasy',
          },
        ],
      },
    );
  });
});

function automaticRandomSource() {
  return {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.system',
    sourceVersion: 1,
  };
}

function emptyResponse(): PlayWorkspaceResponseDto {
  return {
    ok: true,
    status: 'noActivePlayBundle',
    activeArtifact: null,
    candidateArtifact: null,
    upgradeImpact: null,
    activationRevision: 0,
    hostRandomSource: automaticRandomSource(),
    supportedRandomSources: [automaticRandomSource()],
    scenarioSetupRequired: false,
    gameplayAvailable: false,
    gameplay: null,
    diagnostics: [],
  };
}
