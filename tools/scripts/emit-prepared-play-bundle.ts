import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

import { loadPlayBundleWorkspace } from './load-play-bundle-workspace.js';

const outputArgument = process.argv.indexOf('--output');
const outputPath = process.argv[outputArgument + 1];
const rulesetRoot = argumentValue('--ruleset-root');
const contentPackIds = argumentValues('--content-pack');
if (
  outputArgument < 0 ||
  outputPath === undefined ||
  outputPath.trim().length === 0
) {
  throw new Error(
    'usage: emit-prepared-play-bundle --output <path> --ruleset-root <rulesets/name> --content-pack <id>',
  );
}

const result = await loadPlayBundleWorkspace(
  { operation: 'compile', rulesetRoot, contentPackIds },
  process.cwd(),
);
if (!result.ok) {
  process.stderr.write(`${JSON.stringify(result.diagnostics, null, 2)}\n`);
  process.exitCode = 1;
} else {
  const destination = resolve(outputPath);
  await mkdir(dirname(destination), { recursive: true });
  if (result.preparedSource === null) {
    throw new Error('PlayBundle compilation did not produce prepared source');
  }
  await writeFile(destination, `${result.preparedSource}\n`, 'utf8');
  process.stdout.write(`prepared PlayBundle from ${rulesetRoot}\n`);
}

function argumentValues(name: string): readonly string[] {
  return process.argv
    .flatMap((argument, index) =>
      argument === name ? [process.argv[index + 1]] : [],
    )
    .filter(
      (value): value is string =>
        value !== undefined && value.trim().length > 0,
    );
}

function argumentValue(name: string): string {
  const index = process.argv.indexOf(name);
  const value = process.argv[index + 1];
  if (index < 0 || value === undefined || value.trim().length === 0) {
    throw new Error(`${name} requires a value`);
  }
  return value;
}
