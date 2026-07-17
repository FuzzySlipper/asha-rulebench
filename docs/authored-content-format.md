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

Version 3 preserves the v2 catalogs and requires `modifiers` and `actions`
catalogs, either of which may be empty. A v3 action is portable authored
content, not a runtime `ActionDefinition`: it has no actor id, concrete target
ids, visible-target cache, or concrete reaction participant ids. Those values
belong to the later Rust binding step. Its stable fields are:

- action id, referenced ability id, name, action text, and effect text;
- a closed target kind, selection, team constraint, range, visibility policy,
  and optional bounded multi-target or Manhattan-area pipeline;
- one attack, saving-throw, or contested check declaration supported by the
  selected ruleset;
- ordered effect operations for damage, healing, temporary vitality, modifier
  application by id, forced movement, resource change, and reaction hooks;
- action resource costs and optional movement allowance; and
- reaction selectors (`declaredTargets`, `actorAllies`, `targetAllies`, or
  `allOtherParticipants`) and selector-bound response options.

A v3 modifier owns its stable id, label, summary, tenure, stacking group and
policy, duration policy, and stat adjustments. An action references its ability
and applied modifiers across the complete exact dependency set. Missing
references, incompatible checks, malformed reaction hooks, duplicate resource
costs, and definition collisions fail before persistence.

Version 4 preserves the exact v1-v3 readers and adds dependency-closed scenario
composition. Its required catalogs add reusable classes/archetypes, stat
definitions, items/loadouts, and scenarios. A scenario declares its grid,
participant instances, archetype inputs, initial vitality, stats, defenses,
resource pools (including an explicit initial amount), inventory/equipped
loadout, ability and action grants, visibility inputs, selected runtime action,
and manual or exact registered automatic control policy. One authored action
definition may be granted to multiple participants under distinct scenario-
local runtime ids; Rust resolves all references and derives executable targets,
grants, resources, and state.

V4 import materializes every root-owned scenario before persistence. Missing or
collided references, duplicate runtime action ids, invalid selected actions,
out-of-bounds/overlapping/blocked placement, invalid vitality or resource
amounts, unsupported provider capabilities, and unknown automation policies
fail closed. An accepted scenario retains an exact composition receipt with the
pack set, scenario, archetype inputs, loadout items, action definition/runtime
mapping, and control policy. That receipt is projected in live snapshots and
finalized replay review and is reconstructed from the original exact pack set
on recovery.

The current executable effect profile is deliberately fail-closed. An effect
program must contain exactly one leading damage operation,
followed by at most one healing, temporary-vitality, modifier, and forced-
movement operation in that order, then zero or more resource changes, and at
most one final reaction hook. Resource changes retain their authored order.
Damage-free programs, repeated fixed-slot operations, and operations reordered
outside that executable sequence reject as `unsupportedAuthoredActionEffect`
before persistence. This prevents a material fingerprint change from being
silently ignored or executed in a different order by the current resolver.
Top-level authored `movement` declarations also reject as
`unsupportedAuthoredActionEffect`. The cell-movement resolver does not evaluate
the authored targeting, check, or effect program, so changing any of those
fields must not produce a distinct accepted fingerprint with identical runtime
behavior. Movement remains available through compiled Rust scenario actions
until one Rust execution path can honor the complete authored declaration.

The committed fixtures
`rulebench-process-host/src/fixtures/authored-content-v1.json` and
`authored-content-v2.json` are permanent reader compatibility evidence. The
committed `authored-content-v3.json` fixture exercises the executable authored
schema. `shatterline-foundation-v4.json` adds five reusable archetypes, two
baseline actions, and manual and automatic scenarios that are materialized from
configuration rather than a compiled scenario fixture. All four versions pass
through the same Rust import workspace and durable restart path; none is
converted or validated by TypeScript.

## Rust conversion and diagnostics

`rulebench-protocol` owns the version readers and the single authored-catalog
converter. Adding another definition kind changes that protocol owner and its
generated DTO metadata; the host, store, diff, definition-index, and component
paths remain generic over canonical definition kind and id.

Rust rejects incomplete or duplicate definitions before persistence. Stable
diagnostics include `emptyContentImportField`,
`duplicateContentImportDefinition`, `invalidAuthoredActionDeclaration`,
`invalidAuthoredModifierDeclaration`, `missingAuthoredActionAbility`,
`missingAuthoredActionModifier`, `unsupportedAuthoredActionCheck`,
`unsupportedAuthoredActionTargeting`, `unsupportedAuthoredActionEffect`,
`authoredActionRulesetProviderUnavailable`,
`authoredActionRulesetProviderIncompatible`,
`duplicateAuthoredActionResourceCost`, `invalidAuthoredReactionDeclaration`,
`invalidAuthoredScenarioDeclaration`, `invalidAuthoredScenarioInitialState`,
`unsupportedAuthoredScenarioAutomationPolicy`,
`contentImportLimitExceeded`, and `unsupportedAuthoredContentVersion`. Closed
enum decoding and runtime-only action fields such as `actorId` are rejected as
`invalidAuthoredContentPayload`.
Exact dependency references still require id, version, fingerprint algorithm,
and fingerprint value. The default canonical importer limits each catalog to
10,000 definitions, the complete pack to 50,000 definitions, dependencies to 64,
each inspected string to 16 KiB, each action to 64 effects, each reaction hook
to four eligible selectors, and each hook to 16 options.

Rulesets, entities, abilities, modifiers, and actions are canonicalized and
fingerprinted in Rust. Catalog order, resource-cost order, reaction-option
order, reaction selector order, and movement terrain-tag order do not affect a
v3 receipt. Authored effect order is executable sequence and therefore remains
material. V3 receipts use `fnv1a64.rulebench-content-pack.v1`; v1 and v2
receipts retain `fnv1a64.rulebench-content-pack.v0`, so adding v3 cannot
reinterpret their canonical bytes. Generic definition indexes and structured
pack diffs report all definition kinds without a kind-specific TypeScript
branch.

V4 canonicalizes classes, stats, items, scenarios, participant sets, grants,
placements, resource initial amounts, and control provenance and uses
`fnv1a64.rulebench-content-pack.v2`. V1-v3 canonical bytes and fingerprint
algorithms are unchanged.

`POST /api/rulebench/v1/content/validate` accepts
`{ "authoredPayload": "..." }` and runs the same Rust decode, canonicalization,
dependency resolution, semantic validation, and receipt generation as import.
An accepted dry-run returns the canonical fingerprint with no import outcome;
it does not write pack storage, activation state, or audit history. The live and
fake TypeScript transports expose this operation as `validateContent`, but the
generated DTOs are only projections of the Rust-owned wire contract.

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
or capability. Import and dry-run validation select the exact compiled provider
by ruleset id and version, require its module configuration to match, and reject
unsupported check, targeting, or effect capability identities before
persistence. Session binding repeats compatibility checks against the selected
scenario and exact pack ruleset. The selected action, ability, and every applied
modifier must each be owned by a pack whose exact provider provenance matches
the scenario provider; a dependency definition is never relabeled to the root
pack's provider. Binding derives living, non-self, team-legal, in-range targets
and intersects required visibility with the selected actor's existing Rust
scenario visibility snapshot. Combatant targeting applies range from actor to
target. Area targeting instead includes a combatant when at least one
registered grid cell is within the action's actor-to-center maximum range and
the combatant is within the authored Manhattan-burst radius of that center,
matching runtime area execution. An empty legal or required-visible set rejects
before session creation.

## Certified executable boundaries

The shipped product binds one exact active v3 root, action id, scenario, and
actor through Rust. Rust re-imports the exact pack set, resolves the action,
ability, and modifiers, derives legal and visible targets, expands reaction
selectors, creates a session-local ability grant, and validates the composed
scenario before creating a session. The binding receipt records the exact
pack, set, action-definition, ability, actor, scenario, grant, and vocabulary
identities in live explanation, finalized replay, and replay-verified active
session recovery.

The v4 path selects an active authored scenario instead of injecting one action
into a compiled scenario. Rust materializes all scenario action grants together,
creates the session through the same combat API used by built-in scenarios, and
retains the composition receipt through checkpoints, forks, finalized replay,
and verification. The Angular setup surface only selects an exact active pack,
scenario, participant order, and projected control configuration; it does not
compute grants, targets, legality, derived state, or automation decisions.

`content.authored-action@1` in the executable capability manifest means only
this closed v3 product boundary. It covers the target/check/effect declarations
accepted by the selected compiled provider, the ordered executable effect
profile documented above, exact actor resource-pool checks at binding, and the
template/clone/JSON validation/import/activation workflow. The capability does
not widen provider support: each check, targeting operation, and effect must
still appear in the exact selected provider catalog. Top-level authored
movement remains rejected until one Rust resolver executes its complete
targeting, check, and effect program.

## Evolution and migration

- Readers for versions 1, 2, 3, and 4 are permanent compatibility surfaces. Version 1
  is lifted into the current in-memory DTO with an empty ability catalog only
  after its exact v1 shape has decoded; version 2 similarly receives empty
  modifier and action catalogs only after exact v2 decoding.
- A semantic or wire-vocabulary change requires a new `formatVersion` and a
  dedicated Rust reader and, when canonical bytes change, a new fingerprint
  algorithm id. Never reinterpret v1, v2, v3, or v4 with guessed defaults.
- Storage receipt or activation-index changes likewise require an explicit
  reader or migration. Unsupported records are quarantined, not guessed.
- A future migration must decode the old payload, convert it through the owned
  catalog converter, run the current Rust importer, compare canonical receipts,
  and commit through the repository replacement transaction. Failure preserves
  the last known-good payload and activation.

## Non-claims

Version 4 is a closed Shatterline scenario-composition slice, not a general
character builder, progression system, inventory simulation, scripting system,
or provider implementation surface. Its scenario-bound ids, initial state, and
visibility are authored inputs, but target legality, derived stats, grants,
runtime resources, automation choice, and mutation remain Rust authority.
Neither version adds arbitrary scripts, plugins, callbacks, or mutable
TypeScript authority. Support is guaranteed only for the closed v3/v4
vocabulary and exact compiled-provider capabilities described above. The
filesystem adapter remains trusted-local and
single-writer. It does not claim multi-process locking, authenticated upload,
network-filesystem atomicity, power-loss durability, or durable in-progress
policy experiments.
