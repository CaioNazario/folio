import type { RefObject } from 'react'
import type { Format } from '@/lib/bindings/Format'
import type { ReadingPosition } from '@/lib/bindings/ReadingPosition'
import type { Pager } from './pdf/usePdfView'

// View models the document view renders, mapped from the ts-rs bindings in
// documentData.ts.

export type DocumentFormat = Format

export type OpenDocument = {
  id: string
  name: string
  format: DocumentFormat
  /** Project and folders for the breadcrumb, outermost first; empty at the root. */
  folders: string[]
  /** Created by a conversion (badge "derivado"). */
  derived: boolean
  /** Formats this document can be converted to; empty when invalid. */
  conversions: DocumentFormat[]
  valid: boolean
  /** Where the reader stopped: a page (PDF) or a scroll fraction (MD, TXT). */
  position: ReadingPosition | null
}

export type StageContent =
  | { status: 'loading' }
  | { status: 'text'; text: string; truncated: boolean }
  | { status: 'html'; html: string; truncated: boolean }
  /** pdf.js takes these bytes; the display keeps its own copy only in development. */
  | { status: 'pdf'; bytes: ArrayBuffer }
  | { status: 'invalid' }
  | { status: 'notFound' }
  | { status: 'failed' }
  /** The view for this format is not built yet (DOCX waits for spike S4). */
  | { status: 'unsupported' }

export type DocumentActions = {
  onConvert: (format: DocumentFormat) => void
  onRename: () => void
  onShowInFolder: () => void
  onCopyFile: () => void
  onProperties: () => void
  onTrash: () => void
  onRetry: () => void
  /** A link clicked inside the document; the core decides what it is. */
  onLink: (href: string) => void
}

// How the stage reads and reports where the reader is (specs/07-reader.md,
// "Posição"), and what the page keys move.
export type Reading = {
  /** Where to open: where the reader stopped. */
  start: ReadingPosition | null
  onPosition: (position: ReadingPosition) => void
  /** The scrolling page of MD and TXT. */
  pageRef: RefObject<HTMLElement | null>
  /** The pages of a PDF, while one is shown. */
  pagerRef: RefObject<Pager | null>
}
