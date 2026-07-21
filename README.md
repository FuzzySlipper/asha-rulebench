# ASHA Rulebench

ASHA Rulebench is an interactive browser workbench for selecting authored RPG
content, compiling it through Asha RPG, and playing against the Rust authority.
The product boundary is explicit:

```text
Ruleset + selected Content Packs -> compiled PlayBundle -> Scenario -> Session
```

- A **Ruleset** selects semantic models and declares the operations,
  capabilities, values, and numeric domains it provides.
- A **Content Pack** contributes authored definitions and states its Ruleset
  requirements.
- A **PlayBundle** is a declared, compatible composition of one Ruleset and an
  exact Content Pack selection. It is the unit Rust compiles and activates.
- A **Scenario** is setup-only data bound to the active PlayBundle.
- A **Session** is the live Rust-owned state created from an accepted Scenario.

Rulebench bundles no product Ruleset or demo content. The small roots under
`test-fixtures/` exercise the loader contract only. Complete playable content
belongs in independent repositories, such as `asha-d20-fantasy`, and reaches
Rulebench through a configured or explicitly entered source set.

## Loading play content

Rulebench starts with no active PlayBundle. **Play -> Choose Ruleset and Content
Packs...** opens the primary content flow:

1. select a configured source set or enter a Ruleset root plus independent content roots;
2. inspect its exported Ruleset, Content Packs, and declared PlayBundles;
3. explicitly select Content Packs;
4. compile the matching compatible PlayBundle;
5. activate the accepted candidate;
6. create or load a Scenario.

The local server reads `.rulebench/source-sets.json`. It only gives source roots
friendly menu labels; it cannot choose a default, compile, or activate content.
The ignored local file in this checkout points at the separately cloned
`/home/dev/asha-d20-fantasy/rulesets/d20-fantasy` root. See
[Explicit PlayBundle source sets](docs/ruleset-workspaces.md) for the repository
contract and a portable configuration example.

Each inspect or compile request builds the declared source entry modules in one
fresh constrained TypeScript subprocess. It discovers immutable exported
`Ruleset`, `ContentPackSource`, `PlayBundleManifest`, and setup-only
`ScenarioTemplate` values without ambient registration or directory scanning.
Compile sends the exact selected Content Pack IDs through `preparePlayBundle`;
Rust compiles and reloads the closed portable artifact before it can become an
activation candidate. Failure never replaces the active PlayBundle or Session.

After activation, Scenario examples and participant profiles exported by the
selected repository provide editable starting points. They never provide a
command order, targets, reactions, rolls, expected events, or outcome. Named
Ruleset values such as Strength and Armor Class remain visible by identity in
setup and in the live participant state.

## Runtime ownership

TypeScript references and configures Rust behavior. Rust defines and executes
rule logic.

The browser sends generated Scenario, command, reaction, turn-control,
checkpoint, and replay DTOs. Rust owns validation, legality, state mutation,
random requests, accepted events, outcomes, checkpoints, and replay. The
interactive combat grid, participants, current turn, available actions,
targets, reactions, automatic rolls, and log are projections of that authority.

`rulebench-rs/hosts/play-host` is the narrow loopback product host. The Angular
client reaches it through the same-origin local gateway and generated protocol;
there is no TypeScript rules engine or legacy disposable-session path.

## Repository surfaces

- `apps/app` and `apps/app-e2e`: Angular bootstrap and focused browser checks.
- `libs/content-authoring`: structural guards for immutable authored exports.
- `libs/protocol`: generated Rust DTOs and strict decoders.
- `libs/transport`, `libs/store`, `libs/domain`: HTTP, product state, and pure
  view mapping.
- `libs/scenario-viewer`: the interactive play surface and secondary dialogs.
- `libs/components`, `libs/platform`, `libs/shell`, `libs/theme`: shared UI,
  host ports, composition, and tokens.
- `rulebench-rs/hosts/play-host`: compile, activation, Scenario, and Session
  lifecycle host.
- `test-fixtures/rulesets` and `test-fixtures/source-sets`: non-product contract
  fixtures.

## Validation

```bash
pnpm run verify
pnpm run verify:change -- --profile frontend
pnpm run verify:change -- --profile content-authoring
pnpm run verify:change -- --profile rust-owner
pnpm run verify:change -- --profile protocol-generated
pnpm run verify:change -- --profile host-transport
pnpm run verify:change -- --profile browser
pnpm run verify:change -- --profile docs
```

For managed visual evidence:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
```

Use the printed local or LAN URL to conduct the interaction itself. Capture and
inspect desktop and narrow screenshots, record the content revisions and
authority outcomes, and state the non-claims in the Den handoff. Rulebench does
not keep a scripted product journey for this interaction-first campaign.

The focused gate proves the product boundary and one explicit activation flow.
It is not an exhaustive content certification. Persistent libraries and saves,
multiplayer, AI control, migration policy, and compatibility with retired
Rulebench fixture content remain non-claims.
