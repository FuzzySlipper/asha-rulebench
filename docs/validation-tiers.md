# Validation Tiers And Ownership

## Focused checks

`pnpm run verify:change` has a closed vocabulary:

| Profile              | Checks                                                              |
| -------------------- | ------------------------------------------------------------------- |
| `frontend`           | structure, TypeScript authority, lint, types, unit tests, build     |
| `content-authoring`  | TypeScript authority, prepared-artifact emission, types, unit tests |
| `rust-owner`         | fresh ruleset-host workspace tests                                  |
| `protocol-generated` | Rust protocol drift, types, decoder tests                           |
| `host-transport`     | prepared artifact, Rust host, types, transport/store tests          |
| `browser`            | types and the focused `@gate` compile/inspect/activate journey      |
| `docs`               | executable documentation command references                         |

Profiles may be repeated and their command union is deduplicated. Unknown
profiles and arguments fail closed. Use `--dry-run` to inspect a selection.

## Blocking project gate

```bash
pnpm run verify
```

The required check runs static product validation, the pinned Rust host tests,
generated-protocol drift, and the focused browser gate. It proves an explicit
ruleset root is freshly built through inferred `ruleset.ts#ruleset` by the
trusted authoring subprocess for a user-selected request, reaches Rust compilation and a closed artifact
loader, appears in the user-facing inspection dialog, and activates atomically.
It also proves subsequent invalid package and TypeScript build graphs display
diagnostics while preserving the active artifact, authority session, and
activation revision. It
does not prove process-restart storage, migration policy, or an exhaustive
content corpus. The focused gate drives the DOM battlefield, action palette,
highlighted authority targets, inline reaction choice, host-supplied exact
automatic rolls, visible accepted events, keyboard grid movement, narrow
layout, portable checkpoint restore, deterministic Rust replay, and
derivation/mixin/overlay graph, including visible materialization provenance,
independent-root activation, and recent-root switching from the top menu.

## Downstream posture

`asha-rulebench-testing` remains the downstream home for exact-SHA,
cross-repository milestone evidence after this task publishes its product SHA.
Rulebench owns focused product behavior; downstream testing must consume the
public revisions and must not become another compiler or authority path.

## Managed live evidence

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Inspect all milestone screenshots and the evidence packet. The current live
claim covers the inactive startup state, per-click source preparation,
artifact inspection as a secondary dialog, activation, the desktop and narrow
combat workspace, authority target choice, real system-supplied automatic roll
feedback, inline reaction completion, and replay of the exact recorded values.
The fixed bootstrap encounter is not general participant, turn, or board setup.
Persistence across activation/process restart, migration policy, and exhaustive
certification remain explicit non-claims.
