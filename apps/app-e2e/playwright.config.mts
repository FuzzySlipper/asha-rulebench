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
        env: {
          ...process.env,
          RULEBENCH_EPHEMERAL_ARTIFACTS: '1',
          RULEBENCH_ANGULAR_CONFIGURATION: 'e2e',
          RULEBENCH_RANDOM_TAPE: '10,3,4,1,2,3,4,1',
          RULEBENCH_SOURCE_SET_CONFIG: '.rulebench/source-sets.example.json',
        },
      },
    };

export default defineConfig({
  ...nxE2EPreset(import.meta.dirname, { testDir: './src' }),
  retries: 0,
  preserveOutput: 'always',
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
