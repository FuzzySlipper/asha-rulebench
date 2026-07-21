import { contentPackSource, defineContentPack } from '@asha-rpg/authoring';

const contentPack = defineContentPack({
  identity: { id: 'rulebench.minimal.content', version: '1.0.0' },
  entry: { module: 'src/index.ts', declaration: 'duplicateContentSource' },
  definitions: [],
});

export const duplicateContentSource = contentPackSource(contentPack);
