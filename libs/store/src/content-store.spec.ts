import { describe, expect, it } from "vitest";
import type { ClockPort } from "@asha-rulebench/platform";
import type { ClassifiedError } from "@asha-rulebench/protocol";
import { createFakeRulebenchTransport } from "@asha-rulebench/transport";
import { ContentStore } from "./index";

const fixedClock: ClockPort = {
  now: () => new Date("2026-07-10T00:00:00.000Z"),
  setTimeout: () => 1,
  clearTimeout: () => undefined,
};

describe("ContentStore", () => {
  it("carries Rust valid, warning, and error import outcomes into workbench state", async () => {
    const store = new ContentStore(createFakeRulebenchTransport(), fixedClock);

    await store.loadImportExamples();

    const state = store.imports();
    expect(state.kind).toBe("data");
    if (state.kind !== "data") {
      return;
    }
    expect(state.value.map((example) => example.statusLabel)).toEqual([
      "Accepted",
      "Accepted",
      "Rejected",
    ]);
    expect(state.value[0]).toMatchObject({
      exampleId: "content-import-valid",
      errorCount: 0,
      warningCount: 0,
    });
    expect(state.value[1]?.diagnostics[0]).toMatchObject({
      severityLabel: "Warning",
      code: "duplicateContentTagCanonicalized",
      locationLabel: "tags / fixture",
    });
    expect(state.value[2]?.diagnostics[0]).toMatchObject({
      severityLabel: "Error",
      code: "missingContentPackDependency",
      locationLabel: "pack / pack.dependency",
    });
  });

  it("classifies transport failures through AsyncState without TS rule inference", async () => {
    const error: ClassifiedError = {
      kind: "network",
      message: "Content authority unavailable",
      retryable: true,
    };
    const transport = {
      ...createFakeRulebenchTransport(),
      loadContentImportExamples: async () => ({ ok: false as const, error }),
    };
    const store = new ContentStore(transport, fixedClock);

    await store.loadImportExamples();

    expect(store.imports()).toEqual({ kind: "error", error });
  });
});
