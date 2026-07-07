import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['libs/**/*.spec.ts'],
    exclude: ['**/node_modules/**', 'dist/**', 'coverage/**', 'apps/app-e2e/**'],
  },
});
