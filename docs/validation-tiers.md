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
TypeScript composition is freshly prepared by the trusted authoring host for a
user-selected request, reaches Rust compilation and a closed artifact loader,
appears in the user-facing inspection view, and activates atomically. It also
proves that a subsequent invalid source graph displays its TypeScript
diagnostics while preserving the active artifact and activation revision. It
does not prove process-restart storage, migration policy, or an exhaustive
content corpus. The focused gate does cover the fresh gameplay, portable
checkpoint restore, deterministic Rust replay, and
derivation/mixin/overlay graph, including visible materialization provenance
and a pre-activation 1.0-to-1.1 upgrade-impact report.

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
claim covers the inactive startup state, per-click source preparation, explicit
compilation, artifact inspection, activation revision, persistent gameplay
commands and reaction completion, a pre-activation upgrade-impact report, and
failed-recompilation preservation. It also covers the in-memory checkpoint and
replay archive, exact binding inspection, checkpoint restore, and deterministic
Rust replay. Persistence across activation/process restart, migration policy,
and exhaustive certification remain explicit non-claims.
