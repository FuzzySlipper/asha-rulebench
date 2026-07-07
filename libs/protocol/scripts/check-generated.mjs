import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { renderApiTypes } from './generate-api-types.mjs';

const generatedRoot = join(process.cwd(), 'libs/protocol/src/generated');
const apiTypesPath = join(generatedRoot, 'api-types.ts');
const failures = [];

for (const file of collectTsFiles(generatedRoot)) {
  const text = readFileSync(file, 'utf8');
  if (!text.startsWith('// GENERATED')) {
    failures.push(`${file} must start with // GENERATED`);
  }
}

if (readFileSync(apiTypesPath, 'utf8') !== renderApiTypes()) {
  failures.push(`${apiTypesPath} is out of date. Run node libs/protocol/scripts/generate-api-types.mjs.`);
}

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log('protocol generated files ok');

function collectTsFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      files.push(...collectTsFiles(path));
    } else if (entry.endsWith('.ts')) {
      files.push(path);
    }
  }
  return files;
}
