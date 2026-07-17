import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const root = process.cwd();

export function parseReviewArguments(argumentsList) {
  let output = null;
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === "--") continue;
    if (argument === "--output") {
      const value = argumentsList[++index];
      if (value === undefined || value.startsWith("--")) {
        throw new Error("--output requires a path.");
      }
      if (output !== null)
        throw new Error("--output may be supplied only once.");
      output = value;
      continue;
    }
    throw new Error(`Unknown claims-review argument: ${argument}`);
  }
  return { output };
}

export function validateGovernanceReview(review) {
  const failures = [];
  if (review.schemaVersion !== 2) {
    failures.push("docs/verification-claims.json must use schemaVersion 2.");
  }

  const claimsDocuments = review.authority?.claimsDocuments;
  if (!Array.isArray(claimsDocuments) || claimsDocuments.length === 0) {
    failures.push(
      "claims authority must name at least one Den document handle.",
    );
  } else {
    for (const handle of claimsDocuments)
      validateDocumentHandle(handle, failures);
  }
  validateDocumentHandle(review.authority?.limitationsDocument, failures);

  const snapshot = review.limitationSnapshot;
  if (!isRecord(snapshot)) {
    failures.push("limitationSnapshot must be an object.");
  } else {
    requireTimestamp(
      snapshot.sourceDocumentUpdatedAt,
      "sourceDocumentUpdatedAt",
      failures,
    );
    requireTimestamp(snapshot.reviewedAt, "reviewedAt", failures);
    if (!nonEmptyString(snapshot.reviewedBy)) {
      failures.push(
        "limitationSnapshot.reviewedBy must identify the reviewer.",
      );
    }
    validateLimitationRows(snapshot.active, "active", failures);
    validateLimitationRows(snapshot.resolved, "resolved", failures);
    const activeIds = new Set(
      Array.isArray(snapshot.active)
        ? snapshot.active.map((entry) => entry.id)
        : [],
    );
    for (const entry of Array.isArray(snapshot.resolved)
      ? snapshot.resolved
      : []) {
      if (activeIds.has(entry.id)) {
        failures.push(
          `limitation ${entry.id} cannot be both active and resolved.`,
        );
      }
    }
  }

  if (
    review.freshnessPolicy !==
    "report-snapshot-provenance-without-blocking-unrelated-source-edits"
  ) {
    failures.push(
      "verification governance must use the non-literal freshness policy.",
    );
  }
  return failures;
}

export function buildGovernanceReceipt({
  review,
  capabilityManifest,
  sourceCommit,
  sourceTreeDirty,
  reviewer,
  reviewedAt,
}) {
  return {
    schema: "asha-rulebench.certification-governance-receipt",
    schemaVersion: 1,
    source: {
      commit: sourceCommit,
      treeDirty: sourceTreeDirty,
      protocol: {
        id: capabilityManifest.protocolId,
        version: capabilityManifest.protocolVersion,
      },
      governedAshaRevision: capabilityManifest.governedAshaRevision,
    },
    review: {
      reviewer,
      reviewedAt,
      denDocuments: [
        ...review.authority.claimsDocuments,
        review.authority.limitationsDocument,
      ],
    },
    executableInventory: {
      providers: capabilityManifest.providers.length,
      rulesets: capabilityManifest.rulesets.length,
      packages: capabilityManifest.packages.length,
      scenarios: capabilityManifest.scenarios.length,
      capabilities: capabilityManifest.capabilities.length,
      executableCapabilities: capabilityManifest.capabilities.filter(
        (entry) => entry.support.runtimeExecutable,
      ).length,
      regressionCoveredCapabilities: capabilityManifest.capabilities.filter(
        (entry) => entry.support.regressionCovered,
      ).length,
    },
    limitations: {
      authority: review.authority.limitationsDocument,
      snapshotSourceDocumentUpdatedAt:
        review.limitationSnapshot.sourceDocumentUpdatedAt,
      snapshotReviewedAt: review.limitationSnapshot.reviewedAt,
      snapshotReviewedBy: review.limitationSnapshot.reviewedBy,
      active: review.limitationSnapshot.active,
      resolved: review.limitationSnapshot.resolved,
      freshnessEnforcedBySourceGate: false,
      freshnessNote:
        "The receipt exposes the last Den-reviewed limitation snapshot. Policy may require a new review without making unrelated source edits fail on a literal date.",
    },
  };
}

function run() {
  let selection;
  try {
    selection = parseReviewArguments(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(2);
  }

  const review = JSON.parse(
    readFileSync(join(root, "docs", "verification-claims.json"), "utf8"),
  );
  const failures = validateGovernanceReview(review);
  if (failures.length > 0) {
    console.error(failures.join("\n"));
    process.exit(1);
  }

  const capabilityManifest = readCapabilityManifest();
  const receipt = buildGovernanceReceipt({
    review,
    capabilityManifest,
    sourceCommit: runGit(["rev-parse", "HEAD"]).trim(),
    sourceTreeDirty: runGit(["status", "--porcelain"]).trim().length > 0,
    reviewer:
      process.env.RULEBENCH_CERTIFICATION_REVIEWER ??
      process.env.GITHUB_ACTOR ??
      process.env.USER ??
      "unidentified-certification-runner",
    reviewedAt: new Date().toISOString(),
  });
  const rendered = `${JSON.stringify(receipt, null, 2)}\n`;
  if (selection.output !== null) {
    const outputPath = join(root, selection.output);
    mkdirSync(dirname(outputPath), { recursive: true });
    writeFileSync(outputPath, rendered);
    console.log(`claims/limitations receipt written to ${selection.output}`);
  }
  console.log(rendered.trimEnd());
  console.log(
    `review:claims-and-limitations ok (${receipt.executableInventory.capabilities} executable inventory rows; ${receipt.limitations.active.length} active Den-reviewed limitations)`,
  );
}

function validateDocumentHandle(handle, failures) {
  if (
    !isRecord(handle) ||
    !nonEmptyString(handle.projectId) ||
    !nonEmptyString(handle.slug)
  ) {
    failures.push("each Den document handle needs projectId and slug.");
  }
}

function validateLimitationRows(rows, label, failures) {
  if (!Array.isArray(rows)) {
    failures.push(`limitationSnapshot.${label} must be an array.`);
    return;
  }
  const ids = new Set();
  for (const row of rows) {
    if (
      !isRecord(row) ||
      !nonEmptyString(row.id) ||
      !nonEmptyString(row.heading)
    ) {
      failures.push(`${label} limitation rows need non-empty id and heading.`);
      continue;
    }
    if (ids.has(row.id))
      failures.push(`duplicate ${label} limitation id: ${row.id}.`);
    ids.add(row.id);
  }
}

function requireTimestamp(value, label, failures) {
  if (!nonEmptyString(value) || Number.isNaN(Date.parse(value))) {
    failures.push(`limitationSnapshot.${label} must be an ISO timestamp.`);
  }
}

function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function nonEmptyString(value) {
  return typeof value === "string" && value.length > 0;
}

function readCapabilityManifest() {
  const artifact = readFileSync(
    join(
      root,
      "libs",
      "transport",
      "src",
      "generated",
      "rust-capability-manifest.ts",
    ),
    "utf8",
  );
  const match = artifact.match(
    /export const rustCapabilityManifest:[^=]+=(\s*\{[\s\S]*\});\s*$/,
  );
  if (match === null) {
    throw new Error(
      "Generated capability manifest does not contain a JSON export.",
    );
  }
  return JSON.parse(match[1]);
}

function runGit(argumentsList) {
  const result = spawnSync("git", argumentsList, {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || `git ${argumentsList.join(" ")} failed`);
  }
  return result.stdout;
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
