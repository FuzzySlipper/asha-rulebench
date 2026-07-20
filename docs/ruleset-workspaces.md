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
through the same workspace response. Failure at any stage reads the current
Rust workspace state and adds diagnostics; it does not replace the compiled
candidate, active artifact, authority session, checkpoint, or replay archive.

## Loading and switching

Rulebench starts with no root selected. **Ruleset → Load ruleset root…** focuses
the single root input. A successful compile adds that explicit path to the
recent-root section of the same menu. Choosing a recent root selects and
compiles it as a new candidate; activation remains a separate user action.
Invalid roots are not added to the recent list, and failed recompilation leaves
the active artifact and session intact. Recent paths are local presentation
state, not a gameplay catalog or startup default.

The directories under `examples/rulesets` and `examples/foundations` use this
same downstream-repository shape. They have no privileged loader path. Field
Manual, its independently rooted 1.1 variant, and Ember Skirmish demonstrate
root isolation, shared-foundation imports, candidate switching, and explicit
activation.
