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
        spatialSources: [],
        pendingReaction: null,
        pendingForcedMovement: null,
        pendingTurnSave: null,
        log: [
          {
            sequence: '1',
            stateRevision: '1',
            actorId: 'hero',
            actionId: 'action.one',
            itemBinding: null,
            events: [
              {
                kind: 'attackResolved',
                summary:
                  'hero rolled 15 for 23 against raider guard 17; hit=true',
                roll: {
                  kind: 'attack',
                  dieResult: 15,
                  total: 23,
                  thresholdLabel: 'guard',
                  threshold: 17,
                  outcome: 'hit',
                  contributions: [
                    {
                      sourceDefinitionId: 'action.one',
                      sourceInstanceId: null,
                      sourceLabel: 'Attack',
                      amount: 5,
                      reasonKind: 'scalarContribution',
                      contributionId: 'action-check-modifier',
                      selector: 'attack',
                      stackingGroup: 'untyped',
                      disposition: 'applied',
                    },
                    {
                      sourceDefinitionId: 'feature.flanker',
                      sourceInstanceId: null,
                      sourceLabel: 'Flanker',
                      amount: 2,
                      reasonKind: 'scalarContribution',
                      contributionId: 'flanking',
                      selector: 'attack',
                      stackingGroup: 'circumstance',
                      disposition: 'applied',
                    },
                    {
                      sourceDefinitionId: 'feature.surrounded',
                      sourceInstanceId: null,
                      sourceLabel: 'Surrounded',
                      amount: 0,
                      reasonKind: 'scalarContribution',
                      contributionId: 'surrounded',
                      selector: 'attack',
                      stackingGroup: 'circumstance',
                      disposition: 'suppressed by greatest; retained flanking',
                    },
                  ],
                },
                contributions: [],
                details: [
                  { label: 'base', value: '5' },
                  { label: 'final', value: '7' },
                ],
              },
            ],
          },
        ],
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
              heterogeneousValues: [],
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
    const extendedResponse = {
      ...response,
      gameplay: {
        ...response.gameplay,
        spatialSources: [
          {
            instanceId: 'zone-1',
            definitionId: 'spatial.crosswind',
            label: 'Crosswind',
            description: 'A shifting tactical zone.',
            ownerEntityId: 'hero',
            sourceEntityId: 'hero',
            originX: 1,
            originY: 1,
            includedCellIds: ['cell-1-1', 'cell-2-1'],
            radius: 1,
            targetFilter: 'hostiles',
            stacking: 'independentBySource',
            tenure: 'fixed',
            durationAnchor: 'sourceTurnStart',
            remainingCount: 2,
            triggerBoundaries: ['enter', 'exit'],
            triggerEvidence: [
              {
                sequence: '2',
                stateRevision: '1',
                boundary: 'enter',
                cellId: 'cell-2-1',
                participantId: 'raider',
                operationPath: '$.movement',
                disposition: 'applied',
              },
            ],
          },
        ],
        pendingForcedMovement: {
          movementKind: 'push',
          sourceId: 'hero',
          movedParticipantId: 'raider',
          maximumDistance: 2,
          operationPath: '$.program[0]',
          options: [
            {
              sessionBindingId: 'session-1',
              artifactId: 'artifact-1',
              scenarioFingerprintAlgorithm: 'fnv1a64',
              scenarioFingerprintValue: 'scenario-1',
              authorityRevision: 0,
              round: '1',
              turn: '1',
              currentActorId: 'hero',
              actionId: 'action.shove',
              sourceId: 'hero',
              movedParticipantId: 'raider',
              operationPath: '$.program[0]',
              destinationCellId: 'cell-2-1',
              cellIds: ['cell-1-1', 'cell-2-1'],
              movementCost: 1,
            },
          ],
        },
      },
    };
    expect(decodePlayWorkspaceResponse(extendedResponse)).toEqual(
      extendedResponse,
    );
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
      schema: { id: 'asha.rpg.scenario', version: 3 },
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
          classDefinitionId: 'class.fighter',
          featureDefinitionIds: ['feature.flanker'],
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
