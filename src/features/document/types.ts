import type { Format } from '@/lib/bindings/Format'

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
  /** Fraction of the scroll where the reader stopped (MD and TXT). */
  scroll: number | null
}

export type StageContent =
  | { status: 'loading' }
  | { status: 'text'; text: string; truncated: boolean }
  | { status: 'html'; html: string; truncated: boolean }
  | { status: 'invalid' }
  | { status: 'notFound' }
  | { status: 'failed' }
  /** The view for this format is not built yet (PDF and DOCX wait for spikes S2 and S4). */
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
