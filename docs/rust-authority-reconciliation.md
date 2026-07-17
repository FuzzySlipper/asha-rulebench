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
| `rpg-ir` | Strict `asha.rpg.ir@1` decode types plus extracted declaration vocabulary. |
| `rpg-compiler` | Closed operation bindings, requirement/reference/semantic validation, opaque compiled programs, deterministic evaluation, owner staging, DomainEvents, trace, and typed diagnostics. |
| `rpg-runtime` | Private semantic authority sessions plus the public-ASHA-backed decision/reaction fabric. |
| `asha-rpg` | Supported public Rust facade over the active portable owners. |
| `@asha-rpg/ir` | Immutable TypeScript IR package location; semantic execution is forbidden. |
| `@asha-rpg/authoring` | Pure TypeScript authoring package location; complete syntax arrives in #5937. |

The packages are fetched from `https://github.com/FuzzySlipper/asha-rpg.git`
at exact revision `a3d4acd9ac0ff0517a71605bb291a4a844040d9a` with compatible `^0.1`
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
`95907505ffcc942095953e5786186a18119cd97e`, compiles and executes a normalized
action through the semantic kernel, then opens and resumes the typed pre-effect
decision through a consumer-owned authority. It has no Rulebench crate in its
graph. The enclosing repository commit advances after pinning so the consumer
never relies on an unpublished sibling path.

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

The initial #5936 semantic profile deliberately excludes the additional
operations and scheduler/replay surfaces listed in `asha-rpg` non-claims. The
#5937 TypeScript authoring language is not implemented yet. Rulebench content,
automation, storage, archives, experiments, fixtures, certification, Angular,
and filesystem concepts remain outside the public RPG graph. The temporary
adapter is not a stable compatibility API.
