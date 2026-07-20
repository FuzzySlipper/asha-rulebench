# Canonical Ruleset Roots

Rulebench accepts one path per compile request: `rulesetRoot`. The selected
directory is a complete ruleset root with a fixed entry contract:

```text
<content-repository>/
  foundations/
    d20/
      ruleset-package.ts
  rulesets/
    field-manual/
      ruleset.ts
      packages/
        actions.ts
        overlays.ts
    ember-skirmish/
      ruleset.ts
      packages/
        actions.ts
```

Every ruleset root is a direct child of `rulesets/`. Rulebench always loads
`ruleset.ts` and always reads its named `ruleset` export. The UI and generated
compile DTO therefore ask for only a root such as:

```text
/home/dev/my-rules/rulesets/field-manual
```

Relative paths such as `examples/rulesets/field-manual` are resolved from the
Rulebench checkout. Absolute paths allow a separately cloned content repository
to use the same contract without copying its content into Rulebench.

## Root declaration

The fixed export is immutable data with exactly the Asha RPG composition and
package-source surfaces needed by `prepareRulesetCompilation`:

```ts
import {
  composeRuleset,
  defineRulesetPackage,
  rulesetPackageSource,
} from '@asha-rpg/authoring';

const gamePackage = defineRulesetPackage({
  // Identity, entrypoint, exported definitions, dependencies, and requirements.
});

export const ruleset = Object.freeze({
  composition: composeRuleset({
    // One explicit base and any explicitly added or overlaid packages.
  }),
  packages: Object.freeze([rulesetPackageSource(gamePackage)]),
});
```

The root owns its complete composition and ruleset-specific packages. Separate
rulesets never import each other's source files. Code that is genuinely shared
across rulesets belongs beneath the repository's conventional `foundations/`
directory and is imported explicitly by each root. Dependencies still appear
in the authored package manifests and composition; the directory convention is
only a source-authorization boundary, not a package registry or resolver.

## Loader boundary

The trusted authoring subprocess creates a TypeScript program from only the
inferred `ruleset.ts` entrypoint. TypeScript follows explicit relative imports
under the selected root and the sibling `foundations/` directory. Imports into
another child of `rulesets/`, arbitrary local directories, unapproved package
names, dynamic imports, and `require` calls are rejected.

Rulebench does not scan either directory, enumerate exports, discover packages,
or use import-side-effect registration. Only the fixed exported declaration is
passed to Asha RPG's package resolver and materializer. The inferred entrypoint,
authorized roots, and closed source graph contribute to package source
fingerprints.

Build, missing-export, evaluation, package-resolution, graph, compatibility,
materialization, normalization, and Rust artifact diagnostics all return
through the same workspace response. Root-loading and TypeScript-preparation
failures read the current Rust workspace state and add diagnostics without
replacing an already staged candidate. If prepared input reaches Rust but fails
Rust compilation or portable-artifact closure, Rulebench clears the staged
candidate. Neither failure path replaces the active artifact, authority
session, checkpoint, or replay archive.

## Loading and switching

Rulebench starts with no root selected. **Ruleset → Load or switch ruleset…**
opens a friendly configured-ruleset selector plus a custom-root input. Selecting
a configured entry fills the exact root location; loading the candidate and
activating it remain separate user actions. A successful compile also adds that
explicit path to the recent-root section of the same menu. Choosing a recent
root selects and compiles it as a new candidate; activation remains explicit.
Invalid roots are not added to the recent list, and failed recompilation leaves
the active artifact and session intact.

The trusted local server reads `.rulebench/rulesets.json` by default. That file
is ignored by git so machine-specific absolute paths do not leak into product
source. Copy `.rulebench/rulesets.example.json` to start with the repository
examples, then add independent checkouts by friendly label:

```json
{
  "schemaVersion": 1,
  "rulesets": [
    {
      "id": "my-rules",
      "label": "My independent rules",
      "rulesetRoot": "/home/dev/my-rules/rulesets/main"
    }
  ]
}
```

Set `RULEBENCH_RULESET_CONFIG` to use a different configuration file. The list
is read when the local server starts, and invalid shapes fail startup instead
of silently dropping entries. It only names source locations for the human
selector: it cannot select a startup default, activate content, contribute to
artifact identity, or authorize imports. Recent paths and configured paths are
local presentation state, not a gameplay or package catalog.

The directories under `examples/rulesets` and `examples/foundations` use this
same downstream-repository shape. They have no privileged loader path. Field
Manual, its independently rooted 1.1 variant, and Ember Skirmish demonstrate
root isolation, shared-foundation imports, candidate switching, and explicit
activation.

## Encounter setup documents

After activation, **Session → Create encounter…** accepts an explicitly chosen
JSON document with the generated `asha.rpg.encounter.setup@1` shape. The file
is decoded strictly in the browser and its artifact binding is preserved for
Rust to validate; Rulebench does not infer a setup from the ruleset root or
silently substitute the active artifact. A mismatched or otherwise invalid
document leaves the active session unchanged.

The same dialog can author that DTO directly. Participant vitality, stats,
defenses, resources, and turn-bounded modifiers are generic repeatable rows.
Cell traversal, flag, integer, and identifier capabilities are likewise
repeatable and retain their capability identity, version, and optional
definition binding. The random-source selector contains only bindings the
running Rust host reports as supported. Setup diagnostics remain summarized
for inspection and also appear at the matching path-specific control with
`aria-invalid`, `aria-describedby`, and first-error focus.
