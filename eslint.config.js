import js from '@eslint/js'
import { defineConfig, globalIgnores } from 'eslint/config'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import sonarjs from 'eslint-plugin-sonarjs'
import globals from 'globals'
import tseslint from 'typescript-eslint'

// Limites de tamanho do AGENTS.md ("Regras de código" e "Frontend"). Não altere.
export default defineConfig([
  globalIgnores(['dist', 'target', 'src-tauri', 'src/lib/bindings', 'Design']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
      sonarjs.configs.recommended,
    ],
    languageOptions: {
      ecmaVersion: 2023,
      globals: globals.browser,
    },
    rules: {
      'max-lines': ['error', { max: 500 }],
      'max-depth': ['error', 2],
    },
  },
  {
    files: ['**/*.ts'],
    rules: { 'max-lines-per-function': ['error', { max: 20 }] },
  },
  {
    files: ['**/*.tsx'],
    rules: { 'max-lines-per-function': ['error', { max: 80 }] },
  },
  {
    // Blocos describe() agrupam vários testes: o limite vale para funções e
    // componentes, não para o arquivo de teste inteiro.
    files: ['**/*.test.{ts,tsx}'],
    rules: { 'max-lines-per-function': 'off' },
  },
])
