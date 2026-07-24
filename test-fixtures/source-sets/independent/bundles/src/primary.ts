import { composePlayBundle, contentPackRequest } from '@asha-rpg/authoring';

import { contentPack } from '../../content/src/index.js';
import { positionalContentPack } from '../../positional-content/src/index.js';
import { ruleset } from '../../rules/src/index.js';

export const playBundle = composePlayBundle({
  identity: { id: 'rulebench.independent.play', version: '1.0.0' },
  ruleset,
  base: contentPackRequest(contentPack.identity),
  add: [contentPackRequest(positionalContentPack.identity)],
  overlays: [],
  configure: {},
});
