import { defineConfig } from 'vitest/config';
import { fileURLToPath } from 'node:url';

const workspacePath = (path: string): string => fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
  resolve: {
    alias: {
      '@asha-rulebench/components': workspacePath('./libs/components/src/index.ts'),
      '@asha-rulebench/platform': workspacePath('./libs/platform/src/index.ts'),
      '@asha-rulebench/scenario-viewer': workspacePath('./libs/scenario-viewer/src/index.ts'),
      '@asha-rulebench/shell': workspacePath('./libs/shell/src/index.ts'),
      '@asha-rulebench/theme': workspacePath('./libs/theme/src/index.ts'),
    },
  },
  test: {
    include: ['libs/**/*.spec.ts'],
    exclude: ['**/node_modules/**', 'dist/**', 'coverage/**', 'apps/app-e2e/**'],
  },
});
