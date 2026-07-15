import { defineConfig, devices } from '@playwright/test';
import { workspaceRoot } from '@nx/devkit';
import { nxE2EPreset } from '@nx/playwright/preset';

const localPort = process.env['E2E_PORT'] ?? '4317';
const localBaseUrl = `http://127.0.0.1:${localPort}`;
const baseURL = process.env['BASE_URL'] ?? localBaseUrl;

const localWebServer = process.env['BASE_URL']
  ? {}
  : {
      webServer: {
        command: `pnpm dev -- --port ${localPort}`,
        url: localBaseUrl,
        reuseExistingServer: false,
        cwd: workspaceRoot,
        env: { ...process.env, RULEBENCH_EPHEMERAL_ARTIFACTS: '1' },
      },
    };

export default defineConfig({
  ...nxE2EPreset(import.meta.dirname, { testDir: './src' }),
  // Every test project talks to one intentionally single-writer process host.
  // Running stateful files concurrently makes content activation and session
  // creation race through shared authority instead of testing isolated flows.
  workers: 1,
  use: {
    baseURL,
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  ...localWebServer,
});
