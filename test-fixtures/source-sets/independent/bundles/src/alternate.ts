import { composePlayBundle, contentPackRequest } from '@asha-rpg/authoring';

import { ruleset } from '../../alternate-rules/src/index.js';
import { contentPack } from '../../content/src/index.js';

export const playBundle = composePlayBundle({
  identity: { id: 'rulebench.independent.alternate-play', version: '1.0.0' },
  ruleset,
  base: contentPackRequest(contentPack.identity),
  add: [],
  overlays: [],
  configure: {},
});
