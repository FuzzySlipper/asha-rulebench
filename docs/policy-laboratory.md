# Deterministic Policy Laboratory

The policy laboratory compares bounded Rust-owned control strategies. It is an
observation and replay workflow, not a rules engine, AI system, tournament
service, or source of accepted gameplay truth.

## Authority and compatibility

`rulebench-combat` owns the closed, versioned policy registry. A registration
declares its stable id/version, display metadata, selector, and ruleset
requirement. Policy execution receives only Rust-projected command candidates,
preflight decisions, target side/vitality readouts, and selected-ruleset
context. TypeScript selects a registered configuration but never supplies a
callback or chooses a command.

The process host exposes `GET /api/rulebench/v1/automation-policies`. Each row
contains live compatibility results for every registered ruleset identity.
Unsupported ids, versions, and ruleset requirements fail before a session or
trial is mutated.

## Adding a policy

One owner path is required:

1. Add the selector and one registration in
   `rulebench-combat/src/runtime/automation/policy.rs`.
2. Add its exhaustive deterministic score and evidence reason in
   `rulebench-combat/src/runtime/automation.rs`. Tie-breaking remains candidate
   index after policy score.
3. Declare the capability only on compatible providers in
   `rulebench-fixtures/src/providers.rs`.
4. Add focused selection, tie, incompatibility, no-candidate, end, and reaction
   tests as applicable. Add a package-owned automatic run plus verified replay
   so the conformance gate covers `policy.<id>`.
5. Regenerate checked protocol, session, and capability artifacts. The shared
   UI reads catalog metadata and does not gain policy-specific decision code.

Do not add dynamic dispatch supplied by a host, mutable callbacks, fixture
field access from policy code, or a TypeScript implementation of the ranking.

## Bounded experiment lifecycle

A matrix expands scenario/ruleset selections, registered policy
configurations, and explicit roll seeds. Creation validates the complete matrix
before state creation. The current limits are 16 trials and 64 automatic steps
per trial.

Experiments are advanced one trial per request. This makes progress observable
and cancellation explicit without background threads or ambient scheduling.
Every completed trial records:

- scenario, content, ruleset, policy, configuration, and seed provenance;
- stop/finalization state, fingerprints, decision evidence, and bounded metrics;
- the exact replay package id and replay-verification result.

A max-step stop is explicitly finalized for archival while retaining
`stoppedAtMaxSteps` as the trial stop reason. The replay package is stored in the
same archive as other finalized evidence and therefore remains reviewable after
a filesystem-host restart. The in-process experiment progress catalog itself is
not claimed as restart-durable.

Comparison reports the first differing decision-evidence index, or a
decision-count/final-fingerprint boundary when the decision prefix matches.
Metrics such as executed steps and observed hit-point delta are review aids
only.
