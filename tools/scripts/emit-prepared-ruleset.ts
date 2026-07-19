import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

import { loadRulesetWorkspace } from './load-ruleset-workspace.js';

const outputArgument = process.argv.indexOf('--output');
const outputPath = process.argv[outputArgument + 1];
const workspaceRoot = argumentValue('--workspace-root');
const module = argumentValue('--module');
const declaration = argumentValue('--declaration');
const packageRoots = argumentValues('--package-root');
if (
  outputArgument < 0 ||
  outputPath === undefined ||
  outputPath.trim().length === 0
) {
  throw new Error(
    'usage: emit-prepared-ruleset --output <path> --workspace-root <path> --package-root <path>... --module <path> --declaration <export>',
  );
}

const result = await loadRulesetWorkspace(
  { workspaceRoot, packageRoots, module, declaration },
  process.cwd(),
);
if (!result.ok) {
  process.stderr.write(`${JSON.stringify(result.diagnostics, null, 2)}\n`);
  process.exitCode = 1;
} else {
  const destination = resolve(outputPath);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, `${result.preparedSource}\n`, 'utf8');
  process.stdout.write(
    `prepared explicit workspace ${module}#${declaration}\n`,
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

function argumentValues(name: string): readonly string[] {
  const values: string[] = [];
  for (let index = 0; index < process.argv.length; index += 1) {
    if (process.argv[index] !== name) continue;
    const value = process.argv[index + 1];
    if (value === undefined || value.trim().length === 0) {
      throw new Error(`${name} requires a value`);
    }
    values.push(value);
  }
  if (values.length === 0) throw new Error(`${name} is required`);
  return values;
}
