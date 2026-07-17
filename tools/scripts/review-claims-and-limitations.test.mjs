import assert from "node:assert/strict";
import test from "node:test";
import {
  buildGovernanceReceipt,
  parseReviewArguments,
  validateGovernanceReview,
} from "./review-claims-and-limitations.mjs";

const review = {
  schemaVersion: 2,
  authority: {
    claimsDocuments: [{ projectId: "asha-rulebench", slug: "basic-design" }],
    limitationsDocument: {
      projectId: "asha-rulebench",
      slug: "known-limitations",
    },
  },
  limitationSnapshot: {
    sourceDocumentUpdatedAt: "2026-07-17T03:56:16.098Z",
    reviewedAt: "2026-07-17T05:38:03.267Z",
    reviewedBy: "fixture-reviewer",
    active: [{ id: "trusted-local", heading: "Trusted local" }],
    resolved: [{ id: "disconnect", heading: "Disconnect", resolvedByTask: 1 }],
  },
  freshnessPolicy:
    "report-snapshot-provenance-without-blocking-unrelated-source-edits",
};

test("governance review validates handles and limitation identities without literal prose counts", () => {
  assert.deepEqual(validateGovernanceReview(review), []);
  assert.equal("reviewedOn" in review, false);
  assert.equal("requiredClaims" in review, false);
});

test("governance review rejects duplicate and contradictory limitation identities", () => {
  const invalid = structuredClone(review);
  invalid.limitationSnapshot.active.push({
    id: "trusted-local",
    heading: "Duplicate",
  });
  invalid.limitationSnapshot.resolved.push({
    id: "trusted-local",
    heading: "Contradictory",
  });

  const failures = validateGovernanceReview(invalid);
  assert.ok(failures.some((failure) => failure.includes("duplicate active")));
  assert.ok(
    failures.some((failure) => failure.includes("both active and resolved")),
  );
});

test("governance receipt derives current counts instead of storing expected literals", () => {
  const capabilityManifest = {
    protocolId: "asha-rulebench.protocol",
    protocolVersion: 9,
    governedAshaRevision: "a".repeat(40),
    providers: [{}, {}],
    rulesets: [{}, {}],
    packages: [{}, {}, {}],
    scenarios: [{}],
    capabilities: [
      { support: { runtimeExecutable: true, regressionCovered: true } },
      { support: { runtimeExecutable: true, regressionCovered: false } },
    ],
  };
  const receipt = buildGovernanceReceipt({
    review,
    capabilityManifest,
    sourceCommit: "b".repeat(40),
    sourceTreeDirty: false,
    reviewer: "fixture-runner",
    reviewedAt: "2026-07-17T06:00:00.000Z",
  });

  assert.equal(receipt.executableInventory.providers, 2);
  assert.equal(receipt.executableInventory.capabilities, 2);
  assert.equal(receipt.executableInventory.regressionCoveredCapabilities, 1);
  assert.equal(receipt.limitations.freshnessEnforcedBySourceGate, false);
});

test("claims review output selection fails closed", () => {
  assert.deepEqual(parseReviewArguments(["--output", "local/review.json"]), {
    output: "local/review.json",
  });
  assert.throws(() => parseReviewArguments(["--output"]), /requires a path/);
  assert.throws(
    () => parseReviewArguments(["--today"]),
    /Unknown claims-review argument/,
  );
});
