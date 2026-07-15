# Authored Content Pack Format

The live workbench imports one Rust-owned JSON document format:
`asha-rulebench.content-pack` with `formatVersion: 1`. The process host accepts
at most 512 KiB per authored payload. Unknown fields, unknown format names, and
unknown versions fail closed; callers must not infer or repair a newer format.

Version 1 carries pack identity and display metadata, stable source provenance,
an exact selected ruleset, exact dependency references, authored ruleset
declarations, and entity definitions. Rulesets use the closed module DTO
vocabulary already owned by `rulebench-protocol`. Entity damage adjustments use
a closed policy vocabulary. Other canonical definition kinds already flow
through generic storage indexes, structured diffs, and UI readouts; adding a
new authored wire definition requires a protocol version change and a Rust
conversion, but no kind-specific TypeScript authority branch.

The stored payload is never authority merely because a receipt exists. At
startup Rust decodes the original document, resolves dependencies in
deterministic order, imports and canonicalizes it again, and requires the exact
stored fingerprint. Only revalidated exact references may be activated or
selected for a session. A selected session records the resolved pack-set root,
members, and fingerprint in replay evidence.

## Evolution and migration

- Additive display-only response fields may remain compatible with the current
  host protocol, but authored input fields stay strict.
- A semantic or wire-vocabulary change requires a new `formatVersion` and a
  dedicated Rust reader or explicit migration command before the host accepts
  it. Never reinterpret v1 data with v2 defaults.
- Storage receipt or activation-index changes likewise require an explicit
  version reader/migration. Unsupported records are quarantined, not guessed.
- Migrations must decode the old payload, convert it into a new authored DTO,
  run the current Rust importer, compare canonical receipts, and commit through
  the repository replacement transaction. Failed migration leaves the last
  known-good payload and activation unchanged.

The current filesystem adapter is trusted-local and single-writer. It does not
claim multi-process locking, authenticated upload, network-filesystem atomicity,
power-loss durability, or durable active combat sessions.
