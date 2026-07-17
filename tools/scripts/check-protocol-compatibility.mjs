import { readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();

export function readRustProtocolContract(source) {
  return {
    id: requireMatch(
      source,
      /pub const PROTOCOL_ID:\s*&str\s*=\s*"([^"]+)";/,
      "Rust PROTOCOL_ID",
    ),
    version: Number(
      requireMatch(
        source,
        /pub const PROTOCOL_VERSION:\s*u32\s*=\s*(\d+);/,
        "Rust PROTOCOL_VERSION",
      ),
    ),
  };
}

export function readTypeScriptProtocolContract(source) {
  return {
    id: requireMatch(
      source,
      /export const RULEBENCH_PROTOCOL_ID\s*=\s*"([^"]+)";/,
      "TypeScript RULEBENCH_PROTOCOL_ID",
    ),
    version: Number(
      requireMatch(
        source,
        /export const RULEBENCH_PROTOCOL_VERSION\s*=\s*(\d+);/,
        "TypeScript RULEBENCH_PROTOCOL_VERSION",
      ),
    ),
  };
}

export function readGeneratedProtocolContract(source) {
  const objectSource = requireMatch(
    source,
    /export const rustCapabilityManifest:[^=]+=(\s*\{[\s\S]*\});\s*$/,
    "generated capability manifest",
  );
  const manifest = JSON.parse(objectSource);
  return {
    id: manifest.protocolId,
    version: manifest.protocolVersion,
  };
}

export function compareProtocolContracts(contracts) {
  const failures = [];
  for (const [owner, contract] of Object.entries(contracts)) {
    if (typeof contract.id !== "string" || contract.id.length === 0) {
      failures.push(`${owner} protocol id must be a non-empty string.`);
    }
    if (!Number.isInteger(contract.version) || contract.version <= 0) {
      failures.push(`${owner} protocol version must be a positive integer.`);
    }
  }

  const ids = new Set(Object.values(contracts).map((contract) => contract.id));
  if (ids.size !== 1) {
    failures.push(
      `Protocol id mismatch: ${formatContracts(contracts, (contract) => contract.id)}.`,
    );
  }
  const versions = new Set(
    Object.values(contracts).map((contract) => contract.version),
  );
  if (versions.size !== 1) {
    failures.push(
      `Protocol version mismatch: ${formatContracts(contracts, (contract) => contract.version)}.`,
    );
  }
  return failures;
}

function run() {
  const contracts = {
    rust: readRustProtocolContract(
      readFileSync(
        join(
          root,
          "rulebench-rs",
          "crates",
          "rulebench-protocol",
          "src",
          "bridge.rs",
        ),
        "utf8",
      ),
    ),
    typescript: readTypeScriptProtocolContract(
      readFileSync(join(root, "libs", "transport", "src", "live.ts"), "utf8"),
    ),
    generated: readGeneratedProtocolContract(
      readFileSync(
        join(
          root,
          "libs",
          "transport",
          "src",
          "generated",
          "rust-capability-manifest.ts",
        ),
        "utf8",
      ),
    ),
  };
  const failures = compareProtocolContracts(contracts);
  if (failures.length > 0) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  const contract = contracts.rust;
  console.log(
    `check:protocol-compatibility ok (${contract.id} v${contract.version}; Rust, TypeScript, generated manifest)`,
  );
}

function requireMatch(source, pattern, label) {
  const match = source.match(pattern);
  if (match === null) throw new Error(`Could not read ${label}.`);
  return match[1];
}

function formatContracts(contracts, select) {
  return Object.entries(contracts)
    .map(([owner, contract]) => `${owner}=${select(contract)}`)
    .join(", ");
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
