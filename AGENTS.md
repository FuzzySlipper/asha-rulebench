# AGENTS.md

## Den Guidance Bootstrap

- Project ID: `asha-rulebench`
- Resolve live guidance with Den MCP `get_agent_guidance` or `den guidance`
  before substantial work.
- Treat the guidance packet and referenced Den documents as authoritative.
- If Den is unreachable, stop and report the failed tool and intended action.

## Architecture Soul

> TypeScript references and configures Rust behavior. Rust defines and executes
> rule logic.

Rulesets are explicit, dependency-closed artifacts compiled from one TypeScript
manifest. Files organize authoring, manifests define packages, exported roots
define closure, and compiled artifacts define runtime truth.

Rulebench is currently intentionally empty under #5952. Do not restore named
prototype content, implicit rulesets, ambient registration, runtime discovery,
scenario-defined partial rulesets, hidden defaults, product-owned semantic
state, replay compatibility, or legacy authority adapters.

Fresh compiler and activation work belongs to #5953. Fresh runtime and visible
gameplay work belongs to #5955. Until then, the honest product state is
`No compiled ruleset active`.

## Current Repository Structure

```text
/apps/app              Angular bootstrap
/apps/app-e2e          focused empty-state and managed-live evidence
/libs/components       reusable presentation primitives
/libs/platform         browser/host ports
/libs/scenario-viewer  empty workspace; future compiled-artifact consumer
/libs/shell            routes and composition
/libs/theme            approved tokens
```

Every retained surface must have a concrete #5953 or #5955 consumer. Do not add
new dependencies or authority layers without the owning Den task.

## Commands

```bash
pnpm run verify
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile browser
pnpm run verify:change -- --profile docs

den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

`verify:change` accepts only the closed current vocabulary `frontend`,
`browser`, and `docs`. Run `pnpm run verify` when classification is
uncertain.

## Frontend Boundaries

- Use public library entrypoints; never deep-import internals.
- Shell owns routes/composition only.
- Components contain no domain or authority logic.
- Browser/host APIs go through platform ports.
- Generated files, when reintroduced by an owning task, are never hand-edited.
- Do not use `any`, non-null assertions, unsafe casts, or lint disables.
- User-facing work requires managed live evidence and inspected artifacts.

Prefer explicit, typed, mechanically reviewable TypeScript. TypeScript may
author immutable declarations over Rust-published meanings; it never executes
gameplay semantics or mutates authoritative state.

## Non-Claims

The empty boundary does not claim compilation, activation, gameplay execution,
persistence, migration, replay compatibility, or exhaustive certification.
