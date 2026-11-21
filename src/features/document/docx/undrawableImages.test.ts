import { expect, test } from 'vitest'
import { fixUndrawableImage } from './undrawableImages'

test('shows a placeholder for an image the webview cannot draw', async () => {
  const paragraph = document.createElement('p')
  const img = paragraph.appendChild(document.createElement('img'))
  img.alt = 'Organograma'
  img.src = 'blob:http://localhost/wmf'
  const wmf = new Uint8Array([0xd7, 0xcd, 0xc6, 0x9a, 0, 0, 0, 0]).buffer

  await fixUndrawableImage(img, async () => wmf)

  const placeholder = paragraph.querySelector<HTMLElement>('[data-folio-image]')
  expect(placeholder?.dataset.folioImage).toBe('unsupported')
  expect(placeholder).toHaveTextContent('Organograma')
  expect(paragraph.querySelector('img')).toBeNull()
})
