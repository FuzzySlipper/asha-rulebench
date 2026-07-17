import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

const root = process.cwd();
const workspaceManifestPath = join(root, "rulebench-rs", "Cargo.toml");
const cratesRoot = join(root, "rulebench-rs", "crates");
const hostsRoot = join(root, "rulebench-rs", "hosts");
const rpgRepository = "https://github.com/FuzzySlipper/asha-rpg.git";
const rpgRevision = "ea3e3803d4736268f2a10996a34bc5b8dfefcffc";
const rpgVersionRequirement = "^0.1";

const allowedDependencies = new Map([
  ["rulebench-content", new Set(["rpg-compiler", "rpg-core", "rpg-ir"])],
  [
    "rulebench-combat",
    new Set(["rpg-core", "rpg-ir", "rpg-runtime", "rulebench-content"]),
  ],
  ["rulebench-replay", new Set(["rpg-core", "rulebench-combat"])],
  [
    "rulebench-protocol",
    new Set([
      "rpg-core",
      "rpg-ir",
      "rulebench-combat",
      "rulebench-content",
      "rulebench-replay",
    ]),
  ],
  [
    "rulebench-bridge",
    new Set([
      "rpg-core",
      "rpg-ir",
      "rulebench-combat",
      "rulebench-content",
      "rulebench-protocol",
      "rulebench-replay",
    ]),
  ],
  [
    "rulebench-product-content",
    new Set([
      "rpg-core",
      "rpg-ir",
      "rulebench-combat",
      "rulebench-content",
      "rulebench-protocol",
      "rulebench-replay",
    ]),
  ],
  [
    "rulebench-codegen",
    new Set(["rulebench-protocol"]),
  ],
  [
    "rulebench-process-host",
    new Set(["rulebench-bridge", "rulebench-product-content", "rulebench-protocol"]),
  ],
]);

const publicRpgDependencies = new Set([
  "rpg-compiler",
  "rpg-core",
  "rpg-ir",
  "rpg-runtime",
]);
const manifests = readWorkspaceManifests();
const workspaceDependencies = parseDependencies(
  readFileSync(workspaceManifestPath, "utf8"),
  new Set(["workspace.dependencies"]),
);
const failures = validateWorkspace(manifests, workspaceDependencies);

runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(
  `check:rust-boundaries ok (${manifests.size} Rulebench crates; pinned public RPG revision ${rpgRevision.slice(0, 8)})`,
);

function readWorkspaceManifests() {
  const manifests = new Map();
  for (const workspaceRoot of [cratesRoot, hostsRoot]) {
    for (const entry of readdirSync(workspaceRoot, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const manifestPath = join(workspaceRoot, entry.name, "Cargo.toml");
      const manifest = readFileSync(manifestPath, "utf8");
      manifests.set(entry.name, {
        manifestPath,
        dependencies: parseDependencies(
          manifest,
          new Set(["dependencies", "dev-dependencies", "build-dependencies"]),
        ),
      });
    }
  }
  return manifests;
}

function parseDependencies(manifest, includedSections) {
  const dependencies = [];
  let section = "";
  for (const [index, line] of manifest.split("\n").entries()) {
    const sectionMatch = /^\[([^\]]+)]\s*$/.exec(line);
    if (sectionMatch !== null) {
      section = sectionMatch[1];
      continue;
    }
    if (!includedSections.has(section)) continue;

    const dependencyMatch = /^([A-Za-z0-9_-]+)(\.workspace)?\s*=\s*(.*)$/.exec(
      line,
    );
    if (dependencyMatch === null) continue;
    const specification = dependencyMatch[3];
    const path = /\bpath\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null;
    const name = dependencyMatch[1];
    const relevant =
      name.startsWith("rulebench-") ||
      name.startsWith("rpg-") ||
      name.startsWith("asha-") ||
      path !== null;
    if (!relevant) continue;

    dependencies.push({
      name,
      path,
      workspace:
        dependencyMatch[2] !== undefined ||
        /\bworkspace\s*=\s*true\b/.test(specification),
      git: /\bgit\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      revision: /\brev\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      version: /\bversion\s*=\s*"([^"]+)"/.exec(specification)?.[1] ?? null,
      line: index + 1,
    });
  }
  return dependencies;
}

function validateWorkspace(currentManifests, rootDependencies) {
  const errors = [];
  const actualCrates = new Set(currentManifests.keys());
  const rootByName = new Map(
    rootDependencies.map((dependency) => [dependency.name, dependency]),
  );

  for (const crateName of actualCrates) {
    if (!allowedDependencies.has(crateName)) {
      errors.push(
        `Unknown Rust workspace crate: ${crateName}. Add an explicit boundary policy before it can join the workspace.`,
      );
    }
  }
  for (const crateName of allowedDependencies.keys()) {
    if (!actualCrates.has(crateName))
      errors.push(
        `Boundary policy names missing workspace crate: ${crateName}.`,
      );
  }

  for (const dependencyName of publicRpgDependencies) {
    const dependency = rootByName.get(dependencyName);
    if (dependency === undefined) {
      errors.push(
        `rulebench-rs/Cargo.toml must pin public RPG package ${dependencyName}.`,
      );
    } else {
      errors.push(
        ...validateRpgDistribution(
          dependency,
          `rulebench-rs/Cargo.toml:${dependency.line}`,
        ),
      );
    }
  }

  for (const [crateName, manifest] of currentManifests) {
    for (const dependency of manifest.dependencies) {
      const location = `${relative(root, manifest.manifestPath)}:${dependency.line}`;
      errors.push(...validateDependency(crateName, dependency.name, location));
      if (publicRpgDependencies.has(dependency.name)) {
        if (!dependency.workspace) {
          errors.push(
            `${location}: ${dependency.name} must inherit the governed exact workspace pin.`,
          );
        }
      }
      if (dependency.name.startsWith("asha-")) {
        errors.push(
          `${location}: Rulebench must consume ASHA-backed RPG behavior through asha-rpg, not ASHA directly.`,
        );
      }
      if (dependency.path !== null) {
        errors.push(
          ...validateLocalPath(
            dependency.name,
            manifest.manifestPath,
            dependency.path,
            location,
            actualCrates,
          ),
        );
      }
    }
  }

  return errors;
}

function validateDependency(crateName, dependencyName, location) {
  const allowed = allowedDependencies.get(crateName);
  if (allowed === undefined) return [];
  if (publicRpgDependencies.has(dependencyName)) {
    return allowed.has(dependencyName)
      ? []
      : [
          `${location}: ${crateName} may not consume ${dependencyName} without an explicit focused-owner boundary.`,
        ];
  }
  if (dependencyName.startsWith("asha-")) return [];
  if (!dependencyName.startsWith("rulebench-")) return [];
  if (!allowedDependencies.has(dependencyName)) {
    return [
      `${location}: ${crateName} depends on unknown local crate ${dependencyName}.`,
    ];
  }
  return allowed.has(dependencyName)
    ? []
    : [`${location}: ${crateName} must not depend on ${dependencyName}.`];
}

function validateRpgDistribution(dependency, location) {
  const errors = [];
  if (dependency.path !== null)
    errors.push(
      `${location}: public RPG dependencies must not use sibling paths.`,
    );
  if (dependency.git !== rpgRepository)
    errors.push(`${location}: RPG dependency must use ${rpgRepository}.`);
  if (dependency.revision !== rpgRevision)
    errors.push(
      `${location}: RPG dependency must use exact revision ${rpgRevision}.`,
    );
  if (dependency.version !== rpgVersionRequirement)
    errors.push(
      `${location}: RPG dependency must use compatible version ${rpgVersionRequirement}.`,
    );
  return errors;
}

function validateLocalPath(
  dependencyName,
  manifestPath,
  dependencyPath,
  location,
  actualCrates,
) {
  if (!dependencyName.startsWith("rulebench-")) {
    return [
      `${location}: non-Rulebench path dependency ${dependencyName} is forbidden.`,
    ];
  }
  const resolvedPath = resolve(dirname(manifestPath), dependencyPath);
  const relativeToCrates = relative(cratesRoot, resolvedPath);
  const relativeToHosts = relative(hostsRoot, resolvedPath);
  const targetName = !relativeToCrates.startsWith("..")
    ? relativeToCrates.split("/")[0]
    : relativeToHosts.split("/")[0];
  return actualCrates.has(targetName)
    ? []
    : [
        `${location}: local path does not resolve to a governed Rulebench crate (${dependencyPath}).`,
      ];
}

function runFocusedFailureTests() {
  assertRejected(
    validateDependency(
      "rulebench-content",
      "rulebench-product-content",
      "self-test:reverse-dependency",
    ),
    "a product authority reverse dependency was accepted",
  );
  assertRejected(
    validateDependency(
      "rulebench-protocol",
      "rpg-runtime",
      "self-test:rpg-bypass",
    ),
    "protocol bypassed its focused RPG dependency boundary",
  );
  assertRejected(
    validateRpgDistribution(
      {
        name: "rpg-core",
        path: null,
        git: rpgRepository,
        revision: "0".repeat(40),
        version: rpgVersionRequirement,
      },
      "self-test:stale-rpg-revision",
    ),
    "a stale RPG revision was accepted",
  );
  assertRejected(
    validateRpgDistribution(
      {
        name: "rpg-core",
        path: "../../asha-rpg",
        git: null,
        revision: null,
        version: null,
      },
      "self-test:sibling-rpg-path",
    ),
    "a sibling RPG path was accepted",
  );
}

function assertRejected(errors, message) {
  if (errors.length === 0)
    throw new Error(`Boundary self-test failed: ${message}.`);
}
