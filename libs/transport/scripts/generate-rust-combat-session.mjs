import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const generatedPath = join(process.cwd(), 'libs/transport/src/generated/rust-combat-session.ts');
const rustManifestPath = join(process.cwd(), 'rulebench-rs/Cargo.toml');
const owningPackage = 'asha-rulebench.hexing-bolt';

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
      console.error(`Fixture package ${owningPackage} combat-session artifact ${generatedPath} is out of date. Run pnpm run session:write.`);
      process.exit(1);
    }
  } else if (process.argv.includes('--write')) {
    writeFileSync(generatedPath, nextContent);
  } else {
    console.error('Pass --check to verify or --write to regenerate the combat session artifact.');
    process.exit(1);
  }
}
