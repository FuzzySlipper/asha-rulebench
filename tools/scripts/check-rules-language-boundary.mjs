import { pathToFileURL } from "node:url";

export const semanticOperationLayers = Object.freeze([
  "Rust operation registration and complete extension contract",
  "normalized IR decode and compatibility schema",
  "Rust reference, requirement, and semantic validation",
  "Rust staged execution and capability mutation owner",
  "accepted DomainEvents, trace, and replay behavior",
  "generated public operation vocabulary",
  "TypeScript authoring sugar and owner-local tests",
]);

const generatedArtifact =
  "rulebench-rs/crates/rulebench-content/src/generated/representative-rpg-content.json";
const canonicalContentOnlyChange = Object.freeze([
  "libs/content-authoring/src/example-action.ts",
  "libs/content-authoring/src/example-action.spec.ts",
  generatedArtifact,
]);

export function inspectContentOnlyChange(files) {
  const normalized = files.map((file) => file.replaceAll("\\", "/"));
  const layers = new Set();
  const failures = [];

  for (const file of normalized) {
    if (file === generatedArtifact) {
      layers.add("normalized artifact");
      continue;
    }
    if (
      file.startsWith("libs/content-authoring/src/") &&
      file.endsWith(".ts")
    ) {
      layers.add(
        file.endsWith(".spec.ts") || file.endsWith(".test.ts")
          ? "owner expectation"
          : "TypeScript content",
      );
      continue;
    }
    failures.push(
      `content-only change has forbidden structural amplification: ${file}`,
    );
  }

  for (const layer of [
    "TypeScript content",
    "owner expectation",
    "normalized artifact",
  ]) {
    if (!layers.has(layer)) {
      failures.push(`content-only change is missing required ${layer} layer`);
    }
  }

  return {
    contentOnlyLayerCount: layers.size,
    semanticOperationLayerCount: semanticOperationLayers.length,
    failures,
  };
}

function run() {
  const files =
    process.argv.length > 2
      ? process.argv.slice(2)
      : canonicalContentOnlyChange;
  const report = inspectContentOnlyChange(files);
  if (report.failures.length > 0) {
    console.error(report.failures.join("\n"));
    process.exit(1);
  }
  console.log(
    `rules-language amplification ok (content-only=${report.contentOnlyLayerCount} layers; semantic-operation=${report.semanticOperationLayerCount} owner layers)`,
  );
  console.log(
    "content-only excludes Rust, product protocol, host routes, capability manifests, and certification/proof manifests",
  );
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
