# Validation Tiers And Ownership

## Focused checks

`pnpm run verify:change` has a closed vocabulary:

| Profile | Checks |
| --- | --- |
| `frontend` | structure, TypeScript authority guard, lint, types, tests, build |
| `browser` | types and the focused `@gate` empty-state journey |
| `docs` | executable documentation command references |

Profiles may be repeated and their command union is deduplicated. Unknown
profiles and arguments fail closed. Use `--dry-run` to inspect a selection.

## Blocking project gate

```bash
pnpm run verify
```

The required check runs static product validation and the focused browser gate.
In the #5952 deletion phase it intentionally has no Rust, protocol-generation,
content-corpus, persistence, replay, or compatibility cells because those
owners and claims were retired with the prototype.

## Downstream posture

`asha-rulebench-testing` remains the downstream home for future exact-SHA,
cross-repository, milestone evidence. It no longer preserves or certifies the
deleted prototype corpus. A new suite must wait for fresh compiled-artifact
consumers from #5953/#5955; an absent suite is not reported as passed.

## Managed live evidence

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Inspect the screenshots and evidence packet. The current live claim is limited
to the rendered no-active-ruleset state.
