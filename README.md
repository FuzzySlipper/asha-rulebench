# ASHA Rulebench

ASHA Rulebench is an Angular/Nx workbench for experimenting with RPG-shaped gameplay rules on top of ASHA/ECRP architecture.

It is not trying to become a full game. It is a place to author, run, inspect, and test rules and scenarios while the underlying authority model stays clean enough to be useful in downstream games.

The name is intentional: this is a bench for rules, not an RPG product shell.

## Design Intent

The first useful version should let a user load a simple two-combatant scenario, execute or replay a deterministic rule resolution, and inspect:

- board state;
- chosen actor, action, and target;
- accepted DomainEvents;
- rule trace;
- final state diff;
- fixture or golden result.

The UI can stay modest. The product value is explainability. If the bench says a target took damage, gained a modifier, failed a target legality check, or triggered a reaction window, the user should be able to inspect why.

This repo is also a proving ground. Local Rust crates may incubate RPG-domain or game-rules behavior while ideas are still forming. Pieces that become clearly game-generic can later move upstream into ASHA.

## Rust Authority Workspace

Local Rust authority work lives under `rulebench-rs/`. It is an incubation
workspace with explicit portable-authority, protocol/adapter, and
Rulebench-local crate boundaries, not an upstream ASHA crate set.

The implemented crates have explicit portable-authority, product-adapter, and
repository-harness owners. See `rulebench-rs/README.md` and
`docs/rust-authority-reconciliation.md` for the supported consumer surface and
dependency direction. The `rulebench-gameplay-module` crate consumes only
governed public ASHA facades through one exact Git revision and compatible
facade versions; it does not reach into upstream internal crates or require a
sibling checkout. Its concrete combat owner is installed through the preferred
composed RuntimeSession seam added by ASHA #5797, so Rulebench no longer
declares the quarantined standalone gameplay host.
Planner-approved `serde` protocol DTOs and the `serde_json` process host
provide the live local bridge.

The current gameplay-fabric slice and its explicit local/upstream ownership
boundary are documented in
[docs/gameplay-fabric-integration.md](docs/gameplay-fabric-integration.md).

Run the focused Rust gates with:

```bash
pnpm run rust:check
pnpm run rust:test
pnpm run check:rust-boundaries
```

`pnpm dev` starts both the loopback Rust process host and the Angular server.
Angular remains the only `0.0.0.0` LAN-facing process and proxies the versioned
`/api/rulebench/v1` path to Rust on the same origin. The typed client in
`libs/transport/src/live.ts` owns protocol handshakes, generated DTO requests,
classified host errors, and request cancellation. Stores and components must
consume that public transport boundary rather than call the host directly.
`LiveCombatStore` owns live connection/session selection, command inputs,
preflight, submission, snapshot refresh, stale-response suppression, and
cleanup; domain projections turn Rust evidence into display labels only.
The same live boundary reads `/api/rulebench/v1/capabilities`, whose manifest
is assembled from Rust owner registries plus the selected host composition.
The workbench presents that evidence under **View → Runtime capabilities**;
neither the generated artifact nor the UI grants runtime permission.
`ContentWorkbenchStore` owns the live authored-pack lifecycle through the same
transport: file text crosses a platform port, Rust decodes and semantically
validates the versioned document, and TypeScript displays generic diagnostics,
canonical definitions, diffs, activation state, and audit evidence.
The canonical `pnpm run e2e` gate starts this combined stack and completes a
real Rust-owned combat session through the transport, including cleanup and
classified failure/version-mismatch checks.

For restart-stable authored content, finalized replays, and active sessions,
start the Rust host
with `--artifact-root PATH` (or set `RULEBENCH_ARTIFACT_ROOT`). The concrete
host owns that versioned, single-writer filesystem repository; portable Rust
crates remain serialization-free and the bridge remains host-neutral. At each
accepted command boundary the host atomically stores the canonical scenario,
exact ruleset provenance, typed command history, generation, and verified
authority frame. Startup reconstructs valid sessions by replaying those typed
commands through fresh Rust authority; incompatible or corrupt records are
quarantined instead of becoming live state. The setup tool distinguishes new,
restored, and explicitly forked sessions and provides explicit fork and discard
actions. See `docs/session-recovery.md` for the recovery, migration, and
non-claim boundaries.
The deterministic policy laboratory executes bounded scenario × policy × seed
matrices one trial at a time, archives every trial through the same replay
repository, and exposes cancellation and first-divergence comparison. See
`docs/policy-laboratory.md` for its registration path and non-claims.
Stored authored packs are re-decoded and re-imported on every host start before
their exact activation can be used. A new session may select a compatible
activated pack set; its exact references and set fingerprint are then retained
in the finalized replay. Authored-content v3 adds strict portable modifier and
action declarations while preserving the permanent strict v1 and v2 readers.
Through the `content.authored-action@1` product boundary, Rust binds one exact
active action and actor, derives targets and reaction participants, grants its
ability for that session, executes the selected provider's closed vocabulary,
and retains exact pack/action/ability/fingerprint/grant provenance through
replay-verified restart recovery. See `docs/authored-content-format.md` for the
wire contracts, executable profile, migration posture, and non-claims.
Finalized replay files use the portable, versioned canonical identity described
in `docs/replay-archive-identity.md`; the process host atomically migrates only
recognized legacy identities and quarantines unknown or mismatched records.

## Source Material

This repo starts from four planning references:

- `ruleweaver/rpg-rules-engine-successor-concept`: RuleWeaver successor as an ECRP-aligned RPG rules substrate.
- `ruleweaver/asha-upstream-game-rules-substrate`: game-generic Rust substrate candidates for ASHA main.
- `ruleweaver/asha-rpg-combat-sim-plan`: first workbench/product shape.
- `asha/ecrp-pattern-guide`: reusable ECRP pattern and authority rules.

The old RuleWeaver project is evidence, not structure to preserve. Borrow its useful lessons: tactical expressiveness, action slots, target legality, conditions, modifiers, reaction windows, deterministic traces, and content fixtures. Do not inherit event-handler extensibility, mutable TypeScript contexts, UI/runtime coupling, or 4e ceremony just because those shapes existed before.

## TypeScript And Rust Split

The central rule is:

> TypeScript references and configures Rust behavior. Rust defines and executes rule logic.

TypeScript should be used for:

- typed catalog authoring;
- ergonomic builders over generated protocol shapes;
- scenario fixtures and goldens;
- policy code that reads generated views and proposes typed intents;
- UI view models and display mapping;
- tests that prove authored content produces expected protocol inputs and rendered outputs.

Rust should own:

- content validation;
- rule semantics;
- capability storage and mutation;
- target legality;
- deterministic dice and random streams;
- effect interpretation;
- modifier, condition, and duration lifecycle;
- reaction windows;
- accepted DomainEvents;
- trace, replay, and state hashes.

Good TypeScript describes which Rust operations to run:

```ts
action({
  id: "hexing_bolt",
  target: target.singleEnemy({ range: 10, lineOfSight: true }),
  attack: attack.vs("Nerve").using("Mind"),
  onHit: [
    effects.damage({ dice: "1d8", stat: "Mind", type: "psychic" }),
    effects.modifier({
      id: "rattled",
      duration: duration.untilEndOfNextTurn(),
    }),
  ],
});
```

Bad TypeScript implements authority:

```ts
onHit(ctx) {
  ctx.target.hp -= rollSomeDice();
  ctx.bus.emit('damage-applied');
}
```

If a ruleset needs behavior that is not expressible in the current operation vocabulary, add or incubate a Rust operation instead of smuggling authority into TypeScript.

## ECRP Fit

ASHA Rulebench follows the ECRP framing:

> Entities carry Capabilities; Rules validate; Policies propose; Events record.

In this repo that means:

- stored TS catalogs and scenario files are inputs, not authority;
- runtime authority lives in Rust services, local harnesses, or upstream ASHA runtime surfaces;
- UI projections display truth, they do not own it;
- traces explain resolution decisions, they are not committed facts;
- DomainEvents are the accepted replay/audit spine.

Likely upstream candidates include bounded values, modifiers, periodic effects, hit/hurt facts, reaction windows, traces, replay, and content validation rails. RPG action economy, powers, classes, encounter vocabulary, and mutant ruleset details should stay local until they prove otherwise.

## Frontend Architecture

This frontend is built as layered infrastructure. Keep the existing Angular/Nx boundaries intact unless a task explicitly asks for architecture work.

Layer intent:

- `protocol`: generated protocol exports and shared result/error types.
- `transport`: backend, native, WASM, or fake runtime communication through protocol types.
- `domain`: pure view/domain mapping over protocol data, with no Angular or browser APIs.
- `store`: application state mutation, async state, and transport orchestration.
- `renderer`: feature rendering composition over domain views and presentational components.
- `components`: reusable presentational Angular components.
- `platform`: browser/host ports and fakes.
- `shell`: routing and application composition only.
- `theme`: approved tokens and theme entrypoints.
- `testing-fixtures`: typed fixtures for tests and scenario examples.

Use workspace generators for new components, libraries, features, stores, platform ports, and tests.

## First Slice

The first slice should prove the repository shape before it tries to prove a full rules engine:

1. Replace template identity with ASHA Rulebench naming.
2. Define a canned scenario readout through generated or generated-shaped protocol exports.
3. Add testing fixtures for a two-combatant tactical scenario.
4. Map protocol readouts into domain view models.
5. Add store and fake transport state for loading the scenario result as `AsyncState<T>`.
6. Render a static board, timeline, trace, and final-state summary.
7. Cover the flow with unit tests, deterministic E2E, and a live browser inspection pass.

After that, the next slice can route one typed intent through a Rust-backed local harness or upstream ASHA runtime surface.

## Non-Claims

ASHA Rulebench is not:

- a full RPG or adventure game;
- a straight RuleWeaver port;
- a D&D 4e compatibility target;
- an ASHA fork;
- a generic rules engine;
- a place for mutable TypeScript authority callbacks;
- a renderer-first game prototype;
- a replacement for upstream ASHA runtime, replay, or validation infrastructure.

## Development

Install dependencies:

```bash
pnpm install
```

Run the app on the LAN:

```bash
pnpm run dev
```

Verify the workspace:

```bash
pnpm run verify
```

`pnpm run verify` is the canonical blocking project gate and the GitHub
required check. For explicit owner-matched feedback, use the closed focused
profiles instead of guessing from a Git diff:

```bash
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile rust-owner --crate rulebench-rules
pnpm run verify:change -- --profile protocol-generated --profile host-transport
pnpm run verify:change -- --profile fixtures-conformance --scenario hexing-bolt-reaction
```

Profiles may be repeated and their commands are deduplicated. The runner prints
the complete selection before execution, rejects missing/unknown ownership,
and supports `--dry-run`. Run the blocking gate when classification is
uncertain. The pre-tiering measurements, exact profile contract, blocking
membership, and retained certification paths are documented in
[docs/validation-tiers.md](docs/validation-tiers.md).

Complete deterministic certification is one Rulebench-owned command:

```bash
pnpm run certify
```

It runs static authority/product contracts, the unfiltered
regression/conformance corpus, the independent portable consumer, every
deterministic browser journey (including mobile and accessibility coverage),
and a claims/limitations receipt. It runs nightly and by manual GitHub workflow
dispatch, not as the required check for every edit. The receipt derives current
inventory counts from generated Rust evidence and records the last reviewed Den
limitation snapshot without letting a literal date or copied prose block fail
unrelated source changes.

For a milestone or release with user-visible UI claims, run the live-required
mode against a managed server and inspect the emitted artifacts:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url> LIVE_RUN=1 pnpm run certify -- --require-live
```

Without `--require-live`, certification explicitly does not claim managed/LAN
visual evidence. See
[docs/validation-evidence-template.md](docs/validation-evidence-template.md)
for task handoff evidence.

For opt-in live evidence:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Use the `local:` URL printed by `den-serve` for Playwright `BASE_URL`. Report
the printed `lan:` URL for human inspection from another machine.

## Current State

The human testing surface is a seven-panel Angular workbench. Application menus
open focused setup tools for content packs, live authority evidence, live Rust
sessions, automatic runs, and replay packages; panels retain the board,
initiative, encounter status, available actions, participants, and evidence.
The focused tools configure or select authority behavior, while panel view models
display Rust-owned outcomes. Scenario and transcript evidence is read from the
running process host; checked TypeScript catalogs are offline fixture and golden
evidence only and are never a live fallback. See
`docs/viewer-evidence-boundaries.md` for the complete consumer inventory.

The current surface proves two compiled ruleset providers: Hexing Bolt's
attack-versus-defense package family and Objective Turn Control's
three-participant saving-throw package. Both use deterministic cases, live
commands, bounded automatic control, and replay inspection. With an artifact
root configured, the content tool imports,
reviews, compares, activates, deactivates, and safely deletes authored packs;
authored packs, finalized replays, and verified active-session checkpoints
survive host restart. Live session snapshots expose authoritative board and
participant positions to the workbench.

The authored-action claim is deliberately narrow: no arbitrary scripts,
plugins, callbacks, general character/class/item/resource authoring, or
TypeScript rule authority, and no guarantee beyond the exact closed v3
vocabulary and capabilities of the selected compiled provider.

The executable capability manifest reports the current compiled providers,
rulesets, packages, scenarios, automation policies, and operation-pipeline
identity. Its checked TypeScript projection records provider
capability/vocabulary compatibility, the exact governed ASHA revision, and the
configured durable-host support matrix; the live workbench always reads the
current process-host manifest instead. The certification receipt derives its
inventory from that generated evidence instead of synchronizing prose counts.
See `docs/capability-manifest.md` and `docs/ruleset-providers.md` for the
authority and evolution contracts.
