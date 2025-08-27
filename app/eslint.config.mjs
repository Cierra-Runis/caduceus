import { FlatCompat } from '@eslint/eslintrc';
/// [eslint-plugin-perfectionist](https://github.com/azat-io/eslint-plugin-perfectionist)
/// ESLint plugin for sorting various data such as objects, imports, types, enums, JSX props, etc.
import eslintPluginPerfectionist from 'eslint-plugin-perfectionist';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const compat = new FlatCompat({
  baseDirectory: __dirname,
});

const eslintConfig = [
  eslintPluginPerfectionist.configs['recommended-alphabetical'],
  ...compat.extends('next/core-web-vitals', 'next/typescript'),
];

export default eslintConfig;
