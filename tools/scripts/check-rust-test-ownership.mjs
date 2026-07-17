import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const rustRoot = join(root, "rulebench-rs");
const owningCrates = [
  "rulebench-content",
  "rulebench-combat",
  "rulebench-replay",
  "rulebench-protocol",
  "rulebench-bridge",
  "rulebench-fixtures",
  "rulebench-codegen",
];
const failures = [];

for (const crate of owningCrates) {
  const sourceRoot = join(rustRoot, "crates", crate, "src");
  const testCount = collectRustFiles(sourceRoot)
    .map((file) => countTests(readFileSync(file, "utf8")))
    .reduce((sum, count) => sum + count, 0);
  if (testCount === 0) {
    failures.push(
      `${crate} has no focused owning-crate #[test] coverage under ${relative(root, sourceRoot)}.`,
    );
  }
}

const authorityHarness = join(
  rustRoot,
  "crates",
  "rulebench-authority",
  "src",
  "tests",
);
const authorityTestCount = collectRustFiles(authorityHarness)
  .map((file) => countTests(readFileSync(file, "utf8")))
  .reduce((sum, count) => sum + count, 0);
if (authorityTestCount === 0) {
  failures.push(
    "rulebench-authority must retain cross-crate product harness tests under src/tests.",
  );
}

runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(
  `check:rust-test-ownership ok (${owningCrates.length} owners, ${authorityTestCount} authority harness tests)`,
);

function collectRustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      files.push(...collectRustFiles(path));
    } else if (entry.endsWith(".rs")) {
      files.push(path);
    }
  }
  return files;
}

function countTests(source) {
  return [...source.matchAll(/^\s*#\[test]\s*$/gm)].length;
}

function runFocusedFailureTests() {
  if (countTests("fn helper() {}") !== 0) {
    throw new Error(
      "Test-ownership self-test failed: ordinary functions count as tests.",
    );
  }
  if (countTests("#[test]\nfn focused_contract() {}") !== 1) {
    throw new Error(
      "Test-ownership self-test failed: a focused Rust test was not counted.",
    );
  }
}
