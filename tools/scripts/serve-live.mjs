import { spawn } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { join } from 'node:path';

const root = process.cwd();
const forwardedArguments = process.argv.slice(2);
if (forwardedArguments[0] === '--') forwardedArguments.shift();

const hostPort = await freePort();
const hostUrl = `http://127.0.0.1:${hostPort}`;
const proxyPath = join(root, 'tmp', 'ruleset', 'proxy.json');
await mkdir(join(root, 'tmp', 'ruleset'), { recursive: true });
await writeFile(
  proxyPath,
  `${JSON.stringify({
    '/api': {
      target: hostUrl,
      secure: false,
      changeOrigin: true,
      logLevel: 'warn',
    },
  })}\n`,
  'utf8',
);

const rustHost = spawn(
  'cargo',
  [
    'run',
    '--quiet',
    '--manifest-path',
    'rulebench-rs/Cargo.toml',
    '-p',
    'rulebench-ruleset-host',
    '--bin',
    'rulebench-ruleset-host',
    '--',
    '--address',
    `127.0.0.1:${hostPort}`,
  ],
  { cwd: root, stdio: 'inherit', shell: false },
);

await waitForHost(`${hostUrl}/api/ruleset/health`, rustHost);

const angular = spawn(
  'pnpm',
  [
    'nx',
    'serve',
    'app',
    '--configuration=e2e',
    '--host',
    '0.0.0.0',
    '--proxy-config',
    proxyPath,
    ...forwardedArguments,
  ],
  { cwd: root, stdio: 'inherit', shell: false },
);

const terminate = (signal) => {
  angular.kill(signal);
  rustHost.kill(signal);
};
process.once('SIGINT', () => terminate('SIGINT'));
process.once('SIGTERM', () => terminate('SIGTERM'));
rustHost.once('exit', (code) => {
  if (angular.exitCode === null) angular.kill('SIGTERM');
  if (code !== null && code !== 0) process.exitCode = code;
});
angular.once('exit', (code) => {
  if (rustHost.exitCode === null) rustHost.kill('SIGTERM');
  process.exit(code ?? 0);
});

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      server.close(() => {
        if (address !== null && typeof address === 'object')
          resolve(address.port);
        else reject(new Error('Could not allocate a ruleset host port'));
      });
    });
    server.on('error', reject);
  });
}

async function waitForHost(url, child) {
  for (let attempt = 0; attempt < 120; attempt += 1) {
    if (child.exitCode !== null) {
      throw new Error(
        `ruleset host exited before startup with ${child.exitCode}`,
      );
    }
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // The compiler may still be building or binding its loopback socket.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  child.kill('SIGTERM');
  throw new Error(`ruleset host did not become ready at ${url}`);
}
