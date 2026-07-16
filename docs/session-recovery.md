# Active-session recovery

The durable process host can recover an active combat session after a clean or
unclean host restart. Recovery preserves Rust authority: the filesystem record
contains reconstruction inputs and verified evidence, never a serialized
`CombatSessionState`, a TypeScript-owned state snapshot, or an opaque ASHA
runtime checkpoint.

## Durable package

After session creation and every accepted command boundary, the bridge asks the
portable replay owner to build a versioned recovery package containing:

- the canonical session creation request and scenario;
- the exact selected ruleset artifact provenance;
- normalized typed commands and the evidence returned when each command ran;
- a monotonically increasing generation equal to the command count;
- the last command sequence and identity;
- the authoritative state fingerprint and gameplay-module state hash; and
- the pending reaction-window identity, when resolution is suspended.

The concrete filesystem adapter wraps that package in storage format v1,
fingerprints its canonical payload, and commits it with a same-filesystem
temporary-file rename. It keeps one package per session identity. A finalized
session first becomes a replay archive and then loses its active recovery
package, so finalized and recoverable sessions cannot be confused.

## Startup reconstruction

Startup reads every recovery record independently. For each valid record it
creates a fresh Rust combat session from the stored canonical scenario and
replays the typed commands through the same authority path used for replay
verification. The bridge installs the reconstructed state only after all
command evidence and the final generation, fingerprint, module hash, and
reaction-window identity match the stored frame.

An unsupported format, malformed JSON, payload-fingerprint mismatch, missing
scenario, changed ruleset provenance, changed command evidence, or mismatched
final frame is classified and quarantined. A leftover temporary file is also
reported as a partial commit. One bad record does not block other sessions or
expose a partially reconstructed session.

The live recovery catalog reports recoverable sessions separately from
unrecoverable storage issues. A recoverable entry identifies whether it was
created in the current process, restored at startup, or explicitly forked. The
operator can discard a recovery package or fork its fully reconstructed state
under a new explicit session identity. There is no implicit fork or silent
fallback to a blank session.

## Evolution and retention

The portable recovery package is version `1.0.0`; the filesystem envelope is
storage format v1. Readers reject and quarantine unknown versions. Any future
format change must add a deliberate reader or atomic migration before either
version advances. Recovery records are retained until the session is finalized
and closed or explicitly discarded. The adapter performs no time-based cleanup.

The repository remains trusted-local and single-writer. It does not claim
multi-process locking, authentication, hostile-storage hardening, database
migrations, power-loss durability, `fsync`, or atomic replacement on network
filesystems.

## Boundary with upstream ASHA

The pinned ASHA `ComposedRuntimeSessionCheckpoint` is intentionally opaque and
in-memory. Rulebench therefore does not serialize it. Durable recovery uses the
upstream-supported replay boundary: canonical scenario input, typed commands,
generation, and fingerprints. This recovers accepted quiescent command frames,
including a suspended reaction window represented by authoritative replay
state. It does not recover an instruction that was interrupted while executing,
arbitrary process memory, callbacks, or host resources.

The executable capability manifest distinguishes the selected adapters:

- `replayVerifiedCheckpoints` means the filesystem host can reconstruct active
  sessions across restart;
- `processLocalCheckpoints` means the in-memory host owns the same lifecycle but
  cannot survive process loss; and
- `none` means active-session recovery is not composed.
