# Replay Archive Identity

Finalized replay archives use four deliberately separate compatibility
identities:

- the process-host JSON envelope `formatVersion` describes filesystem storage;
- `archivePayloadEncodingVersion` names the portable semantic byte encoding;
- `packageVersion` describes the replay package contract;
- `archiveFingerprintAlgorithm` names only the hash applied to canonical bytes.

Current writes use envelope format `2`, canonical payload encoding
`asha-rulebench.replay-archive-payload.v3`, replay package version `1.0.0`, and
hash algorithm `fnv1a64`. The hash is deterministic corruption evidence, not a
cryptographic authenticity proof.

## Canonical payload

`rulebench-replay` owns the encoder. It emits length-prefixed binary values in
an explicit order and covers archive metadata, the complete initial scenario,
ruleset and content provenance, every typed command field, supplied or
generated randomness, ordered events, audits, rolls, trace, gameplay receipts,
final state identity, and optional narration. Multi-target intent fields and
action-resource declarations/events are encoded directly. V3 also encodes the
exact authored-scenario composition receipt: pack set, scenario, archetype
inputs, loadout items, action definition/runtime mappings, and control policy.

The encoding does not use Rust `Debug`, private layout, declaration order,
serde or host JSON, generated TypeScript, fixtures, bridge code, or protocol
DTOs. A stable golden test therefore survives formatting and debug-shape
changes, while mutation tests prove semantic changes alter the fingerprint.
The encoder is currently a Rulebench product owner. The former standalone
consumer was retired when reusable authority moved to `asha-rpg`; replay
archive extraction and its independent proof are assigned to #5942.

Adding an integrity-relevant package field requires an intentional encoder
change and a new canonical payload encoding version. Readers must never guess
the meaning of unknown versions. Adapters may choose their own envelope and
serialization, but must consume the portable canonical identity rather than
reimplement it.

## Filesystem write and migration policy

New and replacement envelopes and the deterministic index are written to a
sibling temporary path and atomically renamed. Startup ignores and reports
temporary files, validates the host JSON payload checksum, reconstructs stored
commands through the registered scenario and current Rust authority, and then
validates the portable canonical identity before exposing an entry.

Envelope v1 records carrying the recognized legacy v0 or v1 archive identity
are decoded, checksum-validated, authority-reconstructed, and rewritten as
envelope v2 before they become visible. The rewrite is atomic. If it cannot be
committed, the legacy record is quarantined for that startup and remains
unchanged. Unknown envelope or archive identity versions are quarantined rather
than silently accepted.

Envelope v2 records carrying the storage-integrity-valid canonical v2 payload
identity are also recognized as a bounded migration input. Startup reconstructs
them through current authority and atomically rewrites the current v3 canonical
identity; it does not reinterpret their stored hash as a v3 hash.

Stable startup diagnostics distinguish unsupported envelope versions,
unsupported legacy identity versions, malformed envelopes, host-payload
corruption, canonical identity mismatch, partial commits, and migration commit
failure. The rebuilt index contains the canonical payload encoding version and
fingerprint, so list/load ordering and identity remain deterministic across
restarts.

## Compatibility limits

Canonical v3 guarantees that internal Rust formatting, `Debug` output, and
host JSON field order do not affect archive identity. It does not promise that
an archive remains executable after its registered scenario, ruleset, content
pack, replay package version, or canonical encoding version becomes
unsupported. Those are explicit compatibility boundaries and must be handled
through a named migration, not an exception that weakens corruption checks.
