import { describe, expect, it } from 'vitest';

import { isRulesetWorkspaceDeclaration } from './content-authoring.js';

describe('ruleset workspace declaration boundary', () => {
  it('accepts immutable package/composition data and rejects callbacks', () => {
    expect(
      isRulesetWorkspaceDeclaration(
        Object.freeze({
          composition: Object.freeze({}),
          packages: Object.freeze([
            Object.freeze({
              manifest: Object.freeze({}),
              sourceFingerprint: 'fnv1a64:0000000000000000',
            }),
          ]),
        }),
      ),
    ).toBe(true);
    expect(
      isRulesetWorkspaceDeclaration({ composition: {}, packages: [] }),
    ).toBe(false);
    expect(
      isRulesetWorkspaceDeclaration(() => ({ composition: {}, packages: [] })),
    ).toBe(false);
  });
});
