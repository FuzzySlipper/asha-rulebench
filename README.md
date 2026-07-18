# ASHA Rulebench

ASHA Rulebench is the authoring and inspection product for compiled Asha RPG
rulesets. Den task #5953 now builds on the deliberate empty boundary from #5952:

- no prototype content is bundled;
- no ruleset is selected by default;
- no scenario, import, filename, or startup behavior can construct a ruleset;
- no Rulebench-owned combat, replay, or semantic authority remains;
- the UI starts and clearly reports **No compiled ruleset active**;
- one explicit fresh TypeScript composition can be compiled, inspected, and
  atomically activated without introducing gameplay authority.

Content enters only through the explicit manifest/compiler boundary. Each
compile click prepares the currently selected TypeScript package graph in the
browser and sends that prepared source through a generated request DTO. The
current fresh composition resolves an exact package lock and exported-root
closure; Rust derives its private normalized execution input from that one
closed definition graph, compiles it, round-trips the portable artifact through
the authoritative loader, and only then creates an activation candidate.
Source directories, server startup, and import side effects never determine
runtime meaning. A selectable invalid graph proves source diagnostics remain
user-reachable without replacing the active artifact.

## Retained product surfaces

- `apps/app`: Angular bootstrap.
- `apps/app-e2e`: focused compiler/activation browser and managed live evidence.
- `libs/content-authoring`: fresh immutable TypeScript package graph choices and
  on-demand preparation; no global registry, discovery, callbacks, scenarios,
  or raw-IR product catalog.
- `libs/protocol`: generated Rust host DTOs plus a strict decoder.
- `libs/transport`, `libs/domain`, `libs/store`: generated-DTO transport,
  pure inspection mapping, and explicit lifecycle state.
- `libs/components`: generic workbench panels and menus used for
  compilation/activation inspection; #5955 may add runtime controls.
- `libs/platform`: browser ports, including the JSON HTTP boundary used by the
  same-origin compiler transport.
- `libs/scenario-viewer`: compiler, exact lock, closure, diagnostics,
  fingerprint, and activation inspection. #5955 consumes the active artifact
  in a visible workflow.
- `libs/shell`: routes only.
- `libs/theme`: product tokens.

`rulebench-rs/hosts/ruleset-host` is a fresh narrow loopback host pinned to the
public Asha RPG revision. It owns only compile/load/inspect/activate lifecycle
state. It does not own gameplay sessions, state mutation, persistence, or
replay.

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

See [docs/empty-ruleset-boundary.md](docs/empty-ruleset-boundary.md) for the
deletion and retention inventory.
