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

| Surface                                    | Current use                                | Required future consumer                                      |
| ------------------------------------------ | ------------------------------------------ | ------------------------------------------------------------- |
| Angular app and shell                      | boot the empty product                     | #5953 compiler/activation composition                         |
| scenario-viewer feature                    | render no-active-ruleset state             | #5953 artifact inspection; #5955 runtime workflow             |
| components                                 | render panels and disabled honest controls | #5953 diagnostics/activation; #5955 candidates/readouts       |
| platform ports                             | generic browser boundary                   | #5953 manifest input, diagnostics copy, local selection state |
| theme                                      | render current product                     | #5953 and #5955 product presentation                          |
| focused Playwright harness                 | verify/inspect empty state                 | #5953 compilation journey; #5955 gameplay journey             |
| structural and TypeScript authority guards | protect current boundaries                 | #5953 immutable authoring boundary                            |

## Next authority boundary

#5953 may activate content only after compiling one explicit manifest and its
exported-root closure into a closed Rust-validated artifact. It must introduce
fresh protocol, transport, store, and host surfaces from that contract. It may
not revive deleted catalogs, startup defaults, scenario-defined rulesets, or
raw-IR product evaluation.

## #5953 transition

The retained boundary now has its first fresh consumer. `rulebench.fresh-start`
names one base package and one contributed support package, resolves three
exact lock edges, and closes three action roots plus typed stat, defense,
resource, modifier, and damage support. A loopback trusted-authoring gateway prepares the
selected package graph for each compile request, and Rust derives execution
semantics only from that closed definition graph before emitting and reloading
the artifact. The selectable missing-support graph exposes source diagnostics
while preserving any active artifact. The product still starts inactive.

## #5955 transition

Activation now creates one fresh persistent Asha RPG authority session. Its
three TypeScript-authored actions form a visible sequential workflow over one
Rust-owned state revision: movement makes later candidates legal, costs reduce
a resource later preflight reads, a modifier remains visible in state, d6 and
five-d4 evidence is consumed without a four-draw cap, and a typed reaction
suspends and resumes the same staged command. Rulebench sends intents and
decisions and renders generated DTOs; it does not interpret IR, evaluate costs
or legality, reapply events, or keep a mirrored combat state.
