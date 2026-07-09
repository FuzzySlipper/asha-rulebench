# Rulebench Rust Workspace

This workspace incubates local Rust authority behavior for ASHA Rulebench.

It is intentionally local to this repo. Crates here may use ASHA/ECRP vocabulary, but they are not upstream ASHA crates and should not pretend to be generic engine infrastructure until the behavior proves itself in Rulebench scenarios.

## Crate Structure

The destination crate boundaries are present now so new work has an explicit
home. Only `rulebench-rules` and `rulebench-authority` currently own substantial
behavior; the other crates are documented, compiling reservation boundaries.
Code moves into them through focused tasks with behavior-preserving tests.

Portable authority layers:

- `rulebench-core`: shared identifiers, values, event/trace primitives, and fingerprints.
- `rulebench-ruleset`: behavior-module declarations and operation vocabulary.
- `rulebench-content`: content packs, references, validation, diagnostics, and indexing.
- `rulebench-combat`: combat state, resolution, lifecycle, and manual/automatic control.
- `rulebench-replay`: audit packages, replay specifications, verification, and mismatch diagnostics.
- `rulebench-rules`: current portable implementation and eventual convenience facade over the layers above.

Boundary and adapter layers:

- `rulebench-protocol`: Rust-owned command/readback DTO contracts.
- `rulebench-bridge`: protocol-to-authority runtime invocation, independent of the eventual host technology.
- `rulebench-codegen`: Rulebench-local TypeScript and checked-artifact generation.

Rulebench-local layers:

- `rulebench-fixtures`: authored scenarios, fixtures, goldens, and regression packs.
- `rulebench-authority`: Rulebench facade/testbench and current compatibility surface.

The boundaries add useful friction. A game repo should be able to share portable
rule behavior without inheriting Rulebench fixtures, generators, or UI machinery.
Rulebench can exercise every portable layer through its harness.

## Dependency Direction

```text
core
  -> ruleset
  -> content
  -> combat
  -> replay
  -> rules (portable facade)
       -> protocol -> bridge
       -> fixtures -> codegen
                       \-> authority
```

The diagram is ownership shorthand rather than a complete Cargo edge list.
Dependencies may point from a higher layer to the lower layers it consumes;
portable layers must never depend on bridge, codegen, fixtures, authority, or UI.

## Migration Posture

- Empty crates are intentional reservation boundaries, not implemented feature claims.
- Move behavior by concern, preserving public compatibility through `rulebench-rules` and `rulebench-authority` while callers migrate.
- Do not create circular dependencies to preserve an old import path.
- Keep scenario-specific assumptions in `rulebench-fixtures` or the Rulebench harness.
- Keep host choice out of `rulebench-bridge`; concrete native, WASM, process, or service adapters can be added after the live bridge design is selected.

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
