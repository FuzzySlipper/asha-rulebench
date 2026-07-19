# Explicit Ruleset Workspaces

Rulebench accepts one explicit TypeScript entrypoint per compile request:

| Field           | Meaning                                                        |
| --------------- | -------------------------------------------------------------- |
| `workspaceRoot` | Directory against which the root module is resolved            |
| `packageRoots`  | Explicit source roots that the selected module graph may enter |
| `module`        | One TypeScript module relative to `workspaceRoot`              |
| `declaration`   | One named export from that module                              |

The declaration is immutable data with exactly the Asha RPG composition and
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

A downstream game repository installs compatible `@asha-rpg/authoring` and
`@asha-rpg/ir` packages, authors this module in its own repository, and enters
that repository path and export in Rulebench. It does not copy game content
into Rulebench and does not register content globally. Monorepos list every
allowed local source root in `packageRoots`; relative imports that escape those
roots are rejected.

For example, a downstream checkout at `/home/dev/my-game` could select:

```text
workspaceRoot: /home/dev/my-game
packageRoots: packages/rules, packages/content
module: packages/rules/src/ruleset.ts
declaration: ruleset
```

The trusted authoring subprocess creates a TypeScript program from only the
selected module. TypeScript follows its explicit imports, but Rulebench does
not scan directories, enumerate exports, or use import-side-effect
registration. Only the named exported declaration is passed to Asha RPG's
package resolver and materializer. The loader adds the selected
module/declaration provenance to package source fingerprints; moving the root
module changes source identity while unchanged materialized definitions retain
their semantic and presentation identity.

Build, missing-export, evaluation, package-resolution, graph, compatibility,
materialization, normalization, and Rust artifact diagnostics all return
through the same workspace response. Failure at any stage reads the current
Rust workspace state and adds diagnostics; it does not replace the compiled
candidate, active artifact, authority session, checkpoint, or replay archive.

The directories under `examples/rulesets` are ordinary explicit workspaces for
tests and demonstrations. Rulebench starts with no entrypoint selected. The
gateway has no example IDs, option registry, fallback, or switch; entering an
example location exercises the same loader as any downstream repository.
