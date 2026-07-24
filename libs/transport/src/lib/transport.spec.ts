import type {
  PlayBundleSourceSetDto,
  PlayWorkspaceResponseDto,
} from '@asha-rulebench/protocol';
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
        if (path === '/api/play-bundle/source-sets') {
          return { schemaVersion: 2, sourceSets: [] };
        }
        if (path === '/api/rulesets/inspect') {
          return { ok: true, catalog: null, diagnostics: [] };
        }
        return emptyResponse();
      },
    };
    const transport = createPlayTransport(http);
    const root = 'test-fixtures/rulesets/minimal';
    const sources = sourceSet(root);

    await transport.sourceSets();
    await transport.inspectRuleset({ sourceSet: sources });
    await transport.compile({
      sourceSet: sources,
      contentPackIds: ['rulebench.minimal.content'],
    });
    await transport.activatePlayBundle();
    await transport.startScenario({
      schema: { id: 'asha.rpg.scenario', version: 2 },
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
      itemBinding: null,
    });

    expect(requests.map(({ method, path }) => `${method} ${path}`)).toEqual([
      'GET /api/play-bundle/source-sets',
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
      itemBinding: null,
    });
  });

  it('strictly decodes configured Ruleset locations without a product default', async () => {
    const http: JsonHttpClient = {
      request: async () => ({
        schemaVersion: 2,
        sourceSets: [
          {
            id: 'd20-fantasy',
            label: 'd20 Fantasy',
            sourceSet: sourceSet('/rules/rulesets/d20-fantasy'),
          },
        ],
      }),
    };

    await expect(createPlayTransport(http).sourceSets()).resolves.toEqual({
      schemaVersion: 2,
      sourceSets: [
        {
          id: 'd20-fantasy',
          label: 'd20 Fantasy',
          sourceSet: sourceSet('/rules/rulesets/d20-fantasy'),
        },
      ],
    });
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

function sourceSet(sourceRoot: string): PlayBundleSourceSetDto {
  return {
    schemaVersion: 1,
    allowedRoots: [sourceRoot],
    entries: [
      {
        id: 'ruleset',
        label: 'Ruleset source',
        sourceRoot,
        module: 'src/index.ts',
        exportKinds: [
          'ruleset',
          'contentPack',
          'playBundle',
          'scenarioTemplate',
        ],
      },
    ],
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
