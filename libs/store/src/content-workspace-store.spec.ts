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
  };
}
