import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

import { canonicalJson } from '@asha-rpg/authoring';

import { prepareFreshRulebenchRuleset } from '../../libs/content-authoring/src/index.js';

const outputArgument = process.argv.indexOf('--output');
const outputPath = process.argv[outputArgument + 1];
if (
  outputArgument < 0 ||
  outputPath === undefined ||
  outputPath.trim().length === 0
) {
  throw new Error('usage: emit-prepared-ruleset --output <path>');
}

const result = prepareFreshRulebenchRuleset();
if (!result.ok) {
  process.stderr.write(`${JSON.stringify(result.diagnostics, null, 2)}\n`);
  process.exitCode = 1;
} else {
  const destination = resolve(outputPath);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, `${canonicalJson(result.prepared)}\n`, 'utf8');
  process.stdout.write(
    `prepared ruleset ${result.prepared.compositionIdentity.id}@${result.prepared.compositionIdentity.version}\n`,
  );
}
