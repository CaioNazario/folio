import { invoke } from '@tauri-apps/api/core'
import type { DisplayText } from '@/lib/bindings/DisplayText'

export function getTextContent(id: string): Promise<DisplayText> {
  return invoke<DisplayText>('get_text_content', { id })
}

// Called once the stage actually shows the document (specs/06).
export function recordOpened(id: string): Promise<void> {
  return invoke('record_opened', { id })
}
