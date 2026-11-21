import { invoke } from '@tauri-apps/api/core'
import type { DisplayText } from '@/lib/bindings/DisplayText'
import type { DocumentView } from '@/lib/bindings/DocumentView'
import type { LinkTarget } from '@/lib/bindings/LinkTarget'
import type { MarkdownContent } from '@/lib/bindings/MarkdownContent'
import type { ReadingPosition } from '@/lib/bindings/ReadingPosition'

export function openDocument(id: string): Promise<DocumentView> {
  return invoke<DocumentView>('open_document', { id })
}

export function getTextContent(id: string): Promise<DisplayText> {
  return invoke<DisplayText>('get_text_content', { id })
}

// Raw bytes over the bridge (specs/07-reader.md, "Exibição"). `password`
// opens a protected DOCX; pdf.js asks for a PDF's itself.
export function getDocumentBytes(id: string, password: string | null = null): Promise<ArrayBuffer> {
  return invoke<ArrayBuffer>('get_document_bytes', { id, password })
}

export function getMarkdownContent(id: string): Promise<MarkdownContent> {
  return invoke<MarkdownContent>('get_markdown_content', { id })
}

// Called once the stage actually shows the document (specs/06). Must come
// before savePosition: the position lives in the recents row.
export function recordOpened(id: string): Promise<void> {
  return invoke('record_opened', { id })
}

export function savePosition(id: string, position: ReadingPosition): Promise<void> {
  return invoke('save_position', { id, position })
}

// The backend resolves the link and opens an External one itself, in the
// system browser or mail app; the rest is left to the UI.
export function followLink(id: string, href: string): Promise<LinkTarget> {
  return invoke<LinkTarget>('follow_link', { id, href })
}
