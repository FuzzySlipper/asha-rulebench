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
  ConfiguredRulesetLocationDto,
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

export class PlayWorkspaceStore {
  private readonly mutableState = signal<AsyncState<PlayWorkspaceView>>({
    kind: 'idle',
  });
  private readonly mutableRulesetRoot = signal('');
  private readonly mutableRecentRulesetRoots = signal<readonly string[]>([]);
  private readonly mutableConfiguredRulesets = signal<
    readonly ConfiguredRulesetLocationDto[]
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
  private readonly mutableRulesetConfigurationError = signal<string | null>(
    null,
  );
  public readonly state = this.mutableState.asReadonly();
  public readonly view = computed(() => currentView(this.mutableState()));
  public readonly busy = computed(
    () => this.mutableState().kind === 'loading' || this.mutableCatalogBusy(),
  );
  public readonly rulesetRoot = this.mutableRulesetRoot.asReadonly();
  public readonly recentRulesetRoots =
    this.mutableRecentRulesetRoots.asReadonly();
  public readonly configuredRulesets =
    this.mutableConfiguredRulesets.asReadonly();
  public readonly rulesetCatalog = this.mutableRulesetCatalog.asReadonly();
  public readonly selectedContentPackIds =
    this.mutableSelectedContentPackIds.asReadonly();
  public readonly catalogDiagnostics =
    this.mutableCatalogDiagnostics.asReadonly();
  public readonly rulesetConfigurationError =
    this.mutableRulesetConfigurationError.asReadonly();

  public constructor(
    private readonly transport: PlayTransport,
    private readonly storage: KeyValueStoragePort,
  ) {
    this.mutableRecentRulesetRoots.set(readRecentRulesetRoots(storage));
  }

  public async refresh(): Promise<void> {
    await this.run(() => this.transport.status());
  }

  public async refreshConfiguredRulesets(): Promise<void> {
    try {
      const configuration = await this.transport.rulesetLocations();
      this.mutableConfiguredRulesets.set(configuration.rulesets);
      this.mutableRulesetConfigurationError.set(null);
    } catch (error: unknown) {
      this.mutableConfiguredRulesets.set([]);
      this.mutableRulesetConfigurationError.set(
        error instanceof Error
          ? error.message
          : 'Unknown ruleset configuration failure',
      );
    }
  }

  public async inspectSelectedRuleset(): Promise<boolean> {
    const rulesetRoot = this.mutableRulesetRoot().trim();
    if (rulesetRoot.length === 0) return false;
    this.mutableCatalogBusy.set(true);
    try {
      const response = await this.transport.inspectRuleset({ rulesetRoot });
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
          '$.rulesetRoot',
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
    const contentPackIds = [...this.mutableSelectedContentPackIds()];
    const response = await this.run(() =>
      this.transport.compile({ rulesetRoot, contentPackIds }),
    );
    if (response?.ok === true) this.rememberRulesetRoot(rulesetRoot);
  }

  public selectRulesetRoot(rulesetRoot: string): void {
    this.mutableRulesetRoot.set(rulesetRoot);
    this.mutableRulesetCatalog.set(null);
    this.mutableSelectedContentPackIds.set([]);
    this.mutableCatalogDiagnostics.set([]);
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
      this.mutableState.set({
        kind: 'ready',
        value: playWorkspaceView(response),
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
