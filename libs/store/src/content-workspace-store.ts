import { Injectable, signal } from "@angular/core";
import type { Provider, Signal } from "@angular/core";
import {
  projectContentDiff,
  projectContentImportAttempt,
  projectContentPackReview,
  projectContentWorkspace,
  type RulebenchContentDiffView,
  type RulebenchContentImportAttemptView,
  type RulebenchContentPackReviewView,
  type RulebenchContentWorkspaceView,
} from "@asha-rulebench/domain";
import { browserClock, type ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchContentPackReferenceDto,
  RulebenchLiveTransportErrorDto,
} from "@asha-rulebench/protocol";
import type { RulebenchLiveTransport } from "@asha-rulebench/transport";
import type { AsyncState } from "./async-state";
import { RULEBENCH_LIVE_TRANSPORT } from "./live-combat-store";

type ContentState<T> = AsyncState<T, RulebenchLiveTransportErrorDto>;

@Injectable()
export class ContentWorkbenchStore {
  private readonly _workspace = signal<ContentState<RulebenchContentWorkspaceView>>({
    kind: "idle",
  });
  readonly workspace: Signal<ContentState<RulebenchContentWorkspaceView>> =
    this._workspace.asReadonly();
  private readonly _importAttempt = signal<ContentState<RulebenchContentImportAttemptView>>({
    kind: "idle",
  });
  readonly importAttempt = this._importAttempt.asReadonly();
  private readonly _review = signal<ContentState<RulebenchContentPackReviewView>>({
    kind: "idle",
  });
  readonly review = this._review.asReadonly();
  private readonly _diff = signal<ContentState<RulebenchContentDiffView>>({ kind: "idle" });
  readonly diff = this._diff.asReadonly();
  private readonly _selectedReference = signal<RulebenchContentPackReferenceDto | null>(null);
  readonly selectedReference = this._selectedReference.asReadonly();
  private readonly _stagedPayload = signal<string | null>(null);
  readonly stagedPayload = this._stagedPayload.asReadonly();
  private requestGeneration = 0;

  constructor(
    private readonly transport: RulebenchLiveTransport,
    private readonly clock: ClockPort,
  ) {}

  async loadWorkspace(): Promise<void> {
    const generation = ++this.requestGeneration;
    this._workspace.set({ kind: "loading" });
    const result = await this.transport.listContentWorkspace();
    if (generation !== this.requestGeneration) return;
    this._workspace.set(
      result.ok
        ? { kind: "data", value: projectContentWorkspace(result.value) }
        : { kind: "error", error: result.error },
    );
    if (result.ok && this._selectedReference() === null) {
      this._selectedReference.set(result.value.packs[0]?.reference ?? null);
    }
    this.clock.now();
  }

  stagePayload(payload: string): void {
    this.requestGeneration += 1;
    this._stagedPayload.set(payload);
    this._importAttempt.set({ kind: "idle" });
    this._diff.set({ kind: "idle" });
    this.clock.now();
  }

  async importStaged(replaceSameIdentity = false): Promise<void> {
    const payload = this._stagedPayload();
    if (payload === null) return;
    const generation = ++this.requestGeneration;
    this._importAttempt.set({ kind: "loading" });
    const result = await this.transport.importContent(
      payload,
      replaceSameIdentity ? "replaceSameIdentity" : "reject",
    );
    if (generation !== this.requestGeneration) return;
    this._importAttempt.set(
      result.ok
        ? { kind: "data", value: projectContentImportAttempt(result.value) }
        : { kind: "error", error: result.error },
    );
    if (result.ok && result.value.accepted && result.value.outcome !== null) {
      this._review.set({
        kind: "data",
        value: projectContentPackReview(result.value.outcome.review),
      });
      this._selectedReference.set(result.value.outcome.review.pack.reference);
      await this.refreshWorkspaceAfterMutation(generation);
    }
    this.clock.now();
  }

  async compareStaged(): Promise<void> {
    const payload = this._stagedPayload();
    if (payload === null) return;
    const generation = ++this.requestGeneration;
    this._diff.set({ kind: "loading" });
    const result = await this.transport.compareContent(payload);
    if (generation !== this.requestGeneration) return;
    this._diff.set(
      result.ok
        ? { kind: "data", value: projectContentDiff(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async selectPack(reference: RulebenchContentPackReferenceDto): Promise<void> {
    this._selectedReference.set(reference);
    const generation = ++this.requestGeneration;
    this._review.set({ kind: "loading" });
    const result = await this.transport.reviewContent(reference);
    if (generation !== this.requestGeneration) return;
    this._review.set(
      result.ok
        ? { kind: "data", value: projectContentPackReview(result.value) }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  async activateSelected(): Promise<void> {
    await this.mutateSelected("activate");
  }

  async deactivateSelected(): Promise<void> {
    await this.mutateSelected("deactivate");
  }

  async deleteSelected(): Promise<void> {
    await this.mutateSelected("delete");
  }

  private async mutateSelected(operation: "activate" | "deactivate" | "delete"): Promise<void> {
    const reference = this._selectedReference();
    if (reference === null) return;
    const generation = ++this.requestGeneration;
    this._workspace.set({ kind: "loading" });
    const result =
      operation === "activate"
        ? await this.transport.activateContent(reference)
        : operation === "deactivate"
          ? await this.transport.deactivateContent(reference)
          : await this.transport.deleteContent(reference);
    if (generation !== this.requestGeneration) return;
    if (!result.ok) {
      this._workspace.set({ kind: "error", error: result.error });
      this.clock.now();
      return;
    }
    this._workspace.set({ kind: "data", value: projectContentWorkspace(result.value) });
    if (operation === "delete") {
      this._selectedReference.set(result.value.packs[0]?.reference ?? null);
      this._review.set({ kind: "idle" });
    } else {
      await this.selectPack(reference);
    }
    this.clock.now();
  }

  private async refreshWorkspaceAfterMutation(generation: number): Promise<void> {
    const result = await this.transport.listContentWorkspace();
    if (generation !== this.requestGeneration) return;
    this._workspace.set(
      result.ok
        ? { kind: "data", value: projectContentWorkspace(result.value) }
        : { kind: "error", error: result.error },
    );
  }
}

export function provideContentWorkbenchStoreKernel(): Provider[] {
  return [
    {
      provide: ContentWorkbenchStore,
      deps: [RULEBENCH_LIVE_TRANSPORT],
      useFactory: (transport: RulebenchLiveTransport) =>
        new ContentWorkbenchStore(transport, browserClock),
    },
  ];
}
