// pdf.js with its worker bundled by Vite (specs/07-reader.md, "PDF").
// pdf_viewer.mjs reads pdf.js from globalThis.pdfjsLib when it loads, so
// this module must be imported before it.
import * as pdfjs from 'pdfjs-dist'
import workerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url'

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl
Object.assign(globalThis, { pdfjsLib: pdfjs })

export { pdfjs }

// Run-time files copied to pdfjs/ by vite.config.ts. No WebAssembly: the
// CSP blocks it, and the JavaScript decoders render the same (spike S2).
export const PDFJS_OPTIONS = {
  cMapUrl: '/pdfjs/cmaps/',
  standardFontDataUrl: '/pdfjs/standard_fonts/',
  wasmUrl: '/pdfjs/wasm/',
  iccUrl: '/pdfjs/iccs/',
  useWasm: false,
}
