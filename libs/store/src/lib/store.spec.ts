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

    await store.compile();

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
    await store.compile();

    const state = store.state();
    expect(state.kind).toBe('error');
    if (state.kind !== 'error') return;
    expect(state.message).toBe('host offline');
    expect(state.previous?.headline).toBe('No compiled ruleset active');
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
