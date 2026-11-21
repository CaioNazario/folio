import type { DocumentView } from '@/lib/bindings/DocumentView'
import { isBridgeError } from '@/lib/i18n/errors'
import { getDocumentBytes, getMarkdownContent, getTextContent } from './api'
import type { OpenDocument, StageContent } from './types'

export function toOpenDocument({ document, folders, derived, conversionTargets, position }: DocumentView): OpenDocument {
  const { id, name, format, valid } = document
  return { id, name, format, folders, derived, conversions: conversionTargets, valid, position }
}

// Which stage state a failed load shows (specs/07-reader.md, "Estados do palco").
export function failedContent(error: unknown): StageContent {
  if (!isBridgeError(error)) return { status: 'failed' }
  if (error.kind === 'DocumentNotFound') return { status: 'notFound' }
  if (error.kind === 'InvalidDocument') return { status: 'invalid' }
  if (error.kind === 'PasswordRequired') return { status: 'password', wrong: false }
  if (error.kind === 'WrongPassword') return { status: 'password', wrong: true }
  return { status: 'failed' }
}

// The stage content for an opened document, by format. `password` opens a
// protected DOCX; pdf.js asks for a PDF's itself.
export async function loadContent(document: OpenDocument, password: string | null = null): Promise<StageContent> {
  if (!document.valid) return { status: 'invalid' }
  try {
    if (document.format === 'txt') return { status: 'text', ...(await getTextContent(document.id)) }
    if (document.format === 'md') return { status: 'html', ...(await getMarkdownContent(document.id)) }
    if (document.format === 'pdf') return { status: 'pdf', bytes: await getDocumentBytes(document.id) }
    return { status: 'docx', bytes: await getDocumentBytes(document.id, password) }
  } catch (error) {
    return failedContent(error)
  }
}
