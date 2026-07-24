import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

// Unit/component tests run under jsdom. The `@` alias mirrors tsconfig so tests
// import the same way app code does. Test files live next to what they cover
// (`*.test.ts[x]`) and are excluded from `next build` (see tsconfig) — Vitest
// type-checks and runs them instead.
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
