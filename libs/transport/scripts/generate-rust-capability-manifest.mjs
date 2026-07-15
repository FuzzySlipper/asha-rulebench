import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const generatedPath = join(process.cwd(), 'libs/transport/src/generated/rust-capability-manifest.ts');
const rustManifestPath = join(process.cwd(), 'rulebench-rs/Cargo.toml');

export function renderCapabilityManifest() {
  const result = spawnSync(
    'cargo',
    [
      'run',
      '--quiet',
      '--manifest-path',
      rustManifestPath,
      '-p',
      'rulebench-process-host',
      '--bin',
      'emit_capability_manifest',
    ],
    { cwd: process.cwd(), encoding: 'utf8' },
  );

  if (result.status !== 0) {
    throw new Error(`Rust capability manifest emitter failed:\n${result.stderr || result.stdout}`);
  }
  return result.stdout;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const nextContent = renderCapabilityManifest();
  if (process.argv.includes('--check')) {
    const currentContent = readFileSync(generatedPath, 'utf8');
    if (currentContent !== nextContent) {
      console.error(`${generatedPath} is out of date. Run pnpm run generated:write.`);
      process.exit(1);
    }
  } else if (process.argv.includes('--write')) {
    writeFileSync(generatedPath, nextContent);
  } else {
    console.error('Pass --check to verify or --write to regenerate the capability manifest.');
    process.exit(1);
  }
}
