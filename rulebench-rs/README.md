# Rulebench Rust Workspace

This workspace incubates local Rust authority behavior for ASHA Rulebench.

It is intentionally local to this repo. Crates here may use ASHA/ECRP vocabulary, but they are not upstream ASHA crates and should not pretend to be generic engine infrastructure until the behavior proves itself in Rulebench scenarios.

## Current Crates

- `rulebench-rules`: reusable rule model, state, resolution, content validation, modifiers, and session runtime substrate. This crate should stay free of Rulebench artifact emitters, canned transcript catalogs, and UI fixture generation.
- `rulebench-authority`: Rulebench-local facade and testbench crate. It re-exports `rulebench-rules` for existing callers and owns fixtures, scenario/session catalogs, checked artifact emitters, and transport-generation support.

The crate boundary is meant to add useful friction. A game repo should be able to share `rulebench-rules` behavior without inheriting Rulebench's testing/artifact machinery, while Rulebench can still exercise and inspect that substrate through the facade.

## Commands

From the repo root:

```bash
pnpm run rust:check
pnpm run rust:test
```

Or directly:

```bash
cargo check --manifest-path rulebench-rs/Cargo.toml
cargo test --manifest-path rulebench-rs/Cargo.toml
```

## Dependency Posture

The initial workspace has no external Rust dependencies and no path dependency on `/home/dev/asha-engine`.

Add dependencies only after planner approval. This includes serialization/codegen crates such as `serde`, `serde_json`, `schemars`, or `ts-rs`, and also local path dependencies into ASHA engine crates.

## Non-Claims

This workspace is not an ASHA fork, not a generic rules engine, not a RuntimeSession replacement, not a TypeScript callback host, and not the place to rebuild old RuleWeaver assumptions by default.
