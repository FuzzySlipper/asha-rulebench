# Executable Capability Manifest

The Rulebench capability manifest is executable evidence, not a feature flag
file and not permission for TypeScript to perform rules work. Rust assembles it
from the same registries and host composition that own runtime behavior.

## Authority inputs

- public `rpg-ir` owns the versioned effect-operation and targeting
  declarations plus the validated compiled-provider catalog contract.
- `rulebench-combat` owns the executable operation registries and the closed
  automation-policy registry.
- `rulebench-fixtures` composes the concrete provider catalog and derives
  package, ruleset, scenario, and regression evidence from provider-validated,
  executed registered cases. Declaration metadata or a test name is not
  coverage.
- public `rpg-runtime` exposes the exact governed ASHA Git revision compiled
  behind the pinned `asha-rpg` boundary.
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
`pnpm run check:claims:executable` validates its identity, ordering, owner
evidence, and support invariants.

The running process host serves `GET /api/rulebench/v1/capabilities`. The
transport, `LiveCombatStore`, domain projector, and Runtime capabilities dialog
consume that generated DTO. They do not fall back to the checked durable-host
artifact when the live route is unavailable, because that would turn build
evidence into a false runtime claim.

Manifest v4 adds the `session.active-recovery` capability and makes its support
levels depend on the selected recovery adapter. The checked filesystem-host
artifact reports replay-verified checkpoints and restart durability; an
in-memory host reports process-local checkpoints without durability. Protocol,
host, UI, regression, and durability evidence remain separate.

Manifest v4 also reports `content.authored-action@1`. This row claims only the
strict v3 template/clone/text workflow and the Rust-owned exact-pack action
binding described in `authored-content-format.md`: provider-gated targeting,
checks, ordered effects, modifiers/durations, reaction selector expansion,
actor resource-pool checks, session-local ability grants, and exact replay and
recovery provenance. It does not claim top-level authored movement, arbitrary
scripts/plugins/callbacks, general character/class/item/resource authoring, or
TypeScript authority. Its regression flag remains separate from independent
milestone certification evidence.

`content.authored-scenario@1` reports the strict v4 scenario-composition path:
Rust-owned archetype/build inputs, initial participant and grid state, multiple
action grants, exact manual/automatic control provenance, live materialization,
and recovery/replay reconstruction from an exact active pack set. It does not
claim general character progression, inventory simulation, arbitrary scripts,
or TypeScript legality and automation.

Manifest v3 added an explicit `authorityViewerMode` host readback and the
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
5. Run `pnpm run review:claims-and-limitations` to emit the current executable
   inventory together with the last Den-reviewed claims and limitation
   snapshot. Refresh that snapshot from Den when policy requires a governance
   review; do not copy counts, prose, or a literal freshness date into the
   source gate.
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
