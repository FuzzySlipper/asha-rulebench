import { spawnSync } from "node:child_process";

const scenarios = [
  "hexing-bolt-reaction",
  "watchtower-storm-pulse-multiple",
  "binding-glyph-failed-save",
];

for (const scenario of scenarios) {
  console.log(`[regression:gate] ${scenario}`);
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rulebench-rs/Cargo.toml",
      "-p",
      "rulebench-fixtures",
      "--bin",
      "check_regressions",
      "--",
      "--scenario",
      scenario,
    ],
    { cwd: process.cwd(), env: process.env, stdio: "inherit" },
  );
  if (result.error !== undefined) {
    console.error(result.error.message);
    process.exit(1);
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}

console.log(
  `regression:gate ok (${scenarios.length} independently selected cases)`,
);
