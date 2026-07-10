import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const generatedPath = join(process.cwd(), 'libs/protocol/src/generated/api-types.ts');
const rustManifestPath = join(process.cwd(), 'rulebench-rs/Cargo.toml');

export function renderApiTypes() {
  const result = spawnSync(
    'cargo',
    ['run', '--quiet', '--manifest-path', rustManifestPath, '-p', 'rulebench-protocol', '--bin', 'emit_protocol_types'],
    { cwd: process.cwd(), encoding: 'utf8' },
  );

  if (result.status !== 0) {
    throw new Error(`Rust protocol emitter failed:\n${result.stderr || result.stdout}`);
  }

  return result.stdout;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const nextContent = renderApiTypes();
  if (process.argv.includes('--check')) {
    const currentContent = readFileSync(generatedPath, 'utf8');
    if (currentContent !== nextContent) {
      console.error(`${generatedPath} is out of date. Run node libs/protocol/scripts/generate-api-types.mjs.`);
      process.exit(1);
    }
  } else {
    writeFileSync(generatedPath, nextContent);
  }
}
