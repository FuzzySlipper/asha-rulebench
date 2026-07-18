import type { RulesetWorkspaceResponseDto } from '@asha-rulebench/protocol';
import type { RulesetTransport } from '@asha-rulebench/transport';
import { describe, expect, it } from 'vitest';

import { RulesetWorkspaceStore } from './store.js';

describe('ruleset workspace store', () => {
  it('keeps compiler diagnostics as inspected state rather than inventing authority results', async () => {
    const rejected: RulesetWorkspaceResponseDto = {
      ...emptyResponse(),
      ok: false,
      diagnostics: [
        {
          stage: 'references',
          severity: 'error',
          code: 'RULESET_EXPORTED_ROOT_MISSING',
          path: '$.exportedRoots[0]',
          message: 'missing root',
        },
      ],
    };
    const store = new RulesetWorkspaceStore(transportReturning(rejected));

    await store.compile(validPreparation());

    expect(store.state().kind).toBe('ready');
    expect(store.view()?.phase).toBe('empty');
    expect(store.view()?.diagnostics[0]?.code).toBe(
      'RULESET_EXPORTED_ROOT_MISSING',
    );
  });

  it('classifies transport failures without discarding the previous view', async () => {
    const transport: RulesetTransport = {
      status: async () => emptyResponse(),
      compile: async () => {
        throw new Error('host offline');
      },
      activate: async () => emptyResponse(),
    };
    const store = new RulesetWorkspaceStore(transport);
    await store.refresh();
    await store.compile(validPreparation());

    const state = store.state();
    expect(state.kind).toBe('error');
    if (state.kind !== 'error') return;
    expect(state.message).toBe('host offline');
    expect(state.previous?.headline).toBe('No compiled ruleset active');
  });

  it('keeps the active artifact when fresh TypeScript preparation fails', async () => {
    const active = {
      ...emptyResponse(),
      status: 'active' as const,
      activationRevision: 3,
      activeArtifact: artifactSummary('artifact-active'),
    };
    let compileRequests = 0;
    const transport: RulesetTransport = {
      status: async () => active,
      compile: async () => {
        compileRequests += 1;
        return emptyResponse();
      },
      activate: async () => active,
    };
    const store = new RulesetWorkspaceStore(transport);
    await store.refresh();
    await store.compile({
      ok: false,
      diagnostics: [
        {
          stage: 'graph',
          severity: 'error',
          code: 'RULESET_DEFINITION_REFERENCE_MISSING',
          path: '$.packages[rulebench.field-manual@1.0.0].definitions[0]',
          message: 'missing support definition',
        },
      ],
    });

    expect(compileRequests).toBe(0);
    expect(store.view()?.phase).toBe('active');
    expect(store.view()?.activationRevision).toBe(3);
    expect(store.view()?.activeArtifactId).toBe('artifact-active');
    expect(store.view()?.diagnostics[0]?.code).toBe(
      'RULESET_DEFINITION_REFERENCE_MISSING',
    );
  });
});

function transportReturning(
  response: RulesetWorkspaceResponseDto,
): RulesetTransport {
  return {
    status: async () => response,
    compile: async () => response,
    activate: async () => response,
  };
}

function emptyResponse(): RulesetWorkspaceResponseDto {
  return {
    ok: true,
    status: 'noActiveRuleset',
    activeArtifact: null,
    candidateArtifact: null,
    activationRevision: 0,
    gameplayAvailable: false,
    diagnostics: [],
  };
}

function validPreparation() {
  return { ok: true as const, preparedSource: '{}', diagnostics: [] as const };
}

function artifactSummary(artifactId: string) {
  return {
    schema: { id: 'asha.rpg.ruleset.compiled', version: '1' },
    artifactId,
    composition: { id: 'rulebench.fresh-start', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '1.0.0' },
    sourcePackages: [],
    dependencyLock: [],
    requiredOperations: [],
    requiredCapabilities: [],
    exportedRoots: [],
    definitions: [],
    policyBindingIds: [],
    relationships: [],
    derivationSlots: 0,
    overlaySlots: 0,
    fingerprints: {
      source: 'fnv1a64:0000000000000000',
      semantic: 'fnv1a64:0000000000000000',
      presentation: 'fnv1a64:0000000000000000',
    },
  };
}
