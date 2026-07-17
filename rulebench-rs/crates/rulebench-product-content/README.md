# Rulebench Product Content

`rulebench-product-content` owns the named scenarios and primary workflow
samples composed into the Rulebench process host. Its packages reference
compiled providers and Rust-owned behavior; they do not define portable RPG
semantics or a parallel rules engine.

The product repository keeps focused tests for package validity, provider
compatibility, and the primary scenarios shown in the UI:

```bash
cargo test --manifest-path rulebench-rs/Cargo.toml -p rulebench-product-content
pnpm run verify:change -- --profile product-content
```

Synthetic conformance cases, golden expectations, full regression matrices,
generated scenario/session/capability proof artifacts, and certification
receipts are owned by the public downstream consumer
`FuzzySlipper/asha-rulebench-testing`. Do not add those surfaces back to this
crate or introduce a dependency on the downstream repository.

New product content must use the public `asha-rpg` language and current
Rulebench provider catalog. Unsupported semantics require an owner change in
the public RPG language or a planner-approved Rulebench authority extension;
they may not be implemented as callbacks or hidden scenario-specific logic.
