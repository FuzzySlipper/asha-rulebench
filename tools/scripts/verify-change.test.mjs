import assert from "node:assert/strict";
import test from "node:test";
import {
  buildVerifyChangePlan,
  parseVerifyChangeArguments,
} from "./verify-change.mjs";

test("verify:change rejects missing and unknown profiles", () => {
  assert.throws(
    () => parseVerifyChangeArguments([]),
    /At least one --profile is required/,
  );
  assert.throws(
    () => parseVerifyChangeArguments(["--profile", "automatic"]),
    /Unknown verify:change profile/,
  );
});

test("verify:change accepts the pnpm argument separator", () => {
  const selection = parseVerifyChangeArguments([
    "--",
    "--profile",
    "docs",
    "--dry-run",
  ]);

  assert.deepEqual(selection.profiles, ["docs"]);
  assert.equal(selection.dryRun, true);
});

test("verify:change unions profiles and deduplicates shared commands", () => {
  const selection = parseVerifyChangeArguments([
    "--profile",
    "frontend",
    "--profile",
    "browser",
  ]);
  const commandIds = buildVerifyChangePlan(selection).map((entry) => entry.id);

  assert.equal(commandIds.filter((id) => id === "pnpm:typecheck").length, 1);
  assert.ok(commandIds.includes("pnpm:check:typescript-authority"));
  assert.ok(commandIds.includes("pnpm:check:rules-language-boundary"));
  assert.ok(commandIds.includes("pnpm:e2e:gate"));
});

test("rust-owner requires an exact governed crate", () => {
  assert.throws(
    () => parseVerifyChangeArguments(["--profile", "rust-owner"]),
    /--crate is required/,
  );
  assert.throws(
    () =>
      parseVerifyChangeArguments([
        "--profile",
        "rust-owner",
        "--crate",
        "rulebench-unknown",
      ]),
    /Unknown or ungoverned Rust crate/,
  );
});

test("fixtures profile preserves exact filters and defaults to the full corpus", () => {
  const filtered = parseVerifyChangeArguments([
    "--profile",
    "fixtures-conformance",
    "--scenario",
    "hexing-bolt-reaction",
  ]);
  const filteredRegression = buildVerifyChangePlan(filtered).find((entry) =>
    entry.id.startsWith("regression:"),
  );
  assert.deepEqual(filteredRegression.arguments.slice(-2), [
    "--scenario",
    "hexing-bolt-reaction",
  ]);

  const unfiltered = parseVerifyChangeArguments([
    "--profile",
    "fixtures-conformance",
  ]);
  const unfilteredRegression = buildVerifyChangePlan(unfiltered).find((entry) =>
    entry.id.startsWith("regression:"),
  );
  assert.equal(unfilteredRegression.arguments.at(-1), "--");
});

test("identity filters fail closed outside fixtures-conformance", () => {
  assert.throws(
    () =>
      parseVerifyChangeArguments([
        "--profile",
        "frontend",
        "--capability",
        "operation.damage",
      ]),
    /valid only with fixtures-conformance/,
  );
});
