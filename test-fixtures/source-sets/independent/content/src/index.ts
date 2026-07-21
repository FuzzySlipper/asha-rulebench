import { contentPackSource, defineContentPack } from '@asha-rpg/authoring';

export const contentPack = defineContentPack({
  identity: { id: 'rulebench.independent.content', version: '1.0.0' },
  entry: { module: 'src/index.ts', declaration: 'contentPack' },
  definitions: [],
});

export const contentSource = contentPackSource(contentPack);
