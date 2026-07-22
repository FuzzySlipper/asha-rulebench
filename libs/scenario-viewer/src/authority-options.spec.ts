import { describe, expect, it } from 'vitest';

import {
  authorityReadbackRequiresTargetingReset,
  authorityTargetingReadbackIdentity,
  authorityTargetIds,
  toggleAuthorityOption,
  type AuthorityOptionSelection,
} from './index.js';

const targetingReadback = (
  actorId: string,
  turn: number,
  stateHash: string,
) => ({
  activationRevision: 3,
  gameplay: {
    artifactId: 'artifact-1',
    actorId,
    stateRevision: turn,
    turn: {
      currentActorId: actorId,
      round: 1,
      turn,
    },
    outcome: {
      status: 'inProgress',
      winningTeamIds: [],
    },
    result: null,
    archive: {
      stateHash,
    },
  },
});

describe('authority action option selection', () => {
  it('resets targeting when an independent authority readback changes', () => {
    const initialIdentity = authorityTargetingReadbackIdentity(
      targetingReadback('hero', 1, 'state-a'),
      5,
    );
    const nextTurnIdentity = authorityTargetingReadbackIdentity(
      targetingReadback('rival', 2, 'state-b'),
      6,
    );
    const refreshedIdentity = authorityTargetingReadbackIdentity(
      targetingReadback('hero', 1, 'state-a'),
      7,
    );

    expect(
      authorityReadbackRequiresTargetingReset(undefined, initialIdentity),
    ).toBe(false);
    expect(
      authorityReadbackRequiresTargetingReset(initialIdentity, initialIdentity),
    ).toBe(false);
    expect(
      authorityReadbackRequiresTargetingReset(
        initialIdentity,
        nextTurnIdentity,
      ),
    ).toBe(true);
    expect(
      authorityReadbackRequiresTargetingReset(
        initialIdentity,
        refreshedIdentity,
      ),
    ).toBe(true);
  });

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
