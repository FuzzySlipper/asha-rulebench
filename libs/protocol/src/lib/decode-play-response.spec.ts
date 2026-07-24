import { describe, expect, it } from 'vitest';

import {
  decodeScenarioDocument,
  decodePlayWorkspaceResponse,
  PlayProtocolDecodeError,
} from './decode-play-response.js';

const emptyResponse = {
  ok: true,
  status: 'noActivePlayBundle',
  activeArtifact: null,
  candidateArtifact: null,
  upgradeImpact: null,
  activationRevision: 0,
  hostRandomSource: {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.system',
    sourceVersion: 1,
  },
  supportedRandomSources: [
    {
      policyId: 'random.automatic',
      policyVersion: 1,
      sourceId: 'random.system',
      sourceVersion: 1,
    },
  ],
  scenarioSetupRequired: false,
  gameplayAvailable: false,
  gameplay: null,
  diagnostics: [],
};

describe('play protocol decoder', () => {
  it('accepts the generated empty lifecycle response', () => {
    expect(decodePlayWorkspaceResponse(emptyResponse)).toEqual(emptyResponse);
  });

  it('fails closed for unknown protocol fields and unsafe revision values', () => {
    expect(() =>
      decodePlayWorkspaceResponse({
        ...emptyResponse,
        hiddenRuntimeState: {},
      }),
    ).toThrow(PlayProtocolDecodeError);
    expect(() =>
      decodePlayWorkspaceResponse({
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
          code: 'CONTENT_DEFINITION_REFERENCE_MISSING',
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

    expect(decodePlayWorkspaceResponse(response)).toEqual(response);
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

    expect(decodePlayWorkspaceResponse(response)).toEqual(response);
  });

  it('strictly decodes the portable checkpoint and replay archive', () => {
    const response = {
      ...emptyResponse,
      status: 'active',
      gameplayAvailable: true,
      gameplay: {
        artifactId: 'artifact-1',
        actorId: 'hero',
        stateRevision: 0,
        acceptedRandomValues: '0',
        randomSource: {
          policyId: 'random.automatic',
          policyVersion: 1,
          sourceId: 'random.system',
          sourceVersion: 1,
        },
        board: { width: 5, height: 3, cells: [] },
        turn: {
          initiativeOrder: ['hero', 'raider'],
          currentActorId: 'hero',
          round: 1,
          turn: 1,
        },
        actions: [],
        controls: [],
        entities: [],
        pendingReaction: null,
        log: [],
        outcome: { status: 'inProgress', winningTeamIds: [] },
        lastResult: {
          status: 'accepted',
          code: null,
          message: 'Accepted action.one at state revision 1',
          events: [],
          trace: [],
          randomConsumed: '1',
          randomEvidence: [
            {
              kind: 'formulaDice',
              count: 1,
              sides: 6,
              path: '$.damage',
              values: [4],
            },
          ],
          stateRevision: 1,
          randomRequest: null,
        },
        archive: {
          checkpointSchema: 'asha.rpg.session.checkpoint@1',
          replaySchemaVersion: 1,
          eventSchemaVersion: 1,
          artifactId: 'artifact-1',
          artifactSchema: 'asha.rpg.play-bundle.compiled@1',
          playBundle: 'rules@1.0.0',
          ruleset: 'asha.d20@1.0.0',
          operationSchemas: ['operation.damage@1'],
          capabilitySchemas: ['capability.vitality@1'],
          contentPacks: ['rules@1.0.0 · source'],
          dependencyLock: [],
          fingerprints: {
            source: 'source',
            semantic: 'semantic',
            presentation: 'presentation',
          },
          definitionFingerprints: ['action.one · definition'],
          stateRevision: '0',
          acceptedRandomPosition: '0',
          phase: 'ready',
          stateHash: 'fnv1a64.rpg-session.v1:state',
          checkpointBytes: 2048,
          replayEntries: [
            {
              sequence: 1,
              operation: 'submit action.one',
              outcome: 'accepted',
              before: {
                revision: '0',
                acceptedRandomPosition: '0',
                phase: 'ready',
                stateHash: 'before',
              },
              after: {
                revision: '1',
                acceptedRandomPosition: '1',
                phase: 'ready',
                stateHash: 'after',
              },
              randomEvidence: ['formulaDice 1d6 at $.damage = 4'],
              events: ['4 force damage to raider'],
            },
          ],
          verificationStatus: 'verified',
          verificationMessage: 'Rust replay verified 1 record',
        },
      },
    };

    expect(decodePlayWorkspaceResponse(response)).toEqual(response);
    expect(() =>
      decodePlayWorkspaceResponse({
        ...response,
        gameplay: { ...response.gameplay, acceptedRandomValues: 0 },
      }),
    ).toThrow('$.gameplay.acceptedRandomValues');
    expect(() =>
      decodePlayWorkspaceResponse({
        ...response,
        gameplay: { ...response.gameplay, acceptedRandomValues: '00' },
      }),
    ).toThrow('$.gameplay.acceptedRandomValues');
  });

  it('strictly decodes an explicit Scenario document', () => {
    const setup = {
      schema: { id: 'asha.rpg.scenario', version: 1 },
      playBundleId: 'artifact-1',
      board: {
        width: 3,
        height: 2,
        cells: [
          {
            id: 'cover',
            position: { x: 1, y: 1 },
            capabilities: [
              {
                id: 'capability.traversal',
                version: 1,
                definitionId: null,
                value: {
                  kind: 'traversal',
                  passable: false,
                  movementCost: 2,
                },
              },
            ],
          },
        ],
      },
      participants: [
        {
          id: 'hero',
          label: 'Hero',
          teamId: 'allies',
          position: { x: 0, y: 0 },
          definitionIds: ['action.one'],
          items: [{ id: 'sword-1', definitionId: 'item.sword' }],
          equipment: [{ slotId: 'main-hand', itemInstanceId: 'sword-1' }],
          capabilities: [
            { owner: 'vitality', value: { current: 10, max: 10 } },
            { owner: 'stat', id: 'power', value: 3 },
            {
              owner: 'modifier',
              stackingGroup: 'stance',
              id: 'braced',
              value: 1,
              remainingTurns: 2,
            },
          ],
        },
      ],
      turn: {
        initiativeOrder: ['hero'],
        currentActorId: 'hero',
        round: 1,
        turn: 1,
      },
      randomSource: emptyResponse.hostRandomSource,
    };

    expect(decodeScenarioDocument(setup)).toEqual(setup);
    expect(() =>
      decodeScenarioDocument({ ...setup, expectedEvents: [] }),
    ).toThrow('$.expectedEvents: unknown field');
  });
});
