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
fails closed for reverse or unknown Rulebench edges, product fixtures imported
by portable owners, ASHA imports outside the downstream module adapter, sibling
ASHA paths, unapproved ASHA crates, forked repositories, stale revisions, and
unbounded compatibility requirements. `pnpm run check:portable-consumer`
separately compiles the supported `rulebench-rules` facade from an independent
Cargo workspace and rejects product-only crates in its transitive tree.

`pnpm run check:rust-test-ownership` keeps focused tests beside every active
authority, adapter, protocol, fixture, and codegen owner while retaining the
`rulebench-authority/src/tests` suite as cross-crate product evidence. Cargo
dev-dependencies remain subject to the same one-way boundary policy.

`pnpm run generated:check` is the canonical generated-artifact gate. It emits
the protocol, scenario catalog, and combat session projections to a temporary
directory and compares all three with their committed outputs. Failures name
the Rust emitter and artifact; `pnpm run generated:write` is the only supported
update path. Every generated header records its emitter and protocol schema.

`pnpm run regression:check` executes every package-owned catalog case twice
through Rust authority, checks its declared outcome plus deterministic
decisions/events/rolls/trace/final state, and reports the first mismatch with a
replay-compatible path. Exact package, package-version, ruleset,
ruleset-version, and scenario filters are accepted by the underlying binary;
`pnpm run regression:list` prints the registered identities before an
intentional package expectation or generated-artifact update.

Operation pipeline v2 adds bounded explicit multi-target and Manhattan-area
actions without changing the legacy single-target path. Rust owns target-set
derivation, roll policy, atomic stateful effects, reaction suspension, replay,
and resource-ledger fingerprints; generated TypeScript only submits and renders
those facts. See `../docs/operation-pipeline-v2.md` for the compatibility and
migration contract.

`pnpm run rust:test` is part of `pnpm run verify`, so clean CI executes the
focused owner suites, cross-crate authority harness, host-neutral bridge
contracts, composed-owner reaction rollback checks, and real process-host
lifecycle/TCP tests rather than relying on TypeScript or generated fixtures.

`pnpm run e2e` exercises the real local process host through the Angular
same-origin proxy, including classified failure recovery and the full
Rust-owned reaction response/archive path. `pnpm run e2e:live` includes that
reaction path plus the artifact-collecting den-serve scenario; live evidence is
still opt-in and must be inspected rather than inferred from the exit code.

To approve a real boundary change, update the owning north-star task and
systems map, revise this dependency direction, change the checker policy and
its focused failure coverage, then land the crate migration with its callers
and tests. Do not add a path dependency simply to preserve an old import path.

## Migration Posture

- Every current workspace crate is an active owner, adapter, fixture, generator, harness, or host surface with focused tests; new empty reservation crates are forbidden as implementation claims.
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
pnpm run rust:host -- --artifact-root .rulebench-artifacts
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

The process host uses in-memory fixture/replay storage unless an artifact root
is configured with `--artifact-root PATH` or `RULEBENCH_ARTIFACT_ROOT`. In the
configured mode it opens separate `content/` and `replays/` directories,
commits replay envelopes and their deterministic index through temporary-file
renames, and prints a repository summary plus classified startup issues.
Unknown format versions, corrupt
fingerprints, and interrupted temporary files are ignored with explicit issue
codes rather than interpreted as current data. Replay envelopes reconstruct
through the registered scenario and Rust authority. Current entries must
reproduce their v1 archive fingerprint before becoming visible. Legacy v0
archive fingerprints were coupled to Rust debug shape; their integrity-checked
command payloads are read-migrated into a new self-consistent v1 entry without
rewriting the source artifact.

The authored content route accepts only `asha-rulebench.content-pack` version
1 documents up to 512 KiB. The protocol DTO owns decoding; portable content
and ruleset crates remain free of JSON/serde concerns. Rust converts the closed
wire vocabulary, validates structural limits and exact dependencies, resolves
ruleset compatibility, canonicalizes the pack, and stores the original payload
beside its canonical receipt. On restart every payload is decoded and imported
again; corrupt, unsupported, dependency-incomplete, or canonically drifted
payloads are classified and excluded from activation. Activation is an atomic
exact-reference index, replacement clears the old activation, and deletion is
denied for active packs or packs with stored dependents. The host audit log
distinguishes payload acceptance, canonical receipt storage, activation,
replacement, deletion, and session binding.

This filesystem repository is deliberately single-writer. Run only one host
against an artifact root; it does not provide locking, multi-process conflict
resolution, or a database migration service. Storage format changes require an
explicit version reader or migration before the current version is advanced.
Replay replacement keeps a temporary copy of the last committed file until the
new envelope and index both commit, rolls back on an index failure, and removes
the copy after success. Content replacement stages the old receipt and payload
until the new pair commits. Startup reports leftover temporary files for manual
cleanup. These guarantees assume same-filesystem local rename behavior; the
adapter does not claim power-loss durability, `fsync`, or atomic replacement on
network filesystems. Future database adapters belong behind the bridge storage
composition seam. In-memory adapters remain the default for isolated tests.

The process host is trusted local development infrastructure. It is not an
authenticated, internet-facing, multi-user, or durable active-session service.
Configured content and finalized replay artifacts survive restart; active
combat sessions do not.

## Dependency Posture

`rulebench-gameplay-module` consumes governed ASHA public Rust facades through
the canonical Git repository at one exact reviewed revision and compatible
`^0.1` versions. No crate imports `asha-engine/engine-rs/crates/*`, and no
sibling checkout is required. Update every ASHA dependency and the boundary
gate to the same reviewed revision, regenerate the lockfiles, and require both
the full local gate and the clean GitHub gate for upgrades. The gameplay module
uses ASHA #5797's preferred composed RuntimeSession owner seam; the quarantined
standalone gameplay host is not a direct Rulebench dependency.

Planner approval in task #5560 authorizes `serde` with derive support for
protocol DTOs and `serde_json` for the concrete process host. Portable authority
crates remain serialization-free. Further external or ASHA dependencies still
require planner approval.

## Non-Claims

This workspace is not an ASHA fork, not a generic rules engine, not a RuntimeSession replacement, not a TypeScript callback host, and not the place to rebuild old RuleWeaver assumptions by default.
