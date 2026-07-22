# Explicit PlayBundle Source Sets

Rulebench composes a PlayBundle from an explicit, versioned source set. A source
set declares exactly one Ruleset entry, one or more content/PlayBundle entries,
optional scenario entries, and every filesystem root the TypeScript graph may
read. A one-root Ruleset uses the same contract with one entry.

A production repository can contain several unrelated roots without
intermixing their files:

```text
<content-repository>/
  shared-rules/                    # optional, explicitly imported foundations
    d20/
  rulesets/
    d20-fantasy/
      src/
        index.ts                   # canonical public entry
        ruleset.ts                 # semantic Ruleset
    another-game/
      src/index.ts
  content-packs/
    starter/
      src/
        index.ts                   # canonical public entry
        content-pack.ts
        actions.ts
        profiles.ts
  play-bundles/
    starter.ts
  scenarios/                       # optional setup-only documents/helpers
    starter-skirmish.ts
```

Each entry names its own source root and module. The d20 repository therefore
declares peer roots such as:

```text
/home/dev/asha-d20-fantasy/rulesets/d20-fantasy
/home/dev/asha-d20-fantasy/content-packs/starter
/home/dev/asha-d20-fantasy/play-bundles
/home/dev/asha-d20-fantasy/scenarios
```

Unrelated Rulesets do not import each other's files. Truly shared semantic
foundations may live in an explicitly allowed root and are ordinary explicit
imports; they are not a registry and Rulebench does not scan them.

## Source-set contract

```json
{
  "schemaVersion": 1,
  "allowedRoots": ["/repos/d20-rules", "/repos/my-content"],
  "entries": [
    {
      "id": "rules",
      "label": "d20 rules",
      "sourceRoot": "/repos/d20-rules",
      "module": "src/index.ts",
      "exportKinds": ["ruleset"]
    },
    {
      "id": "content",
      "label": "My content",
      "sourceRoot": "/repos/my-content",
      "module": "src/index.ts",
      "exportKinds": ["contentPack", "playBundle", "scenarioTemplate"]
    }
  ]
}
```

Each `sourceRoot` must be inside `allowedRoots`; every authored file reached by
the combined TypeScript graph must also remain inside those roots. Entry IDs,
module paths, and exported kinds are declarations, not discovery hints.

## Public entry modules

Each peer root exports only the authoring kind it owns. For example:

```ts
// rulesets/d20-fantasy/src/index.ts
export { myRuleset } from './ruleset.js';

// content-packs/starter/src/index.ts
export { starterContentSource } from './content-pack.js';

// play-bundles/starter.ts
export const starterPlayBundle = composePlayBundle(/* ... */);

// scenarios/starter-skirmish.ts
export const starterScenario = defineScenarioTemplate(/* ... */);
```

The complete module graph may expose other authoring helpers, but each entry
may export only the authoring kinds it declares. The loader selects structurally branded `Ruleset`, `ContentPackSource`,
`PlayBundleManifest`, and `ScenarioTemplate` values. It rejects duplicate
identities and escaped or unapproved imports. There is no required aggregate
export, magic filename beyond `src/index.ts`, side-effect registration,
directory enumeration, or Rulebench-owned content catalog.

Relative helper imports may use either explicit emitted extensions such as
`./ruleset.js` or ordinary extensionless TypeScript specifiers such as
`./ruleset`. Rulebench resolves the declared graph during compilation and emits
exact relative output-file specifiers before Node evaluates it; source package
module metadata is not part of the runtime contract.

Inspecting a source set returns:

- the one semantic Ruleset;
- all exported Content Packs and their explicit requirements;
- all declared PlayBundles and compatibility diagnostics;
- all setup-only Scenario templates and their declared PlayBundle binding.

Compiling additionally requires an explicit list of Content Pack IDs. That
selection must match exactly one declared PlayBundle. The loader then calls
Asha RPG's `preparePlayBundle`; Rust compiles and reloads the closed result.

## Local source configuration

The trusted local server reads `.rulebench/source-sets.json` by default. The file
is ignored by git so machine paths stay local:

```json
{
  "schemaVersion": 2,
  "sourceSets": [
    {
      "id": "d20-fantasy",
      "label": "d20 Fantasy",
      "sourceSet": {
        "schemaVersion": 1,
        "allowedRoots": [
          "/home/dev/asha-d20-fantasy/rulesets/d20-fantasy",
          "/home/dev/asha-d20-fantasy/content-packs/starter",
          "/home/dev/asha-d20-fantasy/play-bundles",
          "/home/dev/asha-d20-fantasy/scenarios"
        ],
        "entries": [
          {
            "id": "d20-fantasy-rules",
            "label": "d20 Fantasy Ruleset",
            "sourceRoot": "/home/dev/asha-d20-fantasy/rulesets/d20-fantasy",
            "module": "src/index.ts",
            "exportKinds": ["ruleset"]
          },
          {
            "id": "d20-fantasy-starter-content",
            "label": "d20 Fantasy Starter Content",
            "sourceRoot": "/home/dev/asha-d20-fantasy/content-packs/starter",
            "module": "src/index.ts",
            "exportKinds": ["contentPack"]
          },
          {
            "id": "d20-fantasy-starter-play",
            "label": "d20 Fantasy Starter PlayBundle",
            "sourceRoot": "/home/dev/asha-d20-fantasy/play-bundles",
            "module": "starter.ts",
            "exportKinds": ["playBundle"]
          },
          {
            "id": "d20-fantasy-starter-scenarios",
            "label": "d20 Fantasy Starter Scenarios",
            "sourceRoot": "/home/dev/asha-d20-fantasy/scenarios",
            "module": "starter-skirmish.ts",
            "exportKinds": ["scenarioTemplate"]
          }
        ]
      }
    }
  ]
}
```

`RULEBENCH_SOURCE_SET_CONFIG` can name another file. Configuration entries are
only friendly source locations. They cannot preselect Content Packs, compile,
activate, or contribute to artifact identity. Their explicit `allowedRoots`
authorize local imports. Custom Ruleset and independent-content root inputs
remain available for ad hoc checkouts.

## Product lifecycle

**Play -> Choose Ruleset and Content Packs...** inspects the selected source set as a
separate step. The user then selects Content Packs, reviews whether a declared
PlayBundle is compatible, compiles it, and explicitly activates the candidate.
Successfully inspected paths may appear as recent locations, but selecting one
never activates it.

Source, graph, compatibility, materialization, normalization, Rust compilation,
and artifact-closure diagnostics use the same product response. A failed
inspection or compile does not replace the active PlayBundle or Session.

After activation, **Session -> Create Scenario...** accepts or authors a strict
`asha.rpg.scenario@1` setup document bound to the exact PlayBundle artifact.
Scenario data contains the board, participants, capabilities, initiative, and
random-source binding. It does not contain a scripted action order, target
choices, reactions, requested roll results, expected events, or winner. Those
decisions happen interactively against the Rust-owned Session. A content
repository may also publish participant profiles in ordinary authored semantic
data; Rulebench presents those public defaults as setup conveniences while Rust
continues to validate the resulting Scenario.
