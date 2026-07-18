import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const boundaries = JSON.parse(
  readFileSync(join(root, "boundaries.json"), "utf8"),
);
const knownScopes = new Set(boundaries.scopes);
const failures = [];
const requiredFiles = [
  "boundaries.json",
  ".playwright-service.json",
  "AGENTS.md",
  "apps/app-e2e/src/live/support/artifact-collector.ts",
  "apps/app-e2e/src/live/support/live-gate.ts",
  "apps/app-e2e/src/live/support/visual-impact.ts",
  "apps/app-e2e/src/live/boot.live.spec.ts",
  "apps/app-e2e/src/live/docs/live-testing.md",
];

for (const file of requiredFiles) {
  if (!exists(file)) {
    failures.push(`Missing required pattern file: ${file}`);
  }
}

runCheck("node", ["tools/scripts/gen-eslint-boundaries.mjs", "--check"]);

const libs = readdirSync(join(root, "libs")).filter((entry) =>
  statSync(join(root, "libs", entry)).isDirectory(),
);
for (const lib of libs) {
  validateLib(lib);
}

validateSourceImports();
validateLiveHarness();
parseJson(".playwright-service.json");
runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`check:pattern ok (${libs.length} libs)`);

function validateLib(lib) {
  const project = parseJson(`libs/${lib}/project.json`);
  if (project === null) {
    failures.push(`Missing or invalid project.json for libs/${lib}`);
    return;
  }
  const tags = new Set(project.tags ?? []);
  const typeTags = [...tags].filter((tag) => tag.startsWith("type:"));
  const scopeTags = [...tags].filter((tag) => tag.startsWith("scope:"));
  if (typeTags.length !== 1)
    failures.push(`libs/${lib} must have exactly one type: tag`);
  if (scopeTags.length !== 1)
    failures.push(`libs/${lib} must have exactly one scope: tag`);
  const scopeTag = scopeTags[0];
  const scope =
    scopeTag === undefined ? undefined : scopeTag.slice("scope:".length);
  if (scope !== undefined && !knownScopes.has(scope))
    failures.push(`libs/${lib} has unknown scope: ${scope}`);
  if (!exists(`libs/${lib}/src/index.ts`))
    failures.push(`libs/${lib} must expose libs/${lib}/src/index.ts`);
}

function validateSourceImports() {
  const files = collectFiles(
    ["apps", "libs", "tools"],
    [".ts", ".mts", ".mjs", ".js"],
  );
  for (const file of files) {
    const rel = relative(root, file);
    const text = readFileSync(file, "utf8");
    failures.push(...sourceImportFailures(rel, text));
  }
}

function sourceImportFailures(rel, text) {
  const importFailures = [];
  if (/@(?:template|asha-rulebench)\/[^'"]+\/src\//.test(text)) {
    importFailures.push(`${rel} deep-imports another lib src path`);
  }
  if (/from ['"](?:\.\.\/)*apps\//.test(text)) {
    importFailures.push(`${rel} imports from apps/`);
  }
  if (
    !rel.startsWith("libs/testing-fixtures/") &&
    !rel.includes(".spec.") &&
    !rel.startsWith("apps/app-e2e/") &&
    /(?:from|import)\s*['"]@(?:template|asha-rulebench)\/testing-fixtures['"]/.test(
      text,
    )
  ) {
    importFailures.push(`${rel} imports testing-fixtures from production code`);
  }
  return importFailures;
}

function validateLiveHarness() {
  for (const file of collectFiles(["apps/app-e2e/src/live"], [".ts"])) {
    if (!file.endsWith(".live.spec.ts")) continue;
    const rel = relative(root, file);
    const text = readFileSync(file, "utf8");
    if (!text.includes("liveScenario") && !text.includes("requireLiveRun")) {
      failures.push(`${rel} must be LIVE_RUN gated`);
    }
    if (!text.includes("collector")) {
      failures.push(`${rel} must use the artifact collector`);
    }
    if (!text.includes("@live-artifact")) {
      failures.push(`${rel} must carry the @live-artifact certification tag`);
    }
  }

  for (const file of collectFiles(
    ["apps/app-e2e/src/integration", "apps/app-e2e/src/smoke"],
    [".ts"],
  )) {
    const rel = relative(root, file);
    const text = readFileSync(file, "utf8");
    if (text.includes("@live-artifact")) {
      failures.push(
        `${rel} mixes deterministic browser proof with the managed live-artifact group`,
      );
    }
  }

  for (const lib of libs) {
    if (
      lib.startsWith("feature-") &&
      !exists(`apps/app-e2e/src/live/${lib}.live.spec.ts`)
    ) {
      failures.push(
        `Feature lib ${lib} must have apps/app-e2e/src/live/${lib}.live.spec.ts`,
      );
    }
  }
}

function runCheck(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8" });
  if (result.status !== 0) {
    failures.push(
      result.stderr.trim() ||
        result.stdout.trim() ||
        `${command} ${args.join(" ")} failed`,
    );
  }
}

function runFocusedFailureTests() {
  const deepImport = sourceImportFailures(
    "libs/domain/src/failure-injection.ts",
    ["import { secret } from '@asha-rulebench/store", "/src/internal';"].join(
      "",
    ),
  );
  if (deepImport.length === 0) {
    throw new Error(
      "Pattern self-test failed to reject a frontend deep import.",
    );
  }

  const fixtureImport = sourceImportFailures(
    "libs/store/src/failure-injection.ts",
    ["import '@asha-rulebench/testing", "-fixtures';"].join(""),
  );
  if (fixtureImport.length === 0) {
    throw new Error(
      "Pattern self-test failed to reject a production testing-fixture import.",
    );
  }

  const testImport = sourceImportFailures(
    "libs/store/src/store.spec.ts",
    ["import '@asha-rulebench/testing", "-fixtures';"].join(""),
  );
  if (testImport.length !== 0) {
    throw new Error(
      "Pattern self-test rejected an allowed test fixture import.",
    );
  }
}

function parseJson(path) {
  try {
    return JSON.parse(readFileSync(join(root, path), "utf8"));
  } catch (error) {
    failures.push(
      `Invalid JSON ${path}: ${error instanceof Error ? error.message : String(error)}`,
    );
    return null;
  }
}

function exists(path) {
  try {
    statSync(join(root, path));
    return true;
  } catch {
    return false;
  }
}

function collectFiles(roots, extensions) {
  const files = [];
  for (const sourceRoot of roots) {
    const fullRoot = join(root, sourceRoot);
    if (!exists(sourceRoot)) continue;
    walk(fullRoot, files, extensions);
  }
  return files;
}

function walk(directory, files, extensions) {
  for (const entry of readdirSync(directory)) {
    if (["node_modules", "dist", "coverage", ".git", ".nx"].includes(entry))
      continue;
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      walk(path, files, extensions);
    } else if (extensions.some((extension) => entry.endsWith(extension))) {
      files.push(path);
    }
  }
}
