import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const manifestPath = join(root, "docs", "verification-claims.json");
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const capabilityManifest = readCapabilityManifest();
const failures = [];

if (manifest.schemaVersion !== 1)
  failures.push("docs/verification-claims.json must use schemaVersion 1.");
if (manifest.reviewedOn !== "2026-07-15")
  failures.push("verification claims review date is stale.");
for (const slug of [
  "basic-design",
  "north-star-systems-map",
  "known-limitations",
]) {
  if (!manifest.denDocuments.includes(slug))
    failures.push(`verification review omits Den document ${slug}.`);
}
for (const entry of manifest.requiredClaims)
  requireText(entry, "required current claim");
for (const entry of manifest.requiredNonClaims)
  requireText(entry, "required non-claim");
for (const entry of manifest.forbiddenClaims) forbidText(entry);

const limitationIds = new Set(
  manifest.activeLimitations.map((entry) => entry.id),
);
for (const id of [
  "trusted-local-process-host",
  "checked-viewer-artifacts",
  "active-session-recovery",
  "authored-content-v1-vocabulary",
  "schema-coupled-replay-fingerprint",
]) {
  if (!limitationIds.has(id))
    failures.push(`verification review omits active limitation ${id}.`);
}

checkCapabilityManifest(capabilityManifest);

for (const crateRoot of [
  join(root, "rulebench-rs", "crates"),
  join(root, "rulebench-rs", "hosts"),
]) {
  for (const entry of readdirSync(crateRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const crate = entry.name;
    const sourceRoot = join(crateRoot, crate, "src");
    const sources = collectRustFiles(sourceRoot);
    const production = sources.filter(
      (path) =>
        !path.includes(`${join("src", "tests")}`) && !path.endsWith("tests.rs"),
    );
    if (production.length === 0) {
      failures.push(
        `${relative(root, sourceRoot)} has no production Rust source and must not be presented as implemented.`,
      );
      continue;
    }
    const text = production
      .map((path) => readFileSync(path, "utf8"))
      .join("\n");
    if (/\b(?:todo|unimplemented)!\s*\(/.test(text)) {
      failures.push(
        `${crate} contains a production todo!/unimplemented! stub; resolve it or record a scoped limitation.`,
      );
    }
  }
}

runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(
  `check:claims ok (${manifest.requiredClaims.length} claims, ${manifest.requiredNonClaims.length} non-claims, ${manifest.activeLimitations.length} active limitations)`,
);

function requireText(entry, kind) {
  const text = readFileSync(join(root, entry.file), "utf8");
  if (!normalizeWhitespace(text).includes(normalizeWhitespace(entry.text))) {
    failures.push(`${entry.file} is missing ${kind}: ${entry.text}`);
  }
}

function forbidText(entry) {
  const text = readFileSync(join(root, entry.file), "utf8");
  if (text.includes(entry.text))
    failures.push(`${entry.file} contains stale claim: ${entry.text}`);
}

function collectRustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) files.push(...collectRustFiles(path));
    else if (entry.endsWith(".rs")) files.push(path);
  }
  return files;
}

function runFocusedFailureTests() {
  if (!/\b(?:todo|unimplemented)!\s*\(/.test("fn pending() { todo!() }")) {
    throw new Error(
      "Claims self-test failed to detect a production authority stub.",
    );
  }
  if (/\b(?:todo|unimplemented)!\s*\(/.test("fn complete() {}")) {
    throw new Error("Claims self-test classified complete code as a stub.");
  }
}

function readCapabilityManifest() {
  const artifactPath = join(
    root,
    "libs",
    "transport",
    "src",
    "generated",
    "rust-capability-manifest.ts",
  );
  const artifact = readFileSync(artifactPath, "utf8");
  const match = artifact.match(
    /export const rustCapabilityManifest:[^=]+=(\s*\{[\s\S]*\});\s*$/,
  );
  if (match === null) {
    throw new Error(
      "Generated capability manifest does not contain a JSON object export.",
    );
  }
  return JSON.parse(match[1]);
}

function checkCapabilityManifest(capabilities) {
  if (capabilities.manifestId !== "asha-rulebench.capabilities") {
    failures.push("capability manifest has the wrong manifestId.");
  }
  if (capabilities.manifestVersion !== 2) {
    failures.push("capability manifest must use manifestVersion 2.");
  }
  if (
    capabilities.generatedArtifactSchema !== "asha-rulebench.capabilities.ts@2"
  ) {
    failures.push(
      "capability manifest has the wrong generated artifact schema.",
    );
  }
  if (!/^[0-9a-f]{40}$/.test(capabilities.governedAshaRevision)) {
    failures.push(
      "capability manifest must record one exact governed ASHA revision.",
    );
  }

  const identities = [
    ...capabilities.providers.map(
      (entry) =>
        `provider:${entry.provider.id}@${entry.provider.version}:${entry.ruleset.id}@${entry.ruleset.version}`,
    ),
    ...capabilities.rulesets.map(
      (entry) => `ruleset:${entry.id}@${entry.version}`,
    ),
    ...capabilities.packages.map(
      (entry) => `package:${entry.id}@${entry.version}`,
    ),
    ...capabilities.scenarios.map(
      (entry) => `scenario:${entry.id}@${entry.version}`,
    ),
  ];
  requireUnique(identities, "capability identity");

  if (capabilities.providers.length !== capabilities.rulesets.length) {
    failures.push(
      "every runtime ruleset identity must have one compiled provider entry.",
    );
  }
  const providerRows = capabilities.providers.map(
    (entry) =>
      `${entry.provider.id}@${entry.provider.version}:${entry.ruleset.id}@${entry.ruleset.version}`,
  );
  requireUnique(
    capabilities.providers.map((entry) => entry.provider.id),
    "provider id",
  );
  if (providerRows.join("\n") !== [...providerRows].sort().join("\n")) {
    failures.push("compiled providers must use deterministic identity ordering.");
  }
  for (const provider of capabilities.providers) {
    if (
      provider.operationVocabularyVersion !==
        capabilities.operationVocabularyVersion ||
      provider.effectOperationVocabularyVersion !==
        capabilities.effectVocabularyVersion
    ) {
      failures.push(
        `${provider.provider.id} has incompatible operation vocabulary metadata.`,
      );
    }
    if (provider.capabilities.length === 0) {
      failures.push(`${provider.provider.id} declares no provider capabilities.`);
    }
    const providerCapabilities = provider.capabilities.map(
      (capability) => `${capability.id}@${capability.version}`,
    );
    requireUnique(
      providerCapabilities,
      `${provider.provider.id} capability identity`,
    );
    if (
      providerCapabilities.join("\n") !==
      [...providerCapabilities].sort().join("\n")
    ) {
      failures.push(
        `${provider.provider.id} capabilities must use deterministic identity ordering.`,
      );
    }
    if (
      !capabilities.rulesets.some(
        (ruleset) =>
          ruleset.id === provider.ruleset.id &&
          ruleset.version === provider.ruleset.version,
      )
    ) {
      failures.push(
        `${provider.provider.id} references a ruleset absent from the runtime inventory.`,
      );
    }
  }

  const capabilityIds = capabilities.capabilities.map((entry) => entry.id);
  requireUnique(capabilityIds, "capability id");
  const capabilityRows = capabilities.capabilities.map(
    (entry) => `${capabilityKindRank(entry.kind)}:${entry.id}:${entry.version}`,
  );
  const sortedCapabilityRows = [...capabilityRows].sort();
  if (capabilityRows.join("\n") !== sortedCapabilityRows.join("\n")) {
    failures.push(
      "capability manifest rows must use deterministic kind/id/version ordering.",
    );
  }

  for (const entry of capabilities.capabilities) {
    const support = entry.support;
    requireProgression(
      entry.id,
      support.validationSupported,
      support.declared,
      "validation",
      "declaration",
    );
    requireProgression(
      entry.id,
      support.runtimeExecutable,
      support.validationSupported,
      "execution",
      "validation",
    );
    requireProgression(
      entry.id,
      support.protocolExposed,
      support.runtimeExecutable,
      "protocol exposure",
      "execution",
    );
    requireProgression(
      entry.id,
      support.liveHostExposed,
      support.protocolExposed,
      "live-host exposure",
      "protocol exposure",
    );
    requireProgression(
      entry.id,
      support.uiExposed,
      support.liveHostExposed,
      "UI exposure",
      "live-host exposure",
    );
    requireProgression(
      entry.id,
      support.durableAcrossRestart,
      support.runtimeExecutable,
      "restart durability",
      "execution",
    );
    if (entry.evidence.length === 0) {
      failures.push(`${entry.id} must name at least one Rust evidence owner.`);
    }
    if (
      ["operation", "targeting", "policy"].includes(entry.kind) &&
      support.runtimeExecutable &&
      !support.regressionCovered
    ) {
      failures.push(
        `${entry.id} is executable but lacks a successful owner-level conformance case.`,
      );
    }
  }

  if (capabilities.host.sessionRecoveryMode !== "none") {
    failures.push(
      "claims inventory must be reconciled before active-session recovery is reported.",
    );
  }
  const activeRecovery = capabilities.capabilities.find(
    (entry) => entry.id === "session.active-recovery",
  );
  if (activeRecovery?.support.runtimeExecutable !== false) {
    failures.push(
      "session.active-recovery must remain explicitly non-executable while host recovery mode is none.",
    );
  }
}

function requireUnique(values, kind) {
  if (new Set(values).size !== values.length) {
    failures.push(`${kind} entries must be unique.`);
  }
}

function requireProgression(
  id,
  claimed,
  prerequisite,
  claimLabel,
  prerequisiteLabel,
) {
  if (claimed && !prerequisite) {
    failures.push(`${id} reports ${claimLabel} without ${prerequisiteLabel}.`);
  }
}

function normalizeWhitespace(value) {
  return value.replace(/\s+/g, " ").trim();
}

function capabilityKindRank(kind) {
  const ranks = {
    operation: "0",
    targeting: "1",
    policy: "2",
    content: "3",
    replay: "4",
    session: "5",
  };
  const rank = ranks[kind];
  if (rank === undefined) {
    failures.push(`capability manifest contains unknown kind ${kind}.`);
    return "9";
  }
  return rank;
}
