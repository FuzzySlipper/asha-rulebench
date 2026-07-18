import { computed, signal } from '@angular/core';
import {
  rulesetWorkspaceView,
  type RulesetWorkspaceView,
} from '@asha-rulebench/domain';
import { browserJsonHttp } from '@asha-rulebench/platform';
import type {
  GameplayCommandRequestDto,
  GameplayReactionRequestDto,
} from '@asha-rulebench/protocol';
import {
  createRulesetTransport,
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

export class RulesetWorkspaceStore {
  private readonly mutableState = signal<AsyncState<RulesetWorkspaceView>>({
    kind: 'idle',
  });
  public readonly state = this.mutableState.asReadonly();
  public readonly view = computed(() => currentView(this.mutableState()));
  public readonly busy = computed(() => this.mutableState().kind === 'loading');

  public constructor(private readonly transport: RulesetTransport) {}

  public async refresh(): Promise<void> {
    await this.run(() => this.transport.status());
  }

  public async compile(sourceId: string): Promise<void> {
    await this.run(() => this.transport.compile(sourceId));
  }

  public async activate(): Promise<void> {
    await this.run(() => this.transport.activate());
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
  ): Promise<void> {
    const previous = currentView(this.mutableState());
    this.mutableState.set({ kind: 'loading', previous });
    try {
      const response = await request();
      this.mutableState.set({
        kind: 'ready',
        value: rulesetWorkspaceView(response),
      });
    } catch (error: unknown) {
      this.mutableState.set({
        kind: 'error',
        message:
          error instanceof Error
            ? error.message
            : 'Unknown ruleset transport failure',
        previous,
      });
    }
  }
}

export function createBrowserRulesetWorkspaceStore(): RulesetWorkspaceStore {
  return new RulesetWorkspaceStore(createRulesetTransport(browserJsonHttp()));
}

function currentView(
  state: AsyncState<RulesetWorkspaceView>,
): RulesetWorkspaceView | null {
  if (state.kind === 'ready') return state.value;
  if (state.kind === 'loading' || state.kind === 'error') return state.previous;
  return null;
}
