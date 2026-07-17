# Validation Tiers And Risk Inventory

Status: implemented three-tier contract through Den tasks #5868–#5870
Baseline commit: `74623131d9c9e558773c2844a991df357bb30ede`
Measured: 2026-07-16

This document records the current Rulebench verification graph, the defects
each stage is meant to catch, its measured cost, and the target three-tier
contract. Task #5869 implemented the focused checks and blocking project gate.
Task #5870 implemented certification routing, proof cleanup, and final
guidance. The baseline commit/date and counts in the next section are
historical measurement evidence, not current command guidance or synchronized
support claims.

The governing invariant is unchanged:

> TypeScript references and configures Rust behavior. Rust defines and
> executes rule logic.

Reducing default proof cost must not weaken Rust authority, generated protocol
ownership, deterministic replay, architecture direction, protocol
compatibility, or fake/live transport parity.

## Recorded Pre-Tiering Baseline

GitHub Actions has one `verify` job for every pull request and push to `main`.
After dependency and Chromium installation, it runs `pnpm run verify`. That
command serially executes fourteen stages:

1. `check:pattern`
2. `generated:check`
3. `regression:check`
4. `check:rust-boundaries`
5. `check:rust-test-ownership`
6. historical repository-local portable-consumer proof (retired after extraction)
7. `rust:test`
8. `check:claims`
9. `check:docs`
10. `lint`
11. `typecheck`
12. `test`
13. `build`
14. `e2e`

The baseline contains:

- 13 Rust workspace crates, including the concrete process host;
- 488 Rust tests, including 293 cross-crate authority harness tests;
- 11 scenario regressions and 16 capability conformance cases covering 14
  executable capabilities;
- 61 Vitest tests;
- 22 Playwright tests: 21 execute in the normal local stack and one
  artifact-collecting live scenario skips unless live mode is enabled;
- four Rust-generated TypeScript artifacts totaling 610,543 bytes.

At the recorded baseline, five Playwright titles contained `@live`. Four were in
`integration/live-rust.spec.ts` and still execute under ordinary `e2e`; the
artifact-collecting `boot.live.spec.ts` scenario is the one that skips without
live mode. `e2e:live` selects all five, so four browser journeys are executed
both by the ordinary gate and by the live command.

### Timing method

The cold measurement used empty isolated Cargo targets for the main workspace
and portable-consumer workspace. Nx lint, typecheck, and build ran with cache
reuse disabled. Dependencies and Chromium were already installed because
installation is outside `pnpm run verify`.

An already-managed `app:serve:e2e` process caused new Nx invocations of the
same continuous target to wait rather than start on a second port. The cold
browser number therefore used an isolated ephemeral Rust host plus the
equivalent Angular development build/proxy configuration and ran the same
Playwright config with `BASE_URL`. It passed 21 tests and skipped the one
live-only artifact scenario. This measures the browser work without stopping
or mutating the managed development server.

The warm measurement immediately repeated the exact current stage graph.
Nx lint, typecheck, build, and E2E were cache hits; the warm E2E number is a
cached task receipt, not a second browser execution.

| Current stage                | Cold seconds | Warm seconds | Warm evidence note                               |
| ---------------------------- | -----------: | -----------: | ------------------------------------------------ |
| `check:pattern`              |        0.385 |        0.412 | Executed                                         |
| `generated:check`            |       23.998 |        1.028 | Executed four emitters                           |
| `regression:check`           |        1.060 |        0.658 | Executed full corpus                             |
| `check:rust-boundaries`      |        0.371 |        0.366 | Executed                                         |
| `check:rust-test-ownership`  |        0.373 |        0.371 | Executed                                         |
| historical portable consumer |       17.101 |        0.508 | Retired after ownership moved to `asha-rpg`      |
| `rust:test`                  |       10.086 |        3.863 | Executed 488 tests                               |
| `check:claims`               |        0.362 |        0.373 | Executed                                         |
| `check:docs`                 |        0.355 |        0.350 | Executed                                         |
| `lint`                       |        5.384 |        0.678 | Warm result came from Nx cache                   |
| `typecheck`                  |        3.521 |        0.625 | Warm result came from Nx cache                   |
| `test`                       |        1.355 |        1.106 | Executed 61 tests                                |
| `build`                      |        3.825 |        0.603 | Warm result came from Nx cache                   |
| `e2e`                        |       21.104 |        0.623 | Warm result came from Nx cache                   |
| **Measured stage sum**       |   **89.280** |   **11.564** | Shell orchestration reported 11.615 seconds warm |

Cold cost is dominated by four Rust emitters, the independent portable build,
workspace Rust tests, and the real-host browser stack. The full regression
corpus is inexpensive after generated checks have compiled the main Rust
target, but it repeats authority cases and replay proof already exercised at
other layers.

### Focused and blocking implementation measurement

Task #5869 measured the implemented commands with installed dependencies. The
focused cold probe set `NX_SKIP_NX_CACHE=true`; its warm probe immediately
repeated the same profile with normal Nx caching. The blocking cold probe used
an empty isolated Cargo target and disabled Nx caching. Both blocking probes
ran the four actual `e2e:gate` workflows against an already-running local
`den-serve` URL so the user-visible development instance did not have to be
stopped merely to acquire a second Nx continuous target.

| Implemented path                          | Cold seconds | Warm seconds | Evidence boundary                                                                            |
| ----------------------------------------- | -----------: | -----------: | -------------------------------------------------------------------------------------------- |
| `verify:change --profile frontend`        |       12.110 |        5.310 | Actual pattern/authority checks and Vitest; cold lint/typecheck bypassed Nx cache            |
| `verify` blocking composition             |       65.660 |       20.630 | Empty Cargo target for cold; both runs executed three regressions and four browser workflows |
| Recorded pre-tiering `verify` for context |       89.280 |       11.615 | Baseline above; its warm E2E value was an Nx cache receipt rather than a browser execution   |

The focused cold path is materially cheaper than the old all-purpose gate, and
the blocking cold composition removes the portable-consumer rebuild, broad
regression corpus, and broad browser permutations. Its measured warm wall time
is intentionally higher than the old cached receipt because `e2e:gate` really
executed the primary browser workflows. The after measurements exclude local
web-server startup, while the recorded cold baseline included an isolated
real-host browser stack; the exact-SHA GitHub gate remains the clean-checkout
proof for normal self-starting CI composition.

Task #5870 measured the canonical deterministic certification command against
the managed local URL. The cold probe used an empty isolated Cargo target and
disabled Nx cache reuse. The warm probe immediately repeated the same command;
unlike the historical warm baseline, all 22 deterministic browser journeys
executed in both probes.

| Final path                                | Cold seconds | Warm seconds | Evidence boundary                                                                  |
| ----------------------------------------- | -----------: | -----------: | ---------------------------------------------------------------------------------- |
| `verify:change --profile frontend`        |       12.110 |        5.310 | Focused frontend feedback; actual lint/typecheck/Vitest work                       |
| `verify` blocking composition             |       65.660 |       20.630 | Three regressions and four primary browser workflows                               |
| `certify` deterministic certification     |       83.200 |       38.700 | Full static suite, unfiltered corpus, portable consumer, and 22 browser journeys   |
| Recorded pre-tiering `verify` for context |       89.280 |       11.615 | Warm result included cached lint/typecheck/build/E2E receipts, not a browser rerun |

The certification command is intentionally more expensive than the blocking
gate and materially cheaper cold than the recorded pre-tiering composition.
Its warmer number remains honest about actual browser execution instead of
presenting an Nx E2E cache receipt as a second run.

## Verification Risk Inventory

The target tier column is the decision for #5869 and #5870. “Focused” means the
stage is selected when its owning surface changes; it does not imply that the
stage is safe to omit for that owner.

| Stage                        | Concrete defect or drift detected                                                                                                                                                                                                             | Scope and current overlap                                                                                                                                                                                                                    | Target tier, owner, and cadence                                                                                                                                                                                                        |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `check:pattern`              | Missing template/governance files, malformed library tags, deep imports, production imports of testing fixtures, stale generated ESLint boundary policy, and an ungated live harness                                                          | Local frontend structure. Partially overlaps ESLint module boundaries, but also checks repository shape and live-test conventions that lint does not.                                                                                        | Focused for any frontend/tooling change and blocking project gate. Rulebench-owned; run on every applicable edit and every integration.                                                                                                |
| `generated:check`            | A missing, hand-edited, unmarked, or stale protocol, capability, scenario, or combat-session projection from four Rust emitters                                                                                                               | Crosses Rust authority into TypeScript. `protocol:check`, `catalog:check`, and `session:check` are narrower duplicate entrypoints. Typecheck and fixture tests consume the artifacts but cannot prove emitter equality.                      | Focused whenever an emitter, protocol, registry, fixture, host composition, or generated consumer changes; blocking project gate. Rulebench-owned and fail-closed.                                                                     |
| `regression:check`           | Scenario outcome/event/trace/fingerprint drift, nondeterminism, rejection drift, executable capability coverage gaps, replay failure, and unclassified replay mismatches                                                                      | Local semantic certification. It runs 11 registered scenarios twice and 16 conformance cases, several of which reuse the same Hexing Bolt, Watchtower, movement, policy, and replay behavior covered by Rust tests and browser journeys.     | A three-case representative subset blocks the project gate; the unfiltered corpus belongs to certification and focused fixture/capability work. Rulebench-owned.                                                                       |
| `check:rust-boundaries`      | Reverse, unknown, or forbidden crate edges; fixture imports from portable owners; unapproved ASHA crates, paths, repositories, versions, or revisions                                                                                         | Local plus pinned upstream compatibility. Cargo compilation will accept many policy-invalid graphs, so this is not replaced by `rust:test`.                                                                                                  | Focused for Cargo/Rust ownership changes and blocking project gate. Rulebench-owned and fail-closed.                                                                                                                                   |
| `check:rust-test-ownership`  | An active owner crate or authority harness losing all colocated `#[test]` coverage                                                                                                                                                            | Local governance. It does not prove semantics and partially overlaps the existence of tests run by Cargo, but Cargo succeeds for a crate with zero tests.                                                                                    | Focused for Rust owner/test moves and blocking project gate because it costs less than 0.4 seconds. Rulebench-owned.                                                                                                                   |
| `asha-rpg` consumer boundary | Public Git consumption or the ECRP authority loop failing without any Rulebench crate                                                                                                                                                         | Owner-local external-workspace proof. It lives in `asha-rpg/consumers/minimal-game` and is not duplicated by the Rulebench product gate.                                                                                                     | `asha-rpg`-owned and run by that repository's CI. Rulebench validates only its exact public revision and allowed dependency direction.                                                                                                 |
| `rust:test`                  | Owner semantics, validation and resolution, replay/recovery, protocol mapping, bridge behavior, host lifecycle/storage/TCP, and cross-crate authority integration regressions                                                                 | Local authority truth. The 488 tests overlap scenario fixtures and host/browser paths, but provide narrower diagnostics and many rejection/rollback cases absent elsewhere.                                                                  | Focused owner package tests during edits; the full workspace remains in the blocking project gate as core semantic coverage. Rulebench-owned.                                                                                          |
| `check:claims`               | Capability manifest identity/order/progression, evidence ownership, executable-without-conformance claims, host composition, production Rust stubs, exact required prose, exact non-claims, exact limitations, and one hard-coded review date | Mixes executable invariants with manually synchronized policy prose. Manifest checks overlap generation/regression but uniquely prevent capability promotion. Exact text/date checks create documentation ceremony without proving behavior. | Split: executable manifest/stub invariants remain focused and blocking; prose, review date, exact counts, and Den-document review move to milestone certification/governance without hard-coded source-gate literals. Rulebench-owned. |
| `check:docs`                 | Markdown referencing a package script that does not exist                                                                                                                                                                                     | Local documentation freshness with no semantic overlap.                                                                                                                                                                                      | Focused for command/doc edits and blocking project gate because it is cheap. Rulebench-owned.                                                                                                                                          |
| `lint`                       | TypeScript/Angular/Playwright style, unsafe constructs, and Nx module-boundary violations                                                                                                                                                     | Local frontend quality. Boundary rules partially overlap `check:pattern`; the two use different enforcement mechanisms and catch different defects.                                                                                          | Focused for frontend changes and blocking project gate. Rulebench-owned.                                                                                                                                                               |
| `typecheck`                  | TypeScript contract and public API incompatibility, including generated DTO consumer drift                                                                                                                                                    | Local frontend/protocol consumer compatibility. It complements generation equality and unit tests.                                                                                                                                           | Focused for frontend/generated changes and blocking project gate. Rulebench-owned.                                                                                                                                                     |
| `test`                       | Transport parity/error handling, AsyncState orchestration, stale-response behavior, protocol-to-view projection, and fixture usage regressions                                                                                                | Local frontend behavior. Some fixture assertions repeat generated/Rust outcomes, but the unique value is proving TypeScript carries rather than re-derives authority facts.                                                                  | Focused for frontend/generated changes and blocking project gate. Rulebench-owned.                                                                                                                                                     |
| `build`                      | Angular compilation, bundling, entrypoint/style/asset failures, and size-budget violations                                                                                                                                                    | Local product integration. Typecheck overlaps compilation but not the production builder or budgets.                                                                                                                                         | Blocking project gate; focused when app/build/theme composition changes. Rulebench-owned.                                                                                                                                              |
| `e2e:certification`          | Same-origin Rust host integration, handshake/error mapping, content lifecycle, live sessions, visible rule evidence, replay review, policy laboratory, responsive/mobile behavior, and accessibility interactions                             | Product integration plus broad deterministic certification. It repeats Rust scenario/replay semantics only where the transport or rendered user workflow is the distinct defect boundary. The live artifact scenario is excluded.            | Complete deterministic browser group inside `certify`; Rulebench-owned and run nightly, on manual dispatch, and for milestones/releases.                                                                                               |
| `e2e:live-artifacts`         | LAN-served rendered behavior and artifact collection, screenshots, console/page errors, visible text, and explicit non-claims                                                                                                                 | One opt-in managed-server scenario. Deterministic journeys are no longer selected merely because they talk to the live Rust host. A passing process is not sufficient; artifacts must be inspected.                                          | Required for milestone/release user-visible claims and user-deliverable UI tasks. `e2e:live` remains a compatibility alias. Rulebench-owned.                                                                                           |

### Other current package entrypoints

- `rust:check` is a useful compile-only focused command. It is not in the
  current full gate because `rust:test` compiles the same workspace before
  executing tests.
- `protocol:check`, `catalog:check`, and `session:check` are focused aliases for
  subsets of `generated:check`. They remain useful for owner feedback but must
  not be composed beside `generated:check` in the same gate.
- `regression:list` is an inventory and filter-discovery command, not pass/fail
  certification.
- `generated:write`, `protocol:generate`, `catalog:generate`, and
  `session:generate`/`session:write` mutate generated outputs and are repair
  paths, not verification stages.
- `dev`, `serve:local`, and `rust:host` start development surfaces and do not
  themselves establish a verification claim.
- `certify` is the exhaustive deterministic composition. It deliberately calls
  shared primitives rather than `verify`, so regression and browser subsets
  are not executed twice. `--require-live` adds only the managed artifact
  group after deterministic certification.

## Target Tier Contract

### Tier 1: focused change checks

Canonical command:

```bash
pnpm run verify:change -- --profile <profile>
```

The selector must use an explicit, closed profile vocabulary, accept more than
one profile by union, print the commands it selected, and reject a missing or
unknown profile. It must not infer safety from an incomplete Git diff. A
caller that cannot classify a change must run the blocking project gate.

| Profile                | Required contents                                                                                                                                                                                                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `frontend`             | `check:pattern`, `check:typescript-authority`, `check:rules-language-boundary`, lint, typecheck, and all Vitest tests. Add `browser` when a user workflow, route, transport call, app composition, or E2E support surface changes.                                                                     |
| `browser`              | Typecheck plus the four `e2e:gate` workflows. Live/artifact evidence is additionally required when the task is user-deliverable UI work.                                                                                                                                                               |
| `rust-owner`           | `check:rust-boundaries`, `check:rust-test-ownership`, and `cargo test -p <changed-owner>`. `--crate` is required and must be one of the governed workspace owners.                                                                                                                                     |
| `protocol-generated`   | `check:rust-boundaries`, `generated:check`, `check:protocol-compatibility`, executable claims checks, typecheck, and Vitest.                                                                                                                                                                           |
| `fixtures-conformance` | `cargo test -p rulebench-fixtures` plus filtered scenario/capability conformance selected by exact package, ruleset, scenario, or capability identity. With no safe exact filter, run unfiltered `regression:check`. Add `generated:check` when an emitted catalog/session/capability fact can change. |
| `host-transport`       | Process-host and bridge owner tests plus transport/store Vitest. Add `browser` when a route, DTO, lifecycle, recovery, or user-visible error changes.                                                                                                                                                  |
| `docs`                 | `check:docs` plus the cheap executable claims check. Governance/prose freshness remains a certification concern.                                                                                                                                                                                       |

Repeat `--profile` to select the union of multiple owners. `rust-owner`
requires one or more exact `--crate` values from the closed owner set.
`fixtures-conformance` accepts exact `--package`, `--package-version`,
`--ruleset`, `--ruleset-version`, `--scenario`, and `--capability` filters; it
runs the unfiltered corpus when no safe exact filter is supplied. Filters are
rejected outside that profile. `--dry-run` prints the deduplicated command plan
without executing it. Missing or unknown profiles, arguments, filters, and
crate owners exit nonzero. Callers combine `fixtures-conformance` with
`protocol-generated` when emitted catalog/session/capability facts can change,
and combine frontend/host work with `browser` when the visible workflow can
change.

The focused runner may deduplicate identical commands selected by multiple
profiles. It must fail closed for these surfaces:

- any Rust emitter or generated DTO change selects `protocol-generated`;
- any Cargo dependency or governed ASHA revision change selects `rust-owner`
  and either `portable` or the blocking project gate as applicable;
- any fixture/registry/capability change selects `fixtures-conformance`;
- any production TypeScript change that could introduce rule decisions selects
  `frontend` and the TypeScript-authority invariant check described below.

### Tier 2: blocking project gate

Canonical command and GitHub check:

```bash
pnpm run verify
```

The existing required check name stays `verify`. Its target composition is:

```text
verify:static
  test:validation-scripts
  check:pattern
  check:typescript-authority
  generated:check
  check:protocol-compatibility
  check:rust-boundaries
  check:rust-test-ownership
  rust:test
  check:claims:executable
  check:docs
  lint
  typecheck
  test
  build

regression:gate
  hexing-bolt-reaction
  watchtower-storm-pulse-multiple
  binding-glyph-failed-save

e2e:gate
  boots the rulebench shell
  imports, activates, preserves, and selects Rust-owned authored content
  completes a supported scenario through the visible panel workbench
  reviews and compares archived Rust replay evidence
```

The three regression cases retain a reaction/replay path, operation-pipeline-v2
multi-target and classified-rejection path, and a second-provider ruleset
saving-throw/effect path. They are representative blocking evidence, not a
claim of exhaustive capability conformance.

Each entry is an exact scenario-catalog identity that must remain independently
runnable through the current fail-closed selector:

```bash
cargo run --quiet --manifest-path rulebench-rs/Cargo.toml -p rulebench-fixtures --bin check_regressions -- --scenario <scenario-id>
```

`regression:gate` invokes that command once for each identity above. A named
identity succeeds only when both the scenario regression and its capability
conformance case are selected and accepted. An unknown, renamed, catalog-only,
or conformance-only identity exits nonzero; the gate may not treat an empty
selection in either runner as success. The fixture test
`documented_project_gate_cases_are_independently_executable` locks this
composition contract so #5869 can wire the command without rediscovering which
selector owns each case.

The four browser workflows cover the primary rules-designer loop:

1. open the bench and inspect accepted/rejected authority evidence;
2. import, review, activate, reject, and select authored content;
3. configure and complete a live authority scenario through the visible
   panels, including archive creation;
4. verify and compare archived replay evidence.

Those four tests carry the explicit `@gate` tag and `e2e:gate` selects that tag
through the existing Playwright configuration. The public `e2e` command is not
deleted or narrowed; it continues to reach the complete deterministic set. The
canonical `certify` command selects that set through the explicit
`e2e:certification` group.

Responsive permutations, broad capability-matrix rendering, specialized
targeting/policy/recovery journeys, second-provider permutations, and complete
accessibility-media coverage move to certification. No browser test moves
merely because it is slow; it moves when its unique defect class is exhaustive
rather than integration-blocking.

`verify:static` is an internal composition primitive, not a weaker public
approval path. GitHub runs `verify`, not `verify:static`.

`test:validation-scripts` injects one-sided protocol drift, TypeScript rule
calculation/state mutation, and invalid focused selections into pure checker
fixtures. `check:pattern`, `generated:check`, and the Rust boundary/regression
owners retain their own synthetic rejection probes. These meta-checks are
distinct from product semantics: they prove the lighter selectors themselves
still recognize the defect classes they claim.

### Tier 3: certification suite

Canonical command:

```bash
pnpm run certify
```

Certification composes shared primitives once. It must not run `verify` and
then rerun subsets already executed by `verify`:

```text
certify
  verify:static
  regression:check             # unfiltered registered semantic corpus
  e2e:certification            # complete deterministic browser set, once
  review:claims-and-limitations # governance freshness without hard-coded prose/date/counts
  e2e:live-artifacts           # only in explicit --require-live mode
```

Implemented cadence and triggers:

- nightly scheduled and manual GitHub workflow in
  `.github/workflows/certification.yml`;
- every milestone and release candidate;
- every governed ASHA revision/version update;
- every protocol/storage/replay compatibility or migration change;
- before removing, narrowing, or relocating an existing proof surface.

For a milestone/release with user-visible UI claims, use the live-required mode
of the same canonical command:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url> LIVE_RUN=1 pnpm run certify -- --require-live
```

`--require-live` must fail when `BASE_URL` or `LIVE_RUN=1` is missing and must
invoke the artifact-collecting `e2e:live-artifacts` group after deterministic
certification. That live result is accepted only with inspected screenshots/artifacts,
rendered behavior, console/page-error evidence, and explicit non-claims. A
scheduled deterministic certification run without live artifacts must say that
it did not establish live/LAN visual evidence.

Certification remains Rulebench-owned. A read-only scan of
the local `asha-testing` checkout and other local repository GitHub/package/
Cargo entrypoints found no Rulebench caller. Den also has no registered
`asha-testing` project scope. Therefore there is no current destination owner,
invocation path, or retained failure channel that would make a migration safe.

Moving a certification surface later requires all of the following before the
Rulebench path is removed:

1. a destination Den task in the owning project;
2. an exact source revision or published artifact consumed by the destination;
3. a runnable destination command and cadence;
4. a failure receipt visible to the Rulebench milestone/release workflow;
5. one overlapping successful run before source-side removal.

## Duplicate Execution Decisions

| Current duplication                                                            | Decision and retained detection path                                                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `protocol:check`, `catalog:check`, or `session:check` beside `generated:check` | Never compose both in one tier. Focused aliases remain owner conveniences; project and certification gates use only `generated:check`.                                                                                                                                                                                     |
| Rust, TypeScript, and generated protocol identity/version                      | Keep `check:protocol-compatibility` beside generation equality. Generation proves emitted DTO equality; the compatibility check uniquely catches a one-sided hand-maintained live-client identity/version change before a host handshake.                                                                                  |
| Every scenario and conformance case plus all Rust tests                        | The project gate runs three representative regression cases plus all Rust owner tests. Certification runs the unfiltered regression/conformance corpus once. Rust tests remain because their narrow validation, rollback, storage, protocol, and error diagnostics are not supplied by catalog cases.                      |
| The same semantic outcome in Rust, regression, and browser tests               | Retain the Rust test for authority semantics, a regression case for deterministic package/replay compatibility, and a browser case only when it proves transport/rendered workflow behavior. Remove browser assertions that merely restate every event/fingerprint permutation without a visible integration claim.        |
| Deterministic live-host journeys formerly selected by both E2E and `e2e:live`  | Removed the ambiguous `@live` tag from deterministic journeys. `e2e:certification` runs every non-artifact browser case once; `e2e:live-artifacts` selects only the managed screenshot/evidence scenario, and `e2e:live` is a compatibility alias.                                                                         |
| `check:claims` exact prose/date/count checks plus executable manifest checks   | Executable capability progression, owner evidence, host composition, and production-stub checks remain blocking. Prose/date/count literals were removed; certification generates a receipt from the live generated inventory plus a provenance-labelled Den limitation snapshot whose age is policy, not a source literal. |
| Nx cached E2E result presented like browser execution                          | Keep cache use for local speed, but command output/evidence must distinguish a cache receipt from an executed browser run. Milestone/live claims require an actual run.                                                                                                                                                    |

## Checked Artifact Decisions

| Artifact                                                   |  Current size | Unique local value                                                                          | Decision                                                                                                                                                                                                                                                                                                                           |
| ---------------------------------------------------------- | ------------: | ------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `libs/protocol/src/generated/api-types.ts`                 |  74,030 bytes | The generated DTO contract compiled by every TypeScript consumer                            | Keep checked and blocking through `generated:check`.                                                                                                                                                                                                                                                                               |
| `libs/transport/src/generated/rust-capability-manifest.ts` |  14,527 bytes | Offline capability fixture, evidence inventory, and executable claims input                 | Keep checked and blocking while offline consumers use it. Generate counts/descriptions from it rather than copying them into prose.                                                                                                                                                                                                |
| `libs/transport/src/generated/rust-scenario-catalog.ts`    |  67,096 bytes | Offline fixture/golden data for transport and viewer tests; never a live viewer fallback    | Keep checked for now. Narrow only after the affected tests consume an equivalent generated-on-demand fixture with equal diagnostics.                                                                                                                                                                                               |
| `libs/transport/src/generated/rust-combat-session.ts`      | 101,321 bytes | Representative offline session/control/automatic-run/replay fixture used by transport tests | Narrowed from 454,890 bytes. The Rust emitter now selects one complete representative per offline category; generated equality and transport tests retain local drift detection, while all providers/scenarios and broad semantics remain in Rust owner tests, unfiltered regression/conformance, and live-host browser workflows. |

No generated authority contract was deleted. The combat-session artifact
dropped by 353,569 bytes (77.7%). Its generation path is unchanged, so
`generated:check` still identifies the owning emitter and stale file. The
TypeScript offline transport continues to receive a complete session,
control-history, script, automatic-run, and automatic-run-replay representative.
The Rust registry and certification corpus, rather than this offline projection,
retain exhaustive provider and semantic coverage.

## Claims And Non-Claims

The blocking claims check must be executable. It should validate:

- manifest/schema/protocol identity and deterministic ordering;
- exact governed ASHA revision shape and approved dependency policy;
- capability support progression and owner evidence;
- executable operations with conformance ownership;
- actual selected host composition and recovery/viewer modes;
- absence of production `todo!`/`unimplemented!` authority stubs.

It must not hard-code:

- a review date in the checker source;
- exact prose copied from README/AGENTS;
- exact capability/provider/scenario counts in manually synchronized prose;
- an exact list of active limitations whose authority is Den.

Certification generates a review receipt containing current executable
counts, the source commit, reviewed Den document handles, active limitations,
reviewer identity, and review time. That receipt can become stale by policy
without making an unrelated source edit fail because a literal date changed.

## Required Failure Injections

#5869 must demonstrate the lighter focused/blocking path against temporary or
self-test mutations and leave the worktree clean afterward:

| Invariant                         | Representative injection                                                                                                       | Required catching path                                                                     |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| Rust authority                    | Change a representative accepted event/fingerprint or expected rejection in a gate regression fixture                          | Owner Rust test and/or `regression:gate` fails                                             |
| Generated ownership               | Alter or remove a generated header/body without running the emitter                                                            | `generated:check` fails and names the emitter/artifact                                     |
| Protocol compatibility            | Change the protocol version/DTO contract on only one side                                                                      | Generation/type/transport handshake evidence fails before integration                      |
| Architecture direction            | Add a forbidden Rust edge and a frontend deep/testing-fixture import in checker self-test fixtures                             | Rust boundary and frontend pattern/module-boundary checks fail                             |
| Unauthorized TypeScript semantics | Add a synthetic production mapper/store example that rolls dice, derives acceptance/damage, or mutates returned gameplay state | A focused TypeScript-authority invariant check plus behavioral projection/store tests fail |

The TypeScript-authority injection closes a current gap: existing tests strongly
exercise carry-and-project behavior, but `check:pattern` does not directly
classify newly authored rule calculations. #5869 may extend the existing
checker or add a narrowly named checker using existing TypeScript tooling; it
must include focused reject/accept self-tests and may not rely on a prose-only
review rule.

## Scope Accounting

The #5866 acceptance contract is fully routed with no acceptance item deferred:

- #5868 inventoried the old graph, defect classes, overlap, ownership, recorded
  costs, and the focused/blocking/certification target contract.
- #5869 implemented fail-closed `verify:change` profiles, representative
  regression and four-workflow browser groups, revised blocking `verify`, and
  selector failure-injection tests.
- #5870 implemented one exhaustive `certify` command, a nightly/manual
  Rulebench-owned workflow, and explicit live-required milestone mode.
- Blocking and certification use shared primitives, never compose `verify`
  inside `certify`, and execute representative versus exhaustive regression
  and browser groups at their documented tiers.
- Deterministic browser journeys no longer also carry the live-artifact tag.
  Certification executes 22 deterministic journeys once; live mode adds only
  the managed screenshot/evidence scenario.
- Executable capability claims remain blocking. Certification derives current
  inventory and writes a source/Den provenance receipt without hard-coded prose,
  counts, or freshness dates in the source gate.
- The generated combat-session projection retains one complete representative
  for each offline consumer category and is 353,569 bytes (77.7%) smaller;
  exhaustive provider and semantic coverage remains with Rust owners, the
  unfiltered corpus, and live-host workflows.
- Twenty-one validation-runner self-tests plus existing Rust boundary,
  generated-equality, protocol-compatibility, and regression rejection probes
  fail closed around the lighter routing.
- The focused, blocking, and certification measurements above distinguish
  actual execution from cached receipts.
- `AGENTS.md`, README, the evidence template, and Den guidance record the exact
  commands, ownership, cadence, receipt, artifact, and non-claim boundaries.

No proof was moved to `asha-testing`: no destination project, runnable consumer,
or retained Rulebench failure signal exists. The migration prerequisites above
remain the gate for any future relocation, not deferred #5866 acceptance work.
