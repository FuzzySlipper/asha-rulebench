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
```

Use the printed local or LAN URL for human interaction. Record the exact content
and dependency revisions, the user controls exercised, accepted Rust outcomes,
desktop and narrow screenshots actually inspected, and explicit non-claims in
the Den handoff. A scripted product journey is intentionally not the evidence
source for the interaction-first campaign. Persistent setup libraries,
persistence across activation/process restart, multiplayer, AI, migration
policy, and exhaustive certification remain explicit non-claims.
