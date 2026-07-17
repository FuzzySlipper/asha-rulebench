# Rulebench Fixtures

`rulebench-fixtures` owns Rulebench-only scenario packages, deterministic
receipts, transcripts, scripts, and their regression manifests. It is not
portable rule authority and must never be imported by a game consumer.

Each package declares its expected authority evidence in Rust data. Generated
TypeScript catalog and session files are projections of that evidence, not the
golden source. Check them with:

```bash
pnpm run catalog:check
pnpm run session:check
```

After an intentional, reviewed Rust evidence change, regenerate only through:

```bash
pnpm run catalog:write
pnpm run session:write
```

Then re-run both checks and the full `pnpm run verify` gate before committing.

## Capability conformance

`pnpm run regression:check` is also the owner-level capability conformance
gate. It derives the required operation, targeting, and automation-policy
identities from the executable Rust registries and derives coverage only from
cases that actually execute. Each accepted case must provide deterministic
events, rolls, trace, changed-state fingerprint, classified invalid-state
behavior, verified replay, and a classified replay mismatch.

To extend the vocabulary:

1. Register the declaration and runtime handler with an explicit version.
2. Add or reuse package-owned scenario data that executes the capability; do
   not add a capability-name-only checklist row.
3. Add positive effect evidence and invalid/stale-state probes. Multi-target
   cases must prove canonical execution and atomic rollback. Stateful vitality
   cases must prove bounds and event payloads.
4. Run the focused binary with `--capability <id>` while iterating, then run
   the unfiltered `pnpm run regression:check`. Only the unfiltered run certifies
   that every executable owner-registry identity has coverage.
5. Regenerate through `pnpm run generated:write` and run `pnpm run verify`.

Unknown capability ids and owner/case version drift are classified failures.
Declared or validated capabilities that are not runtime-executable remain
honest incubation rows and do not require executable conformance evidence.

## Compiled ruleset providers

Provider types live in the pinned public `rpg-ir` owner and are temporarily
re-exported by `rulebench-rpg-adapter`; the concrete closed catalog lives in
`providers.rs`. Task #5938 removes that adapter edge.
Adding a provider requires an exact provider id/version, exact ruleset metadata,
module configuration, operation/effect vocabulary versions, and a sorted
capability set. Register a separate package with independently owned content,
scenarios, goldens, automatic evidence, and replay evidence. The strict package
registry rejects provider collisions, unknown versions, vocabulary drift,
missing capabilities, and cross-ruleset actions before execution.

Do not add a resolver switch on provider or fixture identity. New behavior must
enter through an existing Rust module/operation seam or a separately reviewed
Rust vocabulary addition. Run `regression:check` with package/ruleset filters for
both identities, then run the unfiltered gate.

## Non-Claims

This crate does not define gameplay behavior, accept TypeScript callbacks, or
serve as a generic game content-pack format. Rust rule crates remain the
authority for validation and resolution.
