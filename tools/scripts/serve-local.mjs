import { createServer } from 'node:net';
import { spawn } from 'node:child_process';

const host = '0.0.0.0';
const probeHost = '127.0.0.1';
const port = await freePort();
const publicUrl = `http://${probeHost}:${port}`;
const child = spawn('pnpm', ['nx', 'serve', 'app', '--host', host, '--port', String(port)], {
  stdio: 'inherit',
  shell: false,
});

console.log(`BASE_URL=${publicUrl}`);

process.on('SIGINT', () => {
  child.kill('SIGINT');
});

process.on('SIGTERM', () => {
  child.kill('SIGTERM');
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, probeHost, () => {
      const address = server.address();
      server.close(() => {
        if (address !== null && typeof address === 'object') {
          resolve(address.port);
        } else {
          reject(new Error('Could not allocate a local port'));
        }
      });
    });
    server.on('error', reject);
  });
}
