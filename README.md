# ASHA Rulebench

ASHA Rulebench is the authoring and inspection product for compiled Asha RPG
rulesets. Den tasks #5953, #5955, and #5957 now build on the deliberate empty
boundary from #5952:

- no prototype content is bundled;
- no ruleset is selected by default;
- no scenario, import, filename, or startup behavior can construct a ruleset;
- no Rulebench-owned combat, replay, or semantic authority remains;
- the UI starts and clearly reports **No compiled ruleset active**;
- one explicit fresh TypeScript composition can be compiled, inspected,
  atomically activated, and exercised through Asha RPG authority without
  introducing TypeScript gameplay authority.

Content enters only through the explicit manifest/compiler boundary. Each
compile click sends the explicit source selection through a generated request
DTO. A loopback trusted-authoring gateway prepares that TypeScript package graph
for the request and passes only its closed prepared source to Rust. The current
fresh composition resolves an exact package lock and exported-root closure;
Rust derives its private normalized execution input from that one closed
definition graph, compiles it, round-trips the portable artifact through the
authoritative loader, and only then creates an activation candidate. Source
directories, server startup, browser execution, and import side effects never
determine runtime meaning. A selectable invalid graph proves source diagnostics
remain user-reachable without replacing the active artifact or its persistent
session.

Activation creates one fresh Asha RPG authority session. The browser displays
Rust-provided action catalogs, source identity, candidates, preflight, random
requests, accepted events, trace, and state. It sends only typed intents,
explicit random evidence, and typed reaction decisions. The current field
manual has four TypeScript-authored actions, including one materialized derived
action. Their sequential workflow moves an actor, applies a modifier, spends a
bounded resource, resolves d6 and five-d4 requests, and suspends/resumes a
reaction against the same state revision. Rust owns all interpretation and
mutation.

## Retained product surfaces

- `apps/app`: Angular bootstrap.
- `apps/app-e2e`: focused compiler/activation browser and managed live evidence.
- `libs/content-authoring`: fresh immutable TypeScript package graph choices and
  on-demand preparation; no global registry, discovery, callbacks, scenarios,
  or raw-IR product catalog.
- `libs/protocol`: generated Rust host DTOs plus a strict decoder.
- `libs/transport`, `libs/domain`, `libs/store`: generated-DTO transport,
  pure inspection mapping, and explicit lifecycle state.
- `libs/components`: generic workbench panels and menus used for artifact and
  authority-session inspection.
- `libs/platform`: browser ports, including the JSON HTTP boundary used by the
  same-origin compiler transport.
- `libs/scenario-viewer`: compiler, exact lock, closure, diagnostics,
  fingerprint, activation, commands, reactions, and state inspection.
- `libs/shell`: routes only.
- `libs/theme`: product tokens.

`rulebench-rs/hosts/ruleset-host` is a fresh narrow loopback host pinned to the
public Asha RPG revision. It owns compile/load/inspect/activate lifecycle state
and the product lifecycle of one Asha RPG session. Asha RPG owns the session's
rules, legality, state mutation, randomness, events, trace, and reaction
transaction. Rulebench does not reapply events or mirror semantic state.

## Commands

```bash
pnpm run verify
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile content-authoring
pnpm run verify:change -- --profile rust-owner
pnpm run verify:change -- --profile protocol-generated
pnpm run verify:change -- --profile host-transport
pnpm run verify:change -- --profile browser
pnpm run verify:change -- --profile docs
```

For managed visual evidence:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Rulebench's blocking gate is focused product validation, including the fresh
Rust host and generated protocol. Exhaustive
cross-repository certification remains downstream, but the old prototype
expectations have been retired rather than preserved there.

Non-claims: session persistence across process restarts or artifact activation,
replay/checkpoints, migration, nested reaction windows, upgrade migration
policy, and exhaustive cross-product certification are not implemented here.

The fresh composition now demonstrates one primary base, two ordered typed
mixins, a local relational patch, an authorized semantic overlay, and a
presentation-only overlay. Rulebench displays the exact base/mixin chain,
parameters, patch fingerprints, before/after values, effectiveness, impact
planes, and final definition fingerprints emitted by Asha RPG; it does not
reimplement materialization semantics.

After an artifact is active, the field-manual 1.1 source can be compiled as a
candidate without activation. The Rust host structurally compares the two
fully materialized accepted artifacts and Rulebench displays changed package
sources, changed definitions, derived descendants, causes, and exact semantic
or presentation field transitions. Source selection is an open string request
validated against the authoring-owned options, so adding ordinary content does
not extend a Rust enum or product protocol vocabulary.

See [docs/empty-ruleset-boundary.md](docs/empty-ruleset-boundary.md) for the
deletion and retention inventory.
