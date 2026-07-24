import type {
  PlayBundleCompileRequestDto,
  PlayWorkspaceResponseDto,
  RulesetCatalogResponseDto,
  PlayBundleSourceSetDto,
} from '@asha-rulebench/protocol';
import type { KeyValueStoragePort } from '@asha-rulebench/platform';
import type { PlayTransport } from '@asha-rulebench/transport';
import { describe, expect, it } from 'vitest';

import { PlayWorkspaceStore } from './store.js';

describe('play workspace store', () => {
  it('persists distinct gameplay shortcut preferences with stable defaults', () => {
    const storage = memoryStorage();
    const store = new PlayWorkspaceStore(baseTransport(), storage);

    expect(store.executeActionKey()).toBe(' ');
    expect(store.cancelActionKey()).toBe('Escape');
    expect(store.setExecuteActionKey('Escape')).toBe(false);
    expect(store.setExecuteActionKey('Enter')).toBe(true);
    expect(storage.getItem('asha-rulebench.gameplay-shortcuts.v1')).toBe(
      '{"executeAction":"Enter","cancelAction":"Escape"}',
    );

    const restored = new PlayWorkspaceStore(baseTransport(), storage);
    expect(restored.executeActionKey()).toBe('Enter');
    expect(restored.cancelActionKey()).toBe('Escape');

    restored.resetGameplayShortcuts();
    expect(restored.executeActionKey()).toBe(' ');
    expect(restored.cancelActionKey()).toBe('Escape');
  });

  it('loads configured source sets without selecting or activating one', async () => {
    const store = new PlayWorkspaceStore(baseTransport(), memoryStorage());

    await store.refresh();
    await store.refreshConfiguredSourceSets();

    expect(store.view()?.headline).toBe('No PlayBundle active');
    expect(store.rulesetRoot()).toBe('');
    expect(store.configuredSourceSets()).toEqual([
      {
        id: 'minimal',
        label: 'Minimal',
        sourceSet: sourceSet('test-fixtures/rulesets/minimal'),
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
        sourceSet: sourceSet('test-fixtures/rulesets/minimal'),
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
          diagnostics: [diagnostic('PLAY_BUNDLE_SOURCE_ENTRY_NOT_FOUND')],
        }),
      }),
      memoryStorage(),
    );
    await store.refresh();
    store.selectRulesetRoot('/missing/rulesets/game');

    expect(await store.inspectSelectedRuleset()).toBe(false);

    expect(store.view()?.phase).toBe('active');
    expect(store.catalogDiagnostics()[0]?.code).toBe(
      'PLAY_BUNDLE_SOURCE_ENTRY_NOT_FOUND',
    );
  });

  it('replaces roll contributions on every authority lifecycle readback', async () => {
    const stale = activeGameplayResponse('Stale Talent');
    const current = activeGameplayResponse(null);
    const operations: readonly {
      readonly name: string;
      readonly run: (store: PlayWorkspaceStore) => Promise<void>;
    }[] = [
      { name: 'refresh', run: (store) => store.refresh() },
      {
        name: 'activation',
        run: (store) => store.activatePlayBundle(),
      },
      {
        name: 'turn control',
        run: (store) =>
          store.control({
            expectedRevision: 1,
            actorId: 'fighter',
            kind: 'endTurn',
          }),
      },
      {
        name: 'checkpoint restoration',
        run: (store) => store.restoreCheckpoint(),
      },
      { name: 'replay', run: (store) => store.replay() },
    ];

    for (const operation of operations) {
      let statusCalls = 0;
      const store = new PlayWorkspaceStore(
        baseTransport({
          status: async () => {
            statusCalls += 1;
            return statusCalls === 1 ? stale : current;
          },
          activatePlayBundle: async () => current,
          control: async () => current,
          restoreCheckpoint: async () => current,
          replay: async () => current,
        }),
        memoryStorage(),
      );
      await store.refresh();
      expect(
        store.view()?.gameplay?.log[0]?.events[0]?.roll?.contributions[0]
          ?.sourceLabel,
        operation.name,
      ).toBe('Stale Talent');

      await operation.run(store);

      expect(store.view()?.gameplay?.log, operation.name).toEqual([]);
    }
  });
});

function baseTransport(overrides: Partial<PlayTransport> = {}): PlayTransport {
  const transport: PlayTransport = {
    sourceSets: async () => ({
      schemaVersion: 2,
      sourceSets: [
        {
          id: 'minimal',
          label: 'Minimal',
          sourceSet: sourceSet('test-fixtures/rulesets/minimal'),
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
      sourceSet: sourceSet('test-fixtures/rulesets/minimal'),
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
      scenarios: [],
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

function activeGameplayResponse(
  contributionLabel: string | null,
): PlayWorkspaceResponseDto {
  const value = activeResponse();
  const artifactId = value.activeArtifact?.artifactId ?? '';
  return {
    ...value,
    scenarioSetupRequired: false,
    gameplayAvailable: true,
    gameplay: {
      artifactId,
      actorId: 'fighter',
      stateRevision: 1,
      acceptedRandomValues: '1',
      randomSource: randomSource(),
      board: { width: 1, height: 1, cells: [] },
      turn: {
        initiativeOrder: ['fighter'],
        currentActorId: 'fighter',
        round: 1,
        turn: 1,
      },
      actions: [],
      controls: [],
      entities: [],
      pendingReaction: null,
      log:
        contributionLabel === null
          ? []
          : [
              {
                sequence: '1',
                stateRevision: '1',
                actorId: 'fighter',
                actionId: 'action.basic-attack',
                itemBinding: null,
                events: [
                  {
                    kind: 'attackResolved',
                    summary: 'authoritative attack',
                    roll: {
                      kind: 'attack',
                      dieResult: 10,
                      total: 12,
                      thresholdLabel: 'armorClass',
                      threshold: 11,
                      outcome: 'hit',
                      contributions: [
                        {
                          sourceDefinitionId: 'feature.stale',
                          sourceLabel: contributionLabel,
                          amount: 2,
                          reasonKind: 'characterFeature',
                          contributionId: 'stale',
                          selector: 'attack',
                          condition: 'always',
                        },
                      ],
                    },
                  },
                ],
              },
            ],
      outcome: { status: 'inProgress', winningTeamIds: [] },
      lastResult: null,
      archive: {
        checkpointSchema: 'asha.rpg.session.checkpoint@5',
        replaySchemaVersion: 6,
        eventSchemaVersion: 3,
        artifactId,
        artifactSchema: 'asha.rpg.play-bundle.compiled@2',
        playBundle: 'rulebench.minimal.play@1.0.0',
        ruleset: 'rulebench.minimal@1.0.0',
        operationSchemas: [],
        capabilitySchemas: [],
        contentPacks: [],
        dependencyLock: [],
        fingerprints: {
          source: 'source',
          semantic: 'semantic',
          presentation: 'presentation',
        },
        definitionFingerprints: [],
        stateRevision: '1',
        acceptedRandomPosition: '1',
        phase: 'ready',
        stateHash: 'state',
        checkpointBytes: 1,
        replayEntries: [],
        verificationStatus: 'verified',
        verificationMessage: 'verified',
      },
    },
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
    rulesetValues: [],
    participantProfiles: [],
    itemDefinitions: [],
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
    path: '$.sourceSet',
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

function memoryStorage(): KeyValueStoragePort {
  const values = new Map<string, string>();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
    removeItem: (key) => values.delete(key),
  };
}
