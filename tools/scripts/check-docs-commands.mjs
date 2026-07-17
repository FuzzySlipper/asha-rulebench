import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8"),
);
const scripts = new Set(Object.keys(packageJson.scripts ?? {}));
const references = collectReferences([
  "README.md",
  "README.template.md",
  "AGENTS.md",
  "docs",
  "apps/app-e2e/src/live/docs",
  ".github/workflows",
]);
const failures = [];

for (const referencePath of references) {
  const rel = relative(root, referencePath);
  const text = readFileSync(referencePath, "utf8");
  for (const match of text.matchAll(/\b(?:pnpm|npm) run ([a-zA-Z0-9:_-]+)/g)) {
    const script = match[1];
    if (!scripts.has(script)) {
      failures.push(`${rel} cites missing package script: ${script}`);
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`docs/workflow command references ok (${references.length} files)`);

function collectReferences(entries) {
  const references = [];
  for (const entry of entries) {
    const path = join(root, entry);
    try {
      const stats = statSync(path);
      if (stats.isDirectory()) {
        references.push(...collectReferenceFiles(path));
      } else {
        references.push(path);
      }
    } catch {
      // Optional docs paths are allowed to be absent.
    }
  }
  return references;
}

function collectReferenceFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      files.push(...collectReferenceFiles(path));
    } else if (
      entry.endsWith(".md") ||
      entry.endsWith(".yml") ||
      entry.endsWith(".yaml")
    ) {
      files.push(path);
    }
  }
  return files;
}
