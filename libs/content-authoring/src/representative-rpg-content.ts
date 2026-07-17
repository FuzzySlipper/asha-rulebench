import {
  action,
  actionId,
  add,
  ally,
  applyModifier as authorModifier,
  attack,
  changeResource,
  constant,
  damage,
  damageType,
  defenseId,
  defineArchetype,
  defineItem,
  definePackage,
  defineScenario,
  dice,
  forEachTarget,
  half,
  heal,
  hostile,
  modifierId,
  moveEntity,
  noRoll,
  onCheck,
  readStat,
  refresh,
  resourceId,
  savingThrow,
  sequence,
  spend,
  stackingGroup,
  statId,
  turns,
} from "@asha-rpg/authoring";
import type { RpgActionId } from "@asha-rpg/authoring";

export interface RulebenchReactionOrchestration {
  readonly window: "beforeEffect";
  readonly eligibleReactors: "declaredTargets";
  readonly optionId: string;
}

export interface RulebenchActionBindingMetadata {
  readonly actionId: RpgActionId;
  readonly abilityId: string;
  readonly actionText: string;
  readonly effectText: string;
  readonly reaction: RulebenchReactionOrchestration | null;
}

const guard = defenseId("guard");
const resolve = defenseId("resolve");
const power = statId("power");
const focusStat = statId("focus");
const standardAction = resourceId("standard-action");
const focus = resourceId("focus");

const hexingBolt = action({
  id: actionId("hexing_bolt"),
  name: "Hexing Bolt",
  sourcePath: "rulebench/scenarios/hexing-bolt",
  targets: hostile({ range: 10 }),
  check: attack({
    modifier: add(constant(4), readStat("actor", statId("mind"))),
    defense: defenseId("nerve"),
  }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: onCheck({
    hit: sequence(
      damage({
        amount: dice({ count: 1, sides: 8, bonus: 4 }),
        type: damageType("psychic"),
      }),
      authorModifier({
        modifier: modifierId("rattled"),
        value: constant(-2),
        duration: turns(1),
        stacking: refresh(stackingGroup("rattled")),
      }),
    ),
  }),
});

const anchorLash = action({
  id: actionId("action.anchor-lash"),
  name: "Anchor Lash",
  sourcePath: "rulebench/actions/anchor-lash",
  targets: hostile({ range: 3 }),
  check: attack({
    modifier: add(constant(1), readStat("actor", focusStat)),
    defense: guard,
  }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: onCheck({
    hit: damage({
      amount: dice({ count: 1, sides: 6, bonus: 2 }),
      type: damageType("kinetic"),
    }),
  }),
});

const bindingSpark = action({
  id: actionId("action.binding-spark"),
  name: "Binding Spark",
  sourcePath: "rulebench/actions/binding-spark",
  targets: hostile({ range: 5 }),
  check: savingThrow({ difficulty: constant(11), defense: resolve }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: onCheck({
    failed: sequence(
      damage({
        amount: dice({ count: 1, sides: 6, bonus: 1 }),
        type: damageType("arcane"),
      }),
      authorModifier({
        modifier: modifierId("modifier.binding-spark.anchored"),
        value: constant(-2),
        duration: turns(2),
        stacking: refresh(stackingGroup("movement-control")),
      }),
    ),
    saved: damage({
      amount: half(dice({ count: 1, sides: 6, bonus: 1 })),
      type: damageType("arcane"),
    }),
  }),
});

const rallyingMend = action({
  id: actionId("action.rallying-mend"),
  name: "Rallying Mend",
  sourcePath: "rulebench/archetypes/anchor/rallying-mend",
  targets: ally({ range: 4 }),
  check: noRoll(),
  program: onCheck({
    noRoll: sequence(
      heal({ amount: dice({ count: 1, sides: 6, bonus: 2 }) }),
      changeResource({ subject: "actor", resource: focus, delta: constant(1) }),
    ),
  }),
});

const shatterlineBurst = action({
  id: actionId("action.shatterline-burst"),
  name: "Shatterline Burst",
  sourcePath: "rulebench/items/shatterline-focus",
  targets: hostile({ range: 6, maximum: 3 }),
  check: savingThrow({ difficulty: constant(13), defense: resolve }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: forEachTarget(
    3,
    onCheck({
      failed: damage({
        amount: dice({ count: 2, sides: 4 }),
        type: damageType("force"),
      }),
      saved: damage({
        amount: half(dice({ count: 2, sides: 4 })),
        type: damageType("force"),
      }),
    }),
  ),
});

const tacticalShift = action({
  id: actionId("action.tactical-shift"),
  name: "Tactical Shift",
  sourcePath: "rulebench/scenarios/shatterline/tactical-shift",
  targets: ally({ range: 4 }),
  check: noRoll(),
  program: onCheck({
    noRoll: moveEntity({
      subject: "target",
      deltaX: constant(2),
      deltaY: constant(0),
      maximumDistance: 2,
      provokes: false,
    }),
  }),
});

const interceptingStrike = action({
  id: actionId("action.intercepting-strike"),
  name: "Intercepting Strike",
  sourcePath: "rulebench/scenarios/shatterline/intercepting-strike",
  targets: hostile({ range: 2 }),
  check: attack({ modifier: readStat("actor", power), defense: guard }),
  rollScope: "perTarget",
  program: onCheck({
    hit: damage({
      amount: dice({ count: 1, sides: 8 }),
      type: damageType("kinetic"),
    }),
  }),
});

const bindingGlyph = action({
  id: actionId("action.binding-glyph"),
  name: "Binding Glyph",
  sourcePath: "rulebench/compatibility/binding-glyph",
  targets: hostile({ range: 6 }),
  check: savingThrow({ difficulty: constant(12), defense: defenseId("body") }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: onCheck({
    failed: sequence(
      damage({
        amount: dice({ count: 1, sides: 6, bonus: 4 }),
        type: damageType("arcane"),
      }),
      authorModifier({
        modifier: modifierId("modifier.binding-glyph.anchored"),
        value: constant(-1),
        duration: turns(1),
        stacking: refresh(stackingGroup("binding-glyph-anchor")),
      }),
    ),
  }),
});

const authoredReactionCompatibility = action({
  id: actionId("action.authored-reaction"),
  name: "Authored Reaction Compatibility",
  sourcePath: "rulebench/compatibility/authored-reaction",
  targets: hostile({ range: 10 }),
  check: attack({
    modifier: add(constant(4), readStat("actor", statId("mind"))),
    defense: defenseId("nerve"),
  }),
  rollScope: "perTarget",
  costs: [spend(standardAction, 1)],
  program: onCheck({
    hit: sequence(
      damage({
        amount: dice({ count: 1, sides: 6, bonus: 4 }),
        type: damageType("arcane"),
      }),
      authorModifier({
        modifier: modifierId("modifier.binding-glyph.anchored"),
        value: constant(-1),
        duration: turns(1),
        stacking: refresh(stackingGroup("binding-glyph-anchor")),
      }),
    ),
  }),
});

export const rulebenchAuthoredRpgPackage = definePackage({
  id: "asha-rulebench.representative-rpg",
  version: "1.0.0",
  sources: [
    defineArchetype("archetype.anchor", [
      hexingBolt,
      anchorLash,
      bindingSpark,
      rallyingMend,
    ]),
    defineItem("item.shatterline-focus", [shatterlineBurst]),
    defineScenario("scenario.shatterline-foundation", [
      tacticalShift,
      interceptingStrike,
    ]),
    defineScenario("scenario.rulebench-compatibility", [
      bindingGlyph,
      authoredReactionCompatibility,
    ]),
  ],
});

export const rulebenchActionBindings: readonly RulebenchActionBindingMetadata[] =
  Object.freeze([
    metadata(
      hexingBolt.id,
      "ability.hexing-bolt",
      "Mind vs Nerve at range ten.",
      "Deal psychic damage and apply Rattled on a hit.",
      {
        window: "beforeEffect",
        eligibleReactors: "declaredTargets",
        optionId: "reaction.brace",
      },
    ),
    metadata(
      anchorLash.id,
      "ability.anchor-lash",
      "Lash one hostile within three cells.",
      "Deal kinetic damage on a hit.",
    ),
    metadata(
      bindingSpark.id,
      "ability.binding-spark",
      "Bind one hostile within five cells.",
      "Deal arcane damage and refresh Anchored on a failed save.",
    ),
    metadata(
      rallyingMend.id,
      "ability.rallying-mend",
      "Restore a nearby ally.",
      "Heal the ally and restore one focus to the actor.",
    ),
    metadata(
      shatterlineBurst.id,
      "ability.shatterline-burst",
      "Burst against up to three hostiles.",
      "Each target saves independently for half force damage.",
    ),
    metadata(
      tacticalShift.id,
      "ability.tactical-shift",
      "Reposition a nearby ally.",
      "Move the ally two cells without provoking.",
    ),
    metadata(
      interceptingStrike.id,
      "ability.intercepting-strike",
      "Strike a nearby hostile.",
      "Deal kinetic damage on a hit.",
    ),
    metadata(
      bindingGlyph.id,
      "ability.binding-glyph",
      "Inscribe a binding glyph beneath a visible hostile combatant.",
      "Deal arcane damage and apply Anchored on a failed save.",
    ),
    metadata(
      authoredReactionCompatibility.id,
      "ability.authored-reaction",
      "Strike a visible hostile and allow product reaction orchestration.",
      "Deal arcane damage and apply Anchored on a hit.",
    ),
  ]);

function metadata(
  actionIdValue: RpgActionId,
  abilityId: string,
  actionText: string,
  effectText: string,
  reaction: RulebenchReactionOrchestration | null = null,
): RulebenchActionBindingMetadata {
  return Object.freeze({
    actionId: actionIdValue,
    abilityId,
    actionText,
    effectText,
    reaction,
  });
}
