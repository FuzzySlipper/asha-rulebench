import { createServer } from 'node:net';
import { spawn } from 'node:child_process';
import { join } from 'node:path';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';

const root = process.cwd();
const protocolVersion = readProtocolVersion(root);
const hostAddress = '127.0.0.1';
const hostPort = await freePort(hostAddress);
const hostUrl = `http://${hostAddress}:${hostPort}`;
const proxyPath = join(root, 'tools/config/rulebench-proxy.cjs');
const forwardedArguments = process.argv.slice(2);
if (forwardedArguments[0] === '--') forwardedArguments.shift();
const ephemeralArtifacts = process.env['RULEBENCH_EPHEMERAL_ARTIFACTS'] === '1';
const artifactRoot =
  process.env['RULEBENCH_ARTIFACT_ROOT'] ??
  (ephemeralArtifacts
    ? mkdtempSync(join(tmpdir(), 'asha-rulebench-e2e-artifacts-'))
    : null);
const artifactArguments = artifactRoot === null ? [] : ['--artifact-root', artifactRoot];

const rustHost = spawn(
  'cargo',
  [
    'run',
    '--manifest-path',
    'rulebench-rs/Cargo.toml',
    '-p',
    'rulebench-process-host',
    '--bin',
    'rulebench_process_host',
    '--',
    '--bind',
    `${hostAddress}:${hostPort}`,
    ...artifactArguments,
  ],
  { cwd: root, stdio: 'inherit', shell: false },
);

let angular = null;
let stopping = false;

try {
  await waitForHost(rustHost, hostUrl);
  console.log(`RULEBENCH_HOST_URL=${hostUrl}`);
  angular = spawn(
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
    {
      cwd: root,
      stdio: 'inherit',
      shell: false,
      env: { ...process.env, RULEBENCH_HOST_URL: hostUrl },
    },
  );

  process.once('SIGINT', () => void stop(130));
  process.once('SIGTERM', () => void stop(143));
  rustHost.once('exit', (code) => {
    if (!stopping) {
      console.error(`Rust host exited while Angular was running (code ${code ?? 'signal'}).`);
      void stop(code ?? 1);
    }
  });
  angular.once('exit', (code) => void stop(code ?? 0));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  await stop(1);
}

async function stop(exitCode) {
  if (stopping) return;
  stopping = true;
  angular?.kill('SIGTERM');
  rustHost.kill('SIGTERM');
  if (ephemeralArtifacts && artifactRoot !== null) {
    rmSync(artifactRoot, { recursive: true, force: true });
  }
  process.exit(exitCode);
}

async function waitForHost(child, baseUrl) {
  let childExit = null;
  child.once('exit', (code) => {
    childExit = code ?? 1;
  });
  for (let attempt = 0; attempt < 300; attempt += 1) {
    if (childExit !== null) {
      throw new Error(`Rust host exited before readiness (code ${childExit}).`);
    }
    try {
      const response = await fetch(`${baseUrl}/api/rulebench/v1/handshake`, {
        headers: {
          'x-rulebench-protocol-version': String(protocolVersion),
        },
        signal: AbortSignal.timeout(500),
      });
      if (response.ok) return;
    } catch {
      // Compilation and process startup are still in progress.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Rust host did not become ready at ${baseUrl}.`);
}

function readProtocolVersion(workspaceRoot) {
  const liveTransportSource = readFileSync(
    join(workspaceRoot, 'libs/transport/src/live.ts'),
    'utf8',
  );
  const match = liveTransportSource.match(
    /export const RULEBENCH_PROTOCOL_VERSION\s*=\s*(\d+);/,
  );
  if (match?.[1] === undefined) {
    throw new Error('Could not resolve the live transport protocol version.');
  }
  return Number(match[1]);
}

function freePort(host) {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, host, () => {
      const address = server.address();
      server.close((error) => {
        if (error !== undefined) {
          reject(error);
          return;
        }
        if (address === null || typeof address === 'string') {
          reject(new Error('Could not allocate a Rust host port.'));
          return;
        }
        resolve(address.port);
      });
    });
  });
}
