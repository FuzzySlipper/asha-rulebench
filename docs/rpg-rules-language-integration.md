# RPG Rules Language Integration

Rulebench owns downstream RPG content in `libs/content-authoring`. The public
`@asha-rpg/authoring` package supplies immutable builders and normalization;
it does not execute semantics. `tools/scripts/rpg-content-artifact.mjs`
normalizes the authored package into canonical `asha.rpg.ir@1` data, and the
generated artifact is checked by the normal generated-artifact gate.

At runtime, `rulebench-content` strictly decodes that artifact and
`rpg-compiler` validates and compiles it. `rulebench-combat` constructs the
portable capability state, submits typed intents to `RpgAuthoritySession`, and
maps accepted RPG DomainEvents, trace, bounded random evidence, and final
projection into product readouts. Workbench session lifecycle, reaction-window
orchestration, archives, experiments, transport, and UI state remain
Rulebench-owned product concerns.

## Content-only extension path

An ordinary new action or reusable composition helper changes:

1. a TypeScript source under `libs/content-authoring`;
2. its owner-local normalization expectation; and
3. the mechanically generated normalized content artifact.

It does not require a Rust source change, protocol DTO, host route, capability
manifest registration, or certification fixture. If the desired behavior is
not expressible with the published operation vocabulary, stop: that is a new
semantic operation and must begin with the Rust-first extension contract in
`asha-rpg`.

`pnpm run check:rules-language-boundary` reports the expected amplification:
three downstream layers for content-only work and seven owner layers for a new
semantic operation. Its focused fixtures reject Rust, protocol, host-route,
capability-manifest, and certification/proof changes falsely classified as
content-only.

## Enforced extension checklists

For a content-only action or pure reusable helper:

1. change TypeScript under `libs/content-authoring/src`;
2. update its owner-local normalization expectation;
3. regenerate the normalized RPG IR artifact;
4. confirm no Rust source, product protocol, host route, capability manifest,
   or certification/proof manifest changed.

For a new semantic operation:

1. begin in `asha-rpg` with the strict IR declaration and version;
2. declare Rust reads, mutation owner, validation behavior, accepted
   DomainEvents, trace behavior, and replay implications;
3. implement Rust compatibility/reference/semantic validation and staged
   owner mutation with atomic rejection tests;
4. regenerate the published operation vocabulary;
5. only then publish TypeScript authoring sugar and normalization/type tests;
6. advance Rulebench's exact public revision and add only focused product
   mapping/regression where the operation is actually consumed.

TypeScript authoring cannot contain executable callbacks, mutable gameplay
contexts, semantic evaluators, capability-store access, browser/Angular APIs,
transport, or product imports. A future `libs/rpg-policy` lane may read typed
protocol views and propose typed intents; it has no authority access and cannot
evaluate rule meaning.

## Migration boundary

The representative migrated corpus covers target legality, attack and saving
throw checks, damage, healing, modifiers and durations, action resources,
movement, product reaction orchestration, conditional branches, and
multi-target resolution. Scenario-local runtime ids are resolved through exact
authored binding receipts. A missing compiled source id rejects at the RPG
boundary; migrated authored actions never fall through to the legacy resolver.

Older Rust fixture catalogs remain only as proof inputs pending task #5942,
which moves exhaustive synthetic evidence to `asha-rulebench-testing`. They are
not a second authoring path for the migrated Shatterline workflow. This task
does not claim that archive browsing, policy experiments, storage, or product
UI state are portable RPG authority.

The focused fixtures in Rulebench and `asha-rpg` enforce the attractive
nuisance boundaries. They are not exhaustive proof: cross-product forbidden
fixtures and consumer certification belong to `asha-rulebench-testing` under
#5942.
