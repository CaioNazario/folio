// View models the document view renders. The feature's api.ts maps the ts-rs
// bindings into them once the reader commands exist.

export type DocumentFormat = 'pdf' | 'docx' | 'md' | 'txt'

export type OpenDocument = {
  id: string
  name: string
  format: DocumentFormat
  /** Project or folder shown in the breadcrumb; null at the workspace root. */
  folder: string | null
  /** Created by a conversion (badge "derivado"). */
  derived: boolean
  /** Formats this document can be converted to; empty when invalid. */
  conversions: DocumentFormat[]
}

export type StageContent =
  | { status: 'loading' }
  | { status: 'text'; text: string; truncated: boolean }
  | { status: 'html'; html: string; truncated: boolean }
  | { status: 'invalid' }
  | { status: 'notFound' }
  | { status: 'failed' }

export type DocumentActions = {
  onConvert: (format: DocumentFormat) => void
  onRename: () => void
  onShowInFolder: () => void
  onCopyFile: () => void
  onProperties: () => void
  onTrash: () => void
  onRetry: () => void
  /** A link clicked inside the document; the core decides what it is. */
  onLink: (target: string) => void
}
