# Viewer Evidence Boundaries

The product scenario and transcript viewer reads Rust authority evidence from
the running process host. Checked TypeScript catalogs remain useful build and
test artifacts, but they are not a runtime fallback.

## Product readback path

`rulebench-fixtures` composes scenario cases and combat-session transcripts
from the owner package registry. At process-host startup, every registered case
is executed through Rust authority and mapped to generated protocol DTOs. The
host-neutral `RulebenchBridge` indexes those readbacks and exposes them through
the versioned routes:

- `GET /api/rulebench/v1/viewer/scenarios`
- `GET /api/rulebench/v1/viewer/scenarios/{scenario_id}`
- `GET /api/rulebench/v1/viewer/sessions`
- `GET /api/rulebench/v1/viewer/sessions/{session_id}/steps/{step_id}`

`RulebenchLiveTransport`, `SessionStore`, the domain projector, and the Live
authority viewer consume those routes. The store cancels superseded requests,
ignores stale completions, preserves classified protocol errors, and offers an
explicit retry. An unavailable route is an error state; it never selects a
checked catalog instead.

Future scenario-package providers participate by registering with the owner
registry. The same aggregation used by regression coverage and capability
inventory feeds the viewer host composition, so no TypeScript catalog edit or
viewer-specific provider list is required.

## Checked artifact inventory

| Artifact or consumer | Classification | Runtime rule |
| --- | --- | --- |
| `libs/transport/src/generated/rust-scenario-catalog.ts` | Offline fixture and golden evidence | Imported only by `RulebenchOfflineFixtureTransport`; never injected into `SessionStore` |
| `libs/transport/src/generated/rust-combat-session.ts` | Offline fixture and golden evidence | Used by deterministic transport/codegen tests; never injected into the product viewer |
| `libs/transport/src/index.ts` default catalog exports | Offline fixture compatibility API | Names and comments identify offline ownership; product providers use `RulebenchLiveTransport` |
| `ContentStore` generated import/validation examples | Offline fixture evidence | Retained for deterministic compatibility tests and not presented as live authored-content state |
| `libs/transport/src/generated/rust-capability-manifest.ts` | Checked durable-host evidence | The Runtime capabilities dialog reads the live `/capabilities` route and has no fallback |
| `libs/protocol/src/generated/api-types.ts` | Generated wire contract | Product and offline flows both use these Rust-owned DTO definitions |

`pnpm run generated:check` proves that checked artifacts match their Rust
emitters. `pnpm run regression:check` proves registered cases execute. Neither
gate is a claim that the current host is reachable or that build-time evidence
is current runtime state.

## Evolution rules

Add viewer fields to Rust protocol DTOs and regenerate the TypeScript contract;
do not hand-edit generated files. Keep bridge methods host-neutral and keep
HTTP, process lifecycle, and JSON routing in the process host. New product
viewer flows must use the live transport/store path with cancellation,
classified errors, and visible retry. New checked catalogs must state their
offline or golden purpose at the import boundary.
