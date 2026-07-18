import { computed, signal } from '@angular/core';
import {
  rulesetWorkspaceView,
  type RulesetWorkspaceView,
} from '@asha-rulebench/domain';
import { browserJsonHttp } from '@asha-rulebench/platform';
import type { RulesetDiagnosticDto } from '@asha-rulebench/protocol';
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

export type RulesetSourcePreparation =
  | {
      readonly ok: true;
      readonly preparedSource: string;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly RulesetDiagnosticDto[];
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

  public async compile(preparation: RulesetSourcePreparation): Promise<void> {
    if (!preparation.ok) {
      const previous = currentView(this.mutableState()) ?? emptyWorkspaceView();
      this.mutableState.set({
        kind: 'ready',
        value: { ...previous, diagnostics: preparation.diagnostics },
      });
      return;
    }
    await this.run(() => this.transport.compile(preparation.preparedSource));
  }

  public async activate(): Promise<void> {
    await this.run(() => this.transport.activate());
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

function emptyWorkspaceView(): RulesetWorkspaceView {
  return rulesetWorkspaceView({
    ok: true,
    status: 'noActiveRuleset',
    activeArtifact: null,
    candidateArtifact: null,
    activationRevision: 0,
    gameplayAvailable: false,
    diagnostics: [],
  });
}
