import {
  composeRuleset,
  rulesetPackageRequest,
  rulesetPackageSource,
} from '@asha-rpg/authoring';
import type {
  RulesetCompositionManifest,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

import { primitivesPackage } from '../../foundations/d20/ruleset-package.js';
import { emberSkirmishPackage } from './packages/ember-skirmish.js';

export const ruleset: {
  readonly composition: RulesetCompositionManifest;
  readonly packages: readonly RulesetPackageSource[];
} = Object.freeze({
  composition: composeRuleset({
    identity: { id: 'rulebench.ember-skirmish.demo', version: '1.0.0' },
    language: { id: 'asha-rpg', version: '^1.0.0' },
    base: rulesetPackageRequest({
      id: 'rulebench.ember-skirmish',
      version: '1.0.0',
    }),
    add: [],
    overlays: [],
    configure: {},
  }),
  packages: Object.freeze([
    rulesetPackageSource(emberSkirmishPackage),
    rulesetPackageSource(primitivesPackage),
  ]),
});
