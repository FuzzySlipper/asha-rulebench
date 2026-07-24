import type { GameplayItemBindingDto } from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import {
  authorityItemBindingLabel,
  type AuthorityItemLabelEntity,
} from './index.js';

describe('authority item binding presentation', () => {
  const entities: readonly AuthorityItemLabelEntity[] = [
    {
      id: 'fighter',
      items: [
        {
          id: 'weapon',
          definitionId: 'item.long-sword',
          label: 'Longsword',
        },
      ],
    },
    {
      id: 'skeleton',
      items: [
        {
          id: 'weapon',
          definitionId: 'item.short-sword',
          label: 'Shortsword',
        },
      ],
    },
  ];

  it('scopes participant-local item instance IDs to the logged actor', () => {
    const binding: GameplayItemBindingDto = {
      bindingId: 'weapon',
      itemInstanceId: 'weapon',
      itemDefinitionId: 'item.short-sword',
      slotId: 'hand.main',
    };

    expect(authorityItemBindingLabel(entities, 'skeleton', binding)).toBe(
      'Shortsword · weapon · hand.main',
    );
  });

  it('does not borrow a label when the authoritative definition differs', () => {
    const binding: GameplayItemBindingDto = {
      bindingId: 'weapon',
      itemInstanceId: 'weapon',
      itemDefinitionId: 'item.long-sword',
      slotId: 'hand.main',
    };

    expect(authorityItemBindingLabel(entities, 'skeleton', binding)).toBe(
      'item.long-sword · weapon · hand.main',
    );
  });
});
