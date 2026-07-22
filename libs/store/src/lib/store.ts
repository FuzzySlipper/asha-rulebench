import { computed, signal } from '@angular/core';
import {
  playWorkspaceView,
  type PlayWorkspaceView,
} from '@asha-rulebench/domain';
import {
  browserJsonHttp,
  browserStorage,
  type KeyValueStoragePort,
} from '@asha-rulebench/platform';
import type {
  ScenarioSetupRequestDto,
  GameplayCommandRequestDto,
  GameplayReactionRequestDto,
  GameplayTurnControlRequestDto,
  PlayDiagnosticDto,
  PlayWorkspaceResponseDto,
  RulesetCatalogDto,
  ConfiguredPlayBundleSourceSetDto,
  PlayBundleSourceSetDto,
  PlayBundleSourceEntryDto,
} from '@asha-rulebench/protocol';
import {
  createPlayTransport,
  type PlayTransport,
} from '@asha-rulebench/transport';

export type AsyncState<Value> =
  | { readonly kind: 'idle' }
  | { readonly kind: 'loading'; readonly previous: Value | null }
  | { readonly kind: 'ready'; readonly value: Value }
  | {
      readonly kind: 'error';
      readonly message: string;
      readonly previous: Value | null;
    };

const RECENT_RULESET_ROOTS_KEY = 'asha-rulebench.recent-ruleset-roots.v1';
const RECENT_RULESET_ROOT_LIMIT = 8;
const GAMEPLAY_SHORTCUTS_KEY = 'asha-rulebench.gameplay-shortcuts.v1';

export const DEFAULT_EXECUTE_ACTION_KEY = ' ';
export const DEFAULT_CANCEL_ACTION_KEY = 'Escape';
export const GAMEPLAY_SHORTCUT_KEYS: readonly string[] = [
  DEFAULT_EXECUTE_ACTION_KEY,
  'Enter',
  DEFAULT_CANCEL_ACTION_KEY,
  'Backspace',
  ...'abcdefghijklmnopqrstuvwxyz',
];

export class PlayWorkspaceStore {
  private readonly mutableState = signal<AsyncState<PlayWorkspaceView>>({
    kind: 'idle',
  });
  private readonly mutableAuthorityReadbackRevision = signal(0);
  private readonly mutableExecuteActionKey = signal(DEFAULT_EXECUTE_ACTION_KEY);
  private readonly mutableCancelActionKey = signal(DEFAULT_CANCEL_ACTION_KEY);
  private readonly mutableRulesetRoot = signal('');
  private readonly mutableAdditionalSourceRoots = signal('');
  private readonly mutableConfiguredSourceSetId = signal<string | null>(null);
  private readonly mutableRecentRulesetRoots = signal<readonly string[]>([]);
  private readonly mutableConfiguredSourceSets = signal<
    readonly ConfiguredPlayBundleSourceSetDto[]
  >([]);
  private readonly mutableRulesetCatalog = signal<RulesetCatalogDto | null>(
    null,
  );
  private readonly mutableSelectedContentPackIds = signal<readonly string[]>(
    [],
  );
  private readonly mutableCatalogDiagnostics = signal<
    readonly PlayDiagnosticDto[]
  >([]);
  private readonly mutableCatalogBusy = signal(false);
  private readonly mutableSourceSetConfigurationError = signal<string | null>(
    null,
  );
  public readonly state = this.mutableState.asReadonly();
  public readonly view = computed(() => currentView(this.mutableState()));
  public readonly authorityReadbackRevision =
    this.mutableAuthorityReadbackRevision.asReadonly();
  public readonly executeActionKey = this.mutableExecuteActionKey.asReadonly();
  public readonly cancelActionKey = this.mutableCancelActionKey.asReadonly();
  public readonly busy = computed(
    () => this.mutableState().kind === 'loading' || this.mutableCatalogBusy(),
  );
  public readonly rulesetRoot = this.mutableRulesetRoot.asReadonly();
  public readonly additionalSourceRoots =
    this.mutableAdditionalSourceRoots.asReadonly();
  public readonly configuredSourceSetId =
    this.mutableConfiguredSourceSetId.asReadonly();
  public readonly recentRulesetRoots =
    this.mutableRecentRulesetRoots.asReadonly();
  public readonly configuredSourceSets =
    this.mutableConfiguredSourceSets.asReadonly();
  public readonly rulesetCatalog = this.mutableRulesetCatalog.asReadonly();
  public readonly selectedContentPackIds =
    this.mutableSelectedContentPackIds.asReadonly();
  public readonly catalogDiagnostics =
    this.mutableCatalogDiagnostics.asReadonly();
  public readonly sourceSetConfigurationError =
    this.mutableSourceSetConfigurationError.asReadonly();

  public constructor(
    private readonly transport: PlayTransport,
    private readonly storage: KeyValueStoragePort,
  ) {
    this.mutableRecentRulesetRoots.set(readRecentRulesetRoots(storage));
    const shortcuts = readGameplayShortcuts(storage);
    this.mutableExecuteActionKey.set(shortcuts.executeAction);
    this.mutableCancelActionKey.set(shortcuts.cancelAction);
  }

  public setExecuteActionKey(key: string): boolean {
    if (!isGameplayShortcutKey(key) || key === this.mutableCancelActionKey()) {
      return false;
    }
    this.mutableExecuteActionKey.set(key);
    this.persistGameplayShortcuts();
    return true;
  }

  public setCancelActionKey(key: string): boolean {
    if (!isGameplayShortcutKey(key) || key === this.mutableExecuteActionKey()) {
      return false;
    }
    this.mutableCancelActionKey.set(key);
    this.persistGameplayShortcuts();
    return true;
  }

  public resetGameplayShortcuts(): void {
    this.mutableExecuteActionKey.set(DEFAULT_EXECUTE_ACTION_KEY);
    this.mutableCancelActionKey.set(DEFAULT_CANCEL_ACTION_KEY);
    this.persistGameplayShortcuts();
  }

  public async refresh(): Promise<void> {
    await this.run(() => this.transport.status());
  }

  public async refreshConfiguredSourceSets(): Promise<void> {
    try {
      const configuration = await this.transport.sourceSets();
      this.mutableConfiguredSourceSets.set(configuration.sourceSets);
      this.mutableSourceSetConfigurationError.set(null);
    } catch (error: unknown) {
      this.mutableConfiguredSourceSets.set([]);
      this.mutableSourceSetConfigurationError.set(
        error instanceof Error
          ? error.message
          : 'Unknown PlayBundle source-set configuration failure',
      );
    }
  }

  public async inspectSelectedRuleset(): Promise<boolean> {
    const sourceSet = this.selectedSourceSet();
    if (sourceSet === null) return false;
    this.mutableCatalogBusy.set(true);
    try {
      const response = await this.transport.inspectRuleset({ sourceSet });
      this.mutableRulesetCatalog.set(response.catalog);
      this.mutableCatalogDiagnostics.set(response.diagnostics);
      this.mutableSelectedContentPackIds.set([]);
      return response.ok && response.catalog !== null;
    } catch (error: unknown) {
      this.mutableRulesetCatalog.set(null);
      this.mutableSelectedContentPackIds.set([]);
      this.mutableCatalogDiagnostics.set([
        clientDiagnostic(
          'RULESET_CATALOG_REQUEST_FAILED',
          '$.sourceSet',
          error instanceof Error
            ? error.message
            : 'Unknown Ruleset catalog failure',
        ),
      ]);
      return false;
    } finally {
      this.mutableCatalogBusy.set(false);
    }
  }

  public async compileSelectedPlayBundle(): Promise<void> {
    const rulesetRoot = this.mutableRulesetRoot().trim();
    const sourceSet = this.selectedSourceSet();
    if (sourceSet === null) return;
    const contentPackIds = [...this.mutableSelectedContentPackIds()];
    const response = await this.run(() =>
      this.transport.compile({ sourceSet, contentPackIds }),
    );
    if (response?.ok === true) this.rememberRulesetRoot(rulesetRoot);
  }

  public selectRulesetRoot(rulesetRoot: string): void {
    this.mutableRulesetRoot.set(rulesetRoot);
    this.mutableConfiguredSourceSetId.set(null);
    this.clearCatalogSelection();
  }

  public selectAdditionalSourceRoots(sourceRoots: string): void {
    this.mutableAdditionalSourceRoots.set(sourceRoots);
    this.mutableConfiguredSourceSetId.set(null);
    this.clearCatalogSelection();
  }

  public selectConfiguredSourceSet(
    location: ConfiguredPlayBundleSourceSetDto | null,
  ): void {
    this.mutableConfiguredSourceSetId.set(location?.id ?? null);
    const entries = location?.sourceSet.entries ?? [];
    const rulesetEntry = entries.find((entry) =>
      entry.exportKinds.includes('ruleset'),
    );
    this.mutableRulesetRoot.set(rulesetEntry?.sourceRoot ?? '');
    this.mutableAdditionalSourceRoots.set(
      entries
        .filter((entry) => entry !== rulesetEntry)
        .map((entry) => entry.sourceRoot)
        .join('\n'),
    );
    this.clearCatalogSelection();
  }

  private clearCatalogSelection(): void {
    this.mutableRulesetCatalog.set(null);
    this.mutableSelectedContentPackIds.set([]);
    this.mutableCatalogDiagnostics.set([]);
  }

  private selectedSourceSet(): PlayBundleSourceSetDto | null {
    const configuredId = this.mutableConfiguredSourceSetId();
    const configured = this.mutableConfiguredSourceSets().find(
      (location) => location.id === configuredId,
    );
    if (configured !== undefined) return configured.sourceSet;

    const rulesetRoot = this.mutableRulesetRoot().trim();
    if (rulesetRoot.length === 0) return null;
    const additionalRoots = uniqueSourceRoots(
      this.mutableAdditionalSourceRoots(),
    ).filter((root) => root !== rulesetRoot);
    return {
      schemaVersion: 1,
      allowedRoots: [rulesetRoot, ...additionalRoots],
      entries: [
        {
          id: 'ruleset',
          label: 'Ruleset source',
          sourceRoot: rulesetRoot,
          module: 'src/index.ts',
          exportKinds: [
            'ruleset',
            'contentPack',
            'playBundle',
            'scenarioTemplate',
          ],
        },
        ...additionalRoots.map<PlayBundleSourceEntryDto>(
          (sourceRoot, index) => ({
            id: `content-${index + 1}`,
            label: `Content source ${index + 1}`,
            sourceRoot,
            module: 'src/index.ts',
            exportKinds: ['contentPack', 'playBundle', 'scenarioTemplate'],
          }),
        ),
      ],
    };
  }

  public setContentPackSelected(
    contentPackId: string,
    selected: boolean,
  ): void {
    this.mutableSelectedContentPackIds.update((current) => {
      if (selected) {
        return current.includes(contentPackId)
          ? current
          : [...current, contentPackId].sort((left, right) =>
              left.localeCompare(right),
            );
      }
      return current.filter((value) => value !== contentPackId);
    });
  }

  public async activatePlayBundle(): Promise<void> {
    await this.run(() => this.transport.activatePlayBundle());
  }

  public async startScenario(setup: ScenarioSetupRequestDto): Promise<boolean> {
    const response = await this.run(() => this.transport.startScenario(setup));
    return response?.ok === true;
  }

  public async command(command: GameplayCommandRequestDto): Promise<void> {
    await this.run(() => this.transport.command(command));
  }

  public async react(reaction: GameplayReactionRequestDto): Promise<void> {
    await this.run(() => this.transport.react(reaction));
  }

  public async control(control: GameplayTurnControlRequestDto): Promise<void> {
    await this.run(() => this.transport.control(control));
  }

  public async restoreCheckpoint(): Promise<void> {
    await this.run(() => this.transport.restoreCheckpoint());
  }

  public async replay(): Promise<void> {
    await this.run(() => this.transport.replay());
  }

  private async run(
    request: () => ReturnType<PlayTransport['status']>,
  ): Promise<PlayWorkspaceResponseDto | null> {
    const previous = currentView(this.mutableState());
    this.mutableState.set({ kind: 'loading', previous });
    try {
      const response = await request();
      const value = playWorkspaceView(response);
      this.mutableAuthorityReadbackRevision.update((revision) => revision + 1);
      this.mutableState.set({
        kind: 'ready',
        value,
      });
      return response;
    } catch (error: unknown) {
      this.mutableState.set({
        kind: 'error',
        message:
          error instanceof Error
            ? error.message
            : 'Unknown play transport failure',
        previous,
      });
      return null;
    }
  }

  private rememberRulesetRoot(rulesetRoot: string): void {
    if (rulesetRoot.length === 0) return;
    const next = [
      rulesetRoot,
      ...this.mutableRecentRulesetRoots().filter(
        (candidate) => candidate !== rulesetRoot,
      ),
    ].slice(0, RECENT_RULESET_ROOT_LIMIT);
    this.mutableRecentRulesetRoots.set(next);
    this.storage.setItem(RECENT_RULESET_ROOTS_KEY, JSON.stringify(next));
  }

  private persistGameplayShortcuts(): void {
    this.storage.setItem(
      GAMEPLAY_SHORTCUTS_KEY,
      JSON.stringify({
        executeAction: this.mutableExecuteActionKey(),
        cancelAction: this.mutableCancelActionKey(),
      }),
    );
  }
}

function uniqueSourceRoots(value: string): readonly string[] {
  const roots = value
    .split(/\r?\n/)
    .map((root) => root.trim())
    .filter((root) => root.length > 0);
  return [...new Set(roots)];
}

function clientDiagnostic(
  code: string,
  path: string,
  message: string,
): PlayDiagnosticDto {
  return {
    stage: 'source',
    severity: 'error',
    code,
    path,
    message,
    packageId: null,
    definitionId: null,
    source: null,
    graphPath: null,
    expected: null,
    actual: null,
  };
}

export function createBrowserPlayWorkspaceStore(): PlayWorkspaceStore {
  return new PlayWorkspaceStore(
    createPlayTransport(browserJsonHttp()),
    browserStorage(),
  );
}

function currentView(
  state: AsyncState<PlayWorkspaceView>,
): PlayWorkspaceView | null {
  if (state.kind === 'ready') return state.value;
  if (state.kind === 'loading' || state.kind === 'error') return state.previous;
  return null;
}

function readRecentRulesetRoots(
  storage: KeyValueStoragePort,
): readonly string[] {
  const stored = storage.getItem(RECENT_RULESET_ROOTS_KEY);
  if (stored === null) return [];
  try {
    const parsed: unknown = JSON.parse(stored);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter(
        (value): value is string =>
          typeof value === 'string' && value.trim().length > 0,
      )
      .map((value) => value.trim())
      .filter((value, index, values) => values.indexOf(value) === index)
      .slice(0, RECENT_RULESET_ROOT_LIMIT);
  } catch {
    return [];
  }
}

function readGameplayShortcuts(storage: KeyValueStoragePort): {
  readonly executeAction: string;
  readonly cancelAction: string;
} {
  const defaults = {
    executeAction: DEFAULT_EXECUTE_ACTION_KEY,
    cancelAction: DEFAULT_CANCEL_ACTION_KEY,
  };
  const stored = storage.getItem(GAMEPLAY_SHORTCUTS_KEY);
  if (stored === null) return defaults;
  try {
    const parsed: unknown = JSON.parse(stored);
    if (
      typeof parsed !== 'object' ||
      parsed === null ||
      !('executeAction' in parsed) ||
      !('cancelAction' in parsed)
    ) {
      return defaults;
    }
    const executeAction = parsed.executeAction;
    const cancelAction = parsed.cancelAction;
    if (
      typeof executeAction !== 'string' ||
      typeof cancelAction !== 'string' ||
      !isGameplayShortcutKey(executeAction) ||
      !isGameplayShortcutKey(cancelAction) ||
      executeAction === cancelAction
    ) {
      return defaults;
    }
    return { executeAction, cancelAction };
  } catch {
    return defaults;
  }
}

function isGameplayShortcutKey(value: string): boolean {
  return GAMEPLAY_SHORTCUT_KEYS.includes(value);
}
