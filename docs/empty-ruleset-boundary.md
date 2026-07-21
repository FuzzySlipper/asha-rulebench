# Product Content Boundary

Rulebench deliberately owns no playable Ruleset or example content. Its former
Field Manual, Ember Skirmish, shared-foundation, upgrade, escaped-import, and
invalid-build example trees were removed rather than mechanically ported to the
new content contract.

The retained `test-fixtures/rulesets` roots are narrow loader fixtures. They do
not appear in a normal local configuration and make no gameplay claim. Complete
content belongs in independent repositories and enters through the same public
source-set contract available to every downstream author.

## Retained product surfaces

| Surface            | Product responsibility                                                 |
| ------------------ | ---------------------------------------------------------------------- |
| Angular workspace  | interactive play and secondary inspection dialogs                      |
| Content loader     | build explicitly selected source graphs and discover immutable exports |
| Rust play host     | compile/activate PlayBundles and own Scenario/Session lifecycle        |
| Generated protocol | carry product DTOs without a parallel authority model                  |
| Focused fixtures   | reject boundary drift without becoming demo content                    |

## Current lifecycle

The product starts with **No PlayBundle active**. One explicit human flow moves
through distinct concepts:

```text
Ruleset + Content Packs -> PlayBundle -> Scenario -> Session
```

Compilation cannot begin until the selected Content Pack set matches a declared
compatible PlayBundle. Activation remains separate and atomic. Scenario
validation cannot replace an existing Session on failure. Interactive commands,
targets, reactions, random draws, events, outcomes, checkpoints, and replay all
remain Rust-owned.

Rulebench does not claim bundled content, startup defaults, scenario-authored
rules, persistence across process restart, migration policy, multiplayer, AI,
or exhaustive cross-product certification.
