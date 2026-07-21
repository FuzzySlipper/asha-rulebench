# Canonical Ruleset Roots

Rulebench accepts a path to one complete Ruleset root. A production repository
can contain several unrelated roots without intermixing their files:

```text
<content-repository>/
  shared-rules/                    # optional, explicitly imported foundations
    d20/
  rulesets/
    d20-fantasy/
      src/
        index.ts                   # canonical public entry
        ruleset.ts                 # semantic Ruleset
      content-packs/
        starter/
          src/
            content-pack.ts
            actions.ts
            profiles.ts
      play-bundles/
        starter.ts
      scenarios/                   # optional authored setup documents/helpers
        starter-skirmish.ts
    another-game/
      src/index.ts
      ...
```

The requested root is the directory containing `src/index.ts`, for example:

```text
/home/dev/asha-d20-fantasy/rulesets/d20-fantasy
```

Unrelated Rulesets do not import each other's files. Truly shared semantic
foundations may live in an explicitly named repository directory and are
ordinary explicit imports; they are not a registry and Rulebench does not scan
them.

## Public entry contract

`src/index.ts` exports immutable values authored with `@asha-rpg/authoring`:

```ts
export { myRuleset } from './ruleset.js';
export { starterContentSource } from '../content-packs/starter/src/content-pack.js';
export { starterPlayBundle } from '../play-bundles/starter.js';
export { starterScenario } from '../scenarios/starter-skirmish.js';
```

The complete module graph may expose other authoring helpers, but the loader
selects only structurally branded `Ruleset`, `ContentPackSource`,
`PlayBundleManifest`, and `ScenarioTemplate` values. It rejects duplicate
identities and escaped or unapproved imports. There is no required aggregate
export, magic filename beyond `src/index.ts`, side-effect registration,
directory enumeration, or Rulebench-owned content catalog.

Inspecting a root returns:

- the one semantic Ruleset;
- all exported Content Packs and their explicit requirements;
- all declared PlayBundles and compatibility diagnostics;
- all setup-only Scenario templates and their declared PlayBundle binding.

Compiling additionally requires an explicit list of Content Pack IDs. That
selection must match exactly one declared PlayBundle. The loader then calls
Asha RPG's `preparePlayBundle`; Rust compiles and reloads the closed result.

## Local source configuration

The trusted local server reads `.rulebench/rulesets.json` by default. The file
is ignored by git so machine paths stay local:

```json
{
  "schemaVersion": 1,
  "rulesets": [
    {
      "id": "d20-fantasy",
      "label": "d20 Fantasy",
      "rulesetRoot": "/home/dev/asha-d20-fantasy/rulesets/d20-fantasy"
    }
  ]
}
```

`RULEBENCH_RULESET_CONFIG` can name another file. Configuration entries are
only friendly source locations. They cannot preselect a root, select Content
Packs, compile, activate, contribute to artifact identity, or authorize an
import. A custom-root input remains available for any independent checkout.

## Product lifecycle

**Play -> Choose Ruleset and Content Packs...** inspects the selected root as a
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
