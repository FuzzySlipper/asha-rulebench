# AGENTS.md

## Den Guidance Bootstrap

- Project ID: `asha-rulebench`
- Resolve live guidance with the Den MCP `get_agent_guidance` tool or `den guidance` before substantial work.
- Treat the resolved Den guidance packet and its referenced Den documents as the source of truth.
- If Den is unreachable, stop and tell the user which Den tool or command failed and what you were about to do. Do not reconstruct Den state from local files.
- Use the project-local notes below only as bootstrap context for connecting to Den and working in this repository.

# ASHA Rulebench Local Bootstrap

Live project guidance lives in Den. Use project ID `asha-rulebench` for Den tasks, messages, documents, librarian queries, and guidance lookups.

When creating or updating Den tasks for this repository, tag them with `asha-rulebench` plus any lane/system tags.

## Source-of-Truth Posture

This local file is bootstrap context for agents entering the repository. It is not the current planning queue.

- **Den** owns current task state, implementation queues, durable planning docs, review packets, and known limitations.
- **Repo docs** describe architecture and committed implementation surfaces.
- **The code/tests** are the implementation truth when they conflict with old planning prose.
- Resolve live Den guidance with `get_agent_guidance(project_id="asha-rulebench")` before substantial work.
- The initial Angular template state is startup residue; do not infer product direction from old template names.

## Architecture Soul

> TypeScript references and configures Rust behavior. Rust defines and executes rule logic.

- **Rust** is authoritative: content validation, rule semantics, capability mutation, deterministic dice, effect interpretation, reaction windows, accepted DomainEvents, trace, replay, and state hashes.
- **TypeScript** authors catalogs, scenarios, fixtures, policy proposals, UI view models, and display mapping over generated protocol surfaces.
- TypeScript **never mutates** authoritative gameplay state and never becomes a parallel rules engine.
- Protocols are **generated** from authority surfaces; hand-editing generated files is forbidden.
- The old RuleWeaver project is evidence, not structure to preserve.

See `README.md` and Den doc `asha-rulebench/basic-design` for current repo orientation.

## Repository Structure

```text
/asha-rulebench
  /apps
    /app                 # Angular shell composition
    /app-e2e             # deterministic and live browser scenarios
  /libs
    /protocol            # generated protocol exports and shared result/error types
    /transport           # backend/native/WASM/fake communication through protocol types
    /domain              # pure protocol-to-view mapping, no Angular/browser APIs
    /store               # app state mutation, AsyncState<T>, transport orchestration
    /renderer            # feature rendering composition
    /components          # reusable presentational Angular components
    /platform            # browser/host ports and fakes
    /shell               # routes and app composition only
    /theme               # approved tokens and theme entrypoints
    /testing-fixtures    # typed fixtures for tests and scenario examples
  /tools                 # workspace scripts and generators
```

Future local Rust crates may be added only when a task explicitly calls for incubating authority behavior here. Keep local Rust separate from the Angular shell and promote generic pieces upstream to ASHA when proven.

The selected concrete live adapter lives at
`rulebench-rs/hosts/rulebench-process-host`. It binds loopback and is reached
through the Angular same-origin proxy; `rulebench-bridge` remains host-neutral.
The matching TypeScript client lives at `libs/transport/src/live.ts`; preserve
its generated-DTO-only API and fake/live implementation parity.

## Local Commands

```bash
# Canonical blocking project gate (the GitHub required check)
pnpm run verify

# Complete deterministic certification (nightly/manual workflow surface)
pnpm run certify

# Explicit focused checks; repeat --profile to take the safe union
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile rust-owner --crate rulebench-rpg-adapter
pnpm run verify:change -- --profile fixtures-conformance --scenario hexing-bolt-reaction

# LAN-first development server
den-serve up asha-rulebench -repo /home/dev/asha-rulebench

# Opt-in live browser evidence
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live

# Milestone/release certification with inspected live artifacts
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run certify -- --require-live
```

`verify:change` has a closed profile vocabulary and never infers safety from a
Git diff. Use `frontend` for production TypeScript, `browser` for routes and
visible workflows, `rust-owner --crate <workspace-owner>` for a changed Rust
owner, `protocol-generated` for protocol/emitter/generated-consumer changes,
`fixtures-conformance` for fixture/registry/capability work, `host-transport`
for bridge/process-host/transport/store work, `portable --crate
<portable-owner>` for portable/public Cargo surfaces, and `docs` for command or
support documentation. Combine profiles when a change crosses owners. Exact
fixture filters are `--package`, `--package-version`, `--ruleset`,
`--ruleset-version`, `--scenario`, and `--capability`; omitting a safe filter
runs the full regression/conformance corpus. Use `--dry-run` to inspect the
selected commands. Missing/unknown profiles, crates, filters, and arguments
fail closed. If classification is uncertain, run `pnpm run verify`.

`pnpm run certify` is the Rulebench-owned exhaustive deterministic suite. It
composes static authority/product contracts, the unfiltered semantic corpus,
the independent portable consumer, the complete deterministic browser set,
and a generated claims/limitations receipt exactly once. GitHub runs it
nightly and on manual dispatch through `.github/workflows/certification.yml`.
It is not a required check for every edit. Use `--require-live` for milestone
or release UI claims; that mode fails closed unless both `BASE_URL` and
`LIVE_RUN=1` are present and collects the managed live-artifact scenario after
deterministic certification. Inspect its screenshots and evidence packet;
process exit alone is not visual proof.

Use [docs/validation-evidence-template.md](docs/validation-evidence-template.md)
for Den task handoffs. Record which tier was selected, why narrower tiers were
safe, whether browser work actually executed or came from cache, and the
explicit non-claims. Do not present a focused or blocking receipt as exhaustive
certification.

## Frontend Boundary Rules

This frontend is built as layered infrastructure. Architecture is fixed unless the task explicitly says otherwise.

Use workspace generators for new components, libraries, features, stores, and tests.
Do not create new dependencies without planner approval.
Do not duplicate backend protocol types; use generated protocol exports only.
Do not import another library's internals; public API entrypoints only.
Do not bypass the transport layer for backend communication.
Do not bypass platform ports for browser/host APIs.
Do not bypass the store for application state mutation.
Do not put domain logic in components. Do not put feature logic in shell.
Expose async state as `AsyncState<T>`; map all failures to classified errors.
Do not close a user-deliverable UI task on deterministic evidence alone: run the live scenario, inspect the rendered artifacts yourself, and report what the UI did, including non-claims. A passing synthetic test is diagnostic, not proof.
Do not use `any`, non-null assertions, unsafe casts, or lint disables.
Do not add global CSS except through approved token/theme files.
Do not create circular dependencies or reverse dependency direction.
Prefer explicit, boring, typed code over clever abstractions.
When a task seems to require breaking a boundary, stop and request planner review.

## Agent Lane Quick Reference

| Lane       | Location                                | May not                                                        |
| ---------- | --------------------------------------- | -------------------------------------------------------------- |
| protocol   | `libs/protocol`                         | Duplicate generated backend types or hand-edit generated files |
| transport  | `libs/transport`                        | Own app state or bypass protocol DTOs                          |
| domain     | `libs/domain`                           | Import Angular, browser APIs, store, renderer, or components   |
| store      | `libs/store`                            | Put rendering logic in state services                          |
| renderer   | `libs/renderer`                         | Mutate application state directly or encode rule authority     |
| components | `libs/components`                       | Know gameplay/domain logic                                     |
| platform   | `libs/platform`                         | Reach into application state or transport                      |
| shell      | `libs/shell`, `apps/app`                | Contain feature/domain logic                                   |
| testing    | `libs/testing-fixtures`, `apps/app-e2e` | Become the only proof for a user-facing UI task                |

## TypeScript House Style

TypeScript in this repo is written for agent governance, not clever human terseness.

Prefer longer, clearer code over compact clever code. Use named intermediate values for meaningful decisions. Split work into small functions with explicit verbs. Avoid generic abstractions until duplication has stabilized. Keep mutation local and visible. Do not create ambient state, manager classes, global registries, or hidden runtime coupling.

A good TypeScript diff should be easy for a reviewer agent to inspect mechanically: imports reveal lane boundaries, functions reveal intent, tests reveal behavior, and public API changes are explicit.

When in doubt, write the boring version.

## Rust House Style

Rust in this repo should be boring authority code. Prefer explicit state, explicit errors, explicit events, and narrow crate APIs. Do not introduce clever abstractions, runtime escape hatches, mutable callback systems, or framework-shaped machinery unless a planner explicitly approves them.

## Non-Claims

ASHA Rulebench is not a full RPG, a straight RuleWeaver port, a D&D 4e compatibility target, an ASHA fork, a generic rules engine, a place for mutable TypeScript authority callbacks, a renderer-first game prototype, or a replacement for upstream ASHA runtime/replay/validation infrastructure.
