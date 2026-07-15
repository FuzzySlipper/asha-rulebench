# Rust Authority Reconciliation

This document records the post-gameplay-fabric disposition of Rulebench's Rust
crates. It is a compatibility map, not a claim that RPG mechanics belong in
ASHA. The governing split remains:

> ASHA owns generic gameplay fabric and RuntimeSession infrastructure.
> Rulebench owns RPG combat semantics and its product adapters.

## Crate disposition

| Surface | Disposition | Authority and consumer posture |
|---|---|---|
| `rulebench-core` | Keep local and portable | RPG-facing identifiers, bounded values, event/trace primitives, and deterministic fingerprints. It has no ASHA or Rulebench product dependency. |
| `rulebench-ruleset` | Keep local and portable | Declarative RPG ruleset/module vocabulary and compatibility checks. This configures local Rust behavior; it is not an alternate ASHA module registry. |
| `rulebench-content` | Keep local and portable | RPG catalogs, canonicalization, references, validation, diagnostics, import, and storage. Its canonicalization remains incubation evidence until a non-RPG ASHA consumer needs the same contract. |
| `rulebench-gameplay-module` | Keep as a downstream module adapter | Implements the Rulebench pre-effect module through `asha-gameplay-module-sdk` and installs its concrete combat-owner adapter through `asha-runtime-session-composition`. It uses canonical codecs, derived schemas and build provenance, declared reads, the closed registry, and ASHA-owned continuation/routing/checkpoint evidence. |
| `rulebench-combat` | Keep local and portable | Authoritative RPG state, targeting, resources, effect interpretation, reaction ordering, duration expiry, and accepted DomainEvents. It calls the gameplay adapter for a bounded pre-effect decision; it does not copy ASHA fabric internals. |
| `rulebench-replay` | Keep local and portable | RPG command packages, archive/review/comparison, randomness provenance, and first-mismatch diagnostics. Gameplay module-state and decision-receipt hashes come from ASHA-owned readouts and are compared as evidence rather than reimplemented. |
| `rulebench-rules` | Supported public Rulebench facade | The one-crate local `v0` consumer root for portable RPG authority. It re-exports the supported core/ruleset/content/combat/replay contract and excludes product fixtures, protocol, bridge, codegen, and hosts. |
| `rulebench-protocol` | Keep product-local | Generated Rust-owned DTOs and TypeScript metadata for Rulebench commands/readbacks, including reaction responses. It maps authority values and does not duplicate ASHA gameplay DTOs. |
| `rulebench-bridge` | Keep product-local and host-neutral | Owns session handles and maps versioned Rulebench protocol requests to the portable authority. It does not select HTTP, browser APIs, or a second rules engine. |
| `rulebench-codegen` | Keep product-local | Emits checked TypeScript protocol and fixture artifacts from Rust sources. Generated outputs are never hand-edited. |
| `rulebench-fixtures` | Keep product-local | Authored scenarios, regression packs, and deterministic evidence. It may exercise portable behavior but is not part of the consumer contract. |
| `rulebench-authority` | Keep as a repository harness | Stable generator commands and integration tests only; no library compatibility API. |
| `rulebench-process-host` | Keep as the concrete local adapter | Loopback HTTP/JSON over `rulebench-bridge`, reached through the Angular same-origin proxy. It owns no RPG semantics and is not a public engine host. |

## Supported public RPG consumer surface

`rulebench-rules` is the supported convenience facade. The independent
`rulebench-rs/portable-consumer-smoke` workspace consumes only that crate,
authors a scenario, executes a command, and reads the resulting state. The
`check:portable-consumer` gate runs that program and rejects any transitive
dependency on fixtures, protocol, bridge, codegen, authority, or the process
host.

Focused consumers may depend directly on `rulebench-core`,
`rulebench-ruleset`, `rulebench-content`, `rulebench-combat`, or
`rulebench-replay`. Those owner crates are public to this repository's RPG
consumer contract, not generic ASHA APIs. New product adapters must not be
added to the portable graph.

## ASHA ownership used directly

The gameplay-module slice uses ASHA's public owners for:

- immutable module and binding registries;
- generated contract references, canonical codecs, schema identities, and
  meaningful build provenance;
- frozen declared reads and typed module state/facts;
- Guard/Transform/React dispatch and wave-owned reaction frames;
- continuation issuance, validation, consumption, and decision receipts;
- proposal-owner routing, scheduler declaration, snapshot text, and module,
  registry, frame, routing, and host hashes.

Rulebench keeps only the product adaptation: opening and ordering RPG reaction
windows, selecting an authored response, validating the local combat owner
revision, and applying the exact accepted damage workspace. The composed owner
checkpoint stages the response and workspace; stale or consumed continuations
cannot retain owner-adapter, module, frame, receipt, or continuation mutation.

ASHA #5797 closed the composed-owner gap. Rulebench now installs one concrete
Rust owner through `StaticRuntimeSessionBuilder::with_gameplay_owner`. The
composed transaction validates the continuation and expected revision, retains
the typed response and committed workspace in the owner checkpoint, routes one
accepted commit, emits opened/resolved facts through the closed registry, and
rolls owner adapter state, module state, frames, receipts, and continuations
back together on rejection. The product combat mutation is validated before
entering that transaction and applied infallibly from the exact accepted
workspace afterward; there is no TypeScript ferry, arbitrary JSON bridge,
mutable owner registry, or private engine import.

## Upstream candidate evaluation

| Candidate | Current result |
|---|---|
| Replay comparison and mismatch diagnostics | Keep local. Rulebench compares RPG commands, rolls, accepted events, narration, and fabric evidence. ASHA owns the underlying fabric receipt/frame integrity; no missing generic API was found. |
| Content canonicalization | Incubate locally. The implementation is RPG catalog-shaped and has no demonstrated second non-RPG consumer. |
| Gameplay-module conformance fixture | Already owned upstream by `asha-gameplay-module-conformance` and ASHA's downstream-module fixtures. Rulebench adds product behavior tests, not a competing generic harness. |
| Scheduler and duration behavior | Split. ASHA owns generic gameplay scheduling and recovery; Rulebench's turn/round modifier expiry is RPG semantics and remains local. No new generic scheduler primitive is required. |
| Composed downstream combat owner | Adopted from ASHA #5797. The standalone host declaration is gone; the preferred composed cell owns the typed adapter checkpoint, continuation transaction, accepted routing evidence, and owner facts. |
| Versioned Rust distribution | Adopted from ASHA #5796. Rulebench consumes the governed public Git workspace at exact revision `67ce55dba602ad61e1b9ca3b0ad01a22fa4fe148` with compatible `^0.1` facade versions. |

## Executable capability evidence

Capability claims are generated from owner registries, not maintained as a
second handwritten inventory. `rulebench-ruleset` declares the versioned
operation and targeting vocabulary; `rulebench-combat` registers executable
operations and closed automation policies; `rulebench-fixtures` contributes
actual package, scenario, and regression identities; and the concrete host
adds its selected storage and recovery composition. The manifest preserves
separate declared, validated, executable, protocol, live-host, UI, regression,
and restart-durability levels so a downstream projection cannot promote a
weaker claim.

`rulebench-process-host/emit_capability_manifest` emits the checked durable-host
projection, while `GET /api/rulebench/v1/capabilities` reports the running
host's actual composition through generated DTOs. The Angular store and view
consume only that live route. See `capability-manifest.md` for the versioning,
negative invariants, and extension procedure.

## Distribution status

ASHA #5796 added the governed `public-rust/Cargo.toml` Git workspace. Rulebench
pins every ASHA facade to the same reviewed 40-character revision and declares
the compatible `^0.1` version range. `check:rust-boundaries` rejects path
dependencies, unknown ASHA crates, noncanonical repositories, mixed or stale
revisions, and incompatible versions. The ordinary GitHub `pnpm run verify`
job runs the portable Rust consumer from a clean Rulebench checkout with no
sibling ASHA tree, so the release boundary is exercised rather than inferred.

To upgrade, select one reviewed ASHA commit from the public compatibility log,
change the shared revision in the gameplay-module manifest and boundary gate,
regenerate both Cargo lockfiles through the normal checks, then require the
full local and clean GitHub gates. Copying or forking engine crates here remains
forbidden.
