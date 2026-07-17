# Validation Tiers And Ownership

Rulebench has two product-owned validation tiers. Exhaustive proof is a
downstream concern.

## Focused checks

`pnpm run verify:change` accepts an explicit, closed profile set and never
infers safety from a Git diff:

| Profile | Product-owned checks |
| --- | --- |
| `frontend` | TypeScript boundaries, lint, typecheck, unit tests |
| `browser` | Typecheck and primary `@gate` browser journeys |
| `rust-owner --crate <owner>` | Rust boundary checks and the named crate tests |
| `protocol-generated` | Protocol generation, compatibility, TypeScript consumers |
| `product-content` | Focused named-content crate tests |
| `host-transport` | Bridge, process-host, transport, and store tests |
| `docs` | Executable documentation command checks |

Profiles may be repeated and selected commands are deduplicated. Unknown
profiles, crates, or arguments fail closed. Use `--dry-run` to inspect the
selection. Run the blocking gate when classification is uncertain.

Focused output is owner feedback. It is not exhaustive certification.

## Blocking project gate

```bash
pnpm run verify
```

The GitHub required check runs the product's static contracts, all focused Rust
owners, TypeScript tests/build, and primary `@gate` browser journeys. These
checks defend source boundaries, current protocol compatibility, host behavior,
and the visible workflows Rulebench directly owns.

The blocking gate deliberately excludes:

- the full synthetic authority harness;
- unfiltered semantic and capability-conformance matrices;
- historical content/replay format matrices;
- generated scenario, session, and capability proof artifacts;
- exhaustive browser permutations;
- certification receipts and claims review.

Those exclusions are not skipped Rulebench checks. Their owner is the public
downstream repository `FuzzySlipper/asha-rulebench-testing`.

## Downstream certification

`asha-rulebench-testing` consumes exact public revisions of `asha-rpg` and
`asha-rulebench`. It owns the relocated exhaustive Rust harness, cross-version
fixtures, generated proof artifacts, portable consumer checks, exhaustive
browser journeys, and honest certification receipts.

The downstream suite runs on its own nightly/manual cadence and for named
milestone or release decisions. Rulebench does not import it, invoke it from
`pnpm run verify`, or wait for it as an ordinary per-change gate. A downstream
failure is routed to the first broken public contract and does not make the
testing repository semantic authority.

## Browser evidence

The product keeps primary visible journeys under `@gate`. User-deliverable UI
work also requires a managed live run and inspected artifacts:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Process exit alone is not visual proof. Record what was rendered and inspected,
plus explicit non-claims. Exhaustive browser certification remains downstream.

## Handoff contract

Use `docs/validation-evidence-template.md`. Record the exact commit, selected
tier, commands, observed results, browser execution and inspection state, and
non-claims. Never label focused or blocking results as downstream
certification.
