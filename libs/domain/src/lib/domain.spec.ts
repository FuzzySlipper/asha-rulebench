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
    expect(view.gameplay?.actions[0]?.randomPlan).toEqual(['formulaDice: 5d4']);
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
        randomRequests: [
          { kind: 'formulaDice', count: 5, sides: 4, path: '$.damage' },
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
    derivationSlots: 0,
    overlaySlots: 0,
    fingerprints: {
      source: 'fnv1a64:1111111111111111',
      semantic: 'fnv1a64:2222222222222222',
      presentation: 'fnv1a64:3333333333333333',
    },
  };
}
