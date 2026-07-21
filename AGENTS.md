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

Rulesets declare semantic models and provisions. Explicitly selected Content
Packs contribute authored definitions. One compatible declared PlayBundle is
the dependency-closed artifact Rust compiles and activates; a Scenario then
creates a Rust-owned Session.

Rulebench starts intentionally inactive. Do not restore named prototype
content, implicit Rulesets, ambient registration, runtime discovery,
Scenario-defined partial rules, hidden defaults, product-owned semantic state,
or legacy authority adapters. Startup remains `No PlayBundle active`; a user
must explicitly select Content Packs, compile the compatible PlayBundle, and
activate its closed artifact.

## Current Repository Structure

```text
/apps/app              Angular bootstrap
/apps/app-e2e          focused compiler lifecycle and managed-live evidence
/libs/components       reusable presentation primitives
/libs/platform         browser/host ports
/libs/content-authoring immutable authored-export declaration boundary
/test-fixtures/rulesets narrow non-product loader fixtures
/libs/protocol         Rust-generated lifecycle DTOs and strict decoder
/libs/transport        generated-DTO-only compiler host client
/libs/domain           pure artifact-to-inspection mapping
/libs/store            explicit async compile/activate orchestration
/libs/scenario-viewer  compiler and artifact inspection feature
/libs/shell            routes and composition
/libs/theme            approved tokens
/rulebench-rs/hosts/play-host narrow loopback PlayBundle/Session host
```

Every retained surface must have a concrete #5953 or #5955 consumer. Do not add
new dependencies or authority layers without the owning Den task.

## Commands

```bash
pnpm run verify
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile content-authoring
pnpm run verify:change -- --profile rust-owner
pnpm run verify:change -- --profile protocol-generated
pnpm run verify:change -- --profile host-transport
pnpm run verify:change -- --profile browser
pnpm run verify:change -- --profile docs

den-serve up asha-rulebench -repo /home/dev/asha-rulebench
```

`verify:change` accepts only the closed current vocabulary `frontend`,
`content-authoring`, `rust-owner`, `protocol-generated`, `host-transport`,
`browser`, and `docs`. Run `pnpm run verify` when classification is uncertain.

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

The current boundary claims explicit package resolution, deterministic
derivation/mixin/overlay materialization, Rust compilation, closed artifact
loading, provenance inspection, gameplay execution, and atomic activation. It
does not claim persistence, migration, replay compatibility, or exhaustive
certification.
