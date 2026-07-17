import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const receiptPath = "local/certification/claims-and-limitations.json";

export function parseCertificationArguments(argumentsList, environment) {
  let dryRun = false;
  let requireLive = false;

  for (const argument of argumentsList) {
    if (argument === "--") continue;
    if (argument === "--dry-run") {
      dryRun = true;
      continue;
    }
    if (argument === "--require-live") {
      requireLive = true;
      continue;
    }
    throw new Error(`Unknown certify argument: ${argument}`);
  }

  if (requireLive) validateLiveEnvironment(environment);
  return { dryRun, requireLive };
}

export function buildCertificationPlan(selection) {
  const groups = [
    {
      id: "authority-and-product-contracts",
      owner: "Rulebench",
      command: "pnpm",
      arguments: ["run", "verify:static"],
      coverage:
        "Rust owners, generated/protocol compatibility, architecture, TypeScript authority, executable claims, frontend tests, and production build",
    },
    {
      id: "exhaustive-semantic-corpus",
      owner: "Rulebench fixtures",
      command: "pnpm",
      arguments: ["run", "regression:check"],
      coverage:
        "all registered scenario regressions and executable capability conformance cases",
    },
    {
      id: "deterministic-browser-certification",
      owner: "Rulebench browser workflows",
      command: "pnpm",
      arguments: ["run", "e2e:certification"],
      coverage:
        "complete deterministic desktop, mobile, accessibility, content, session, policy, recovery, viewer, and replay journeys",
    },
  ];

  if (selection.requireLive) {
    groups.push({
      id: "managed-live-artifacts",
      owner: "Rulebench milestone/release evidence",
      command: "pnpm",
      arguments: ["run", "e2e:live-artifacts"],
      coverage:
        "managed-server screenshots, visible text, browser errors, milestone evidence, and explicit non-claims",
    });
  }

  groups.push({
    id: "claims-and-limitations-receipt",
    owner: "Rulebench governance with Den limitation authority",
    command: "pnpm",
    arguments: [
      "run",
      "review:claims-and-limitations",
      "--",
      "--output",
      receiptPath,
    ],
    coverage:
      "current executable inventory plus the last reviewed Den claims/limitations snapshot, without literal prose, count, or freshness gates",
  });

  return groups;
}

export function formatCertificationCommand(group) {
  return [group.command, ...group.arguments]
    .map((part) =>
      /^[A-Za-z0-9_./:@=-]+$/.test(part) ? part : JSON.stringify(part),
    )
    .join(" ");
}

function validateLiveEnvironment(environment) {
  if (environment.LIVE_RUN !== "1") {
    throw new Error("--require-live requires LIVE_RUN=1.");
  }
  const baseUrl = environment.BASE_URL;
  if (baseUrl === undefined || baseUrl.length === 0) {
    throw new Error("--require-live requires BASE_URL from a managed server.");
  }
  let parsed;
  try {
    parsed = new URL(baseUrl);
  } catch {
    throw new Error("--require-live requires BASE_URL to be a valid URL.");
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("--require-live requires an HTTP(S) BASE_URL.");
  }
}

function run() {
  let selection;
  try {
    selection = parseCertificationArguments(process.argv.slice(2), process.env);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(2);
  }

  const plan = buildCertificationPlan(selection);
  console.log(
    `certify mode: ${selection.requireLive ? "live-required milestone/release" : "deterministic scheduled/milestone"}`,
  );
  console.log(
    "certify owner: Rulebench; no proof is migrated to ASHA Testing without a destination task, runnable consumer, and retained failure signal.",
  );
  console.log("certify groups:");
  for (const [index, group] of plan.entries()) {
    console.log(`${index + 1}. ${group.id} [${group.owner}]`);
    console.log(`   covers: ${group.coverage}`);
    console.log(`   runs: ${formatCertificationCommand(group)}`);
  }
  if (!selection.requireLive) {
    console.log(
      "certify non-claim: this run does not establish managed/LAN visual evidence; use --require-live with BASE_URL and LIVE_RUN=1 for that claim.",
    );
  }
  if (selection.dryRun) return;

  for (const group of plan) {
    console.log(`\n[certify:${group.id}] ${formatCertificationCommand(group)}`);
    const result = spawnSync(group.command, group.arguments, {
      cwd: process.cwd(),
      env: process.env,
      stdio: "inherit",
    });
    if (result.error !== undefined) {
      console.error(result.error.message);
      process.exit(1);
    }
    if (result.status !== 0) process.exit(result.status ?? 1);
  }

  console.log(
    `certify ok (${plan.length} groups; governance receipt: ${receiptPath})`,
  );
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
