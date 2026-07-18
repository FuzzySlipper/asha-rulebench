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
generated-protocol drift, and the focused browser gate. It proves the one fresh
TypeScript composition reaches Rust compilation, a closed artifact loader, the
user-facing inspection view, and atomic activation. It does not prove gameplay
execution, persistence, replay, migration, derivation/overlay execution, or an
exhaustive content corpus.

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
claim covers the inactive startup state, explicit compilation, artifact
inspection, and activation revision. Gameplay remains an explicit non-claim.
