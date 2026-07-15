/// https://www.npmjs.com/package/@eslint/js
/// The beginnings of separating out JavaScript-specific functionality from ESLint.
import eslintPluginJavaScript from '@eslint/js';
/// https://www.npmjs.com/package/@next/eslint-plugin-next
/// Official ESLint plugin for Next.js
import eslintPluginNext from '@next/eslint-plugin-next';
/// https://github.com/schoero/eslint-plugin-better-tailwindcss
/// ESLint plugin to help you write better tailwindcss by improving readability with formatting rules and enforcing best practices with linting rules.
import eslintPluginBetterTailwindCss from 'eslint-plugin-better-tailwindcss';
/// https://github.com/azat-io/eslint-plugin-perfectionist
/// ESLint plugin for sorting various data such as objects, imports, types, enums, JSX props, etc.
import eslintPluginPerfectionist from 'eslint-plugin-perfectionist';
/// https://github.com/marcalexiei/eslint-zod
/// ESLint plugin that adds custom linting rules to enforce best practices when using Zod.
import eslintPluginZod from 'eslint-plugin-zod';
/// https://typescript-eslint.io/getting-started
/// Powerful static analysis for JavaScript and TypeScript.
import { defineConfig } from 'eslint/config';
import eslintPluginTypeScript from 'typescript-eslint';

const eslintPluginBetterTailwindCssConfig = {
  files: ['**/*.{jsx,tsx}'],
  languageOptions: {
    parserOptions: {
      ecmaFeatures: {
        jsx: true,
      },
    },
  },
  name: 'better-tailwindcss/',
  plugins: {
    'better-tailwindcss': eslintPluginBetterTailwindCss,
  },
  rules: {
    ...eslintPluginBetterTailwindCss.configs['recommended-warn'].rules,
    ...eslintPluginBetterTailwindCss.configs['recommended-error'].rules,
    "better-tailwindcss/enforce-consistent-line-wrapping": "off",
  },
  settings: {
    'better-tailwindcss': {
      entryPoint: 'src/styles/globals.css',
    },
  },
};

export default defineConfig([
  {
    ignores: [
      'node_modules/**',
      '.next/**',
      'out/**',
      'build/**',
      'next-env.d.ts',
      './src/components/ui/**',
    ],
    name: 'base/',
  },
  eslintPluginJavaScript.configs.recommended,
  eslintPluginTypeScript.configs.strict,
  eslintPluginTypeScript.configs.stylistic,
  eslintPluginBetterTailwindCssConfig,
  eslintPluginPerfectionist.configs['recommended-alphabetical'],
  eslintPluginNext.configs.recommended,
  eslintPluginZod.configs.recommended,
]);
