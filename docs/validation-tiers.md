# Validation Tiers And Ownership

## Focused checks

`pnpm run verify:change` has a closed vocabulary:

| Profile              | Checks                                                          |
| -------------------- | --------------------------------------------------------------- |
| `frontend`           | structure, TypeScript authority, lint, types, unit tests, build |
| `content-authoring`  | TypeScript authority, loader contract, types, unit tests        |
| `rust-owner`         | fresh play-host workspace tests                                 |
| `protocol-generated` | Rust protocol drift, types, decoder tests                       |
| `host-transport`     | Rust host, types, transport/store tests                         |
| `browser`            | types and the focused `@gate` selection/activation journey      |
| `docs`               | executable documentation command references                     |

Profiles may be repeated and their command union is deduplicated. Unknown
profiles and arguments fail closed. Use `--dry-run` to inspect a selection.

## Blocking project gate

```bash
pnpm run verify
```

The required check runs static product validation, the pinned Rust host tests,
generated-protocol drift, and a focused browser gate. It proves a selected
`src/index.ts` root exposes a distinct Ruleset, Content Packs, and compatible
PlayBundle; compilation reaches Rust's closed artifact loader; and activation
is explicit and atomic before Scenario setup becomes available. It does not
prove a playable content corpus, process-restart storage, or migration policy.

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
claim covers inactive startup, per-click source preparation, artifact
inspection as a secondary dialog, activation, explicit artifact-pinned setup,
the desktop and narrow combat workspace, alternating authority-owned turns,
participant/cell/area authority target choice, explicit turn control, real
system-supplied automatic roll feedback, inline reaction completion, completed
outcome presentation, and replay of the exact recorded values. Persistent
setup libraries, persistence across activation/process restart, multiplayer,
AI, migration policy, and exhaustive certification remain explicit non-claims.
