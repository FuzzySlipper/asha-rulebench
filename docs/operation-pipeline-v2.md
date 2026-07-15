# Operation Pipeline v2

Operation pipeline v2 is the Rust-owned execution contract for bounded area and
explicit multi-target actions. It extends the existing single-target action
path; it does not replace it and does not move rule authority into TypeScript.

## Declaration and compatibility

An action opts in through `TargetingDeclaration.operation_pipeline`. The v2
declaration fixes all behavior that can change target or roll interpretation:

- a positive `maximum_targets` no greater than eight;
- either an explicit combatant set or a Manhattan-burst area with radius one
  through four;
- `shared`, `perTarget`, or `noRoll` roll policy;
- atomic failure policy; and
- canonical participant-id target order.

Legacy single-target and movement declarations keep `operation_pipeline: None`
and retain their existing fingerprints and replay behavior. Stateful `Move`
and `ChangeResource` hit operations are rejected outside v2. The current
content-pack v1 JSON vocabulary still imports rulesets and entities only; it
does not claim a JSON action-authoring format. Rust scenario content and
portable Rust consumers author v2 actions directly until a separately
versioned content wire is designed.

## Resolution contract

Rust derives the complete target set before consuming effects. Explicit sets
reject empty, duplicate, over-limit, missing, defeated, out-of-range, hostile
or ally constraint, and visibility failures. Area intents carry a selected
cell; Rust validates the cell and center range, derives live declared targets
inside the radius, truncates in canonical order, and rejects an empty result.

The resolver validates the complete fixed roll bundle and then evaluates the
operation against a cloned combat state. Damage, healing, temporary vitality,
modifiers, push, pull, shift, and resource changes commit together. A blocked,
occupied, out-of-bounds, missing-resource, or bounded-resource failure returns
no accepted events and leaves every target unchanged. A per-target attack miss
is an accepted target result with no hit effects, not a partial command
failure.

Before-effect reactions suspend the complete receipt. When the reaction frame
resumes, the composed Rust owner commits every target result and the actor cost
as one transaction. While a reaction is open, preflight, control, equipment,
and further action submission remain gated by the existing reaction owner.

## Protocol, evidence, and replay

Generated intents expose `targetIds` and `targetCell`. Current-actor options
expose bounded `targetSets`; manual UI and automatic candidates submit those
exact generated affordances. Manual supplied-roll entry accepts an extensible
comma-separated tail rather than assuming one attack/damage pair. Command readback exposes ordered per-target
attack, damage, movement, and resource evidence. TypeScript projects and
renders these DTOs but never re-derives the target set or applies effects.

Vitality, conditions, and positions retain the established state-fingerprint
contract. Action resources have a separate deterministic
`fnv1a64.rulebench-action-resources.v0` fingerprint plus typed
`changedByEffect` transition entries, so resource-only operations are auditable
without changing the meaning of old state hashes. Replay packages retain the
exact multi-target intent, ordered events, rolls, trace, command audit, and
final evidence. Changing a recorded area cell or target set produces a replay
mismatch.

Replay archive payload identity is now the explicit portable encoding
`asha-rulebench.replay-archive-payload.v2` hashed with `fnv1a64`. It covers the
complete multi-target/action-resource scenario, command, randomness, and
evidence contract without depending on Rust `Debug` or host JSON. The process
host authority-verifies recognized legacy v0/v1 envelopes and atomically
rewrites them to envelope v2 before exposure. See
`replay-archive-identity.md` for compatibility and quarantine behavior.

## Consumer example

`rulebench-rs/portable-consumer-smoke` authors an explicit two-target,
per-target-roll action, records and fingerprints its replay through
`rulebench-rules`, and executes it without fixtures, protocol, bridge, codegen,
or host dependencies. The Watchtower
`Storm Pulse` fixture supplies the product proof for a shared-roll area action
with damage, push, target resource changes, actor cost, UI affordances, replay,
and composed reaction behavior.
