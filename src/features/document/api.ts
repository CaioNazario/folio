import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
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

export function resolveLink(id: string, href: string): Promise<LinkTarget> {
  return invoke<LinkTarget>('resolve_link', { id, href })
}

// http, https and mailto, in the system browser or mail app.
export function openExternal(url: string): Promise<void> {
  return openUrl(url)
}
