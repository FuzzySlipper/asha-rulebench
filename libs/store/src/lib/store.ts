import { computed, signal } from '@angular/core';
import {
  rulesetWorkspaceView,
  type RulesetWorkspaceView,
} from '@asha-rulebench/domain';
import {
  browserJsonHttp,
  browserStorage,
  type KeyValueStoragePort,
} from '@asha-rulebench/platform';
import type {
  EncounterSetupRequestDto,
  GameplayCommandRequestDto,
  GameplayReactionRequestDto,
  RulesetCompileRequestDto,
  RulesetWorkspaceResponseDto,
} from '@asha-rulebench/protocol';
import {
  createRulesetTransport,
  type ConfiguredRulesetLocation,
  type RulesetTransport,
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

export class RulesetWorkspaceStore {
  private readonly mutableState = signal<AsyncState<RulesetWorkspaceView>>({
    kind: 'idle',
  });
  private readonly mutableRulesetRoot = signal('');
  private readonly mutableRecentRulesetRoots = signal<readonly string[]>([]);
  private readonly mutableConfiguredRulesets = signal<
    readonly ConfiguredRulesetLocation[]
  >([]);
  private readonly mutableRulesetConfigurationError = signal<string | null>(
    null,
  );
  public readonly state = this.mutableState.asReadonly();
  public readonly view = computed(() => currentView(this.mutableState()));
  public readonly busy = computed(() => this.mutableState().kind === 'loading');
  public readonly rulesetRoot = this.mutableRulesetRoot.asReadonly();
  public readonly recentRulesetRoots =
    this.mutableRecentRulesetRoots.asReadonly();
  public readonly configuredRulesets =
    this.mutableConfiguredRulesets.asReadonly();
  public readonly rulesetConfigurationError =
    this.mutableRulesetConfigurationError.asReadonly();

  public constructor(
    private readonly transport: RulesetTransport,
    private readonly storage: KeyValueStoragePort,
  ) {
    this.mutableRecentRulesetRoots.set(readRecentRulesetRoots(storage));
  }

  public async refresh(): Promise<void> {
    await this.run(() => this.transport.status());
  }

  public async refreshConfiguredRulesets(): Promise<void> {
    try {
      const configuredRulesets = await this.transport.configuredRulesets();
      this.mutableConfiguredRulesets.set(configuredRulesets);
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

  public async compile(request: RulesetCompileRequestDto): Promise<void> {
    const rulesetRoot = request.rulesetRoot.trim();
    this.mutableRulesetRoot.set(rulesetRoot);
    const response = await this.run(() =>
      this.transport.compile({ rulesetRoot }),
    );
    if (response?.ok === true) this.rememberRulesetRoot(rulesetRoot);
  }

  public selectRulesetRoot(rulesetRoot: string): void {
    this.mutableRulesetRoot.set(rulesetRoot);
  }

  public async activate(): Promise<void> {
    await this.run(() => this.transport.activate());
  }

  public async startEncounter(
    setup: EncounterSetupRequestDto,
  ): Promise<boolean> {
    const response = await this.run(() => this.transport.startEncounter(setup));
    return response?.ok === true;
  }

  public async command(command: GameplayCommandRequestDto): Promise<void> {
    await this.run(() => this.transport.command(command));
  }

  public async react(reaction: GameplayReactionRequestDto): Promise<void> {
    await this.run(() => this.transport.react(reaction));
  }

  public async restoreCheckpoint(): Promise<void> {
    await this.run(() => this.transport.restoreCheckpoint());
  }

  public async replay(): Promise<void> {
    await this.run(() => this.transport.replay());
  }

  private async run(
    request: () => ReturnType<RulesetTransport['status']>,
  ): Promise<RulesetWorkspaceResponseDto | null> {
    const previous = currentView(this.mutableState());
    this.mutableState.set({ kind: 'loading', previous });
    try {
      const response = await request();
      this.mutableState.set({
        kind: 'ready',
        value: rulesetWorkspaceView(response),
      });
      return response;
    } catch (error: unknown) {
      this.mutableState.set({
        kind: 'error',
        message:
          error instanceof Error
            ? error.message
            : 'Unknown ruleset transport failure',
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

export function createBrowserRulesetWorkspaceStore(): RulesetWorkspaceStore {
  return new RulesetWorkspaceStore(
    createRulesetTransport(browserJsonHttp()),
    browserStorage(),
  );
}

function currentView(
  state: AsyncState<RulesetWorkspaceView>,
): RulesetWorkspaceView | null {
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
