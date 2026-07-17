# Executable Capability Manifest

The Rulebench capability manifest is a runtime composition report, not a
feature flag file and not permission for TypeScript to perform rules work.

## Authority inputs

- public `rpg-ir` owns versioned operation and targeting declarations;
- `rulebench-combat` owns executable operations and the closed policy registry;
- `rulebench-product-content` supplies named providers, packages, and scenarios;
- public `rpg-runtime` supplies the exact governed ASHA revision;
- `rulebench-process-host` adds its selected storage and recovery adapters.

The manifest validates one-way support progression: declaration, execution,
protocol exposure, live-host exposure, UI exposure, then restart durability.
A memory host and filesystem host report different durability facts.

Product capability rows do not claim exhaustive regression coverage. That
evidence is evaluated downstream by `FuzzySlipper/asha-rulebench-testing`
against exact public product revisions.

## Product consumer

The running process host serves `GET /api/rulebench/v1/capabilities`. The live
transport, store, domain projector, and Runtime capabilities dialog consume the
generated protocol DTO. There is no checked capability artifact fallback.

Provider entries name exact provider/ruleset identities, accepted vocabulary
versions, and the closed capability set available to packages. Content and
replay formats retain their own strict version/migration rules; the manifest
describes current support but does not migrate stored data.

## Evolution procedure

1. Change the owning public RPG or Rulebench Rust registry and compatibility
   version.
2. Keep declaration, validation, execution, host, UI, and durability support
   separate.
3. Add focused owner tests and primary product workflow coverage.
4. Regenerate the protocol DTO contract when its shape changes.
5. Run `pnpm run verify` and inspect managed live UI evidence for visible
   changes.
6. Update the downstream conformance suite when the public capability contract
   changes; do not add exhaustive proof artifacts back to Rulebench.

Changing manifest structure requires a new `manifestVersion`. Changing
operation semantics requires the public RPG language compatibility process.
