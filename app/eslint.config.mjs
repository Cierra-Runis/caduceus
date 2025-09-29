/// https://www.npmjs.com/package/@eslint/js
/// The beginnings of separating out JavaScript-specific functionality from ESLint.
import eslintPluginJavaScript from '@eslint/js';
/// https://www.npmjs.com/package/@next/eslint-plugin-next
/// Official ESLint plugin for Next.js
import eslintPluginNext from '@next/eslint-plugin-next';
/// https://github.com/azat-io/eslint-plugin-perfectionist
/// ESLint plugin for sorting various data such as objects, imports, types, enums, JSX props, etc.
import eslintPluginPerfectionist from 'eslint-plugin-perfectionist';
/// https://github.com/francoismassart/eslint-plugin-tailwindcss
/// While you can use the official plugin for ordering, this plugin offers more than 5 other rules
import eslintPluginTailwindCSS from 'eslint-plugin-tailwindcss';
/// https://typescript-eslint.io/getting-started
/// Powerful static analysis for JavaScript and TypeScript.
import eslintPluginTypeScript from 'typescript-eslint';

/// FIXME: https://github.com/vercel/next.js/issues/73655#issuecomment-3344699670
const { flatConfig: eslintPluginNextFlatConfig } = eslintPluginNext;

import { defineConfig } from 'eslint/config';

export default defineConfig([
  {
    ignores: [
      'node_modules/**',
      '.next/**',
      'out/**',
      'build/**',
      'next-env.d.ts',
    ],
  },
  eslintPluginJavaScript.configs.recommended,
  eslintPluginTypeScript.configs.strict,
  eslintPluginTypeScript.configs.stylistic,
  eslintPluginTailwindCSS.configs['flat/recommended'],
  eslintPluginPerfectionist.configs['recommended-alphabetical'],
  eslintPluginNextFlatConfig.recommended,
]);
