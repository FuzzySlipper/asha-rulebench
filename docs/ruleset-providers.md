# Compiled Ruleset Providers

Rulebench selects rulesets through a closed Rust provider catalog. A provider
is immutable deployment composition metadata over Rust-owned modules and
operation vocabularies; it is not a callback registry, scripting system, or
dynamic native-code loader.

## Registration contract

Each `RulesetProviderDescriptor` declares:

- stable provider id and provider version;
- exact ruleset id, version, and module configuration;
- exact operation-pipeline and effect-operation vocabulary versions;
- the versioned check, targeting, effect, and policy capabilities packages may
  use.

The concrete catalog is compiled in `rulebench-fixtures/src/providers.rs` and
validated before the built-in package registry exists. The process host and
generated capability manifest consume that same catalog. Shared resolution,
bridge, store, and UI code never switch on provider, package, or scenario ids.

## Adding or evolving a provider

1. Express behavior through existing `rulebench-ruleset`, combat-operation, and
   gameplay-module owner seams. Add a new Rust vocabulary only through its own
   reviewed compatibility change.
2. Register the provider with an exact capability set and compatible vocabulary
   versions. Colliding provider or ruleset identities fail startup tests.
3. Add independently owned content and a multi-participant package with
   deterministic positive/rejection cases, manual and automatic evidence,
   goldens, and a replay package.
4. Prove exact package/ruleset filters, cross-ruleset rejection, missing
   capability rejection, and replay/content failure under a removed, upgraded,
   or wrong provider.
5. Regenerate protocol/catalog/manifest artifacts and inspect both live setup
   selection and the Runtime capabilities dialog.

Provider versions and ruleset versions are exact for stored content and replay
interpretation. Upgrades do not silently retarget old artifacts. Migration must
produce a new validated artifact or leave the old provider available; otherwise
loading fails with a stable compatibility diagnostic before authority mutates.

Future dynamically distributed providers require a separate architecture task
covering trust, signatures, loading, isolation, and migration. This catalog does
not imply hot loading or arbitrary third-party rule code.
