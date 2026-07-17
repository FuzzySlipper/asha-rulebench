import { Injectable, computed, signal } from "@angular/core";
import type { Provider, Signal } from "@angular/core";
import {
  projectContentActionBindingCatalog,
  projectContentAuthoringDraft,
  projectContentDiff,
  projectContentImportAttempt,
  projectContentPackReview,
  projectContentWorkspace,
  type RulebenchContentActionBindingCandidateView,
  type RulebenchContentAuthoringDraftView,
  type RulebenchContentDiffView,
  type RulebenchContentImportAttemptView,
  type RulebenchContentPackReviewView,
  type RulebenchContentWorkspaceView,
} from "@asha-rulebench/domain";
import { browserClock, type ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchContentDraftIdentityDto,
  RulebenchContentPackReferenceDto,
  RulebenchLiveTransportErrorDto,
} from "@asha-rulebench/protocol";
import type { RulebenchLiveTransport } from "@asha-rulebench/transport";
import type { AsyncState } from "./async-state";
import { RULEBENCH_LIVE_TRANSPORT } from "./live-combat-store";

type ContentState<T> = AsyncState<T, RulebenchLiveTransportErrorDto>;

export type ContentDraftSyntaxView =
  | { readonly kind: "empty"; readonly message: string }
  | { readonly kind: "valid"; readonly message: string }
  | { readonly kind: "error"; readonly message: string };

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
  private readonly _validation = signal<ContentState<RulebenchContentImportAttemptView>>({
    kind: "idle",
  });
  readonly validation = this._validation.asReadonly();
  private readonly _review = signal<ContentState<RulebenchContentPackReviewView>>({
    kind: "idle",
  });
  readonly review = this._review.asReadonly();
  private readonly _diff = signal<ContentState<RulebenchContentDiffView>>({ kind: "idle" });
  readonly diff = this._diff.asReadonly();
  private readonly _draft = signal<ContentState<RulebenchContentAuthoringDraftView>>({
    kind: "idle",
  });
  readonly draft = this._draft.asReadonly();
  private readonly _bindingCatalog = signal<
    ContentState<readonly RulebenchContentActionBindingCandidateView[]>
  >({ kind: "idle" });
  readonly bindingCatalog = this._bindingCatalog.asReadonly();
  private readonly _selectedReference = signal<RulebenchContentPackReferenceDto | null>(null);
  readonly selectedReference = this._selectedReference.asReadonly();
  private readonly _draftPayload = signal<string | null>(null);
  readonly stagedPayload = this._draftPayload.asReadonly();
  readonly draftPayload = this._draftPayload.asReadonly();
  private readonly _draftIdentity = signal<RulebenchContentDraftIdentityDto>({
    id: "pack.draft.authored.v3",
    version: "0.1.0",
  });
  readonly draftIdentity = this._draftIdentity.asReadonly();
  private readonly _draftSyntax = signal<ContentDraftSyntaxView>({
    kind: "empty",
    message: "Enter or load a JSON document.",
  });
  readonly draftSyntax = this._draftSyntax.asReadonly();
  private readonly _validatedPayload = signal<string | null>(null);
  readonly canImportDraft = computed(() => {
    const payload = this._draftPayload();
    return payload !== null && this._validatedPayload() === payload;
  });

  private requestGeneration = 0;
  private draftGeneration = 0;
  private bindingGeneration = 0;
  private draftController: AbortController | null = null;
  private validationController: AbortController | null = null;
  private bindingController: AbortController | null = null;

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

  async loadBindingCatalog(): Promise<void> {
    const generation = ++this.bindingGeneration;
    this.bindingController?.abort();
    const controller = new AbortController();
    this.bindingController = controller;
    this._bindingCatalog.set({ kind: "loading" });
    const result = await this.transport.listContentActionBindings({
      signal: controller.signal,
    });
    if (generation !== this.bindingGeneration) return;
    this.bindingController = null;
    this._bindingCatalog.set(
      result.ok
        ? {
            kind: "data",
            value: projectContentActionBindingCatalog(result.value),
          }
        : { kind: "error", error: result.error },
    );
    this.clock.now();
  }

  setDraftIdentity(id: string, version: string): void {
    const current = this._draftIdentity();
    if (current.id === id && current.version === version) return;

    this.draftGeneration += 1;
    this.draftController?.abort();
    this.draftController = null;
    this.invalidateValidationRequest();
    if (this._draft().kind === "loading") {
      this._draft.set({ kind: "idle" });
    }
    this._draftIdentity.set({ id, version });
    this.clock.now();
  }

  async startTemplateDraft(): Promise<void> {
    await this.loadDraft((signal) =>
      this.transport.createContentTemplateDraft(this._draftIdentity(), { signal }),
    );
  }

  async cloneSelectedDraft(): Promise<void> {
    const reference = this._selectedReference();
    if (reference === null) return;
    await this.loadDraft((signal) =>
      this.transport.cloneContentDraft(reference, this._draftIdentity(), { signal }),
    );
  }

  stagePayload(payload: string): void {
    this.applyDraftPayload(payload);
    this._draft.set({ kind: "idle" });
  }

  updateDraftPayload(payload: string): void {
    this.applyDraftPayload(payload);
  }

  async validateDraft(): Promise<void> {
    const payload = this._draftPayload();
    if (payload === null || this._draftSyntax().kind !== "valid") return;
    const generation = ++this.draftGeneration;
    this.validationController?.abort();
    const controller = new AbortController();
    this.validationController = controller;
    this._validation.set({ kind: "loading" });
    const result = await this.transport.validateContent(payload, {
      signal: controller.signal,
    });
    if (generation !== this.draftGeneration) return;
    this.validationController = null;
    this._validation.set(
      result.ok
        ? { kind: "data", value: projectContentImportAttempt(result.value) }
        : { kind: "error", error: result.error },
    );
    this._validatedPayload.set(
      result.ok && result.value.accepted ? payload : null,
    );
    this.clock.now();
  }

  async importStaged(replaceSameIdentity = false): Promise<void> {
    const payload = this._draftPayload();
    if (payload === null || this._validatedPayload() !== payload) return;
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
      await Promise.all([
        this.refreshWorkspaceAfterMutation(generation),
        this.loadBindingCatalog(),
      ]);
    }
    this.clock.now();
  }

  async compareStaged(): Promise<void> {
    const payload = this._draftPayload();
    if (payload === null || this._validatedPayload() !== payload) return;
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

  private async loadDraft(
    request: (
      signal: AbortSignal,
    ) => ReturnType<RulebenchLiveTransport["createContentTemplateDraft"]>,
  ): Promise<void> {
    const generation = ++this.draftGeneration;
    this.draftController?.abort();
    this.invalidateValidationRequest();
    const controller = new AbortController();
    this.draftController = controller;
    this._draft.set({ kind: "loading" });
    const result = await request(controller.signal);
    if (generation !== this.draftGeneration) return;
    this.draftController = null;
    if (!result.ok) {
      this._draft.set({ kind: "error", error: result.error });
      this.clock.now();
      return;
    }
    const draft = projectContentAuthoringDraft(result.value);
    this._draft.set({ kind: "data", value: draft });
    this.applyDraftPayload(draft.authoredPayload, false);
  }

  private applyDraftPayload(
    payload: string,
    advanceGeneration = true,
  ): void {
    if (advanceGeneration) this.draftGeneration += 1;
    this.requestGeneration += 1;
    this.draftController?.abort();
    this.invalidateValidationRequest();
    this._draftPayload.set(payload);
    this._draftSyntax.set(inspectJsonSyntax(payload));
    this._validatedPayload.set(null);
    this._validation.set({ kind: "idle" });
    this._importAttempt.set({ kind: "idle" });
    this._diff.set({ kind: "idle" });
    this.clock.now();
  }

  private invalidateValidationRequest(): void {
    this.validationController?.abort();
    this.validationController = null;
    if (this._validation().kind === "loading") {
      this._validation.set({ kind: "idle" });
    }
  }

  private async mutateSelected(
    operation: "activate" | "deactivate" | "delete",
  ): Promise<void> {
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
    await this.loadBindingCatalog();
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

function inspectJsonSyntax(payload: string): ContentDraftSyntaxView {
  if (payload.trim().length === 0) {
    return { kind: "empty", message: "Enter or load a JSON document." };
  }
  try {
    JSON.parse(payload);
    return {
      kind: "valid",
      message: "JSON syntax is valid. Rust semantic validation has not been inferred.",
    };
  } catch (error: unknown) {
    return {
      kind: "error",
      message: error instanceof Error ? error.message : "Invalid JSON syntax.",
    };
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
