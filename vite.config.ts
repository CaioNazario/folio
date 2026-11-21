import { fileURLToPath, URL } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { viteStaticCopy } from 'vite-plugin-static-copy'
import { defineConfig } from 'vitest/config'

// Files pdf.js loads at run time, served from pdfjs/ (specs/07-reader.md,
// "PDF"). Without WebAssembly, only the JavaScript decoders are needed.
const PDFJS = 'node_modules/pdfjs-dist'
const pdfjsAssets = viteStaticCopy({
  targets: [
    {
      src: ['cmaps', 'standard_fonts', 'iccs', 'wasm/*_nowasm_fallback.js', 'wasm/LICENSE_*'].map((path) => `${PDFJS}/${path}`),
      dest: 'pdfjs',
      // node_modules/pdfjs-dist/cmaps -> pdfjs/cmaps
      rename: { stripBase: 2 },
    },
  ],
})

// Host de rede só existe no dev mobile (tauri android dev).
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [react(), tailwindcss(), pdfjsAssets],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  // Não altere clearScreen, server.port, strictPort nem watch.ignored sem
  // perguntar: a porta precisa bater com o devUrl do src-tauri/tauri.conf.json.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**', '**/target/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test-setup.ts'],
  },
})
