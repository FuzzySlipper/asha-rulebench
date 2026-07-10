import { describe, expect, it } from "vitest";
import type { RulebenchReplayComparisonReadoutDto, RulebenchReplayPackageReviewDto, RulebenchReplayVerificationReadoutDto } from "@asha-rulebench/protocol";
import { comparisonKey, createReplayReviewTransport } from "@asha-rulebench/transport";
import { ReplayReviewStore } from "./replay-review-store";

describe("ReplayReviewStore", () => {
  it("stores projected authority verification and comparison evidence", async () => {
    const review: RulebenchReplayPackageReviewDto = {
      packageVersion: "1.0.0", packageId: "replay", sessionId: "session", scenarioId: "scenario", rulesetId: "rules", rulesetVersion: "1.0.0", commandCount: 2,
      finalStateFingerprint: { algorithm: "test", value: "final" }, fingerprintKind: "deterministicNonCryptographic", narrationTitle: null, narrationSummary: null, commands: [],
    };
    const verification: RulebenchReplayVerificationReadoutDto = {
      accepted: false, decisionKind: "mismatchedEvidence", verifiedStepCount: 1, finalized: false,
      finalStateFingerprint: { algorithm: "test", value: "actual" }, mismatch: { commandSequence: 1, commandId: "second", dimension: "rolls", reason: "Rolls differed." },
    };
    const difference = { code: "replayRollsMismatch" as const, path: "commands[1].expected.rolls", commandSequence: 1, commandId: "second", expectedSummary: "[10]", actualSummary: "[11]" };
    const comparison: RulebenchReplayComparisonReadoutDto = { matches: false, expectedPackageId: "replay", actualPackageId: "actual", comparedCommandCount: 2, firstDifference: difference, differences: [difference] };
    const transport = createReplayReviewTransport({ packages: [], reviews: { replay: review }, verifications: { replay: verification }, comparisons: { [comparisonKey("replay", "actual")]: comparison } });
    const store = new ReplayReviewStore(transport, { now: () => 0 });

    await store.loadReview("replay");
    await store.loadVerification("replay");
    await store.compare("replay", "actual");

    expect(store.review()).toEqual({ kind: "data", value: {
      packageId: "replay", title: "replay", summary: "No replay narration supplied.",
      provenanceLabel: "scenario · rules 1.0.0 · package 1.0.0",
      finalFingerprintLabel: "test:final", commands: [],
    } });
    expect(store.verification()).toEqual({ kind: "data", value: {
      accepted: false, decisionLabel: "Mismatched Evidence", verifiedStepLabel: "1 steps verified",
      finalizedLabel: "Not finalized", mismatchLabel: "Rolls · Rolls differed.", fingerprintLabel: "test:actual",
    } });
    expect(store.comparison()).toEqual({ kind: "data", value: expect.objectContaining({
      matches: false, resultLabel: "Differences found", packageLabel: "replay vs actual",
      firstDifference: expect.objectContaining({ path: "commands[1].expected.rolls", expectedSummary: "[10]", actualSummary: "[11]" }),
    }) });
  });
});
