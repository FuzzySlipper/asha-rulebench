# ASHA Rulebench

ASHA Rulebench is an Angular/Nx workbench for experimenting with RPG-shaped gameplay rules on top of ASHA/ECRP architecture.

It is not trying to become a full game. It is a place to author, run, inspect, and regression-test rules and scenario fixtures while the underlying authority model stays clean enough to promote useful pieces upstream.

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

The complete destination crate skeleton is present so new work has a durable
owner from the start. `rulebench-rules` and `rulebench-authority` currently hold
the substantial implementation; focused tasks will move concerns into the
reserved `core`, `ruleset`, `content`, `combat`, `replay`, `protocol`, `bridge`,
`codegen`, and `fixtures` crates. See `rulebench-rs/README.md` for ownership and
dependency direction. The workspace has no path dependency on
`/home/dev/asha-engine`. Planner-approved `serde` protocol DTOs and the
`serde_json` process host provide the first live local bridge; further
serialization/codegen crates or ASHA path dependencies remain a
planner-approval moment.

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
The canonical `pnpm run e2e` gate starts this combined stack and completes a
real Rust-owned combat session through the transport, including cleanup and
classified failure/version-mismatch checks.

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
  id: 'hexing_bolt',
  target: target.singleEnemy({ range: 10, lineOfSight: true }),
  attack: attack.vs('Nerve').using('Mind'),
  onHit: [
    effects.damage({ dice: '1d8', stat: 'Mind', type: 'psychic' }),
    effects.modifier({ id: 'rattled', duration: duration.untilEndOfNextTurn() }),
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

For opt-in live evidence:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Use the `local:` URL printed by `den-serve` for Playwright `BASE_URL`. Report
the printed `lan:` URL for human inspection from another machine.

## Current State

This repo was bootstrapped from the UI pattern template, so some code still carries template naming. Treat those names as startup residue, not product direction. Clean them up through workspace-aware generators or focused rename tasks, not opportunistic edits during feature work.
