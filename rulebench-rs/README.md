# Rulebench Rust Workspace

This workspace incubates local Rust authority behavior for ASHA Rulebench.

It is intentionally local to this repo. Crates here may use ASHA/ECRP vocabulary, but they are not upstream ASHA crates and should not pretend to be generic engine infrastructure until the behavior proves itself in Rulebench scenarios.

## Crate Structure

The destination crate boundaries are present now so new work has an explicit
home. The portable core, ruleset, content, combat, and replay crates now own
their extracted concerns; Rulebench-local fixtures, codegen, bridge adapters,
and the authority harness remain separate. Further changes move through focused
tasks with behavior-preserving tests.

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
- `hosts/rulebench-process-host`: concrete loopback HTTP/JSON adapter over `rulebench-bridge`; this host is Rulebench-local and not portable authority.

Rulebench-local layers:

- `rulebench-fixtures`: authored scenarios, fixtures, goldens, and regression packs.
- `rulebench-authority`: stable catalog/session emitter commands and the Rulebench integration test harness; it has no library compatibility API.

The boundaries add useful friction. A game repo should be able to share portable
rule behavior without inheriting Rulebench fixtures, generators, or UI machinery.
Rulebench can exercise every portable layer through its harness.

## Supported Portable Facade

`rulebench-rules` is the supported local `v0` convenience facade for a consumer
that wants the complete portable contract from one crate. It exposes:

- authored content and validation values;
- ruleset declarations, module validation, and compatibility identity;
- combat session creation, commands, readbacks, state, resolver, and audit;
- replay specifications and verification.

Consumers that need a smaller dependency surface may use the focused owner
crates directly. The facade does not expose Rulebench fixture catalogs, checked
artifact generation, protocol/bridge adapters, or UI concerns. It is not a
generic rules-engine or host-runtime compatibility promise.

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
       -> bridge + fixtures + protocol -> process host
                       \-> authority
```

The diagram is ownership shorthand rather than a complete Cargo edge list.
Dependencies may point from a higher layer to the lower layers it consumes;
portable layers must never depend on bridge, codegen, fixtures, authority, or UI.

`pnpm run check:rust-boundaries` enforces this workspace graph and is part of
`pnpm run verify`. It also runs focused invalid-dependency checks so the guard
fails closed for a portable-to-Rulebench edge and a portable path dependency on
a frontend surface.

To approve a real boundary change, update the owning north-star task and
systems map, revise this dependency direction, change the checker policy and
its focused failure coverage, then land the crate migration with its callers
and tests. Do not add a path dependency simply to preserve an old import path.

## Migration Posture

- Empty crates are intentional reservation boundaries, not implemented feature claims.
- Move behavior by concern, preserving portable compatibility through `rulebench-rules`; migrate Rulebench callers directly to fixture, protocol, bridge, or codegen owners rather than adding authority forwards.
- Do not create circular dependencies to preserve an old import path.
- Keep scenario-specific assumptions in `rulebench-fixtures` or the Rulebench harness.
- Keep host choice out of `rulebench-bridge`; the selected first concrete adapter lives under `hosts/rulebench-process-host`.

## Commands

From the repo root:

```bash
pnpm run rust:check
pnpm run rust:test
pnpm run rust:host -- --bind 127.0.0.1:4318
```

Or directly:

```bash
cargo check --manifest-path rulebench-rs/Cargo.toml
cargo test --manifest-path rulebench-rs/Cargo.toml
```

## Process Host Lifecycle

`pnpm dev` starts the Rust host on an available loopback port, waits for its
versioned handshake, supplies that URL to the Angular proxy configuration, and
then starts Angular on `0.0.0.0`. The browser uses the same-origin
`/api/rulebench/v1` path; the Rust listener is not exposed directly to the LAN.
Stopping the development command terminates both child processes.

The process host is trusted local development infrastructure. It is not an
authenticated, internet-facing, multi-user, or durable-session service.

## Dependency Posture

The workspace has no path dependency on `/home/dev/asha-engine`.

Planner approval in task #5560 authorizes `serde` with derive support for
protocol DTOs and `serde_json` for the concrete process host. Portable authority
crates remain serialization-free. Further external or ASHA path dependencies
still require planner approval.

## Non-Claims

This workspace is not an ASHA fork, not a generic rules engine, not a RuntimeSession replacement, not a TypeScript callback host, and not the place to rebuild old RuleWeaver assumptions by default.
