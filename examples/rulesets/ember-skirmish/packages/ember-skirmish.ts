import {
  action,
  actionId,
  applyModifier,
  constant,
  defineActionDefinition,
  defineRulesetPackage,
  hostile,
  noRoll,
  onCheck,
  refresh,
  rulesetDependency,
  stackingGroup,
  turns,
} from '@asha-rpg/authoring';

import { primitivesCatalog } from '../../../foundations/d20/ruleset-package.js';

const sourceModule = 'rulesets/ember-skirmish/packages/ember-skirmish.ts';
const emberGuard = action({
  id: actionId('rulebench.ember-guard'),
  name: 'Ember Guard',
  sourcePath: sourceModule,
  targets: hostile({ range: 4 }),
  check: noRoll(),
  program: onCheck({
    noRoll: applyModifier({
      modifier: primitivesCatalog.references.exposed,
      value: constant(-1),
      duration: turns(1),
      stacking: refresh(stackingGroup('ember-guard-penalty')),
    }),
  }),
});

export const emberSkirmishPackage = defineRulesetPackage({
  identity: { id: 'rulebench.ember-skirmish', version: '1.0.0' },
  entry: { module: sourceModule, declaration: 'emberSkirmishPackage' },
  dependencies: [
    rulesetDependency({
      id: 'rulebench.primitives',
      version: '1.0.0',
      importAs: 'primitives',
    }),
  ],
  definitions: [
    defineActionDefinition({
      kind: 'action',
      id: emberGuard.id,
      visibility: 'public',
      extensionPolicy: 'sealed',
      source: { module: sourceModule, declaration: 'emberGuard' },
      presentation: {
        label: emberGuard.name,
        description:
          'A second independent ruleset sharing only the d20 foundation.',
        tags: ['ember', 'playable'],
      },
      action: emberGuard,
    }),
  ],
});
