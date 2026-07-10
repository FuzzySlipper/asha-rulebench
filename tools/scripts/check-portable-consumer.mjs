import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

const manifestPath = join(process.cwd(), 'rulebench-rs', 'portable-consumer-smoke', 'Cargo.toml');
const forbiddenCrates = [
  'rulebench-authority',
  'rulebench-bridge',
  'rulebench-codegen',
  'rulebench-fixtures',
  'rulebench-protocol',
];

runCargo(['run', '--quiet', '--manifest-path', manifestPath]);
const tree = runCargo(['tree', '--manifest-path', manifestPath]);
for (const crate of forbiddenCrates) {
  if (tree.includes(crate)) {
    fail(`portable consumer transitively depends on Rulebench-local crate ${crate}`);
  }
}

console.log('portable consumer contract ok');

function runCargo(args) {
  const result = spawnSync('cargo', args, { cwd: process.cwd(), encoding: 'utf8' });
  if (result.status !== 0) {
    fail(result.stderr || result.stdout || `cargo ${args.join(' ')} failed`);
  }
  return result.stdout;
}

function fail(message) {
  console.error(message.trim());
  process.exit(1);
}
