# Gameplay Fabric Integration

Rulebench now exercises one real downstream gameplay-fabric slice: a typed
`BeforeEffect` reaction decision. The point is not to replace Rulebench's
combat runtime. It is to prove that expressive game behavior can cross ASHA's
public governed boundary without handing mutation authority to a callback or
requiring another bespoke upstream access point.

## The concrete flow

1. Rulebench resolves an authored hit far enough to open a local reaction
   window.
2. Public `rpg-runtime` submits a typed pre-effect workspace through ASHA's
   RuntimeSession composition.
3. The static module registry invokes the RPG fabric's declared Transform and React
   stages. Each invocation receives only its declared, frozen reads.
4. React suspends. The ASHA host owns the continuation token and persists it in
   the host snapshot; Rulebench keeps the typed continuation beside its local
   pending combat resolution.
5. Rulebench restores a staged host from the canonical pre-resolution snapshot
   and records the chosen local outcome there as a typed Observe fact.
6. Resume re-runs on that staged host with the expected Rulebench owner
   revision. A matching accepted reaction transforms the proposed damage by
   the authored reduction. The narrow Rulebench owner validates and commits the
   resulting workspace, and only then does Rulebench publish the staged host.
   A wrong, stale, or consumed continuation leaves the live module state,
   frames, decision readout, and host snapshot unchanged.
7. The live snapshot projects module-state hashes, reaction-frame hashes,
   decision receipt hashes, declared-read hashes, invocation hashes, routing
   evidence, and diagnostics. The Audit panel displays that evidence without
   interpreting it as authority.

The built-in **Hexing Bolt Reaction** scenario exposes the Rust-owned response
window through the generated protocol. The user can pass or select an authored
option; Rust validates response order, resumes the ASHA continuation, commits
the accepted transform, and returns the next authority snapshot. TypeScript
only renders the options and submits the selected typed command.

## Ownership boundary

ASHA owns the generic fabric:

- closed module and binding registries;
- typed module contracts and declared-read plans;
- Guard, Transform, and React invocation;
- suspension-token validation and consumption;
- deterministic module state, Observe frames, receipts, snapshots, and
  evidence hashes;
- owner routing constrained by the resolved registry.

Rulebench owns the gameplay semantics:

- what a pre-effect reaction window means;
- which combatants and options participate, including nested-window ordering;
- the authored two-point accepted-reaction damage reduction;
- combat state and resource validation;
- the narrow owner adapter that validates and commits a pre-effect workspace;
- presentation of the evidence in its protocol and workbench.

This is fabric rather than an event bus. Modules do not subscribe dynamically,
hold mutable callbacks, or gain ambient access to combat state. The registry,
binding, invocation stages, reads, outputs, owner revision, and resulting
evidence are inspectable before and after execution.

## Determinism and replay

The ASHA host snapshot round-trips module state, reaction frames, decisions,
and live continuation state. Rulebench's focused module test restores a
suspended host before resuming and proves the same accepted transform and
receipt readout. It also replays a consumed continuation at the unchanged
expected revision and proves there is no second owner commit and no live
state/frame/readout/snapshot mutation.

Rulebench replay step evidence now includes the gameplay module-state hash and
ordered decision-receipt hashes. Verification rejects the first command whose
gameplay-fabric evidence differs even when its traditional combat-state
fingerprint is unchanged. Replay-package comparison exposes the same mismatch
as a distinct machine-readable dimension.

## Reconciliation and non-claims

- The local reaction-window coordinator remains Rulebench-owned. This slice
  does not claim that ordered or nested RPG reactions belong upstream.
- The public `RpgPreEffectOwner` interface is intentionally narrow. It is the
  consumer-owned commit border, not a mutable callback or generic ASHA owner
  registry.
- Rulebench consumes `rpg-runtime` from the governed public `asha-rpg` Git
  repository at one exact revision and compatible `^0.1` version. The boundary
  gate rejects sibling paths, direct ASHA crates, stale revisions, and
  incompatible versions. The independent minimal consumer is owned and run in
  `asha-rpg`.
- ASHA #5797 supplied the composed downstream-combat-owner seam. Rulebench now
  installs one concrete owner through `StaticRuntimeSessionBuilder`, uses the
  composed transaction for continuation validation, routing, typed opened and
  resolved facts, module state, checkpoints, and rollback, and no longer
  declares the quarantined standalone host.

See [rust-authority-reconciliation.md](rust-authority-reconciliation.md) for
the complete crate disposition and upstream candidate audit.
