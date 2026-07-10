import { describe, expect, it } from "vitest";
import type { RulebenchReplayComparisonReadoutDto, RulebenchReplayPackageReviewDto, RulebenchReplayVerificationReadoutDto } from "@asha-rulebench/protocol";
import { comparisonKey, createReplayReviewTransport } from "./replay-review";

describe("replay review transport", () => {
  it("returns authority evidence without validating or rebuilding it", async () => {
    const review = packageReview();
    const verification = verificationReadout();
    const comparison = comparisonReadout();
    const transport = createReplayReviewTransport({ packages: [], reviews: { replay: review }, verifications: { replay: verification }, comparisons: { [comparisonKey("replay", "actual")]: comparison } });

    const loadedReview = await transport.loadReplayPackage("replay");
    const loadedVerification = await transport.loadReplayVerification("replay");
    const loadedComparison = await transport.compareReplayPackages("replay", "actual");

    expect(loadedReview.ok && loadedReview.value).toBe(review);
    expect(loadedVerification.ok && loadedVerification.value).toBe(verification);
    expect(loadedComparison.ok && loadedComparison.value).toBe(comparison);
  });
});

export const packageReview = (): RulebenchReplayPackageReviewDto => ({
  packageVersion: "1.0.0", packageId: "replay", sessionId: "session", scenarioId: "scenario", rulesetId: "rules", rulesetVersion: "1.0.0", commandCount: 2,
  finalStateFingerprint: { algorithm: "test", value: "final" }, fingerprintKind: "deterministicNonCryptographic", narrationTitle: null, narrationSummary: null,
});

export const verificationReadout = (): RulebenchReplayVerificationReadoutDto => ({
  accepted: false, decisionKind: "mismatchedEvidence", verifiedStepCount: 1, finalized: false,
  finalStateFingerprint: { algorithm: "test", value: "actual" }, mismatch: { commandSequence: 1, commandId: "second", dimension: "rolls", reason: "Rolls differed." },
});

export const comparisonReadout = (): RulebenchReplayComparisonReadoutDto => {
  const difference = { code: "replayRollsMismatch" as const, path: "commands[1].expected.rolls", commandSequence: 1, commandId: "second", expectedSummary: "[10]", actualSummary: "[11]" };
  return { matches: false, expectedPackageId: "replay", actualPackageId: "actual", comparedCommandCount: 2, firstDifference: difference, differences: [difference] };
};
