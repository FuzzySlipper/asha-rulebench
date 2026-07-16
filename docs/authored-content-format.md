# Authored Content Pack Format

The live workbench imports one Rust-owned JSON document family,
`asha-rulebench.content-pack`. The process host accepts at most 512 KiB per
authored payload. Unknown fields in a shipped version, unknown format names,
and unknown format versions fail closed; callers must not infer, repair, or
reinterpret a newer document.

## Shipped versions

Version 1 is the original compatibility contract. It carries pack identity and
display metadata, stable source provenance, one exact selected ruleset, exact
dependency references, authored ruleset declarations, and entity definitions.
Its catalog shape remains strict and does not accept fields introduced later.

Version 2 preserves those fields and requires an `abilities` catalog, which may
be empty. An authored ability contains an id, name, closed `ability` or `spell`
kind, summary, and tags. This is the dependency-root slice exercised by the
second provider's Binding Glyph content. It establishes a durable catalog
converter without pretending that ability metadata alone defines an executable
action.

The committed fixtures
`rulebench-process-host/src/fixtures/authored-content-v1.json` and
`authored-content-v2.json` are reader compatibility evidence. Both pass through
the same Rust import workspace; neither is converted or validated by
TypeScript.

## Rust conversion and diagnostics

`rulebench-protocol` owns the version readers and the single authored-catalog
converter. Adding another definition kind changes that protocol owner and its
generated DTO metadata; the host, store, diff, definition-index, and component
paths remain generic over canonical definition kind and id.

Rust rejects incomplete or duplicate abilities before persistence. Stable
diagnostics include `emptyContentImportField` with paths such as
`catalogs.abilities[0].summary`, `duplicateContentImportDefinition`,
`contentImportLimitExceeded`, and `unsupportedAuthoredContentVersion`. Closed
enum decoding rejects an unknown ability kind as `invalidAuthoredContentPayload`.
Exact dependency references still require id, version, fingerprint algorithm,
and fingerprint value. The default canonical importer limits each catalog to
10,000 definitions, the complete pack to 50,000 definitions, dependencies to 64,
and each inspected string to 16 KiB.

Rulesets, entities, and abilities are canonicalized and fingerprinted in Rust.
Generic definition indexes and structured pack diffs therefore report ability
additions, removals, and material changes without a kind-specific TypeScript
branch. UI metadata and a stored receipt are never substitutes for canonical
import.

## Durability and exact binding

The stored payload is not authority merely because a receipt exists. At startup
Rust decodes the original versioned document, resolves dependencies in
deterministic order, imports and canonicalizes it again, and requires the exact
stored fingerprint. Only revalidated exact references may be activated or
selected for a session. A selected session records the resolved pack-set root,
members, and set fingerprint in replay evidence; replay review renders those
exact references.

Replacement is transactional. A failed decode, semantic import, canonical
store, or activation change leaves the last known-good payload, active set, and
audit history intact. A pack cannot supply a missing compiled ruleset provider
or capability: session binding fails closed when the selected scenario and
exact pack ruleset are incompatible.

## Evolution and migration

- Readers for versions 1 and 2 are permanent compatibility surfaces. Version 1
  is lifted into the current in-memory DTO with an empty ability catalog only
  after its exact v1 shape has decoded.
- A semantic or wire-vocabulary change requires a new `formatVersion` and a
  dedicated Rust reader. Never reinterpret v1 or v2 with guessed defaults.
- Storage receipt or activation-index changes likewise require an explicit
  reader or migration. Unsupported records are quarantined, not guessed.
- A future migration must decode the old payload, convert it through the owned
  catalog converter, run the current Rust importer, compare canonical receipts,
  and commit through the repository replacement transaction. Failure preserves
  the last known-good payload and activation.

## Non-claims

Version 2 does not author actions, checks, effects, modifiers, stats, classes,
items, equipment, entity ability grants, or provider implementations. Binding
Glyph remains executable because the compiled second provider and scenario own
those semantics; the authored ability is canonical identity and descriptive
content. The filesystem adapter remains trusted-local and single-writer. It
does not claim multi-process locking, authenticated upload, network-filesystem
atomicity, power-loss durability, or durable in-progress policy experiments.
