import type { RulesetWorkspaceResponseDto } from '@asha-rulebench/protocol';
import type { RulesetTransport } from '@asha-rulebench/transport';
import { memoryStorage } from '@asha-rulebench/platform';
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
          packageId: null,
          definitionId: null,
          source: null,
          graphPath: null,
          expected: null,
          actual: null,
        },
      ],
    };
    const store = new RulesetWorkspaceStore(
      transportReturning(rejected),
      memoryStorage(),
    );

    await store.compile(exampleWorkspace());

    expect(store.state().kind).toBe('ready');
    expect(store.view()?.phase).toBe('empty');
    expect(store.view()?.diagnostics[0]?.code).toBe(
      'RULESET_EXPORTED_ROOT_MISSING',
    );
  });

  it('classifies transport failures without discarding the previous view', async () => {
    const transport: RulesetTransport = {
      configuredRulesets: async () => [],
      status: async () => emptyResponse(),
      compile: async () => {
        throw new Error('host offline');
      },
      activate: async () => emptyResponse(),
      startEncounter: async () => emptyResponse(),
      command: async () => emptyResponse(),
      react: async () => emptyResponse(),
      restoreCheckpoint: async () => emptyResponse(),
      replay: async () => emptyResponse(),
    };
    const store = new RulesetWorkspaceStore(transport, memoryStorage());
    await store.refresh();
    await store.compile(exampleWorkspace());

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
      configuredRulesets: async () => [],
      status: async () => active,
      compile: async () => {
        compileRequests += 1;
        return {
          ...active,
          ok: false,
          diagnostics: [
            {
              stage: 'graph',
              severity: 'error',
              code: 'RULESET_DEFINITION_REFERENCE_MISSING',
              path: '$.packages[rulebench.field-manual@1.0.0].definitions[0]',
              message: 'missing support definition',
              packageId: 'rulebench.field-manual',
              definitionId: 'rulebench.signal-flare',
              source: {
                module: 'packages/rulebench-field-manual.ts',
                declaration: 'signalFlare',
              },
              graphPath: null,
              expected: null,
              actual: null,
            },
          ],
        };
      },
      activate: async () => active,
      startEncounter: async () => active,
      command: async () => active,
      react: async () => active,
      restoreCheckpoint: async () => active,
      replay: async () => active,
    };
    const store = new RulesetWorkspaceStore(transport, memoryStorage());
    await store.refresh();
    await store.compile(exampleWorkspace());

    expect(compileRequests).toBe(1);
    expect(store.view()?.phase).toBe('active');
    expect(store.view()?.activationRevision).toBe(3);
    expect(store.view()?.activeArtifactId).toBe('artifact-active');
    expect(store.view()?.diagnostics[0]?.code).toBe(
      'RULESET_DEFINITION_REFERENCE_MISSING',
    );
  });

  it('persists successful roots in most-recent order without selecting startup content', async () => {
    const storage = memoryStorage();
    const store = new RulesetWorkspaceStore(
      transportReturning(emptyResponse()),
      storage,
    );

    expect(store.rulesetRoot()).toBe('');
    await store.compile({ rulesetRoot: 'examples/rulesets/field-manual' });
    await store.compile({ rulesetRoot: 'examples/rulesets/ember-skirmish' });
    await store.compile({ rulesetRoot: 'examples/rulesets/field-manual' });

    expect(store.recentRulesetRoots()).toEqual([
      'examples/rulesets/field-manual',
      'examples/rulesets/ember-skirmish',
    ]);
    const reloaded = new RulesetWorkspaceStore(
      transportReturning(emptyResponse()),
      storage,
    );
    expect(reloaded.rulesetRoot()).toBe('');
    expect(reloaded.recentRulesetRoots()).toEqual(store.recentRulesetRoots());
  });

  it('does not add rejected roots to the switch menu', async () => {
    const rejected = { ...emptyResponse(), ok: false };
    const store = new RulesetWorkspaceStore(
      transportReturning(rejected),
      memoryStorage(),
    );

    await store.compile({ rulesetRoot: 'examples/rulesets/invalid-build' });

    expect(store.rulesetRoot()).toBe('examples/rulesets/invalid-build');
    expect(store.recentRulesetRoots()).toEqual([]);
  });

  it('loads configured source locations without selecting startup content', async () => {
    const transport: RulesetTransport = {
      ...transportReturning(emptyResponse()),
      configuredRulesets: async () => [
        {
          id: 'field-manual',
          label: 'Field Manual',
          rulesetRoot: 'examples/rulesets/field-manual',
        },
      ],
    };
    const store = new RulesetWorkspaceStore(transport, memoryStorage());

    await store.refreshConfiguredRulesets();

    expect(store.configuredRulesets()).toEqual([
      {
        id: 'field-manual',
        label: 'Field Manual',
        rulesetRoot: 'examples/rulesets/field-manual',
      },
    ]);
    expect(store.rulesetRoot()).toBe('');
    expect(store.rulesetConfigurationError()).toBeNull();
  });

  it('keeps a configuration failure separate from host workspace state', async () => {
    const transport: RulesetTransport = {
      ...transportReturning(emptyResponse()),
      configuredRulesets: async () => {
        throw new Error('invalid local ruleset config');
      },
    };
    const store = new RulesetWorkspaceStore(transport, memoryStorage());
    await store.refresh();

    await store.refreshConfiguredRulesets();

    expect(store.view()?.headline).toBe('No compiled ruleset active');
    expect(store.configuredRulesets()).toEqual([]);
    expect(store.rulesetConfigurationError()).toBe(
      'invalid local ruleset config',
    );
  });
});

function transportReturning(
  response: RulesetWorkspaceResponseDto,
): RulesetTransport {
  return {
    configuredRulesets: async () => [],
    status: async () => response,
    compile: async () => response,
    activate: async () => response,
    startEncounter: async () => response,
    command: async () => response,
    react: async () => response,
    restoreCheckpoint: async () => response,
    replay: async () => response,
  };
}

function exampleWorkspace() {
  return {
    rulesetRoot: 'examples/rulesets/field-manual',
  };
}

function emptyResponse(): RulesetWorkspaceResponseDto {
  return {
    ok: true,
    status: 'noActiveRuleset',
    activeArtifact: null,
    candidateArtifact: null,
    upgradeImpact: null,
    activationRevision: 0,
    hostRandomSource: {
      policyId: 'rulebench.automatic-random',
      policyVersion: 1,
      sourceId: 'rulebench.system-random',
      sourceVersion: 1,
    },
    encounterSetupRequired: false,
    gameplayAvailable: false,
    gameplay: null,
    diagnostics: [],
  };
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
    derivations: [],
    overlays: [],
    fingerprints: {
      source: 'fnv1a64:0000000000000000',
      semantic: 'fnv1a64:0000000000000000',
      presentation: 'fnv1a64:0000000000000000',
    },
  };
}
