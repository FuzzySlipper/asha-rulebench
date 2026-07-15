# Executable Capability Manifest

The Rulebench capability manifest is executable evidence, not a feature flag
file and not permission for TypeScript to perform rules work. Rust assembles it
from the same registries and host composition that own runtime behavior.

## Authority inputs

- `rulebench-ruleset` owns the versioned effect-operation and targeting
  declarations plus the validated compiled-provider catalog contract.
- `rulebench-combat` owns the executable operation registries and the closed
  automation-policy registry.
- `rulebench-fixtures` composes the concrete provider catalog and derives
  package, ruleset, scenario, and regression evidence from provider-validated,
  executed registered cases. Declaration metadata or a test name is not
  coverage.
- `rulebench-gameplay-module` exposes the exact governed ASHA Git revision
  compiled from the workspace lockfile.
- `rulebench-process-host` adds the selected content, replay, and active-session
  recovery adapters. A memory host and a filesystem host therefore report
  different durability facts.

The portable assembly rejects impossible support progressions. Validation
requires a declaration, execution requires validation, protocol exposure
requires execution, live-host exposure requires protocol exposure, UI
exposure requires live-host exposure, and restart durability requires runtime
execution. Operation, targeting, and policy regression coverage requires a
successful owner-level conformance case. Host capability regression evidence
may remain true when a selected host adapter disables that capability; it
still cannot promote any runtime or exposure level.

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

Manifest v3 adds an explicit `authorityViewerMode` host readback and the
`viewer.authority-readback` capability. `liveAuthorityReadback` means the
versioned viewer routes are composed from the current Rust registries and are
visible in the UI; it never means the checked TypeScript catalogs are a runtime
fallback.

Manifest v2 added compiled provider entries. Each entry names exact provider and
ruleset identities, accepted operation/effect vocabulary versions, and the
closed capability set available to packages. The ruleset inventory is derived
from this catalog, not inferred from scenario metadata. See
`ruleset-providers.md` for registration and compatibility rules.

## Evolution procedure

When adding an operation, targeting mode, policy, provider, storage adapter, or
recovery mode:

1. Add or change the owning Rust registry and its compatibility version.
2. Keep declaration, validation, execution, and host-composition evidence
   separate. Do not mark a downstream level true until its owner path exists.
3. Add a package-owned conformance case keyed to the capability id and version.
   It must execute through portable/session/replay owners and prove positive
   events/state/trace/fingerprint, classified invalid-state behavior, replay
   reproduction, and replay mismatch diagnostics. Multi-target additions also
   prove canonical ordering and atomic rollback.
4. Regenerate protocol and manifest artifacts with `pnpm run generated:write`.
5. Reconcile `docs/verification-claims.json` and Den's basic design, systems
   map, and known-limitations documents.
6. Run focused owner tests, `pnpm run verify`, and live browser inspection for
   user-visible changes.

The unfiltered regression gate compares successful executed coverage against
the executable operation, targeting, and automation-policy registries. Missing
cases, unknown ids, and version drift are classified failures. Filters are for
focused diagnosis only and never certify the complete registry.

Changing manifest structure requires a new `manifestVersion` and generated
artifact schema. Changing operation semantics requires the operation/effect
vocabulary compatibility process rather than silently rewriting an existing
row. Stored replay/content compatibility remains owned by their versioned
formats; the manifest describes support but does not migrate data.
