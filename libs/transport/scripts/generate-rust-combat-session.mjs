import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const generatedPath = join(process.cwd(), 'libs/transport/src/generated/rust-combat-session.ts');
const rustManifestPath = join(process.cwd(), 'rulebench-rs/Cargo.toml');

export function renderCombatSessionCatalog() {
  const result = spawnSync(
    'cargo',
    ['run', '--quiet', '--manifest-path', rustManifestPath, '-p', 'rulebench-authority', '--bin', 'emit_combat_session'],
    { cwd: process.cwd(), encoding: 'utf8' },
  );

  if (result.status !== 0) {
    throw new Error(`Rust combat session emitter failed:\n${result.stderr || result.stdout}`);
  }

  return result.stdout;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const nextContent = renderCombatSessionCatalog();
  if (process.argv.includes('--check')) {
    const currentContent = readFileSync(generatedPath, 'utf8');
    if (currentContent !== nextContent) {
      console.error(`${generatedPath} is out of date. Run node libs/transport/scripts/generate-rust-combat-session.mjs.`);
      process.exit(1);
    }
  } else {
    writeFileSync(generatedPath, nextContent);
  }
}
