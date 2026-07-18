import { spawn } from 'node:child_process';

const forwardedArguments = process.argv.slice(2);
if (forwardedArguments[0] === '--') forwardedArguments.shift();

const angular = spawn(
  'pnpm',
  [
    'nx',
    'serve',
    'app',
    '--configuration=e2e',
    '--host',
    '0.0.0.0',
    ...forwardedArguments,
  ],
  {
    cwd: process.cwd(),
    stdio: 'inherit',
    shell: false,
  },
);

process.once('SIGINT', () => angular.kill('SIGINT'));
process.once('SIGTERM', () => angular.kill('SIGTERM'));
angular.once('exit', (code) => process.exit(code ?? 0));
