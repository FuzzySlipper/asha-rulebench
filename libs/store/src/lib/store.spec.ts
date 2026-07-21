import type {
  PlayBundleCompileRequestDto,
  PlayWorkspaceResponseDto,
  RulesetCatalogResponseDto,
} from '@asha-rulebench/protocol';
import type { KeyValueStoragePort } from '@asha-rulebench/platform';
import type { PlayTransport } from '@asha-rulebench/transport';
import { describe, expect, it } from 'vitest';

import { PlayWorkspaceStore } from './store.js';

describe('play workspace store', () => {
  it('loads configured Ruleset roots without selecting or activating one', async () => {
    const store = new PlayWorkspaceStore(baseTransport(), memoryStorage());

    await store.refresh();
    await store.refreshConfiguredRulesets();

    expect(store.view()?.headline).toBe('No PlayBundle active');
    expect(store.rulesetRoot()).toBe('');
    expect(store.configuredRulesets()).toEqual([
      {
        id: 'minimal',
        label: 'Minimal',
        rulesetRoot: 'test-fixtures/rulesets/minimal',
      },
    ]);
  });

  it('inspects Ruleset contents before any Content Pack selection is made', async () => {
    const store = new PlayWorkspaceStore(baseTransport(), memoryStorage());
    store.selectRulesetRoot('test-fixtures/rulesets/minimal');

    expect(await store.inspectSelectedRuleset()).toBe(true);

    expect(store.rulesetCatalog()?.ruleset.id).toBe('rulebench.minimal');
    expect(store.selectedContentPackIds()).toEqual([]);
    expect(store.view()).toBeNull();
  });

  it('compiles the explicit Content Pack selection and remembers only accepted roots', async () => {
    const requests: PlayBundleCompileRequestDto[] = [];
    const storage = memoryStorage();
    const store = new PlayWorkspaceStore(
      baseTransport({
        compile: async (request) => {
          requests.push(request);
          return candidateResponse();
        },
      }),
      storage,
    );
    store.selectRulesetRoot('test-fixtures/rulesets/minimal');
    await store.inspectSelectedRuleset();
    store.setContentPackSelected('rulebench.minimal.content', true);

    await store.compileSelectedPlayBundle();

    expect(requests).toEqual([
      {
        rulesetRoot: 'test-fixtures/rulesets/minimal',
        contentPackIds: ['rulebench.minimal.content'],
      },
    ]);
    expect(store.view()?.phase).toBe('candidate');
    expect(store.recentRulesetRoots()).toEqual([
      'test-fixtures/rulesets/minimal',
    ]);
    expect(storage.getItem('asha-rulebench.recent-ruleset-roots.v1')).toBe(
      '["test-fixtures/rulesets/minimal"]',
    );
  });

  it('keeps catalog diagnostics separate from an existing active PlayBundle view', async () => {
    const store = new PlayWorkspaceStore(
      baseTransport({
        status: async () => activeResponse(),
        inspectRuleset: async () => ({
          ok: false,
          catalog: null,
          diagnostics: [diagnostic('RULESET_ROOT_ENTRY_NOT_FOUND')],
        }),
      }),
      memoryStorage(),
    );
    await store.refresh();
    store.selectRulesetRoot('/missing/rulesets/game');

    expect(await store.inspectSelectedRuleset()).toBe(false);

    expect(store.view()?.phase).toBe('active');
    expect(store.catalogDiagnostics()[0]?.code).toBe(
      'RULESET_ROOT_ENTRY_NOT_FOUND',
    );
  });
});

function baseTransport(overrides: Partial<PlayTransport> = {}): PlayTransport {
  const transport: PlayTransport = {
    rulesetLocations: async () => ({
      schemaVersion: 1,
      rulesets: [
        {
          id: 'minimal',
          label: 'Minimal',
          rulesetRoot: 'test-fixtures/rulesets/minimal',
        },
      ],
    }),
    inspectRuleset: async () => catalogResponse(),
    status: async () => emptyResponse(),
    compile: async () => candidateResponse(),
    activatePlayBundle: async () => activeResponse(),
    startScenario: async () => activeResponse(),
    command: async () => activeResponse(),
    react: async () => activeResponse(),
    control: async () => activeResponse(),
    restoreCheckpoint: async () => activeResponse(),
    replay: async () => activeResponse(),
  };
  return { ...transport, ...overrides };
}

function catalogResponse(): RulesetCatalogResponseDto {
  return {
    ok: true,
    catalog: {
      rulesetRoot: 'test-fixtures/rulesets/minimal',
      ruleset: { id: 'rulebench.minimal', version: '1.0.0' },
      contentPacks: [
        {
          id: 'rulebench.minimal.content',
          version: '1.0.0',
          label: 'Minimal Content',
          requirements: [],
        },
      ],
      playBundles: [
        {
          id: 'rulebench.minimal.play',
          version: '1.0.0',
          contentPackIds: ['rulebench.minimal.content'],
          compatible: true,
          diagnostics: [],
        },
      ],
    },
    diagnostics: [],
  };
}

function emptyResponse(): PlayWorkspaceResponseDto {
  return response('noActivePlayBundle', null);
}

function candidateResponse(): PlayWorkspaceResponseDto {
  return response('compiledCandidate', artifact());
}

function activeResponse(): PlayWorkspaceResponseDto {
  const value = response('active', null);
  return {
    ...value,
    activeArtifact: artifact(),
    activationRevision: 1,
    scenarioSetupRequired: true,
  };
}

function response(
  status: PlayWorkspaceResponseDto['status'],
  candidateArtifact: PlayWorkspaceResponseDto['candidateArtifact'],
): PlayWorkspaceResponseDto {
  return {
    ok: true,
    status,
    activeArtifact: null,
    candidateArtifact,
    upgradeImpact: null,
    activationRevision: 0,
    hostRandomSource: randomSource(),
    supportedRandomSources: [randomSource()],
    scenarioSetupRequired: false,
    gameplayAvailable: false,
    gameplay: null,
    diagnostics: [],
  };
}

function artifact(): NonNullable<PlayWorkspaceResponseDto['activeArtifact']> {
  return {
    schema: { id: 'asha.rpg.play-bundle.compiled', version: '1' },
    artifactId: 'rulebench.minimal.play@1.0.0:fnv1a64:artifact',
    playBundle: { id: 'rulebench.minimal.play', version: '1.0.0' },
    ruleset: { id: 'rulebench.minimal', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '1.0.0' },
    contentPacks: [],
    dependencyLock: [],
    requiredOperations: [],
    requiredCapabilities: [],
    requiredValues: [],
    requiredNumericDomains: [],
    exportedRoots: [],
    definitions: [],
    policyBindingIds: [],
    relationships: [],
    derivationSlots: 0,
    overlaySlots: 0,
    derivations: [],
    overlays: [],
    fingerprints: {
      source: 'source',
      semantic: 'semantic',
      presentation: 'presentation',
    },
  };
}

function diagnostic(code: string) {
  return {
    stage: 'source',
    severity: 'error',
    code,
    path: '$.rulesetRoot',
    message: 'missing',
    packageId: null,
    definitionId: null,
    source: null,
    graphPath: null,
    expected: null,
    actual: null,
  };
}

function randomSource() {
  return {
    policyId: 'random.automatic',
    policyVersion: 1,
    sourceId: 'random.system',
    sourceVersion: 1,
  };
}

function memoryStorage(): KeyValueStoragePort {
  const values = new Map<string, string>();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };
}
