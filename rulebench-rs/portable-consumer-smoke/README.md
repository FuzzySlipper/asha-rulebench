# Portable Consumer Smoke

This independent Cargo workspace is the minimum supported `rulebench-rules`
consumer contract. It depends only on the portable facade, authors a valid
two-combatant scenario, creates and starts a combat session, submits one typed
action intent with deterministic rolls, and inspects Rust-owned accepted events
and session log readback.

It must not depend on Rulebench fixtures, authority/catalog code, codegen,
protocol/bridge adapters, or frontend code. `pnpm run check:portable-consumer`
runs the scenario and rejects those Rulebench-local crates in the dependency
tree; `pnpm run verify` includes that check.
