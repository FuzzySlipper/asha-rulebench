import { describe, expect, it } from 'vitest';

import {
  authorityTargetIds,
  toggleAuthorityOption,
  type AuthorityOptionSelection,
} from './index.js';

describe('authority action option selection', () => {
  it('retains multiple participant targets up to the authority maximum', () => {
    const first = toggleAuthorityOption(
      [],
      { kind: 'participant', id: 'raider-one' },
      2,
    );
    const second = toggleAuthorityOption(
      first,
      { kind: 'participant', id: 'raider-two' },
      2,
    );

    expect(authorityTargetIds(second)).toEqual(['raider-one', 'raider-two']);
    expect(
      toggleAuthorityOption(
        second,
        { kind: 'participant', id: 'raider-three' },
        2,
      ),
    ).toEqual(second);
  });

  it('submits participant, cell, and area options through one typed selection path', () => {
    const selections: readonly AuthorityOptionSelection[] = [
      { kind: 'participant', id: 'raider' },
      { kind: 'cell', id: 'bridge-cell' },
      { kind: 'area', id: 'blast-zone' },
    ];

    expect(authorityTargetIds(selections)).toEqual([
      'raider',
      'bridge-cell',
      'blast-zone',
    ]);
  });
});
