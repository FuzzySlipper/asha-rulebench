import { spawn, spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { createServer as createHttpServer } from 'node:http';
import { createServer as createNetServer } from 'node:net';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { loadRulesetLocationConfig } from './ruleset-location-config.mjs';

const root = process.cwd();
const rulesetLocationConfig = await loadRulesetLocationConfig(
  root,
  process.env['RULEBENCH_RULESET_CONFIG'] ?? '.rulebench/rulesets.json',
);
const forwardedArguments = process.argv.slice(2);
if (forwardedArguments[0] === '--') forwardedArguments.shift();
const angularConfiguration =
  process.env['RULEBENCH_ANGULAR_CONFIGURATION'] ?? 'rulebench';

const authoringBuild = spawnSync(
  'pnpm',
  ['exec', 'tsc', '-p', 'tools/content-compiler/tsconfig.json'],
  { cwd: root, stdio: 'inherit', shell: false },
);
if (authoringBuild.status !== 0) process.exit(authoringBuild.status ?? 1);
const authoringModuleUrl = pathToFileURL(
  join(
    root,
    'tmp',
    'content-compiler',
    'tools',
    'scripts',
    'load-ruleset-workspace.js',
  ),
);
const { parseWorkspaceLoaderOutput } = await import(
  `${authoringModuleUrl.href}?startup=${Date.now()}`
);
const workspaceLoaderPath = join(
  root,
  'tmp',
  'content-compiler',
  'tools',
  'scripts',
  'load-ruleset-workspace.js',
);

const rustHostPort = await freePort();
const rustHostUrl = `http://127.0.0.1:${rustHostPort}`;
const gatewayPort = await freePort();
const gatewayUrl = `http://127.0.0.1:${gatewayPort}`;
const proxyPath = join(root, 'tmp', 'ruleset', 'proxy.json');
await mkdir(join(root, 'tmp', 'ruleset'), { recursive: true });
await writeFile(
  proxyPath,
  `${JSON.stringify({
    '/api': {
      target: gatewayUrl,
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
    `127.0.0.1:${rustHostPort}`,
  ],
  { cwd: root, stdio: 'inherit', shell: false },
);

await waitForHost(`${rustHostUrl}/api/ruleset/health`, rustHost);
const authoringGateway = await startAuthoringGateway(
  gatewayPort,
  rustHostUrl,
  workspaceLoaderPath,
  parseWorkspaceLoaderOutput,
);

const angular = spawn(
  'pnpm',
  [
    'nx',
    'serve',
    'app',
    `--configuration=${angularConfiguration}`,
    '--host',
    '0.0.0.0',
    '--proxy-config',
    proxyPath,
    ...forwardedArguments,
  ],
  { cwd: root, stdio: 'inherit', shell: false },
);

const terminate = (signal) => {
  authoringGateway.close();
  angular.kill(signal);
  rustHost.kill(signal);
};
process.once('SIGINT', () => terminate('SIGINT'));
process.once('SIGTERM', () => terminate('SIGTERM'));
rustHost.once('exit', (code) => {
  authoringGateway.close();
  if (angular.exitCode === null) angular.kill('SIGTERM');
  if (code !== null && code !== 0) process.exitCode = code;
});
angular.once('exit', (code) => {
  authoringGateway.close();
  if (rustHost.exitCode === null) rustHost.kill('SIGTERM');
  process.exit(code ?? 0);
});

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createNetServer();
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

function startAuthoringGateway(
  port,
  rustHostUrl,
  loaderPath,
  parseLoaderOutput,
) {
  const server = createHttpServer(async (request, response) => {
    try {
      if (request.method === 'GET' && request.url === '/api/ruleset/config') {
        sendJson(response, 200, rulesetLocationConfig);
        return;
      }
      if (request.method === 'POST' && request.url === '/api/ruleset/compile') {
        const body = await readJsonBody(request);
        const preparation = await prepareWorkspace(
          loaderPath,
          parseLoaderOutput,
          body,
        );
        if (!preparation.ok) {
          const workspace = await rustWorkspace(rustHostUrl);
          sendJson(response, 200, {
            ...workspace,
            ok: false,
            diagnostics: preparation.diagnostics.map(authoringDiagnosticDto),
          });
          return;
        }
        await forwardJson(
          response,
          `${rustHostUrl}/api/ruleset/compile`,
          'POST',
          { preparedSource: preparation.preparedSource },
        );
        return;
      }
      const forwardedBody =
        request.method === 'POST'
          ? await readOptionalJsonBody(request)
          : undefined;
      await forwardJson(
        response,
        `${rustHostUrl}${request.url ?? '/api/ruleset'}`,
        request.method === 'POST' ? 'POST' : 'GET',
        forwardedBody,
      );
    } catch (error) {
      await respondWithGatewayDiagnostic(
        response,
        rustHostUrl,
        500,
        'RULESET_AUTHORING_GATEWAY_FAILED',
        '$',
        error instanceof Error ? error.message : String(error),
      );
    }
  });
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => resolve(server));
  });
}

function prepareWorkspace(loaderPath, parseLoaderOutput, input) {
  return new Promise((resolve, reject) => {
    const compiler = spawn(process.execPath, [loaderPath], {
      cwd: root,
      stdio: ['pipe', 'pipe', 'pipe'],
      shell: false,
    });
    const output = [];
    const errors = [];
    let outputLength = 0;
    let errorLength = 0;
    let settled = false;
    const timeout = setTimeout(() => {
      compiler.kill('SIGTERM');
      if (!settled) {
        settled = true;
        reject(new Error('authoring subprocess exceeded 30 seconds'));
      }
    }, 30_000);
    const collect = (chunks, chunk, length, label) => {
      const nextLength = length + chunk.length;
      if (nextLength > 1_048_576) {
        compiler.kill('SIGTERM');
        if (!settled) {
          settled = true;
          clearTimeout(timeout);
          reject(new Error(`${label} exceeded 1 MiB`));
        }
        return length;
      }
      chunks.push(chunk);
      return nextLength;
    };
    compiler.stdout.on('data', (chunk) => {
      outputLength = collect(output, chunk, outputLength, 'authoring output');
    });
    compiler.stderr.on('data', (chunk) => {
      errorLength = collect(errors, chunk, errorLength, 'authoring errors');
    });
    compiler.once('error', (error) => {
      if (!settled) {
        settled = true;
        clearTimeout(timeout);
        reject(error);
      }
    });
    compiler.once('exit', (code) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      if (code !== 0) {
        reject(
          new Error(
            `authoring subprocess exited ${code}: ${Buffer.concat(errors).toString('utf8')}`,
          ),
        );
        return;
      }
      try {
        const parsed = parseLoaderOutput(
          Buffer.concat(output).toString('utf8'),
        );
        if (
          parsed === null ||
          typeof parsed !== 'object' ||
          Array.isArray(parsed) ||
          !('ok' in parsed) ||
          typeof parsed.ok !== 'boolean' ||
          !('diagnostics' in parsed) ||
          !Array.isArray(parsed.diagnostics)
        ) {
          reject(new Error('authoring subprocess returned an invalid result'));
          return;
        }
        resolve(parsed);
      } catch (error) {
        reject(error);
      }
    });
    compiler.stdin.end(JSON.stringify(input));
  });
}

async function forwardJson(response, url, method, body) {
  const upstream = await fetch(url, {
    method,
    headers: {
      accept: 'application/json',
      ...(body === undefined ? {} : { 'content-type': 'application/json' }),
    },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
  sendJson(response, upstream.status, await upstream.json());
}

async function rustWorkspace(rustHostUrl) {
  const response = await fetch(`${rustHostUrl}/api/ruleset`);
  if (!response.ok)
    throw new Error(`ruleset host status failed: ${response.status}`);
  return response.json();
}

async function respondWithGatewayDiagnostic(
  response,
  rustHostUrl,
  status,
  code,
  path,
  message,
) {
  const workspace = await rustWorkspace(rustHostUrl);
  sendJson(response, status, {
    ...workspace,
    ok: false,
    diagnostics: [
      {
        stage: 'source',
        severity: 'error',
        code,
        path,
        message,
        packageId: null,
        definitionId: null,
        source: null,
        graphPath: null,
        expected: null,
        actual: null,
      },
    ],
  });
}

function authoringDiagnosticDto(diagnostic) {
  return {
    stage: diagnostic.stage,
    severity: diagnostic.severity,
    code: diagnostic.code,
    path: diagnostic.path,
    message: diagnostic.message,
    packageId: diagnostic.packageId ?? null,
    definitionId: diagnostic.definitionId ?? null,
    source:
      diagnostic.source === undefined
        ? null
        : {
            module: diagnostic.source.module,
            declaration: diagnostic.source.declaration,
          },
    graphPath:
      diagnostic.graphPath === undefined ? null : [...diagnostic.graphPath],
    expected: diagnostic.expected ?? null,
    actual: diagnostic.actual ?? null,
  };
}

function sendJson(response, status, payload) {
  response.writeHead(status, { 'content-type': 'application/json' });
  response.end(JSON.stringify(payload));
}

async function readJsonBody(request) {
  const source = await readBody(request);
  if (source.length === 0) throw new Error('request body is required');
  return JSON.parse(source);
}

async function readOptionalJsonBody(request) {
  const source = await readBody(request);
  return source.length === 0 ? undefined : JSON.parse(source);
}

async function readBody(request) {
  const chunks = [];
  let length = 0;
  for await (const chunk of request) {
    length += chunk.length;
    if (length > 1_048_576) throw new Error('request body exceeds 1 MiB');
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString('utf8');
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
