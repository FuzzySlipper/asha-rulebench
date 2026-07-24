import type {
  PlayBundleArtifactSummaryDto,
  PlayWorkspaceResponseDto,
} from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { playWorkspaceView } from './domain.js';

describe('play workspace view mapping', () => {
  it('keeps source, semantic, and presentation fingerprints visibly separate', () => {
    const view = playWorkspaceView(
      response('compiledCandidate', null, artifact()),
    );

    expect(view.phase).toBe('candidate');
    expect(view.activeArtifactId).toBeNull();
    expect(view.artifact?.fingerprints).toEqual([
      { plane: 'Source', value: 'fnv1a64:1111111111111111' },
      { plane: 'Semantic', value: 'fnv1a64:2222222222222222' },
      { plane: 'Presentation', value: 'fnv1a64:3333333333333333' },
    ]);
    expect(view.artifact?.exportedRoots).toEqual(['rulebench.signal-flare']);
    expect(view.artifact?.derivations[0]?.mixins[0]).toEqual({
      identity: 'rulebench.primitives@1.0.0#rulebench.double-range',
      fingerprint: 'fnv1a64:7777777777777777',
      parameters: 'factor=2',
      order: 0,
    });
    expect(view.artifact?.overlays[0]?.impact).toBe('semantic · reject');
    expect(view.artifact?.overlays[0]?.changes[0]?.transition).toBe('7 → 8');
  });

  it('reports active artifact identity with the persistent authority view', () => {
    const active = artifact();
    const activeResponse = response('active', active, null);
    activeResponse.scenarioSetupRequired = false;
    activeResponse.gameplayAvailable = true;
    activeResponse.gameplay = gameplay();
    const view = playWorkspaceView(activeResponse);

    expect(view.phase).toBe('active');
    expect(view.activeArtifactId).toBe(active.artifactId);
    expect(view.gameplayAvailable).toBe(true);
    expect(view.gameplay?.stateRevision).toBe(2);
    expect(view.gameplay?.actions[0]?.candidateIds).toEqual(['raider']);
    expect(view.gameplay?.actions[0]?.itemBinding?.itemInstanceId).toBe(
      'longsword-1',
    );
    expect(view.gameplay?.actions[0]?.identity).not.toBe(
      view.gameplay?.actions[1]?.identity,
    );
    expect(view.gameplay?.actions[0]?.id).toBe(view.gameplay?.actions[1]?.id);
    expect(view.gameplay?.actions).toHaveLength(2);
    expect(view.gameplay?.entities[0]?.items[0]?.label).toBe('Longsword');
    expect(view.gameplay?.entities[0]?.equipment[0]).toEqual({
      slotId: 'main-hand',
      itemInstanceId: 'longsword-1',
    });
    expect(view.gameplay?.actions[0]?.cellPaths).toEqual([
      {
        destinationCellId: 'cell-2-1',
        cellIds: ['cell-1-0', 'cell-2-0', 'cell-2-1'],
        movementCost: 4,
      },
    ]);
    expect(view.gameplay?.turn.initiativeOrder).toEqual(['hero', 'raider']);
    expect(view.gameplay?.archive.verificationStatus).toBe('verified');
    expect(view.gameplay?.archive.replayEntries[0]?.transition).toContain(
      'awaitingReaction ward r2',
    );
  });

  it('maps candidate upgrade impact without implying activation', () => {
    const active = artifact();
    const candidate = {
      ...artifact(),
      artifactId: 'rulebench.fresh-start@1.1.0:fnv1a64:aaaaaaaaaaaaaaaa',
    };
    const candidateResponse = response('compiledCandidate', active, candidate);
    candidateResponse.upgradeImpact = {
      fromArtifactId: active.artifactId,
      toArtifactId: candidate.artifactId,
      sourceChanges: ['rulebench.field-manual: 1.0.0 → 1.1.0'],
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
    };

    const view = playWorkspaceView(candidateResponse);

    expect(view.phase).toBe('candidate');
    expect(view.activationRevision).toBe(0);
    expect(view.upgradeImpact?.definitions[0]).toEqual({
      definitionId: 'rulebench.arc-lash-stormfront',
      status: 'changed derived descendant',
      causes: ['primary base identity or fingerprint changed'],
      fields: [
        {
          plane: 'semantic',
          path: '$.semantic.program.hit.amount.right.value',
          transition: '1 → 2',
        },
      ],
    });
  });
});

function response(
  status: PlayWorkspaceResponseDto['status'],
  activeArtifact: PlayBundleArtifactSummaryDto | null,
  candidateArtifact: PlayBundleArtifactSummaryDto | null,
): PlayWorkspaceResponseDto {
  return {
    ok: true,
    status,
    activeArtifact,
    candidateArtifact,
    upgradeImpact: null,
    activationRevision: status === 'active' ? 1 : 0,
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
    scenarioSetupRequired: status === 'active',
    gameplayAvailable: false,
    gameplay: null,
    diagnostics: [],
  };
}

function gameplay(): NonNullable<PlayWorkspaceResponseDto['gameplay']> {
  return {
    artifactId: 'rulebench.fresh-start@1.0.0:fnv1a64:4444444444444444',
    actorId: 'hero',
    stateRevision: 2,
    acceptedRandomValues: '3',
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
      turn: 3,
    },
    actions: [
      {
        definitionId: 'rulebench.wardbreaker-volley',
        label: 'Wardbreaker Volley',
        itemBinding: null,
        available: false,
        unavailable: {
          code: 'RPG_ACTION_ITEM_BINDING_UNAVAILABLE',
          path: '$.action.itemBinding',
          message: 'a compatible equipped item is required',
        },
        maximumTargets: 1,
        options: {
          participantIds: [],
          cellPaths: [],
          areaIds: [],
        },
      },
      {
        definitionId: 'rulebench.wardbreaker-volley',
        label: 'Wardbreaker Volley — Longsword',
        itemBinding: {
          bindingId: 'main-hand:longsword-1',
          itemInstanceId: 'longsword-1',
          itemDefinitionId: 'item.longsword',
          slotId: 'main-hand',
        },
        available: true,
        unavailable: null,
        maximumTargets: 1,
        options: {
          participantIds: ['raider'],
          cellPaths: [
            {
              destinationCellId: 'cell-2-1',
              cellIds: ['cell-1-0', 'cell-2-0', 'cell-2-1'],
              movementCost: 4,
            },
          ],
          areaIds: [],
        },
      },
      {
        definitionId: 'rulebench.wardbreaker-volley',
        label: 'Wardbreaker Volley — Shortsword',
        itemBinding: {
          bindingId: 'off-hand:shortsword-1',
          itemInstanceId: 'shortsword-1',
          itemDefinitionId: 'item.shortsword',
          slotId: 'off-hand',
        },
        available: true,
        unavailable: null,
        maximumTargets: 1,
        options: {
          participantIds: ['raider'],
          cellPaths: [],
          areaIds: [],
        },
      },
    ],
    controls: [
      {
        kind: 'endTurn',
        label: 'End turn',
        available: true,
        unavailable: null,
      },
    ],
    entities: [
      {
        id: 'hero',
        label: 'Hero',
        teamId: 'allies',
        x: 0,
        y: 0,
        definitionIds: [],
        items: [
          {
            id: 'longsword-1',
            definitionId: 'item.longsword',
            label: 'Longsword',
            description: null,
            tags: ['weapon'],
            traits: ['martial'],
            allowedSlots: ['main-hand'],
            attributes: [],
          },
        ],
        equipment: [{ slotId: 'main-hand', itemInstanceId: 'longsword-1' }],
        vitality: { id: 'vitality', current: 10, maximum: 10 },
        stats: [],
        defenses: [],
        resources: [],
        modifiers: [],
      },
    ],
    pendingReaction: null,
    log: [],
    outcome: { status: 'inProgress', winningTeamIds: [] },
    lastResult: null,
    archive: {
      checkpointSchema: 'asha.rpg.session.checkpoint@1',
      replaySchemaVersion: 1,
      eventSchemaVersion: 1,
      artifactId: 'rulebench.fresh-start@1.0.0:fnv1a64:4444444444444444',
      artifactSchema: 'asha.rpg.play-bundle.compiled@1',
      playBundle: 'rulebench.fresh-start@1.0.0',
      ruleset: 'rulebench.semantic-profile@1.0.0',
      operationSchemas: ['operation.damage@1'],
      capabilitySchemas: ['capability.vitality@1'],
      contentPacks: ['rulebench.field-manual@1.0.0 · fnv1a64:source'],
      dependencyLock: [],
      fingerprints: {
        source: 'fnv1a64:1111111111111111',
        semantic: 'fnv1a64:2222222222222222',
        presentation: 'fnv1a64:3333333333333333',
      },
      definitionFingerprints: [
        'rulebench.signal-flare · fnv1a64:6666666666666666',
      ],
      stateRevision: '2',
      acceptedRandomPosition: '3',
      phase: 'awaitingReaction ward',
      stateHash: 'fnv1a64.rpg-session.v1:state',
      checkpointBytes: 4096,
      replayEntries: [
        {
          sequence: 1,
          operation: 'submit rulebench.signal-flare',
          outcome: 'awaitingReaction',
          before: {
            revision: '2',
            acceptedRandomPosition: '3',
            phase: 'ready',
            stateHash: 'hash-before',
          },
          after: {
            revision: '2',
            acceptedRandomPosition: '3',
            phase: 'awaitingReaction ward',
            stateHash: 'hash-after',
          },
          randomEvidence: ['formulaDice 1d6 at $.damage = 4'],
          events: [],
        },
      ],
      verificationStatus: 'verified',
      verificationMessage: 'Rust replay verified 1 record',
    },
  };
}

function artifact(): PlayBundleArtifactSummaryDto {
  return {
    schema: { id: 'asha.rpg.play-bundle.compiled', version: '1' },
    artifactId: 'rulebench.fresh-start@1.0.0:fnv1a64:4444444444444444',
    playBundle: { id: 'rulebench.fresh-start', version: '1.0.0' },
    ruleset: { id: 'rulebench.semantic-profile', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '1.0.0' },
    contentPacks: [
      {
        id: 'rulebench.field-manual',
        version: '1.0.0',
        sourceFingerprint: 'fnv1a64:5555555555555555',
      },
    ],
    dependencyLock: [],
    requiredOperations: [{ id: 'operation.damage', version: 1 }],
    requiredCapabilities: [{ id: 'capability.vitality', version: 1 }],
    requiredValues: [],
    requiredNumericDomains: [],
    rulesetValues: [],
    participantProfiles: [],
    itemDefinitions: [],
    exportedRoots: ['rulebench.signal-flare'],
    definitions: [
      {
        id: 'rulebench.signal-flare',
        fingerprint: 'fnv1a64:6666666666666666',
        label: 'Signal Flare',
        description: null,
        tags: [],
        catalog: null,
        catalogId: null,
        kind: 'action',
        visibility: 'exported',
        extensionPolicy: 'patchable',
        references: [],
        packageId: 'rulebench.field-manual',
        packageVersion: '1.0.0',
        sourceModule: 'packages/rulebench-field-manual.ts',
        sourceDeclaration: 'signalFlare',
      },
    ],
    policyBindingIds: [],
    relationships: [],
    derivationSlots: 1,
    overlaySlots: 1,
    derivations: [
      {
        definitionId: 'rulebench.signal-flare',
        owner: 'rulebench.field-manual@1.0.0',
        base: 'rulebench.field-manual@1.0.0#rulebench.signal-base',
        baseFingerprint: 'fnv1a64:6666666666666666',
        mixins: [
          {
            identity: 'rulebench.primitives@1.0.0#rulebench.double-range',
            fingerprint: 'fnv1a64:7777777777777777',
            parameters: ['factor=2'],
            order: 0,
          },
        ],
        localPatchFingerprint: 'fnv1a64:8888888888888888',
        materializedFingerprint: 'fnv1a64:9999999999999999',
        changes: [],
      },
    ],
    overlays: [
      {
        overlay: 'rulebench.balance@1.0.0',
        target: 'rulebench.field-manual@1.0.0#rulebench.signal-flare',
        expectedFingerprint: 'fnv1a64:9999999999999999',
        beforeFingerprint: 'fnv1a64:9999999999999999',
        afterFingerprint: 'fnv1a64:aaaaaaaaaaaaaaaa',
        plane: 'semantic',
        conflictPolicy: 'reject',
        patchFingerprint: 'fnv1a64:bbbbbbbbbbbbbbbb',
        order: 0,
        changes: [
          {
            plane: 'semantic',
            path: 'targets.maximumRange',
            before: '7',
            after: '8',
            effective: true,
          },
        ],
      },
    ],
    fingerprints: {
      source: 'fnv1a64:1111111111111111',
      semantic: 'fnv1a64:2222222222222222',
      presentation: 'fnv1a64:3333333333333333',
    },
  };
}
