// An EMF that only wraps a bitmap (one EMR_STRETCHDIBITS record, no other
// drawing) becomes a BMP file the webview can draw (specs/07-reader.md,
// "DOCX"). Anything else is null.

const EMR_HEADER = 1
const EMR_EOF = 14
const EMR_GDICOMMENT = 70
const EMR_STRETCHDIBITS = 81
const NOT_DRAWING = [EMR_HEADER, EMR_EOF, EMR_GDICOMMENT]
const BMP_FILE_HEADER = 14

type EmfRecord = { kind: number; offset: number }

// Every record, or none when the sizes do not add up.
function records(view: DataView): EmfRecord[] {
  const found: EmfRecord[] = []
  for (let offset = 0; offset + 8 <= view.byteLength; ) {
    const size = view.getUint32(offset + 4, true)
    if (size < 8 || offset + size > view.byteLength) return []
    found.push({ kind: view.getUint32(offset, true), offset })
    offset += size
  }
  return found
}

// The bitmap header and bits of an EMR_STRETCHDIBITS, behind a BMP file header.
function toBmp(view: DataView, record: number): Uint8Array<ArrayBuffer> {
  const [offBmi, cbBmi, offBits, cbBits] = [48, 52, 56, 60].map((at) => view.getUint32(record + at, true))
  const bmp = new Uint8Array(BMP_FILE_HEADER + cbBmi + cbBits)
  const header = new DataView(bmp.buffer)
  bmp.set([0x42, 0x4d])
  header.setUint32(2, bmp.length, true)
  header.setUint32(10, BMP_FILE_HEADER + cbBmi, true)
  bmp.set(new Uint8Array(view.buffer, view.byteOffset + record + offBmi, cbBmi), BMP_FILE_HEADER)
  bmp.set(new Uint8Array(view.buffer, view.byteOffset + record + offBits, cbBits), BMP_FILE_HEADER + cbBmi)
  return bmp
}

export function emfBitmap(bytes: ArrayBuffer): Uint8Array<ArrayBuffer> | null {
  const view = new DataView(bytes)
  const found = records(view)
  if (found[0]?.kind !== EMR_HEADER) return null
  const drawing = found.filter((record) => !NOT_DRAWING.includes(record.kind))
  if (drawing.length !== 1 || drawing[0].kind !== EMR_STRETCHDIBITS) return null
  return toBmp(view, drawing[0].offset)
}
