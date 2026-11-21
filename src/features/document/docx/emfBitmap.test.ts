import { expect, test } from 'vitest'
import { emfBitmap } from './emfBitmap'

const BITMAP_INFO = 40
const BITS = [0x11, 0x22, 0x33, 0xff]

// EMR_HEADER, one EMR_STRETCHDIBITS with a 1x1, 32-bit bitmap, EMR_EOF.
function wrappedBitmap(): ArrayBuffer {
  const stretch = 80 + BITMAP_INFO + BITS.length
  const view = new DataView(new ArrayBuffer(8 + stretch + 20))
  view.setUint32(0, 1, true)
  view.setUint32(4, 8, true)
  view.setUint32(8, 81, true)
  view.setUint32(12, stretch, true)
  ;[80, BITMAP_INFO, 80 + BITMAP_INFO, BITS.length].forEach((value, i) => view.setUint32(8 + 48 + 4 * i, value, true))
  view.setUint32(8 + 80, BITMAP_INFO, true)
  BITS.forEach((byte, i) => view.setUint8(8 + 80 + BITMAP_INFO + i, byte))
  view.setUint32(8 + stretch, 14, true)
  view.setUint32(8 + stretch + 4, 20, true)
  return view.buffer
}

test('turns an emf that wraps a bitmap into a bmp', () => {
  const bmp = emfBitmap(wrappedBitmap())
  const header = new DataView((bmp as Uint8Array<ArrayBuffer>).buffer)

  expect(String.fromCharCode(...(bmp as Uint8Array).slice(0, 2))).toBe('BM')
  expect(header.getUint32(2, true)).toBe(14 + BITMAP_INFO + BITS.length)
  expect(header.getUint32(10, true)).toBe(14 + BITMAP_INFO)
  expect([...(bmp as Uint8Array).slice(14 + BITMAP_INFO)]).toEqual(BITS)
})
