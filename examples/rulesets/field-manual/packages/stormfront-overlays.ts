import {
  defineRulesetOverlay,
  defineRulesetPackage,
  definitionReference,
  rulesetDependency,
} from '@asha-rpg/authoring';
import type { RulesetPatch } from '@asha-rpg/authoring';

const sourceModule = 'rulesets/field-manual/packages/stormfront-overlays.ts';

export function createStormfrontOverlayPackage(options: {
  readonly id: string;
  readonly version: string;
  readonly expectedFingerprint: string;
  readonly patch: RulesetPatch;
}) {
  return defineRulesetPackage({
    identity: { id: options.id, version: options.version },
    entry: {
      module: sourceModule,
      declaration: 'createStormfrontOverlayPackage',
    },
    dependencies: [
      rulesetDependency({
        id: 'rulebench.field-manual',
        version: options.version,
        importAs: 'fieldManual',
      }),
    ],
    definitions: [],
    relationships: [
      defineRulesetOverlay({
        definitionId: `${options.id}.patch`,
        target: definitionReference({
          importAs: 'fieldManual',
          definitionId: 'rulebench.arc-lash-stormfront',
        }),
        targetPackage: {
          id: 'rulebench.field-manual',
          version: options.version,
        },
        expectedFingerprint: options.expectedFingerprint,
        patch: options.patch,
      }),
    ],
  });
}
