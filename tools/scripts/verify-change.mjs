import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

export const profileOrder = [
  'frontend',
  'content-authoring',
  'rust-owner',
  'protocol-generated',
  'host-transport',
  'browser',
  'docs',
];

export function parseVerifyChangeArguments(argumentsList) {
  const profiles = [];
  let dryRun = false;

  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === '--') continue;
    if (argument === '--dry-run') {
      dryRun = true;
      continue;
    }
    if (argument === '--profile') {
      profiles.push(requireValue(argumentsList, ++index, argument));
      continue;
    }
    throw new Error(`Unknown verify:change argument: ${argument}`);
  }

  const uniqueProfiles = [...new Set(profiles)];
  if (uniqueProfiles.length === 0) {
    throw new Error(
      `At least one --profile is required. Allowed profiles: ${profileOrder.join(', ')}.`,
    );
  }
  for (const profile of uniqueProfiles) {
    if (!profileOrder.includes(profile)) {
      throw new Error(
        `Unknown verify:change profile: ${profile}. Allowed profiles: ${profileOrder.join(', ')}.`,
      );
    }
  }

  return {
    profiles: profileOrder.filter((profile) =>
      uniqueProfiles.includes(profile),
    ),
    dryRun,
  };
}

export function buildVerifyChangePlan(selection) {
  const commands = [];
  const addScript = (script) =>
    addCommand(commands, {
      id: `pnpm:${script}`,
      command: 'pnpm',
      arguments: ['run', script],
    });

  for (const profile of selection.profiles) {
    switch (profile) {
      case 'frontend':
        addScript('check:pattern');
        addScript('check:typescript-authority');
        addScript('lint');
        addScript('typecheck');
        addScript('test');
        addScript('build');
        break;
      case 'content-authoring':
        addScript('check:typescript-authority');
        addScript('ruleset:prepare');
        addScript('typecheck');
        addScript('test');
        break;
      case 'rust-owner':
        addScript('test:rust');
        break;
      case 'protocol-generated':
        addScript('check:generated');
        addScript('typecheck');
        addScript('test');
        break;
      case 'host-transport':
        addScript('ruleset:prepare');
        addScript('test:rust');
        addScript('typecheck');
        addScript('test');
        break;
      case 'browser':
        addScript('typecheck');
        addScript('e2e:gate');
        break;
      case 'docs':
        addScript('check:docs');
        break;
    }
  }

  return commands;
}

export function formatCommand(entry) {
  return [entry.command, ...entry.arguments]
    .map((part) =>
      /^[A-Za-z0-9_./:@=-]+$/.test(part) ? part : JSON.stringify(part),
    )
    .join(' ');
}

function addCommand(commands, command) {
  if (!commands.some((entry) => entry.id === command.id)) {
    commands.push(command);
  }
}

function requireValue(argumentsList, index, flag) {
  const value = argumentsList[index];
  if (value === undefined || value.startsWith('--')) {
    throw new Error(`${flag} requires a value.`);
  }
  return value;
}

function run() {
  let selection;
  try {
    selection = parseVerifyChangeArguments(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(2);
  }

  const plan = buildVerifyChangePlan(selection);
  console.log(`verify:change profiles: ${selection.profiles.join(', ')}`);
  console.log('verify:change selected commands:');
  for (const command of plan) console.log(`- ${formatCommand(command)}`);
  if (selection.dryRun) return;

  for (const entry of plan) {
    console.log(`\n[verify:change] ${formatCommand(entry)}`);
    const result = spawnSync(entry.command, entry.arguments, {
      cwd: process.cwd(),
      env: process.env,
      stdio: 'inherit',
    });
    if (result.error !== undefined) {
      console.error(result.error.message);
      process.exit(1);
    }
    if (result.status !== 0) process.exit(result.status ?? 1);
  }
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
