import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const dirname = path.dirname(fileURLToPath(import.meta.url));

// Minimal Vitest setup for pure unit tests (zod schemas, small helpers, hooks).
// Default environment is `node` since most of the first test slice is pure
// logic; individual files needing a DOM (e.g. renderHook) opt in with a
// `// @vitest-environment jsdom` docblock instead of paying the jsdom cost
// project-wide.
export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(dirname, './src'),
    },
  },
  test: {
    env: {
      JWT_SECRET: 'test-jwt-secret',
      NEXT_PUBLIC_API_URL: 'http://localhost:8080/api',
      NEXT_PUBLIC_WS_URL: 'ws://localhost:8080/ws',
      NODE_ENV: 'test',
    },
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
