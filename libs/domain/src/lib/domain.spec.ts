import type {
  RulesetArtifactSummaryDto,
  RulesetWorkspaceResponseDto,
} from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import { rulesetWorkspaceView } from './domain.js';

describe('ruleset workspace view mapping', () => {
  it('keeps source, semantic, and presentation fingerprints visibly separate', () => {
    const view = rulesetWorkspaceView(
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
    activeResponse.gameplayAvailable = true;
    activeResponse.gameplay = gameplay();
    const view = rulesetWorkspaceView(activeResponse);

    expect(view.phase).toBe('active');
    expect(view.activeArtifactId).toBe(active.artifactId);
    expect(view.gameplayAvailable).toBe(true);
    expect(view.gameplay?.stateRevision).toBe(2);
    expect(view.gameplay?.actions[0]?.randomPlan).toEqual([
      'if no-roll branch: formulaDice 5d4',
    ]);
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

    const view = rulesetWorkspaceView(candidateResponse);

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
  status: RulesetWorkspaceResponseDto['status'],
  activeArtifact: RulesetArtifactSummaryDto | null,
  candidateArtifact: RulesetArtifactSummaryDto | null,
): RulesetWorkspaceResponseDto {
  return {
    ok: true,
    status,
    activeArtifact,
    candidateArtifact,
    upgradeImpact: null,
    activationRevision: status === 'active' ? 1 : 0,
    gameplayAvailable: false,
    gameplay: null,
    diagnostics: [],
  };
}

function gameplay(): NonNullable<RulesetWorkspaceResponseDto['gameplay']> {
  return {
    actorId: 'hero',
    stateRevision: 2,
    acceptedRandomValues: 3,
    actions: [
      {
        id: 'rulebench.wardbreaker-volley',
        name: 'Wardbreaker Volley',
        sourcePath: 'packages/rulebench-field-manual.ts',
        team: 'hostile',
        maximumRange: 3,
        maximumTargets: 1,
        costs: [{ resourceId: 'focus', amount: 1 }],
        randomPlan: [
          {
            request: {
              kind: 'formulaDice',
              count: 5,
              sides: 4,
              path: '$.damage',
            },
            conditions: [{ kind: 'checkNoRoll', path: '$.program' }],
          },
        ],
        candidateIds: ['raider'],
      },
    ],
    preflights: [
      {
        actionId: 'rulebench.wardbreaker-volley',
        targetId: 'raider',
        available: true,
        code: null,
        message: 'accepted',
      },
    ],
    entities: [],
    pendingReaction: null,
    lastResult: null,
  };
}

function artifact(): RulesetArtifactSummaryDto {
  return {
    schema: { id: 'asha.rpg.ruleset.compiled', version: '1' },
    artifactId: 'rulebench.fresh-start@1.0.0:fnv1a64:4444444444444444',
    composition: { id: 'rulebench.fresh-start', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '1.0.0' },
    sourcePackages: [
      {
        id: 'rulebench.field-manual',
        version: '1.0.0',
        sourceFingerprint: 'fnv1a64:5555555555555555',
      },
    ],
    dependencyLock: [],
    requiredOperations: [{ id: 'operation.damage', version: 1 }],
    requiredCapabilities: [{ id: 'capability.vitality', version: 1 }],
    exportedRoots: ['rulebench.signal-flare'],
    definitions: [
      {
        id: 'rulebench.signal-flare',
        fingerprint: 'fnv1a64:6666666666666666',
        label: 'Signal Flare',
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
