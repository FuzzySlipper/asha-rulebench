import assert from "node:assert/strict";
import test from "node:test";
import {
  buildCertificationPlan,
  parseCertificationArguments,
} from "./certify.mjs";

test("certify rejects unknown arguments", () => {
  assert.throws(
    () => parseCertificationArguments(["--fast"], {}),
    /Unknown certify argument/,
  );
});

test("certify deterministic mode composes every exhaustive local group once", () => {
  const selection = parseCertificationArguments(["--dry-run"], {});
  const groupIds = buildCertificationPlan(selection).map((group) => group.id);

  assert.deepEqual(groupIds, [
    "authority-and-product-contracts",
    "exhaustive-semantic-corpus",
    "deterministic-browser-certification",
    "claims-and-limitations-receipt",
  ]);
  assert.equal(new Set(groupIds).size, groupIds.length);
});

test("certify live-required mode fails closed before building a plan", () => {
  assert.throws(
    () => parseCertificationArguments(["--require-live"], {}),
    /LIVE_RUN=1/,
  );
  assert.throws(
    () => parseCertificationArguments(["--require-live"], { LIVE_RUN: "1" }),
    /requires BASE_URL/,
  );
  assert.throws(
    () =>
      parseCertificationArguments(["--require-live"], {
        LIVE_RUN: "1",
        BASE_URL: "file:///tmp/rulebench",
      }),
    /HTTP\(S\)/,
  );
});

test("certify live-required mode appends the artifact group before its receipt", () => {
  const selection = parseCertificationArguments(["--require-live"], {
    LIVE_RUN: "1",
    BASE_URL: "http://127.0.0.1:37300/",
  });
  const groupIds = buildCertificationPlan(selection).map((group) => group.id);

  assert.deepEqual(groupIds.slice(-2), [
    "managed-live-artifacts",
    "claims-and-limitations-receipt",
  ]);
});
