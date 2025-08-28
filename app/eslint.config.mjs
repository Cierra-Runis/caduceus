import { FlatCompat } from '@eslint/eslintrc';
/// [eslint-plugin-perfectionist](https://github.com/azat-io/eslint-plugin-perfectionist)
/// ESLint plugin for sorting various data such as objects, imports, types, enums, JSX props, etc.
import eslintPluginPerfectionist from 'eslint-plugin-perfectionist';
/// [eslint-plugin-tailwindcss](https://github.com/francoismassart/eslint-plugin-tailwindcss)
/// While you can use the official plugin for ordering, this plugin offers more than 5 other rules
import eslintPluginTailwindCSS from 'eslint-plugin-tailwindcss';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const compat = new FlatCompat({
  baseDirectory: __dirname,
});

/**  @type {import('eslint').Linter.Config[]} */
const eslintConfig = [
  {
    ignores: [
      'node_modules/**',
      '.next/**',
      'out/**',
      'build/**',
      'next-env.d.ts',
    ],
  },
  ...compat.extends('next/core-web-vitals', 'next/typescript'),
  eslintPluginPerfectionist.configs['recommended-alphabetical'],
  ...eslintPluginTailwindCSS.configs['flat/recommended'],
];

export default eslintConfig;
