import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();
const projectName = process.argv[2];

if (projectName === undefined || !/^[a-z][a-z0-9-]*$/.test(projectName)) {
  console.error('Usage: pnpm run init -- <kebab-project-name>');
  process.exit(1);
}

const packagePath = join(root, 'package.json');
const packageJson = JSON.parse(readFileSync(packagePath, 'utf8'));
packageJson.name = projectName;
writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

const manifestPath = join(root, 'template-manifest.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
manifest.copiedAt = new Date().toISOString().slice(0, 10);
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

replaceInFiles(['README.md', 'README.template.md', 'apps/app/src/index.html'], 'UI Pattern Bootstrap', titleCase(projectName));

console.log(`Initialized ${projectName}. Review import aliases in tsconfig.base.json when the package namespace is known.`);

function replaceInFiles(paths, search, replacement) {
  for (const path of paths) {
    const fullPath = join(root, path);
    const text = readFileSync(fullPath, 'utf8');
    writeFileSync(fullPath, text.replaceAll(search, replacement));
  }
}

function titleCase(value) {
  return value.split('-').map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`).join(' ');
}
