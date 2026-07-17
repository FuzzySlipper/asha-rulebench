import assert from "node:assert/strict";
import test from "node:test";

import {
  declaredSurfaceFiles,
  inspectContentOnlyChange,
  inspectDeclaredRulesLanguage,
} from "./check-rules-language-boundary.mjs";
import { readFileSync } from "node:fs";

const generated =
  "rulebench-rs/crates/rulebench-content/src/generated/representative-rpg-content.json";

test("content-only action changes stay within three downstream layers", () => {
  const report = inspectContentOnlyChange([
    declaredSurfaceFiles[0],
    declaredSurfaceFiles[1],
    generated,
  ]);

  assert.deepEqual(report.failures, []);
  assert.equal(report.contentOnlyLayerCount, 3);
});

test("content-only classification rejects Rust protocol host and proof amplification", () => {
  for (const forbidden of [
    "rulebench-rs/crates/rulebench-combat/src/frost_bolt.rs",
    "libs/protocol/src/frost-bolt.ts",
    "rulebench-rs/hosts/rulebench-process-host/src/frost_bolt.rs",
    "rulebench-rs/crates/rulebench-protocol/src/capability_manifest.rs",
    ".github/workflows/certification.yml",
  ]) {
    const report = inspectContentOnlyChange([
      "libs/content-authoring/src/frost-bolt.ts",
      "libs/content-authoring/src/frost-bolt.spec.ts",
      generated,
      forbidden,
    ]);
    assert.ok(
      report.failures.some((entry) => entry.includes(forbidden)),
      forbidden,
    );
  }
});

test("content-only classification fails closed when generated IR is omitted", () => {
  const report = inspectContentOnlyChange([
    "libs/content-authoring/src/frost-bolt.ts",
    "libs/content-authoring/src/frost-bolt.spec.ts",
  ]);

  assert.ok(
    report.failures.some((entry) => entry.includes("normalized artifact")),
  );
});

test("declared content reaches the persistent user-facing authority path", () => {
  const report = inspectDeclaredRulesLanguage((file) =>
    readFileSync(file, "utf8"),
  );

  assert.deepEqual(report.failures, []);
  assert.ok(report.actionCount > 0);
  assert.equal(report.actionCount, report.bindingCount);
});
