import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const generatedArtifact =
  "rulebench-rs/crates/rulebench-content/src/generated/representative-rpg-content.json";
export const declaredSurfaceFiles = Object.freeze([
  "libs/content-authoring/src/representative-rpg-content.ts",
  "libs/content-authoring/src/representative-rpg-content.spec.ts",
  generatedArtifact,
  "rulebench-rs/crates/rulebench-combat/src/rpg_resolver.rs",
  "rulebench-rs/crates/rulebench-combat/src/runtime/status.rs",
  "rulebench-rs/crates/rulebench-product-content/src/scenarios/skirmish.rs",
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
    failures,
  };
}

export function inspectDeclaredRulesLanguage(readSource) {
  const failures = [];
  const authored = readSource(declaredSurfaceFiles[0]);
  const artifact = JSON.parse(readSource(generatedArtifact));
  const resolver = readSource(declaredSurfaceFiles[3]);
  const status = readSource(declaredSurfaceFiles[4]);
  const runtimeRegression = readSource(declaredSurfaceFiles[5]);
  const actionIds = new Set(
    artifact.normalizedIr.actions.map((action) => action.id),
  );
  const bindingIds = new Set(
    artifact.bindings.map((binding) => binding.actionId),
  );
  if (actionIds.size === 0 || actionIds.size !== bindingIds.size) {
    failures.push("generated actions and runtime bindings must be non-empty and one-to-one");
  }
  for (const actionId of actionIds) {
    if (!bindingIds.has(actionId)) {
      failures.push(`generated action lacks runtime binding: ${actionId}`);
    }
  }
  for (const required of ["definePackage", "rulebenchActionBindings"]) {
    if (!authored.includes(required)) {
      failures.push(`TypeScript authoring surface is missing ${required}`);
    }
  }
  for (const required of ["RulebenchRpgAuthority", "submit_with_random", "random_request"]) {
    if (!resolver.includes(required)) {
      failures.push(`persistent Rust authority surface is missing ${required}`);
    }
  }
  if (!status.includes("rpg_authored_action_options")) {
    failures.push("user-facing action options do not consume generated RPG content");
  }
  for (const required of [
    "typescript_only_action_reaches_options_preflight_and_authority_runtime",
    ".action_by_id(&action.action_id)",
    "ASHA_RPG_AUTHORITY_SURFACE",
  ]) {
    if (!runtimeRegression.includes(required)) {
      failures.push(`runtime regression is missing ${required}`);
    }
  }
  return {
    actionCount: actionIds.size,
    bindingCount: bindingIds.size,
    failures,
  };
}

function run() {
  const files = process.argv.slice(2);
  const report = files.length > 0
    ? inspectContentOnlyChange(files)
    : inspectDeclaredRulesLanguage((file) => readFileSync(file, "utf8"));
  if (report.failures.length > 0) {
    console.error(report.failures.join("\n"));
    process.exit(1);
  }
  console.log(
    files.length > 0
      ? `rules-language amplification ok (content-only=${report.contentOnlyLayerCount} layers)`
      : `rules-language runtime path ok (actions=${report.actionCount}; bindings=${report.bindingCount})`,
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
