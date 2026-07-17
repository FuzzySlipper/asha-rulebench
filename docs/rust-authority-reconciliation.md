# Rust Ownership Boundary

Portable RPG language and runtime behavior is owned by public repository
`FuzzySlipper/asha-rpg`. Rulebench consumes one exact public revision; sibling
paths, direct ASHA dependencies, and private source imports are rejected by the
Rust boundary check.

Rulebench owns product content, content storage, combat composition, replay,
protocol mapping, bridge behavior, and its loopback process host. The local
workspace is intentionally small and exposes no combined compatibility facade.

Exhaustive synthetic authority tests, semantic/conformance matrices,
cross-version fixtures, generated proof artifacts, and certification receipts
are physically owned by public downstream repository
`FuzzySlipper/asha-rulebench-testing`. That repository consumes exact public
revisions. Neither `asha-rpg` nor Rulebench imports it or waits for it as an
ordinary product gate.

The enforced direction is:

```text
asha-engine <- asha-rpg <- asha-rulebench <- asha-rulebench-testing
```

The arrows mean public dependency/consumption only. A testing failure is routed
to the first broken public contract and does not grant the testing repository
semantic authority.
