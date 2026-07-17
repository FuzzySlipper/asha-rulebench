# Rust Authority Reconciliation

This document records the repository boundary after the first `asha-rpg`
source-authority extraction.

> ASHA owns generic gameplay fabric and RuntimeSession infrastructure.
> `asha-rpg` owns portable RPG values, language IR, and semantic execution.
> Rulebench owns the interactive product, content workflows, storage, and proof consumers.

## Extracted public owners

| Public package | Authority and consumer posture |
|---|---|
| `rpg-core` | Dependency-free RPG values and deterministic fingerprint vocabulary. |
| `rpg-ir` | Normalized rule declarations, operation vocabulary, compatibility, and typed ruleset views. |
| `rpg-runtime` | Public-ASHA-backed RuntimeSession decision/reaction fabric, owner revision checks, accepted commit routing, snapshots, and typed readouts. |
| `asha-rpg` | Supported public Rust facade over the active portable owners. |
| `@asha-rpg/ir` | Immutable TypeScript IR package location; semantic execution is forbidden. |
| `@asha-rpg/authoring` | Pure TypeScript authoring package location; complete syntax arrives in #5937. |

The packages are fetched from `https://github.com/FuzzySlipper/asha-rpg.git`
at exact revision `7d6430bb3fb9809a6d35636275ef2e3e36ae9407` with compatible `^0.1`
versions. Rulebench has no sibling path dependency and no direct ASHA crate
dependency. The extracted repo retains the exact governed ASHA revision
`67ce55dba602ad61e1b9ca3b0ad01a22fa4fe148` behind `rpg-runtime`.

## Rulebench owners

| Surface | Disposition |
|---|---|
| `rulebench-content` | Product content packs, validation, import, storage, and diagnostics. It consumes public RPG values and declarations. |
| `rulebench-combat` | Product combat/session behavior and the current consumer of `rpg-runtime`. Later migration tasks continue separating portable semantic execution from product orchestration. |
| `rulebench-replay` | Product archive, comparison, recovery, and review surfaces. |
| `rulebench-rpg-adapter` | Temporary combined adapter required by current protocol, bridge, and fixture consumers. #5938 owns its deletion. |
| `rulebench-protocol` | Generated product DTOs and TypeScript metadata. |
| `rulebench-bridge` | Host-neutral product invocation. |
| `rulebench-codegen` | Product code and checked-artifact generation. |
| `rulebench-fixtures` | Product scenarios and regression inputs until #5942 moves exhaustive proof ownership. |
| `rulebench-authority` | Repository generator commands and cross-crate harness. |
| `rulebench-process-host` | Concrete loopback product host and durable repository adapters. |

The former `rulebench-core`, `rulebench-ruleset`, and
`rulebench-gameplay-module` crates were deleted after every call site moved to
the public Git packages. The former `rulebench-rules` portability facade and
its repository-local consumer smoke were retired. The replacement adapter is
explicitly product-local and temporary; it is not a supported game-consumer
surface.

## Public consumer evidence

`asha-rpg/consumers/minimal-game` is an independent Cargo workspace. It fetches
the public facade from Git at exact revision
`d8701adcf34f58bc911df0669e3c92aa9919fc7f`, opens and resumes the typed
pre-effect decision, validates and commits through a consumer-owned authority,
and checks the deterministic readout. It has no Rulebench crate in its graph.

This proof belongs to the portable owner. Rulebench certification no longer
contains a self-consuming portability facade or claims that product storage,
archives, fixtures, protocol, or browser workflows are reusable RPG substrate.

## Enforced direction

`pnpm run check:rust-boundaries` verifies:

- the exact public `asha-rpg` repository, revision, and compatible version;
- workspace inheritance for every RPG dependency;
- no sibling RPG paths and no direct ASHA imports;
- protocol and fixture consumers reach the temporary adapter until #5938;
- no unknown or reverse Rulebench crate dependencies.

The checked capability manifest identifies normalized operation and targeting
vocabulary evidence as `asha-rpg.*`; generated artifacts are refreshed only
through `pnpm run generated:write`.

## Non-claims

This extraction does not claim that the complete #5936 compiler or #5937
TypeScript authoring language already exists. It does not make Rulebench
content, automation, storage, archives, experiments, fixtures, certification,
Angular, or filesystem concepts part of the public RPG graph. The temporary
adapter is not a stable compatibility API.
