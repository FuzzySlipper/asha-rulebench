# ASHA Rulebench

ASHA Rulebench is the authoring and inspection product for compiled Asha RPG
rulesets. It is currently at the deliberate empty boundary established by Den
task #5952:

- no prototype content is bundled;
- no ruleset is selected by default;
- no scenario, import, filename, or startup behavior can construct a ruleset;
- no Rulebench-owned combat, replay, or semantic authority remains;
- the UI starts and clearly reports **No compiled ruleset active**.

Future content enters only through the explicit manifest/compiler boundary
owned by #5953. That path will compile one explicit TypeScript package or
composition manifest into a dependency-closed, Rust-validated artifact before
Rulebench can activate it. Source directories and import side effects never
determine runtime meaning.

## Retained product surfaces

- `apps/app`: Angular bootstrap.
- `apps/app-e2e`: focused empty-state browser and managed live evidence.
- `libs/components`: generic workbench panels, menus, and dialogs. #5953 uses
  these for compilation/activation inspection and #5955 for runtime controls.
- `libs/platform`: browser ports. #5953 uses file input, storage, clipboard,
  timing, and document effects without bypassing product layers.
- `libs/scenario-viewer`: the current empty workspace feature. #5953 replaces
  the empty artifact state with compiler/activation readouts; #5955 consumes
  the active artifact in a visible workflow.
- `libs/shell`: routes only.
- `libs/theme`: product tokens.

The deleted Rust workspace, generated protocol, transport/store/domain layers,
content authoring corpus, replay persistence, and proof artifacts were coupled
to the retired prototype. #5953 must introduce fresh owner boundaries from the
compiled-artifact contract rather than restoring those paths.

## Commands

```bash
pnpm run verify
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile browser
pnpm run verify:change -- --profile docs
```

For managed visual evidence:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

Rulebench's blocking gate is focused product validation. Exhaustive
cross-repository certification remains downstream, but the old prototype
expectations have been retired rather than preserved there.

See [docs/empty-ruleset-boundary.md](docs/empty-ruleset-boundary.md) for the
deletion and retention inventory.
