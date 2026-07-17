# Rulebench Rust Workspace

This workspace owns Rulebench product behavior, adapters, storage, named content,
protocol generation, and its concrete process host. Portable RPG substrate is
owned by the public `asha-rpg` repository and enters this workspace only
through exact Git dependencies.

## Crate Structure

Pinned public RPG dependencies:

- `rpg-core`: dependency-free values and deterministic fingerprint vocabulary.
- `rpg-ir`: normalized rules, operation vocabulary, and compatibility identity.
- `rpg-runtime`: the public-ASHA RuntimeSession decision/reaction fabric.

Rulebench product authority:

- `rulebench-content`: content packs, references, validation, diagnostics, and indexing.
- `rulebench-combat`: combat state, resolution, lifecycle, and manual/automatic control.
- `rulebench-replay`: audit packages, replay specifications, verification, and mismatch diagnostics.

Boundary and adapter layers:

- `rulebench-protocol`: Rust-owned command/readback DTO contracts.
- `rulebench-bridge`: protocol-to-authority runtime invocation, independent of the eventual host technology.
- `rulebench-codegen`: Rust protocol DTO generation for the product client.
- `hosts/rulebench-process-host`: concrete loopback HTTP/JSON adapter over `rulebench-bridge`; this host is Rulebench-local and not portable authority.

Rulebench-local layers:

- `rulebench-product-content`: named scenarios and primary workflow samples used by the product host.

The reusable public facade is `asha-rpg` in its own repository. This workspace
does not provide a game-consumer facade or a combined product compatibility
facade.

## Dependency Direction

```text
public asha-rpg: rpg-core + rpg-ir + rpg-compiler + rpg-runtime
                    |          |            |             |
                    +------> content ----> combat ----> replay
                                |            |            |
                                +--------> protocol <-----+
                                             |
                                           bridge
                                             |
                                      process host

named product content imports focused owners directly and feeds the bridge and host.
```

`pnpm run check:rust-boundaries` enforces this workspace graph and is part of
`pnpm run verify`. It fails closed for reverse or unknown Rulebench edges,
direct ASHA imports, sibling RPG paths, noncanonical repositories, stale
revisions, and unbounded version requirements. The independent public consumer
proof lives with `asha-rpg`, not inside this product repository.

`pnpm run check:rust-test-ownership` keeps focused tests beside every active
authority, adapter, protocol, content, and codegen owner. Cargo dev-dependencies
remain subject to the same one-way boundary policy.

`pnpm run generated:check` is the product generated-artifact gate. It emits the
Rust protocol DTO contract to a temporary directory and compares it with the
committed TypeScript output. Scenario, session, capability, and certification
proof artifacts and their emitters are downstream-owned by
`FuzzySlipper/asha-rulebench-testing`.

Operation pipeline v2 adds bounded explicit multi-target and Manhattan-area
actions without changing the legacy single-target path. Rust owns target-set
derivation, roll policy, atomic stateful effects, reaction suspension, replay,
and resource-ledger fingerprints; generated TypeScript only submits and renders
those facts. See `../docs/operation-pipeline-v2.md` for the compatibility and
migration contract.

The executable manifest is assembled at runtime from operation and policy
registries, named product packages, the compiled provider catalog, and the
concrete host's selected storage/recovery adapters. The process host serves the
typed DTO at `GET /api/rulebench/v1/capabilities`; a memory-mode host therefore
cannot inherit durable support from a filesystem composition. Exhaustive
coverage is a downstream result, not a product runtime fact. See
`../docs/capability-manifest.md`.

`pnpm run rust:test` is part of `pnpm run verify`, so clean CI executes the
focused owner suites, host-neutral bridge contracts, composed-owner reaction
rollback checks, and real process-host lifecycle/TCP tests.

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

- Every current workspace crate is an active owner, adapter, product-content, generator, or host surface with focused tests; new empty reservation crates are forbidden as implementation claims.
- Import focused owners directly; do not add a combined compatibility facade.
- Do not create circular dependencies to preserve an old import path.
- Keep scenario-specific assumptions in `rulebench-product-content` or the Rulebench harness.
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
Unknown format or archive-identity versions, corrupt fingerprints, and
interrupted temporary files are ignored with explicit issue codes rather than
interpreted as current data. Replay envelopes reconstruct through the
registered scenario and Rust authority. Current entries must reproduce the
portable `asha-rulebench.replay-archive-payload.v2` canonical identity before
becoming visible. Recognized legacy v0/v1 identities are authority-verified and
atomically rewritten as envelope v2; a failed rewrite quarantines the entry and
leaves its source unchanged. The encoder is owned by `rulebench-replay` and is
independent of Rust `Debug`, private layout, and host JSON. See
`docs/replay-archive-identity.md` for the version boundaries and migration
policy.

The authored content route accepts strict `asha-rulebench.content-pack`
versions 1, 2, and 3 up to 512 KiB. V1 owns ruleset/entity catalogs, v2 adds
ability metadata, and v3 adds the portable modifier and authored-action
declarations documented in `../docs/authored-content-format.md`. The protocol
DTO owns decoding; portable content and ruleset crates remain free of
JSON/serde concerns. Rust converts the closed wire vocabulary, validates
structural limits, exact dependencies, and provider capabilities, canonicalizes
the pack, and stores the original payload beside its canonical receipt. The v3
binder resolves one exact active action/ability/modifier set, derives scenario
targets and reaction participants, checks actor resources, creates a
session-local ability grant, and routes execution through the normal Rust
events, trace, replay, and recovery owners. On restart every payload is decoded
and imported again; corrupt, unsupported, dependency-incomplete, or canonically
drifted payloads are classified and excluded from activation. Activation is an
atomic exact-reference index, replacement clears the old activation, and
deletion is denied for active packs or packs with stored dependents. The host
audit log distinguishes payload acceptance, canonical receipt storage,
activation, replacement, deletion, and session binding.

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
authenticated, internet-facing, multi-user service. With an artifact root,
configured content, finalized replay artifacts, and verified active-session
recovery packages survive restart. Recovery is deliberately limited to fully
accepted command boundaries: startup reconstructs a fresh authority session by
replaying canonical typed commands and admits it only when command evidence,
ruleset provenance, generation, state fingerprint, gameplay-module hash, and
pending reaction-window identity all agree. Corrupt, partial, unknown-version,
or incompatible recovery records are quarantined. The host does not persist
opaque ASHA continuations or arbitrary in-flight CPU state. See
`../docs/session-recovery.md`.

## Dependency Posture

Rulebench consumes governed `asha-rpg` packages through the canonical public
Git repository at one exact reviewed revision and compatible `^0.1` versions.
No Rulebench crate imports ASHA directly or uses a sibling `asha-rpg` path.
Update the shared RPG revision, boundary gate, and Cargo lock together, then
require both the local and exact-SHA GitHub gates. `rpg-runtime` owns the ASHA
RuntimeSession composition behind that public boundary.

Planner approval in task #5560 authorizes `serde` with derive support for
protocol DTOs and `serde_json` for the concrete process host. Further external
dependencies still require planner approval.

## Non-Claims

This workspace is not an ASHA fork, not a generic rules engine, not a RuntimeSession replacement, not a TypeScript callback host, and not the place to rebuild old RuleWeaver assumptions by default.
