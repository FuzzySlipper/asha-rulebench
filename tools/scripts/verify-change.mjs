import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const workspaceManifest = "rulebench-rs/Cargo.toml";

export const profileOrder = [
  "frontend",
  "browser",
  "rust-owner",
  "protocol-generated",
  "product-content",
  "host-transport",
  "docs",
];

export const governedCrates = new Set([
  "rulebench-content",
  "rulebench-combat",
  "rulebench-replay",
  "rulebench-protocol",
  "rulebench-bridge",
  "rulebench-product-content",
  "rulebench-codegen",
  "rulebench-process-host",
]);

export function parseVerifyChangeArguments(argumentsList) {
  const profiles = [];
  const crates = [];
  let dryRun = false;

  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === "--") continue;
    if (argument === "--dry-run") {
      dryRun = true;
      continue;
    }
    if (argument === "--profile") {
      profiles.push(requireValue(argumentsList, ++index, argument));
      continue;
    }
    if (argument === "--crate") {
      crates.push(requireValue(argumentsList, ++index, argument));
      continue;
    }
    throw new Error(`Unknown verify:change argument: ${argument}`);
  }

  const uniqueProfiles = [...new Set(profiles)];
  if (uniqueProfiles.length === 0) {
    throw new Error(
      `At least one --profile is required. Allowed profiles: ${profileOrder.join(", ")}.`,
    );
  }
  for (const profile of uniqueProfiles) {
    if (!profileOrder.includes(profile)) {
      throw new Error(
        `Unknown verify:change profile: ${profile}. Allowed profiles: ${profileOrder.join(", ")}.`,
      );
    }
  }

  const uniqueCrates = [...new Set(crates)];
  for (const crate of uniqueCrates) {
    if (!governedCrates.has(crate)) {
      throw new Error(
        `Unknown or ungoverned Rust crate: ${crate}. Run the blocking project gate when ownership is uncertain.`,
      );
    }
  }

  const selectsRustOwner = uniqueProfiles.includes("rust-owner");
  if (selectsRustOwner && uniqueCrates.length === 0) {
    throw new Error("--crate is required for the rust-owner profile.");
  }
  if (!selectsRustOwner && uniqueCrates.length > 0) {
    throw new Error("--crate is valid only with the rust-owner profile.");
  }

  return {
    profiles: profileOrder.filter((profile) =>
      uniqueProfiles.includes(profile),
    ),
    crates: uniqueCrates,
    dryRun,
  };
}

export function buildVerifyChangePlan(selection) {
  const commands = [];
  const addScript = (script) =>
    addCommand(commands, {
      id: `pnpm:${script}`,
      command: "pnpm",
      arguments: ["run", script],
    });
  const addCargoTest = (crate) =>
    addCommand(commands, {
      id: `cargo:test:${crate}`,
      command: "cargo",
      arguments: ["test", "--manifest-path", workspaceManifest, "-p", crate],
    });

  for (const profile of selection.profiles) {
    switch (profile) {
      case "frontend":
        addScript("check:pattern");
        addScript("check:typescript-authority");
        addScript("check:rules-language-boundary");
        addScript("lint");
        addScript("typecheck");
        addScript("test");
        break;
      case "browser":
        addScript("typecheck");
        addScript("e2e:gate");
        break;
      case "rust-owner":
        addScript("check:rust-boundaries");
        addScript("check:rust-test-ownership");
        for (const crate of selection.crates) addCargoTest(crate);
        break;
      case "protocol-generated":
        addScript("check:rust-boundaries");
        addScript("generated:check");
        addScript("check:protocol-compatibility");
        addScript("typecheck");
        addScript("test");
        break;
      case "product-content":
        addCargoTest("rulebench-product-content");
        break;
      case "host-transport":
        addCargoTest("rulebench-bridge");
        addCargoTest("rulebench-process-host");
        addCommand(commands, {
          id: "vitest:host-transport",
          command: "pnpm",
          arguments: [
            "exec",
            "vitest",
            "run",
            "libs/transport",
            "libs/store",
            "--passWithNoTests",
          ],
        });
        break;
      case "docs":
        addScript("check:docs");
        break;
    }
  }

  return commands;
}

export function formatCommand(entry) {
  return [entry.command, ...entry.arguments]
    .map((part) =>
      /^[A-Za-z0-9_./:@=-]+$/.test(part) ? part : JSON.stringify(part),
    )
    .join(" ");
}

function addCommand(commands, command) {
  if (!commands.some((entry) => entry.id === command.id))
    commands.push(command);
}

function requireValue(argumentsList, index, flag) {
  const value = argumentsList[index];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`${flag} requires a value.`);
  }
  return value;
}

function run() {
  let selection;
  try {
    selection = parseVerifyChangeArguments(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(2);
  }

  const plan = buildVerifyChangePlan(selection);
  console.log(`verify:change profiles: ${selection.profiles.join(", ")}`);
  if (selection.crates.length > 0) {
    console.log(`verify:change crates: ${selection.crates.join(", ")}`);
  }
  console.log("verify:change selected commands:");
  for (const command of plan) console.log(`- ${formatCommand(command)}`);
  if (selection.dryRun) return;

  for (const entry of plan) {
    console.log(`\n[verify:change] ${formatCommand(entry)}`);
    const result = spawnSync(entry.command, entry.arguments, {
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
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
