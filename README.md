# ASHA Rulebench

ASHA Rulebench is the authoring and inspection product for compiled Asha RPG
rulesets. Den tasks #5953, #5955, #5957, and #5956 now build on the deliberate empty
boundary from #5952:

- no prototype content is bundled;
- no ruleset is selected by default;
- no scenario, import, filename, or startup behavior can construct a ruleset;
- no Rulebench-owned combat, replay, or semantic authority remains;
- the UI starts and clearly reports **No compiled ruleset active**;
- one explicitly selected TypeScript ruleset root can be compiled, inspected,
  atomically activated, and exercised through Asha RPG authority without
  introducing TypeScript gameplay authority.

Content enters only through the canonical ruleset-root compiler boundary. Each
compile click sends one `rulesetRoot` through a generated request DTO. The root
must be a direct child of `rulesets/`; Rulebench infers `ruleset.ts#ruleset` and
permits only that root plus its repository's conventional `foundations/`. A loopback
trusted-authoring gateway launches a fresh constrained TypeScript build for
that request and passes only its closed prepared source to Rust. It does not
scan directories or consult a product source catalog. The current example
composition resolves an exact package lock and exported-root closure;
Rust derives its private normalized execution input from that one closed
definition graph, compiles it, round-trips the portable artifact through the
authoritative loader, and only then creates an activation candidate. Source
directories, server startup, browser execution, and import side effects never
determine runtime meaning. Invalid build and package graphs prove source
diagnostics remain user-reachable without replacing the active artifact or its
persistent session.

Activation creates one fresh Asha RPG authority session. The browser's primary
surface is an interaction-first combat workspace: a DOM grid, participants,
current actor and revision, authority-provided actions and targets, reactions,
automatic rolls, and accepted outcomes remain visible in one play loop.
Ruleset lifecycle, artifact/provenance, and replay inspection remain available
as secondary top-menu dialogs. The browser sends only typed intents and typed
reaction decisions; it has no random-value input or roll-plan interpreter.
When Asha RPG rejects a probe with an exact random request, the Rust host draws
only that count and die size from system entropy and retries from an unchanged
checkpoint. It records one terminal command with the exact consumed evidence.
The deterministic browser gate injects a host-side roll tape through process
configuration, never through gameplay UI or a TypeScript semantic path. The current field
manual has four TypeScript-authored actions, including one materialized derived
action. Their sequential workflow moves an actor, applies a modifier, spends a
bounded resource, resolves d6 and five-d4 requests, and suspends/resumes a
reaction against the same state revision. Rust owns all interpretation and
mutation.

The active slot also stores Asha RPG's versioned portable checkpoint and typed
replay entries. Rulebench displays their exact artifact/package/lock bindings,
fingerprint planes, definition fingerprints, pending phase, state revision,
random position, structured evidence, accepted events, and canonical state
hash. Restore and replay buttons delegate to the public Asha RPG APIs; the
product does not rematerialize TypeScript, resolve package ranges, reapply
events, or maintain a second gameplay state path.

## Retained product surfaces

- `apps/app`: Angular bootstrap.
- `apps/app-e2e`: focused interaction-first browser and managed live evidence.
- `libs/content-authoring`: the narrow immutable declaration shape accepted by
  the root loader; no content catalog, global registry, discovery,
  callbacks, scenarios, or raw-IR product catalog.
- `examples/rulesets`: independently owned example roots used by tests and demos.
  They have no privileged loader path.
- `examples/foundations`: explicitly imported packages shared by more than one
  example root; never a ruleset catalog or discovery location.
- `libs/protocol`: generated Rust host DTOs plus a strict decoder.
- `libs/transport`, `libs/domain`, `libs/store`: generated-DTO transport,
  pure inspection mapping, and explicit lifecycle state.
- `libs/components`: generic workbench panels, menus, and dialogs used by the
  combat workspace and its secondary inspection tools.
- `libs/platform`: browser ports, including the JSON HTTP boundary used by the
  same-origin compiler transport.
- `libs/scenario-viewer`: the interactive combat grid/action/reaction loop plus
  secondary compiler, provenance, diagnostics, checkpoint, and replay tools.
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

Non-claims: the fixed hero/raider bootstrap is not general participant setup,
turn sequencing and board metadata are not yet content-authored, and archive
persistence across process restarts or artifact activation, storage migration,
nested reaction windows, upgrade migration policy, and exhaustive
cross-product certification are not implemented here.

The fresh composition now demonstrates one primary base, two ordered typed
mixins, a local relational patch, an authorized semantic overlay, and a
presentation-only overlay. Rulebench displays the exact base/mixin chain,
parameters, patch fingerprints, before/after values, effectiveness, impact
planes, and final definition fingerprints emitted by Asha RPG; it does not
reimplement materialization semantics.

After an artifact is active, the independently rooted Field Manual 1.1 example can be compiled as a
candidate without activation. The Rust host structurally compares the two
fully materialized accepted artifacts and Rulebench displays changed package
sources, changed definitions, derived descendants, causes, and exact semantic
or presentation field transitions. The Ruleset menu also retains successfully
compiled root paths and can compile a recent root as a candidate without
silently activating it. Root selection is an explicit location request rather
than a source ID, so adding ordinary content does not extend a Rust enum,
switch, or product catalog.

See [docs/empty-ruleset-boundary.md](docs/empty-ruleset-boundary.md) for the
deletion and retention inventory.
See [docs/ruleset-workspaces.md](docs/ruleset-workspaces.md) for the downstream
manifest contract and loader constraints.
