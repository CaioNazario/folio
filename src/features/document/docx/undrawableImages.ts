import { emfBitmap } from './emfBitmap'

// Images in a DOCX the webview cannot decode (EMF, WMF): an EMF that wraps a
// bitmap shows that bitmap; anything else becomes the "unsupported" image
// placeholder with its alt text (specs/07-reader.md, "DOCX").

export type ReadImage = (src: string) => Promise<ArrayBuffer>

// docx-preview serves images as blob: URLs of the package parts.
const readImage: ReadImage = async (src) => (await fetch(src)).arrayBuffer()

function placeholder(img: HTMLImageElement): HTMLElement {
  const span = document.createElement('span')
  span.dataset.folioImage = 'unsupported'
  span.textContent = img.alt
  return span
}

export async function fixUndrawableImage(img: HTMLImageElement, read: ReadImage) {
  const bytes = await read(img.src).catch(() => new ArrayBuffer(0))
  const bitmap = emfBitmap(bytes)
  if (bitmap) img.src = URL.createObjectURL(new Blob([bitmap], { type: 'image/bmp' }))
  else img.replaceWith(placeholder(img))
}

// Waits for every image to decode; the ones that fail get fixed.
export async function fixUndrawableImages(host: HTMLElement, read: ReadImage = readImage) {
  const images = [...host.querySelectorAll('img')]
  await Promise.all(images.map((img) => img.decode().catch(() => fixUndrawableImage(img, read))))
}
