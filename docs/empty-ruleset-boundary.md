# Empty Ruleset Boundary

Den task #5952 is a deletion phase, not a migration.

## Deleted surfaces

- the representative TypeScript RPG content and its generated JSON;
- named actions, scenarios, rulesets, item/archetype surrogates, providers, and
  registries;
- Rulebench Rust content, combat, replay, bridge, protocol, codegen, and process
  host crates;
- product-owned discovery, validation, preflight, target legality, costs,
  resolution, state mutation, event application, projections, persistence, and
  replay compatibility;
- generated scenario/session/capability artifacts and old format fixtures;
- transport, domain, store, renderer, dialogs, browser journeys, docs, claims,
  and receipts coupled to the prototype;
- downstream certification expectations, baselines, and artifacts for that
  content.

No compatibility shim or placeholder corpus replaces them.

## Retained surfaces and concrete consumers

| Surface                                    | Current use                                | Required future consumer                                |
| ------------------------------------------ | ------------------------------------------ | ------------------------------------------------------- |
| Angular app and shell                      | boot the empty product                     | #5953 compiler/activation composition                   |
| scenario-viewer feature                    | render no-active-ruleset state             | #5953 artifact inspection; #5955 runtime workflow       |
| components                                 | render panels and disabled honest controls | #5953 diagnostics/activation; #5955 candidates/readouts |
| platform ports                             | generic browser boundary                   | root input, diagnostics copy, recent-root state         |
| theme                                      | render current product                     | #5953 and #5955 product presentation                    |
| focused Playwright harness                 | verify/inspect empty state                 | #5953 compilation journey; #5955 gameplay journey       |
| structural and TypeScript authority guards | protect current boundaries                 | #5953 immutable authoring boundary                      |

## Next authority boundary

#5953 may activate content only after compiling one explicit root declaration and its
exported-root closure into a closed Rust-validated artifact. It must introduce
fresh protocol, transport, store, and host surfaces from that contract. It may
not revive deleted catalogs, startup defaults, scenario-defined rulesets, or
raw-IR product evaluation.

## #5953 transition

The retained boundary now has its first fresh consumer. `rulebench.fresh-start`
names one base package, one contributed support/mixin package, and two ordered
overlay packages. It resolves seven exact lock edges and closes four action
roots plus typed stat, defense,
resource, modifier, and damage support. A loopback trusted-authoring gateway
builds the inferred `ruleset.ts#ruleset` entrypoint under one explicitly selected
canonical root for each compile request,
then Asha RPG prepares only its exported package graph. Rust derives execution
semantics only from that closed definition graph before emitting and reloading
the artifact. Invalid layout, escaped-source, and invalid-build roots expose
source diagnostics while preserving any active artifact and session. No
product source ID catalog selects content. The top menu switches only among
successfully compiled explicit paths, and the product still starts inactive.

## #5955 transition

Activation now creates one fresh persistent Asha RPG authority session. Its
four TypeScript-authored actions, including one materialized derivation, form a
visible sequential workflow over one
Rust-owned state revision: movement makes later candidates legal, costs reduce
a resource later preflight reads, a modifier remains visible in state, d6 and
five-d4 evidence is consumed without a four-draw cap, and a typed reaction
suspends and resumes the same staged command. Rulebench sends intents and
decisions and renders generated DTOs; it does not interpret IR, evaluate costs
or legality, reapply events, or keep a mirrored combat state.

## #5957 transition

The same explicit graph now materializes Arc Lash: Stormfront from one primary
base, two ordered non-commutative typed mixins, and a local presentation patch.
The root composition then applies one fingerprint-pinned semantic overlay and
one presentation-only overlay. Asha RPG emits fully materialized definitions
and typed source-to-effective-value provenance; Rulebench only maps that
generated host summary into the inspection UI.

The field-manual 1.1 candidate changes Arc Lash's materialized damage formula.
With 1.0 still active, the host compares accepted artifact structures and the
UI identifies both the directly changed action and Arc Lash: Stormfront as a
changed derived descendant, including the exact semantic field transition.
The report remains pre-activation and does not define migration policy.

## #5956 transition

The artifact-bound Asha RPG session now emits one versioned portable checkpoint
and typed replay entries for every submitted intent and reaction. Rulebench
stores those public records in the active slot and exposes their exact package,
lock, artifact, definition-fingerprint, schema, phase, random-evidence, event,
revision, and state-hash identity for inspection. Checkpoint restore and replay
are invoked through Asha RPG and replace the session only after complete Rust
validation. Rulebench does not execute TypeScript, float package versions,
reapply events, or own a parallel state path. Process-restart storage,
activation migration, and exhaustive compatibility evidence remain downstream
or future work.

## #6014 interaction-first transition

The active-session surface is now organized for direct play rather than proof
inspection. A DOM combat grid, participants, current actor and revision,
authority action choices, highlighted legal targets, reaction choices, and the
combat log form the primary keyboard-operable loop. Ruleset loading,
artifact/provenance inspection, checkpoints, and replay remain reachable from
the top menu as secondary dialogs.

Gameplay requests do not carry browser-authored random values. The Rust host
passes the selected host source into Asha RPG's public automatic command API,
which requests the exact count and sides for each executed branch and records
the terminal command with its consumed evidence. Normal sessions use system
entropy. The deterministic browser journey may configure a host-side roll
tape; it is not a user control or TypeScript authority path.

Artifact activation and encounter creation are separate operations. The setup
dialog authors the generated setup DTO directly; Rust validates its artifact,
board/terrain, participant definitions and capabilities, initiative, counters,
and random binding before atomically installing a fresh session. The returned
encounter view, rather than inferred frontend state, owns board extent,
participants, current turn, legal action options, log, and outcome.
