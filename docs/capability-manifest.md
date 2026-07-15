# Executable Capability Manifest

The Rulebench capability manifest is executable evidence, not a feature flag
file and not permission for TypeScript to perform rules work. Rust assembles it
from the same registries and host composition that own runtime behavior.

## Authority inputs

- `rulebench-ruleset` owns the versioned effect-operation and targeting
  declarations.
- `rulebench-combat` owns the executable operation registries and the closed
  automation-policy registry.
- `rulebench-fixtures` derives package, ruleset, scenario, and regression
  evidence from the registered package catalog and scenario cases.
- `rulebench-gameplay-module` exposes the exact governed ASHA Git revision
  compiled from the workspace lockfile.
- `rulebench-process-host` adds the selected content, replay, and active-session
  recovery adapters. A memory host and a filesystem host therefore report
  different durability facts.

The portable assembly rejects impossible support progressions. Validation
requires a declaration, execution requires validation, protocol exposure
requires execution, live-host exposure requires protocol exposure, UI
exposure requires live-host exposure, and restart durability requires runtime
execution. Regression coverage is independent evidence and cannot promote any
of those levels.

## Consumers

`rulebench-process-host/emit_capability_manifest` emits
`libs/transport/src/generated/rust-capability-manifest.ts` from a real durable
host composition. `pnpm run generated:check` reconstructs and compares it, and
`pnpm run check:claims` validates its identity, ordering, owner evidence, and
support invariants.

The running process host serves `GET /api/rulebench/v1/capabilities`. The
transport, `LiveCombatStore`, domain projector, and Runtime capabilities dialog
consume that generated DTO. They do not fall back to the checked durable-host
artifact when the live route is unavailable, because that would turn build
evidence into a false runtime claim.

## Evolution procedure

When adding an operation, targeting mode, policy, provider, storage adapter, or
recovery mode:

1. Add or change the owning Rust registry and its compatibility version.
2. Keep declaration, validation, execution, and host-composition evidence
   separate. Do not mark a downstream level true until its owner path exists.
3. Add positive runtime/regression evidence and a negative test that prevents
   metadata, generated output, or UI projection from substituting for runtime
   support.
4. Regenerate protocol and manifest artifacts with `pnpm run generated:write`.
5. Reconcile `docs/verification-claims.json` and Den's basic design, systems
   map, and known-limitations documents.
6. Run focused owner tests, `pnpm run verify`, and live browser inspection for
   user-visible changes.

Changing manifest structure requires a new `manifestVersion` and generated
artifact schema. Changing operation semantics requires the operation/effect
vocabulary compatibility process rather than silently rewriting an existing
row. Stored replay/content compatibility remains owned by their versioned
formats; the manifest describes support but does not migrate data.
