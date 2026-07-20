import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

import { loadRulesetWorkspace } from './load-ruleset-workspace.js';

const outputArgument = process.argv.indexOf('--output');
const outputPath = process.argv[outputArgument + 1];
const rulesetRoot = argumentValue('--ruleset-root');
if (
  outputArgument < 0 ||
  outputPath === undefined ||
  outputPath.trim().length === 0
) {
  throw new Error(
    'usage: emit-prepared-ruleset --output <path> --ruleset-root <rulesets/name>',
  );
}

const result = await loadRulesetWorkspace({ rulesetRoot }, process.cwd());
if (!result.ok) {
  process.stderr.write(`${JSON.stringify(result.diagnostics, null, 2)}\n`);
  process.exitCode = 1;
} else {
  const destination = resolve(outputPath);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, `${result.preparedSource}\n`, 'utf8');
  process.stdout.write(`prepared ruleset root ${rulesetRoot}\n`);
}

function argumentValue(name: string): string {
  const index = process.argv.indexOf(name);
  const value = process.argv[index + 1];
  if (index < 0 || value === undefined || value.trim().length === 0) {
    throw new Error(`${name} requires a value`);
  }
  return value;
}
