import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const root = process.cwd();
const manifestPath = join(root, 'docs', 'verification-claims.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
const failures = [];

if (manifest.schemaVersion !== 1) failures.push('docs/verification-claims.json must use schemaVersion 1.');
if (manifest.reviewedOn !== '2026-07-14') failures.push('verification claims review date is stale.');
for (const slug of ['basic-design', 'north-star-systems-map', 'known-limitations']) {
  if (!manifest.denDocuments.includes(slug)) failures.push(`verification review omits Den document ${slug}.`);
}
for (const entry of manifest.requiredClaims) requireText(entry, 'required current claim');
for (const entry of manifest.requiredNonClaims) requireText(entry, 'required non-claim');
for (const entry of manifest.forbiddenClaims) forbidText(entry);

const limitationIds = new Set(manifest.activeLimitations.map((entry) => entry.id));
for (const id of ['trusted-local-process-host', 'single-ruleset-identity', 'checked-viewer-artifacts']) {
  if (!limitationIds.has(id)) failures.push(`verification review omits active limitation ${id}.`);
}

for (const crateRoot of [join(root, 'rulebench-rs', 'crates'), join(root, 'rulebench-rs', 'hosts')]) {
  for (const entry of readdirSync(crateRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const crate = entry.name;
    const sourceRoot = join(crateRoot, crate, 'src');
    const sources = collectRustFiles(sourceRoot);
    const production = sources.filter((path) => !path.includes(`${join('src', 'tests')}`) && !path.endsWith('tests.rs'));
    if (production.length === 0) {
      failures.push(`${relative(root, sourceRoot)} has no production Rust source and must not be presented as implemented.`);
      continue;
    }
    const text = production.map((path) => readFileSync(path, 'utf8')).join('\n');
    if (/\b(?:todo|unimplemented)!\s*\(/.test(text)) {
      failures.push(`${crate} contains a production todo!/unimplemented! stub; resolve it or record a scoped limitation.`);
    }
  }
}

runFocusedFailureTests();

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(`check:claims ok (${manifest.requiredClaims.length} claims, ${manifest.requiredNonClaims.length} non-claims, ${manifest.activeLimitations.length} active limitations)`);

function requireText(entry, kind) {
  const text = readFileSync(join(root, entry.file), 'utf8');
  if (!text.includes(entry.text)) failures.push(`${entry.file} is missing ${kind}: ${entry.text}`);
}

function forbidText(entry) {
  const text = readFileSync(join(root, entry.file), 'utf8');
  if (text.includes(entry.text)) failures.push(`${entry.file} contains stale claim: ${entry.text}`);
}

function collectRustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) files.push(...collectRustFiles(path));
    else if (entry.endsWith('.rs')) files.push(path);
  }
  return files;
}

function runFocusedFailureTests() {
  if (!/\b(?:todo|unimplemented)!\s*\(/.test('fn pending() { todo!() }')) {
    throw new Error('Claims self-test failed to detect a production authority stub.');
  }
  if (/\b(?:todo|unimplemented)!\s*\(/.test('fn complete() {}')) {
    throw new Error('Claims self-test classified complete code as a stub.');
  }
}
