import { describe, expect, it } from 'vitest';

import {
  isContentPackSource,
  isPlayBundleManifest,
  isRuleset,
} from './content-authoring.js';

const ruleset = Object.freeze({
  schema: Object.freeze({ identity: 'asha.rpg.ruleset', major: 1 }),
  identity: Object.freeze({ id: 'rules', version: '1.0.0' }),
  language: Object.freeze({ id: 'asha-rpg', version: '1.0.0' }),
  models: Object.freeze({}),
  provides: Object.freeze({}),
});

describe('Ruleset root authoring discovery', () => {
  it('recognizes only immutable values at each explicit boundary', () => {
    const source = Object.freeze({
      manifest: Object.freeze({
        identity: Object.freeze({ id: 'content', version: '1.0.0' }),
        definitions: Object.freeze([]),
        exports: Object.freeze([]),
      }),
      sourceFingerprint: 'fnv1a64:0000000000000000',
    });
    const bundle = Object.freeze({
      identity: Object.freeze({ id: 'play', version: '1.0.0' }),
      ruleset,
      base: Object.freeze({ id: 'content', version: '1.0.0' }),
      add: Object.freeze([]),
      overlays: Object.freeze([]),
      configure: Object.freeze({}),
    });

    expect(isRuleset(ruleset)).toBe(true);
    expect(isContentPackSource(source)).toBe(true);
    expect(isPlayBundleManifest(bundle)).toBe(true);
    expect(isRuleset({ ...ruleset })).toBe(false);
    expect(isPlayBundleManifest(() => bundle)).toBe(false);
  });
});
