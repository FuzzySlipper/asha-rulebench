import { describe, expect, it } from "vitest";
import type { ClockPort } from "@asha-rulebench/platform";
import type {
  RulebenchContentPackReferenceDto,
  RulebenchContentPackReviewDto,
  RulebenchContentWorkspaceDto,
} from "@asha-rulebench/protocol";
import { createFakeRulebenchLiveTransport } from "@asha-rulebench/transport";
import { ContentWorkbenchStore } from "./content-workspace-store";

const clock: ClockPort = {
  now: () => new Date("2026-07-15T00:00:00.000Z"),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

const firstReference: RulebenchContentPackReferenceDto = {
  id: "pack.first",
  version: "1.0.0",
  fingerprint: { algorithm: "fnv1a64", value: "first" },
};
const secondReference: RulebenchContentPackReferenceDto = {
  id: "pack.second",
  version: "1.0.0",
  fingerprint: { algorithm: "fnv1a64", value: "second" },
};

describe("ContentWorkbenchStore", () => {
  it("keeps the last active workspace after a rejected Rust import", async () => {
    const initial = workspace(firstReference, true);
    const transport = createFakeRulebenchLiveTransport({
      listContentWorkspace: async () => ({ ok: true, value: initial }),
      validateContent: async () => ({
        ok: true,
        value: acceptedValidation("pack.first", "1.0.0"),
      }),
      importContent: async () => ({
        ok: true,
        value: {
          accepted: false,
          pack: { id: "pack.first", version: "1.0.0", fingerprint: null },
          outcome: null,
          diagnostics: [
            {
              severity: "error",
              code: "unsupportedAuthoredContentVersion",
              path: "formatVersion",
              referenceId: "pack.first",
              definitionKind: null,
              message: "Unsupported version.",
            },
          ],
          errorCode: "unsupportedAuthoredContentVersion",
          errorMessage: "Unsupported version.",
        },
      }),
    });
    const store = new ContentWorkbenchStore(transport, clock);

    await store.loadWorkspace();
    store.stagePayload("{}");
    await store.validateDraft();
    await store.importStaged(true);

    expect(store.workspace()).toMatchObject({
      kind: "data",
      value: { packs: [{ identityLabel: "pack.first@1.0.0", active: true }] },
    });
    expect(store.importAttempt()).toMatchObject({
      kind: "data",
      value: {
        accepted: false,
        diagnostics: [{ code: "unsupportedAuthoredContentVersion" }],
      },
    });
  });

  it("keeps JSON syntax and Rust semantic validation as separate draft states", async () => {
    const payload = '{"formatVersion":3,"pack":{"id":"pack.draft"}}';
    const transport = createFakeRulebenchLiveTransport({
      createContentTemplateDraft: async (identity) => ({
        ok: true,
        value: {
          authoredPayload: payload,
          sourceKind: "rustTemplate",
          sourceLabel: "Rust v3 authored-action starter",
          identity,
          identityExpectation: `New content identity ${identity.id}@${identity.version}.`,
        },
      }),
      validateContent: async () => ({
        ok: true,
        value: {
          accepted: false,
          pack: { id: "pack.draft", version: "0.1.0", fingerprint: null },
          outcome: null,
          diagnostics: [
            {
              severity: "error",
              code: "missingAuthoredContentField",
              path: "pack.catalogs",
              referenceId: "pack.draft",
              definitionKind: null,
              message: "The Rust authority requires pack.catalogs.",
            },
          ],
          errorCode: "missingAuthoredContentField",
          errorMessage: "The Rust authority requires pack.catalogs.",
        },
      }),
    });
    const store = new ContentWorkbenchStore(transport, clock);

    store.setDraftIdentity("pack.draft", "0.1.0");
    await store.startTemplateDraft();

    expect(store.draft()).toMatchObject({
      kind: "data",
      value: {
        sourceLabel: "Rust v3 authored-action starter",
        authoredPayload: payload,
      },
    });
    expect(store.draftSyntax()).toEqual({
      kind: "valid",
      message: "JSON syntax is valid. Rust semantic validation has not been inferred.",
    });
    expect(store.validation()).toEqual({ kind: "idle" });
    expect(store.canImportDraft()).toBe(false);

    await store.validateDraft();

    expect(store.validation()).toMatchObject({
      kind: "data",
      value: {
        accepted: false,
        diagnostics: [
          {
            code: "missingAuthoredContentField",
            locationLabel: "pack.catalogs / pack.draft",
            message: "The Rust authority requires pack.catalogs.",
          },
        ],
      },
    });
    expect(store.canImportDraft()).toBe(false);

    store.updateDraftPayload("{");
    expect(store.draftSyntax()).toMatchObject({ kind: "error" });
    expect(store.validation()).toEqual({ kind: "idle" });
  });

  it("suppresses a stale semantic validation after the draft changes", async () => {
    let resolveValidation:
      | ((value: { ok: true; value: ReturnType<typeof acceptedValidation> }) => void)
      | null = null;
    const validation = new Promise<{
      ok: true;
      value: ReturnType<typeof acceptedValidation>;
    }>((resolve) => {
      resolveValidation = resolve;
    });
    const transport = createFakeRulebenchLiveTransport({
      validateContent: async () => validation,
    });
    const store = new ContentWorkbenchStore(transport, clock);
    store.stagePayload("{}");

    const pending = store.validateDraft();
    store.updateDraftPayload('{"changed":true}');
    const completeValidation = resolveValidation;
    if (completeValidation === null) throw new Error("validation was not requested");
    completeValidation({
      ok: true,
      value: acceptedValidation("pack.old", "1.0.0"),
    });
    await pending;

    expect(store.validation()).toEqual({ kind: "idle" });
    expect(store.canImportDraft()).toBe(false);
  });

  it("ignores an older review response after a newer pack selection", async () => {
    let resolveFirst: ((value: { ok: true; value: RulebenchContentPackReviewDto }) => void) | null = null;
    const firstReview = new Promise<{ ok: true; value: RulebenchContentPackReviewDto }>((resolve) => {
      resolveFirst = resolve;
    });
    const transport = createFakeRulebenchLiveTransport({
      reviewContent: async (reference) =>
        reference.id === firstReference.id
          ? firstReview
          : { ok: true, value: review(secondReference, "Second") },
    });
    const store = new ContentWorkbenchStore(transport, clock);

    const older = store.selectPack(firstReference);
    await store.selectPack(secondReference);
    const completeFirst = resolveFirst;
    if (completeFirst === null) throw new Error("first review was not requested");
    completeFirst({ ok: true, value: review(firstReference, "First") });
    await older;

    expect(store.review()).toMatchObject({
      kind: "data",
      value: { pack: { title: "Second", identityLabel: "pack.second@1.0.0" } },
    });
  });
});

function workspace(
  reference: RulebenchContentPackReferenceDto,
  active: boolean,
): RulebenchContentWorkspaceDto {
  return {
    packs: [{ ...review(reference, "First").pack, active }],
    audit: [],
  };
}

function review(
  reference: RulebenchContentPackReferenceDto,
  title: string,
): RulebenchContentPackReviewDto {
  return {
    pack: {
      reference,
      title,
      summary: "Stored authored pack.",
      sourceKind: "authoredFile",
      sourceId: "test",
      authoredBy: null,
      rulesetId: "rules.test",
      rulesetVersion: "1.0.0",
      dependencies: [],
      definitions: [{ kind: "entity", id: `entity.${title.toLowerCase()}` }],
      active: reference.id === firstReference.id,
    },
    authoredPayload: "{}",
    diagnostics: [],
    abilities: [],
    modifiers: [],
    actions: [],
  };
}

function acceptedValidation(
  id: string,
  version: string,
) {
  return {
    accepted: true,
    pack: {
      id,
      version,
      fingerprint: { algorithm: "fnv1a64", value: "validated" },
    },
    outcome: null,
    diagnostics: [],
    errorCode: null,
    errorMessage: null,
  } as const;
}
