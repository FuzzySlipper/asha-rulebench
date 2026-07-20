import {
  action,
  actionId,
  actionPatch,
  add,
  applyModifier,
  attack,
  compare,
  constant,
  damage,
  defineActionDefinition,
  defineRulesetPackage,
  definitionReference,
  deriveAction,
  dice,
  hostile,
  moveEntity,
  noRoll,
  onCheck,
  openReaction,
  reactionId,
  reactionOptionId,
  readStat,
  refresh,
  rulesetDependency,
  sequence,
  spend,
  stackingGroup,
  turns,
  when,
} from '@asha-rpg/authoring';
import type { AuthoredAction } from '@asha-rpg/authoring';

import { primitivesCatalog } from '../../../foundations/d20/ruleset-package.js';

const sourceModule = 'rulesets/field-manual/packages/field-manual.ts';

export function createFieldManualPackage(options: {
  readonly version: string;
  readonly arcLashDamageBonus: number;
}) {
  const actions = authoredActions(options.arcLashDamageBonus);
  const definitions = actions.map((authored) =>
    defineActionDefinition({
      kind: 'action',
      id: authored.id,
      visibility: 'public',
      extensionPolicy:
        authored.id === 'rulebench.arc-lash' ? 'derivable' : 'patchable',
      source: { module: sourceModule, declaration: authored.id },
      presentation: {
        label: authored.name,
        description: 'Fresh TypeScript content compiled into Rust authority.',
        tags: ['fresh', 'playable'],
      },
      action: authored,
    }),
  );
  const derivedArcLash = deriveAction({
    id: 'rulebench.arc-lash-stormfront',
    visibility: 'public',
    extensionPolicy: 'patchable',
    source: { module: sourceModule, declaration: 'arcLashStormfront' },
    presentation: { label: 'Arc Lash Stormfront' },
    base: definitionReference({ definitionId: 'rulebench.arc-lash' }),
    mixins: [
      {
        target: definitionReference({
          importAs: 'primitives',
          definitionId: 'rulebench.double-range',
        }),
        parameters: { factor: 2 },
      },
      {
        target: definitionReference({
          importAs: 'primitives',
          definitionId: 'rulebench.extend-range',
        }),
        parameters: { amount: 1 },
      },
    ],
    patch: actionPatch.presentation.description.set(
      'Derived from Arc Lash through two ordered mixins and a local patch.',
    ),
  });
  return defineRulesetPackage({
    identity: { id: 'rulebench.field-manual', version: options.version },
    entry: { module: sourceModule, declaration: 'createFieldManualPackage' },
    dependencies: [
      rulesetDependency({
        id: 'rulebench.primitives',
        version: '1.0.0',
        importAs: 'primitives',
      }),
    ],
    definitions: [...definitions, derivedArcLash.definition],
    relationships: [derivedArcLash.relationship],
  });
}

function authoredActions(
  arcLashDamageBonus: number,
): readonly AuthoredAction[] {
  const { storm, power, guard, focus, exposed } = primitivesCatalog.references;
  return Object.freeze([
    action({
      id: actionId('rulebench.tactical-advance'),
      name: 'Tactical Advance',
      sourcePath: sourceModule,
      targets: hostile({ range: 6 }),
      check: noRoll(),
      program: onCheck({
        noRoll: sequence(
          moveEntity({
            subject: 'actor',
            deltaX: constant(2),
            deltaY: constant(0),
            maximumDistance: 2,
            provokes: false,
          }),
          applyModifier({
            modifier: exposed,
            value: constant(-2),
            duration: turns(2),
            stacking: refresh(stackingGroup('guard-penalty')),
          }),
        ),
      }),
    }),
    action({
      id: actionId('rulebench.arc-lash'),
      name: 'Arc Lash',
      sourcePath: sourceModule,
      targets: hostile({ range: 3 }),
      check: attack({ modifier: readStat('actor', power), defense: guard }),
      rollScope: 'perTarget',
      costs: [spend(focus, 1)],
      program: onCheck({
        hit: when(
          compare(readStat('actor', power), 'greaterThan', constant(0)),
          damage({
            amount: add(
              dice({ count: 2, sides: 6 }),
              constant(arcLashDamageBonus),
            ),
            type: storm,
          }),
          damage({ amount: dice({ count: 1, sides: 6 }), type: storm }),
        ),
      }),
    }),
    action({
      id: actionId('rulebench.wardbreaker-volley'),
      name: 'Wardbreaker Volley',
      sourcePath: sourceModule,
      targets: hostile({ range: 3 }),
      check: noRoll(),
      costs: [spend(focus, 1)],
      program: onCheck({
        noRoll: sequence(
          openReaction({
            id: reactionId('reaction.raise-ward'),
            options: [
              {
                id: reactionOptionId('raise-ward'),
                label: 'Raise ward',
                damageReduction: 3,
              },
            ],
          }),
          damage({ amount: dice({ count: 5, sides: 4 }), type: storm }),
        ),
      }),
    }),
  ]);
}
