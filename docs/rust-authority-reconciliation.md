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
| `rulebench-gameplay-module` | Keep as a downstream module adapter | Implements the Rulebench pre-effect module through `asha-gameplay-module-sdk`. It uses canonical codecs, derived schemas and build provenance, declared reads, the closed registry, and ASHA-owned continuation/routing evidence. Its direct `asha-gameplay-runtime-host` use is quarantined pending ASHA #5797. |
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
revision, and committing the transformed damage workspace. The adapter stages
the host snapshot and publishes it only after ASHA accepts the owner route, so a
stale or consumed continuation cannot partially mutate live fabric state.

The current direct `asha-gameplay-runtime-host` dependency is not the final
preferred topology. ASHA now marks that facade quarantined and names #5715 as
its Rulebench deletion owner. The preferred `asha-runtime-session-composition`
cell does not yet expose a governed way for Rulebench's independent Rust combat
owner to publish its typed opened/resolved facts and atomically bind its local
commit to the composed checkpoint. ASHA #5797 owns that real upstream gap.
Rulebench will not replace it with a TypeScript ferry, arbitrary JSON bridge,
or private engine import.

## Upstream candidate evaluation

| Candidate | Current result |
|---|---|
| Replay comparison and mismatch diagnostics | Keep local. Rulebench compares RPG commands, rolls, accepted events, narration, and fabric evidence. ASHA owns the underlying fabric receipt/frame integrity; no missing generic API was found. |
| Content canonicalization | Incubate locally. The implementation is RPG catalog-shaped and has no demonstrated second non-RPG consumer. |
| Gameplay-module conformance fixture | Already owned upstream by `asha-gameplay-module-conformance` and ASHA's downstream-module fixtures. Rulebench adds product behavior tests, not a competing generic harness. |
| Scheduler and duration behavior | Split. ASHA owns generic gameplay scheduling and recovery; Rulebench's turn/round modifier expiry is RPG semantics and remains local. No new generic scheduler primitive is required. |
| Composed downstream combat owner | Missing upstream. ASHA #5797 records the narrow one-cell owner/fact/continuation transaction required to remove the quarantined standalone host. |
| Versioned Rust distribution | Missing upstream. ASHA #5796 records the release boundary required by Rulebench #5680. |

## Distribution status

At ASHA commit `5545ae9ab76253ac6bde91937c6cb906af99760b`, the approved public Rust
facades are `publish = false`, the engine repository has no root Cargo
workspace for git dependency discovery, and the compatibility guide still
documents sibling `public-rust/*` path dependencies. Rulebench therefore keeps
its current development-only paths until ASHA #5796 supplies a supported,
versioned distribution. Copying or forking engine crates here is forbidden.

