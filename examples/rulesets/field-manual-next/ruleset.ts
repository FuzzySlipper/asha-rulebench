import {
  actionPatch,
  canonicalJson,
  composeRuleset,
  prepareRulesetCompilation,
  rulesetPackageRequest,
  rulesetPackageSource,
} from '@asha-rpg/authoring';
import type {
  RulesetCompositionManifest,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

import { primitivesPackage } from '../../foundations/d20/ruleset-package.js';
import { createFieldManualPackage } from './packages/field-manual.js';
import { createStormfrontOverlayPackage } from './packages/stormfront-overlays.js';

const version = '1.1.0';
const fieldManual = createFieldManualPackage({
  version,
  arcLashDamageBonus: 2,
});
const basePackages: readonly RulesetPackageSource[] = Object.freeze([
  rulesetPackageSource(fieldManual),
  rulesetPackageSource(primitivesPackage),
]);
const basePrepared = prepareRulesetCompilation({
  composition: composeFieldManual([], version),
  packages: basePackages,
});
if (!basePrepared.ok) {
  throw new Error(
    `field manual derivation failed: ${canonicalJson(basePrepared.diagnostics)}`,
  );
}
const arcDerivation = basePrepared.prepared.derivationProvenance.find(
  (provenance) => provenance.definitionId === 'rulebench.arc-lash-stormfront',
);
if (arcDerivation === undefined) {
  throw new Error(
    'field manual did not materialize rulebench.arc-lash-stormfront',
  );
}
const semanticOverlay = createStormfrontOverlayPackage({
  id: 'rulebench.stormfront-balance',
  version,
  expectedFingerprint: arcDerivation.materializedFingerprint,
  patch: actionPatch.semantic.maximumRange.set(8),
});
const semanticPrepared = prepareRulesetCompilation({
  composition: composeFieldManual(['rulebench.stormfront-balance'], version),
  packages: [...basePackages, rulesetPackageSource(semanticOverlay)],
});
if (!semanticPrepared.ok) {
  throw new Error(
    `field manual semantic overlay failed: ${canonicalJson(semanticPrepared.diagnostics)}`,
  );
}
const semanticOverlayProvenance =
  semanticPrepared.prepared.overlayProvenance[0];
if (semanticOverlayProvenance === undefined) {
  throw new Error('field manual semantic overlay did not emit provenance');
}
const presentationOverlay = createStormfrontOverlayPackage({
  id: 'rulebench.stormfront-presentation',
  version,
  expectedFingerprint: semanticOverlayProvenance.afterFingerprint,
  patch: actionPatch.presentation.label.set('Arc Lash: Stormfront'),
});

export const ruleset: {
  readonly composition: RulesetCompositionManifest;
  readonly packages: readonly RulesetPackageSource[];
} = Object.freeze({
  composition: composeFieldManual(
    ['rulebench.stormfront-balance', 'rulebench.stormfront-presentation'],
    version,
  ),
  packages: Object.freeze([
    ...basePackages,
    rulesetPackageSource(semanticOverlay),
    rulesetPackageSource(presentationOverlay),
  ]),
});

function composeFieldManual(
  overlays: readonly string[],
  fieldManualVersion: string,
): RulesetCompositionManifest {
  return composeRuleset({
    identity: { id: 'rulebench.fresh-start', version: fieldManualVersion },
    language: { id: 'asha-rpg', version: '^1.0.0' },
    base: rulesetPackageRequest({
      id: 'rulebench.field-manual',
      version: fieldManualVersion,
    }),
    add: [],
    overlays: overlays.map((id) =>
      rulesetPackageRequest({ id, version: fieldManualVersion }),
    ),
    configure: {},
  });
}
