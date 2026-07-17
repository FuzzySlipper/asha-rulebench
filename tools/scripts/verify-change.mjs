import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const workspaceManifest = "rulebench-rs/Cargo.toml";

export const profileOrder = [
  "frontend",
  "browser",
  "rust-owner",
  "protocol-generated",
  "fixtures-conformance",
  "host-transport",
  "portable",
  "docs",
];

export const governedCrates = new Set([
  "rulebench-content",
  "rulebench-combat",
  "rulebench-replay",
  "rulebench-rpg-adapter",
  "rulebench-protocol",
  "rulebench-bridge",
  "rulebench-fixtures",
  "rulebench-codegen",
  "rulebench-authority",
  "rulebench-process-host",
]);

export const portableCrates = new Set([
  "rulebench-rpg-adapter",
]);

const filterFlags = new Map([
  ["--package", "package"],
  ["--package-version", "packageVersion"],
  ["--ruleset", "ruleset"],
  ["--ruleset-version", "rulesetVersion"],
  ["--scenario", "scenario"],
  ["--capability", "capability"],
]);

export function parseVerifyChangeArguments(argumentsList) {
  const profiles = [];
  const crates = [];
  const filters = {};
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
    const filterName = filterFlags.get(argument);
    if (filterName !== undefined) {
      if (filters[filterName] !== undefined) {
        throw new Error(`${argument} may be supplied only once.`);
      }
      filters[filterName] = requireValue(argumentsList, ++index, argument);
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
  const selectsPortable = uniqueProfiles.includes("portable");
  if ((selectsRustOwner || selectsPortable) && uniqueCrates.length === 0) {
    throw new Error(
      "--crate is required for the rust-owner and portable profiles.",
    );
  }
  if (!selectsRustOwner && !selectsPortable && uniqueCrates.length > 0) {
    throw new Error(
      "--crate is valid only with the rust-owner or portable profile.",
    );
  }
  if (selectsPortable) {
    const selectedPortableCrates = uniqueCrates.filter((crate) =>
      portableCrates.has(crate),
    );
    if (selectedPortableCrates.length === 0) {
      throw new Error(
        "The portable profile requires at least one portable --crate owner.",
      );
    }
    if (
      !selectsRustOwner &&
      selectedPortableCrates.length !== uniqueCrates.length
    ) {
      const invalid = uniqueCrates.filter(
        (crate) => !portableCrates.has(crate),
      );
      throw new Error(
        `The portable profile cannot select non-portable crates: ${invalid.join(", ")}.`,
      );
    }
  }

  const hasFilters = Object.keys(filters).length > 0;
  if (hasFilters && !uniqueProfiles.includes("fixtures-conformance")) {
    throw new Error(
      "Regression identity filters are valid only with fixtures-conformance.",
    );
  }

  return {
    profiles: profileOrder.filter((profile) =>
      uniqueProfiles.includes(profile),
    ),
    crates: uniqueCrates,
    filters,
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
        addScript("check:claims:executable");
        addScript("typecheck");
        addScript("test");
        break;
      case "fixtures-conformance":
        addCargoTest("rulebench-fixtures");
        addCommand(commands, regressionCommand(selection.filters));
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
      case "portable":
        addScript("check:rust-boundaries");
        for (const crate of selection.crates.filter((crate) =>
          portableCrates.has(crate),
        )) {
          addCargoTest(crate);
        }
        break;
      case "docs":
        addScript("check:docs");
        addScript("check:claims:executable");
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

function regressionCommand(filters) {
  const argumentsList = [
    "run",
    "--quiet",
    "--manifest-path",
    workspaceManifest,
    "-p",
    "rulebench-fixtures",
    "--bin",
    "check_regressions",
    "--",
  ];
  for (const [flag, filterName] of filterFlags) {
    const value = filters[filterName];
    if (value !== undefined) argumentsList.push(flag, value);
  }
  return {
    id: `regression:${argumentsList.slice(9).join(":") || "all"}`,
    command: "cargo",
    arguments: argumentsList,
  };
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
  if (Object.keys(selection.filters).length > 0) {
    console.log(
      `verify:change filters: ${Object.entries(selection.filters)
        .map(([name, value]) => `${name}=${value}`)
        .join(", ")}`,
    );
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
