import { spawn, spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { createServer as createHttpServer } from 'node:http';
import { createServer as createNetServer } from 'node:net';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = process.cwd();
const forwardedArguments = process.argv.slice(2);
if (forwardedArguments[0] === '--') forwardedArguments.shift();

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
    'libs',
    'content-authoring',
    'src',
    'index.js',
  ),
);
const { prepareRulebenchRulesetSource } = await import(
  `${authoringModuleUrl.href}?startup=${Date.now()}`
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
  prepareRulebenchRulesetSource,
);

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

function startAuthoringGateway(port, rustHostUrl, prepareSource) {
  const server = createHttpServer(async (request, response) => {
    try {
      if (request.method === 'POST' && request.url === '/api/ruleset/compile') {
        const body = await readJsonBody(request);
        if (
          body === null ||
          typeof body !== 'object' ||
          Array.isArray(body) ||
          !('sourceId' in body) ||
          (body.sourceId !== 'fresh' && body.sourceId !== 'missingSupport')
        ) {
          await respondWithGatewayDiagnostic(
            response,
            rustHostUrl,
            400,
            'RULESET_SOURCE_SELECTION_INVALID',
            '$.sourceId',
            'sourceId must be fresh or missingSupport',
          );
          return;
        }
        const preparation = prepareSource(body.sourceId);
        if (!preparation.ok) {
          const workspace = await rustWorkspace(rustHostUrl);
          sendJson(response, 422, {
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
      await forwardJson(
        response,
        `${rustHostUrl}${request.url ?? '/api/ruleset'}`,
        request.method === 'POST' ? 'POST' : 'GET',
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
  const chunks = [];
  let length = 0;
  for await (const chunk of request) {
    length += chunk.length;
    if (length > 1_048_576) throw new Error('request body exceeds 1 MiB');
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
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
